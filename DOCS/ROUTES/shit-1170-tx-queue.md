# SHIT-1170 TX QUEUE

`POST /api/shit-1170/tx-queue` - queue a batch of transactions.

`GET /api/shit-1170/tx-queue` - the queue list, filterable.

the agent queues, the user signs, the queue records what happened.

## THE MODEL

a queued tx is **WHAT, WHEN, WHY, WHO, HOW**.

- **WHAT** - what the transactions do, in words
- **WHEN** - when they should happen, in words, and optionally at a
  machine-readable moment
- **WHY** - why the user wants them
- **WHO** - the accounts the batch is for
- **HOW** - the function calls themselves, or a key action

a queue is a **BATCH** of transactions, not one transaction. the agent
queues a swap, and that swap needs three function calls - a deposit, the
swap itself, and a withdrawal - so they go in together under one
`queue_id`, and the user signs them **as one batch, in one wallet
popup**.

the model is three levels deep:

```
a QUEUE  (queue_id)          what / when / why
  └── a TRANSACTION          contractId / method / args / gas / deposit
        └── an ACCOUNT ROW    accountId / status / txHash
```

## THE TWO KINDS OF TRANSACTION

a queued transaction is EITHER a **contract call** OR a **key action**.

a **contract call** is the usual one: a contract, a method, and the
arguments.

a **key action** is an access key operation - `add_key` or
`delete_key`:

- `add_key` - give the account a new LIMITED key. it names the public
  key, the contract the key may call, the methods it may call, and
  optionally a yoctoNEAR ceiling.
- `delete_key` - take a LIMITED key off the account. it names the
  public key, and nothing else.

> --- WHY --- the queue carries key actions at all. an agent that may
> only queue calls can never offer the user "give me a limited key" or
> "take that limited key away", and those are ordinary things a user
> asks for. a limited key is how an agent is given the right to do one
> thing and not another, and removing one is how a user takes that
> right back.

> --- WHY --- A KEY ACTION HAS NO CONTRACT AND NO METHOD. it is not a
> call, and it does not call anything. the db's `contract_id` and
> `method` columns are `NOT NULL` and nulling them would need a table
> rebuild, so a key action writes the EMPTY STRING there. the empty
> string means "this is not a contract call", and the `keyAction` field
> is what says which it is.

> --- WHY --- ONLY A DELETE IS CHECKED AGAINST THE CHAIN. an `add_key`
> names a key that does not exist yet, so there is nothing on chain to
> check - and it cannot take anything away. a `delete_key` CAN take
> something away, and the thing it takes away may be the user's whole
> wallet.

## THE STATUS

there are four states, and they live **only on the account row**:

- `pending` - on the queue, not signed yet
- `sent` - signed and sent, the outcome is not known yet
- `succeeded` - it went through
- `failed` - it did not work

> --- WHY --- the status lives ONLY on the account row. it is the one
> place where the truth is. a transaction is not "sent" or "failed" on
> its own - it is sent BY an account, and each account's send is its own
> transaction with its own hash and its own outcome. two accounts sending
> the same queued transaction produce two different on-chain
> transactions, and either can succeed while the other fails.

> --- WHY --- there is NO queue-level status and NO transaction-level
> status. a queue is never done. at any moment another account can
> decide they want to send it, and they join. a "is this queue done"
> badge would be a lie the moment somebody new signs it. the ui does not
> need a done badge - it needs to know, per account, whether that
> account has sent it.

> --- WHY --- `sent` is a real state and not a leftover. the outcome is
> not known at the moment the hash is written - the frontend reads it
> off the chain later and reports it back. so `sent` means "it went out
> and nobody knows how it went yet".

## THE TWO RULES

there are two rules here, and they pull in opposite directions.

**the NAMING rule** - the agent queues a tx and names the accounts it is
for. the agent may name **ONLY accounts whose api key it holds**.

**the SIGNING rule** - a user sees a tx in their feed and signs it. any
signed-in user may sign **any** queue they can see.

> --- WHY --- the naming rule exists. the WHO list is what puts a tx in
> somebody's feed. if an agent could name any account id it liked, agent
> A could fill agent B's users' feeds with txs they never asked for -
> and the ui filters by account, so those users would keep seeing them.
> one agent spamming another agent's users through the queue is exactly
> what this stops. so the agent's WHO list is its OWN users, and only
> its own users.

> --- WHY --- the signing rule stays open. the queue is an open offer. a
> user who sees a tx may send it, and their account joins the WHO list.
> that is a user acting for themselves with their own key - no agent is
> putting it in front of them, they found it and chose it.

so the queue never becomes a spam channel between agents, and it never
becomes a closed club either.

the naming check is one query: does the `api_keys` table have a row for
that account. the api_keys table IS the record of the keys that were
issued, and an agent holds the key it was given - so a row in it is the
agent saying "this account is mine".

the agent's OWN account is always allowed to name itself.

## THE CAP

a batch holds at most **50** transactions. more than 50 is a `400`.

> --- WHY --- fifty. the wallet signs the whole batch in ONE popup, and a
> popup with two hundred transactions is unreadable and slow.

## THE POST

### the body

```json
{
  "what": "swap 100 USDC for NEAR on the ref finance dex",
  "when": "when the price is right",
  "why": "you said to move half your usdc into near",
  "whenAt": 1772666400000,
  "transactions": [
    {
      "contractId": "ref.finance.near",
      "method": "ft_swap",
      "args": {
        "receiver_id": "alice.near",
        "amount": "100000000",
        "token_in": "usdt.tether-token.near",
        "token_out": "wrap.near"
      },
      "gas": "30000000000000",
      "deposit": "1"
    },
    {
      "contractId": "ref.finance.near",
      "method": "ft_resolve_transfer",
      "args": {
        "token_id": "usdt.tether-token.near",
        "receiver_id": "alice.near",
        "amount": "100000000"
      },
      "gas": "30000000000000",
      "deposit": "1"
    }
  ],
  "accounts": ["alice.near", "bob.near"]
}
```

`transactions` is a LIST. one entry is a batch of one. the order in the
list is the order the transactions are sent in, and it becomes the
`position` column.

`args` is a json OBJECT on the wire, not a string. it may be left out,
and then it is an empty object - a call with no arguments is an
ordinary thing.

### the raw bytes

some contracts take their arguments as RAW BYTES and not as json. the
FastFS protocol (`__fastdata_fastfs`) is one of them - it wants borsh
bytes, and a json object cannot be borsh bytes.

such a call carries `argsBase64` instead of `args`:

```json
{
  "contractId": "fastfs.near",
  "method": "__fastdata_fastfs",
  "argsBase64": "AAwAAABmYXN0bmVhci5wbmc=",
  "gas": "1",
  "deposit": "0"
}
```

`argsBase64` is **standard base64** - the `A-Z a-z 0-9 + /` alphabet with
`=` padding. it is the encoding the near wallet uses for action
arguments, so the bytes that come out of the wire are the bytes the
chain expects. it is NOT the url-safe kind.

a transaction sets **exactly one** of `args` and `argsBase64`:

- **both** set - a `400` with the code `args_and_args_base64`
- **neither** set - the args are an empty object, which is what a call
  with no arguments has always been
- an `argsBase64` that does not decode - a `400` with the code
  `args_base64_not_base64`

> --- WHY --- the two are ALTERNATIVES and not a pair. they carry
> different things. `args` is a json object the toolbox can name its
> keys from, and `argsBase64` is an opaque run of bytes that only the
> contract can read. a transaction that carries both does not say which
> one the contract gets.
>
> --- WHY --- the browser decodes the base64 back into bytes and hands
> the BYTES to the wallet. the wallet's adapter tries `JSON.parse` on the
> bytes and passes them through untouched when they are not json, so the
> raw borsh bytes reach the chain as they are.

`accounts` is the WHO - the accounts the agent is queuing this for. it
is **optional**, and **every account in it must be one the agent holds
an api key for** - the NAMING rule. an account the agent has no key
for is refused with a `400`, so an agent cannot drop a tx into somebody
else's feed.

those accounts get rows with `status = pending`, so they see the tx in
their feed before anyone has signed it.

> --- WHY --- `what`, `when` and `why` are all optional. the agent may
> know the WHAT and not the WHY. an agent that had to fill all three
> would make one up, and a made-up WHY is worse than no WHY.

> --- WHY --- `when` is text AND `whenAt` is a timestamp. most of the
> time "when" is a CONDITION and not a moment - "when the price is
> right", "after this proposal passes". a string can say that and a
> timestamp cannot. but some requests really are "at 10:00 on friday",
> and that needs a number the machine can sort on and the ui can count
> down to. so both, and `whenAt` is null when there is no moment.

> --- WHY --- the args must be a json OBJECT. the near-kit toolbox takes
> a json object of arguments, and the frontend hands the transactions
> straight to `generic_contract_call_wallet`. an array or a bare string
> is not a set of named arguments, and the call would fail on chain.

### the key actions

a transaction that carries a `keyAction` is a key action and not a
contract call:

```json
{
  "what": "take back the key i gave you",
  "transactions": [
    {
      "keyAction": "delete_key",
      "keyPublicKey": "ed25519:FrtUfbY74kXpBiWmqeQMtn7jKCX3zJhmwyg32ZqANj3c"
    },
    {
      "keyAction": "add_key",
      "keyPublicKey": "ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp",
      "keyReceiverId": "ref.finance.near",
      "keyMethodNames": [ "ft_swap" ],
      "keyAllowance": "1000000000000000000000000"
    }
  ],
  "accounts": ["alice.near"]
}
```

the fields are:

- `keyAction` - `add_key` or `delete_key`. it is **optional**, and
  without it the transaction is an ordinary contract call.
- `keyPublicKey` - the public key the action names. it is **required**
  for a key action.
- `keyReceiverId` - the contract the NEW key may call. it is
  **required** for an `add_key` and is not used by a `delete_key`.
- `keyMethodNames` - the methods the NEW key may call. it is only for
  an `add_key`. an empty list means it may call every method of its
  receiver.
- `keyAllowance` - the yoctoNEAR ceiling on the NEW key. it is only
  for an `add_key`, and it is a **STRING** because a yoctoNEAR amount
  does not fit a json number safely when it is big.

a key action may **not** carry a `contractId` or a `method`. it has
nothing to call, so a contract on it says the request does not know
what it is.

> --- WHY --- THE TARGET ACCOUNT IS THE WHO LIST, OR THE CALLER'S OWN
> ACCOUNT WHEN THE WHO LIST IS EMPTY. a public key does not say whose
> key it is - the same key can be on ten accounts. the WHO list is the
> queue's own list of who is waiting for it, and it is the only thing
> that says whose account the key action is about. an agent that queues
> a key action for itself names nobody, and then the account the key
> belongs to is the account the api key belongs to.

> --- WHY --- A DELETE IS CHECKED AGAINST THE CHAIN, AT QUEUE TIME. the
> key's permission lives on chain, and only the chain knows it. a FULL
> access key is the user's wallet - deleting it locks them out - so the
> queue refuses to carry a delete for a key that is not a LIMITED one.
> a LIMITED key is one whose permission is `FunctionCall` or
> `GasKeyFunctionCall`: it may only call its receiver, and only its
> methods. `FullAccess` and `GasKeyFullAccess` are NOT limited and are
> refused, and so is a key the account does not hold at all.
>
> > --- WHY --- THE CHECK IS AGAINST EVERY NAMED ACCOUNT. a delete that
> > names several accounts has to hold on every one of them. a key that
> > is limited on alice and full access on bob is a full access key on
> > bob, and the batch is refused.
>
> > --- WHY --- THE CHECK RUNS BEFORE THE WRITE. a refused request never
> > leaves half of itself in the db.

> --- WHY --- THE RPC URL IS AN ENV VAR. the chain is read over json-rpc,
> and which node answers is a deployment's business. the variable is
> `SHIT_1170_NEAR_RPC_URL`, and it defaults to
> `https://rpc.mainnet.fastnear.com`.

### the answer

`201` with the queued batch:

```json
{
  "queueId": "8f14e45f-ceea-467a-9b0d-2b1c2e6c1a11",
  "what": "swap 100 USDC for NEAR on the ref finance dex",
  "when": "when the price is right",
  "why": "you said to move half your usdc into near",
  "whenAt": null,
  "transactions": [
    {
      "txId": "1a2b3c4d-0000-0000-0000-000000000001",
      "position": 0,
      "contractId": "ref.finance.near",
      "method": "ft_swap",
      "args": { "receiver_id": "alice.near" },
      "argsBase64": null,
      "gas": "30000000000000",
      "deposit": "1",
      "keyAction": null,
      "keyPublicKey": null,
      "keyReceiverId": null,
      "keyMethodNames": null,
      "keyAllowance": null,
      "accounts": [
        { "accountId": "alice.near", "status": "pending", "txHash": null },
        { "accountId": "bob.near", "status": "pending", "txHash": null }
      ]
    }
  ],
  "accounts": [
    { "accountId": "alice.near", "status": "pending", "txHash": null },
    { "accountId": "bob.near", "status": "pending", "txHash": null }
  ],
  "createdAt": 1772666000000,
  "updatedAt": 1772666000000
}
```

the queue's own `accounts` list is the WHO across the whole batch - the
union of the transactions' account rows, **deduped by account id**. a
batch of three calls has three account rows for alice, and the queue's
list holds ONE entry for alice, not three.

a transaction's `accounts` list is the same rows under that transaction,
with that transaction's status and hash for each.

> --- WHY --- the row that CARRIES A HASH wins in the queue's list. the
> hash is the fact - it means the account actually sent the
> transaction. a pending row under another transaction of the same batch
> is still the same account, and the sent one is the newer truth.

the answer is read back out of the db, not built from what was just
written. so it is the SAME shape whether it came from the queue route,
the one-batch route or the list route.

`argsBase64` is `null` for a call with json arguments, and it holds the
base64 for a call with raw bytes. the json `args` is the empty object
for such a call - the bytes have no names.

the five `key*` fields are **always** on the wire. they are `null` for
an ordinary contract call, and they carry the key action for a key
action transaction. a client reads one shape for every transaction, and
a field that is sometimes missing is a field every client has to check
for.

for a key action, `contractId` and `method` are the **EMPTY STRING** -
the db's columns are `NOT NULL` and nulling them would need a table
rebuild. the empty string means "this is not a contract call", and
`keyAction` says what it is.

### the errors

- `401` - `no_api_key` when the header is missing, or
  `unknown_api_key` when the key is not one of ours
- `400` - `no_transactions` when the batch was empty,
  `too_many_transactions` when it held more than fifty,
  `no_contract_id` or `no_method` when a call was missing one,
  `args_not_an_object` when the args were not a json object,
  `args_base64_not_base64` when the raw args were not base64,
  `args_and_args_base64` when a call carried both kinds of args, or
  `account_not_named_by_agent` when the `accounts` list named an
  account the agent holds **no api key for** - the NAMING rule
- `400` - the KEY ACTION errors:
  - `bad_key_action` when `keyAction` was neither `add_key` nor
    `delete_key`
  - `no_key_public_key` when a key action named no public key
  - `no_key_receiver_id` when an `add_key` named no receiver id
  - `key_action_and_contract_call` when a transaction was a key action
    AND a contract call at once
  - `key_action_not_allowed` when a `delete_key` named a key that is
    **not a limited key** on the account it is about - the message says
    whether the key was not there at all or was a full access key
  - `bad_key_public_key` when the public key was not a NEAR public key
  - `bad_account_id` when the target account was not a NEAR account id
- `503` - `chain_unreachable` when the chain could not be read, so the
  key could not be checked
- `500` - `db_unavailable`

> --- WHY --- a SHORTER answer from the naming check means one of the
> accounts was refused, and the WHOLE request is refused. naming one
> account you may not name would still put a tx in somebody's feed, so
> the batch is not written at all.

> --- WHY --- every call is checked BEFORE anything is written. a batch
> that is going to be refused never leaves half of itself in the db.

> --- WHY --- A CHAIN THAT CANNOT BE READ IS A 503 AND NOT A 400. the
> request was fine - the chain was not. and it is not a 500 either:
> nothing broke, the app just could not answer right now, and the
> client should come back.

> --- WHY --- `key_action_not_allowed` IS ONE CODE FOR BOTH REFUSALS -
> a key that is not there and a key that is full access. the MESSAGE
> says which one it was, because they are different facts and the user
> reads them differently. the code says the same thing to a client:
> this delete may not be queued.

## THE GET

`GET /api/shit-1170/tx-queue` - the queue list, filterable. it is
**OPEN** - it asks for NO api key.

### the query

```
GET /api/shit-1170/tx-queue?account_id=alice.near,bob.near&contract_id=ref.finance.near&hide_sent=1&limit=50
```

the filters are:

- `account_id` - only the queues these accounts are on. it is a **COMMA
  SEPARATED LIST**, so one value still works and many work too. it needs
  **NO API KEY**
- `hide_sent` - `1` drops the ones the reader has sent. it is about ONE
  account - the signed-in reader's OWN account, or the first account named
  in the query when the reader is not signed in
- `contract_id` - only the queues holding a tx for one of these
  contracts. it is a **COMMA SEPARATED LIST** too
- `method` - only the queues holding a tx for this method
- `limit` - default 50, and clamped to 500
- `cursor` - the opaque token of the page before. the FIRST page has
  none

**there is no `status` filter.** the status is per account and per
transaction, and a queue has no global status to filter on.

the query names are snake_case and are matched literally, so
`accountId` is NOT `account_id`.

`hide_sent` takes `1`, `true`, `yes` or `on` for on, and `0`, `false`,
`no` or `off` for off. anything else is read as "off" - it is a filter,
and a filter that is switched on by accident hides the rows the reader
came for.

**`hide_sent` without an account is a `400`.** a signed-in reader always
has their own account, so they are never refused - `?hide_sent=1` with a
valid api key works and filters by the reader's OWN account. a reader
with NO api key must name one in the query -
`?account_id=alice.near&hide_sent=1`.

> --- WHY --- the account and contract filters are COMMA SEPARATED and
> not repeated params. the query extractor reads ONE value per param,
> and the filter bar builds its query string out of a map - a map
> cannot hold the same param twice. one comma separated string carries
> many values, and it reads the same in a hand-written url.
>
> > --- WHY --- the many accounts are an OR. "alice's batches OR bob's
> > batches" is a question one id cannot answer, and the filter bar's
> > whole point is a reader who picked two accounts. a reader who picked
> > two wants the union, and the union is what an OR gives.

> --- WHY --- `hide_sent` needs an account. "has this account sent it" is
> a question about ONE account. a queue can be sent by alice and not by
> bob, so there is no single answer to "has this queue been sent". the
> filter therefore names the account it is about, and saying so is
> better than quietly ignoring it and returning the whole list.
>
> > --- WHY --- the account it is about is the READER'S OWN when they
> >
> > > are signed in, and the FIRST ACCOUNT NAMED in the query when they
> > > are not. a signed-in reader always has an account to be about, so
> > > they are never refused, and `?hide_sent=1` with a valid api key
> > > filters by their own account. a reader with NO key has no account
> > > of their own, so they must name one - and the first one they named
> > > is the account the filter is about.

> --- WHY --- the api does the filtering and not the frontend. the
> frontend would have to fetch every queue, read every account row, and
> work out which ones the reader has sent - and then throw most of them
> away. the api can do it in the query, so the reader gets the twenty
> they asked for and not two hundred they did not.

> --- WHY --- the list is OPEN. anybody can read it - a queue is an
> open offer, and hiding it would defeat the point. a reader who is not
> signed in can already read every batch and see every account on it -
> the WHO is right there on the card. so filtering that PUBLIC list by a
> name is not a leak, it is the filter the reader asked for, and the
> reader does not have to be signed in to use it.
>
> > --- WHY --- the api key is read here ONLY so the reader's OWN
> >
> > > account is known - the `hide_sent` filter is about the reader's
> > > own history. a missing or unknown key is NOT an error on this
> > > route: the reader gets the list, and the other routes are the
> > > ones that ask for the key.

### the answer

`200`:

```json
{
  "items": [ ...one batch each... ],
  "total": 1,
  "nextCursor": null
}
```

each item is the same shape as the `201` answer above.

**the `total` is the count of the WHOLE match, BEFORE the `limit` and
the `cursor` took their cut.** it is how many queues matched, and it
does NOT change as the reader pages - every page of the same filter
answers with the same `total`.

**the `nextCursor` is the cursor of the NEXT page, or `null` when this
page is the last one.** the caller hands it straight back as the
`cursor` of the next request. it is `null` when the page came back
short, because a short page IS the last page and a cursor for it would
send the reader off to look for rows that are not there.

### the cursor

the cursor is an **opaque token**. the caller does not read it, does
not build it and does not change it - the api hands it out and the api
takes it back. it says "the rows after THIS ONE" and it carries the
`created_at` and the `queue_id` of the last row of the page before.

**a cursor the api cannot read is a `400` with the code `bad_cursor`.**

> --- WHY --- a cursor and NOT an offset. an offset says "skip the first
> N rows", and the rows are ordered by `created_at DESC` - new batches
> land at the top of that order all the time. so the first N rows of
> page two are NOT the rows the reader already saw: the reader would
> see a batch twice, or skip one. a cursor says "the rows after this
> one", and that stays true whatever lands above it. a reader scrolling
> a feed of queued work must not see the same batch twice, and must not
> skip one either.
>
> > --- WHY --- the cursor carries the id and not just the time. many
> > batches can be queued in the same millisecond, and `created_at`
> > alone would leave the cursor ambiguous - the same millisecond read
> > twice or skipped. the `queue_id` breaks the tie, and the pair is
> > unique.
>
> > --- WHY --- a bad cursor is a 400 and NOT the first page. the caller
> > sent something the api does not understand, and answering with the
> > first page would show the reader the same rows twice and hide the
> > fact that their paging was broken.

### the errors

- `400` - `hide_sent_without_account` when `hide_sent` came without an
  account, or `bad_cursor` when the cursor could not be read
- `500` - `db_unavailable`, or `args_broken` when the arguments could
  not be read back

**there is no `401` on this route.** a reader with no api key at all
gets the list.

### the reads

the page's batches are read in ONE query, and their transactions and
their accounts are read in TWO more - for the WHOLE page at once.

> --- WHY --- the children are read for the WHOLE PAGE and not per
> batch. reading them per batch is two queries per batch, and a page
> of a hundred is two hundred queries - the single biggest cost in
> serving the feed, and it grows with the page. two queries for the
> whole page is the same cost whatever the page holds.

## THE FACETS

`GET /api/shit-1170/tx-queue/facets` - the contracts and the accounts
that have queued work.

see [`shit-1170-tx-queue-facets.md`](./shit-1170-tx-queue-facets.md)

## ONE BATCH

`GET /api/shit-1170/tx-queue/{queue_id}` - one queued batch.

the answer is the same shape as one item of the list: the queue, its
transactions in `position` order, and the WHO across the batch.

- `401` - `no_api_key` or `unknown_api_key`
- `404` - `no_such_queue` when there is no queue with that id
- `500` - `db_unavailable`, or `args_broken` when the arguments of the
  queue could not be read back

> --- WHY --- the key is checked FIRST. a caller who cannot prove who
> they are gets nothing, not even the 404 that says whether the queue
> exists.

## CLIENT

the typed client lives in `middleman/src/`, published as
`@huggies/shit-1170-rex-engine-middleman`:

- `api_shit_1170_tx_queue_post(base_url, body, options)`
- `api_shit_1170_tx_queue_get(base_url, query, options)`
- `api_shit_1170_tx_queue_one_get(base_url, queue_id, options)`
- `api_shit_1170_tx_queue_facets_get(base_url, query, options)`

the middleman checks the empty batch and the fifty cap before the round
trip, so a caller that goes over gets a clear message instead of a 400
from the wire.

=====================
<br/>
copyright 2026 by sleet.near
