# VAULT

the backend db storage location will be hardcoded to this location for simplicity.

LOCATION: `${HOME}/.shit-1170-rex-engine/shit-1170-rex-engine.db`

that will be the default location that we use to store the sqlite db file.

can be overridden with a an `SHIT_1170_VAULT_DB_LOCATION` env but the defualt is not to use that env or set that.

we will mount this location in docker.

NEVER wipe or delete this db.

## WHAT IS IN IT

the db holds SIX tables of its own:

- `api_keys` — the api keys the users hold. one row per ACCOUNT, and an
  account has exactly one key, ever.
- `tx_queue` — one row per queued BATCH of transactions. it holds the
  WHAT, the WHEN, the WHY and the WHEN_AT of the batch.
- `tx_queue_transactions` — one row per transaction inside a batch. it
  holds the HOW: the contract, the method, the args, the gas and the
  deposit, and the `position` that says the order. a transaction may
  also be a KEY ACTION — `add_key` or `delete_key` — and then the five
  `key_*` columns say which key it acts on.
- `tx_queue_accounts` — one row per account, per transaction. this is
  the WHO of a batch, and it is the ONLY place a status lives.
- `swap_queue` — one row per queued SWAP. it holds the token in, the
  amount in, the token out and the quote that was taken when it was
  queued.
- `swap_queue_accounts` — one row per account, per queued swap. the
  WHO of a swap, and the ONLY place a status lives there too.

the tables were once ONE table called `sessions`, and the api keys were
called session keys. the credential in them is not a browser session: the
user signs in once and hands the key to an agent, and the agent holds it
as a bearer token for as long as it lives. a "session" describes the
browser's short use of it; an "api key" describes what it actually is.

the column-by-column shape of every table is in
[`DB/tables.csv`](./DB/tables.csv), and the indexes are in
[`DB/indexes.csv`](./DB/indexes.csv). the markdown holds the prose and
the csvs hold the field lists.

## THE THREE LEVELS OF A QUEUE

a queued batch is three levels deep:

```
a QUEUE  (queue_id)          what / when / why
  └── a TRANSACTION          contract_id / method / args / gas / deposit
        └── an ACCOUNT ROW    account_id / status / tx_hash
```

a transaction is EITHER a contract call OR a key action. a key action
has no contract and no method, so its `contract_id` and `method` are the
EMPTY STRING — the columns are `NOT NULL` and nulling them would need a
table rebuild. the `key_action` column is what says which kind of row
it is.

a queue is a BATCH, not one transaction. the agent queues a swap, and
that swap needs three function calls — a deposit, the swap itself, and a
withdrawal — so they go in together under one `queue_id`, and the user
signs them as one batch, in one wallet popup.

## WHERE THE STATUS LIVES

the status lives on the account row and NOWHERE ELSE. there is no
status column on `tx_queue` and no status column on
`tx_queue_transactions`.

it is the one place where the truth is. a transaction is not "sent" or
"failed" on its own — it is sent BY an account, and each account's send
is its own transaction with its own hash and its own outcome. two
accounts sending the same queued transaction produce two different
on-chain transactions, and either can succeed while the other fails.

and there is no queue-level status because **a queue is never done**. at
any moment another account can decide they want to send it, and they
join. a "is this queue done" badge would be a lie the moment somebody
new signs it.

the four states are `pending`, `sent`, `succeeded` and `failed`.

## THE ACCOUNTS TABLE IS THE JOIN

the account row hangs off the TRANSACTION and not off the QUEUE. a queue
has three transactions, the user signs all three together, and EACH
transaction produces its OWN hash. the hash cannot live on the account
row of the queue, or three hashes would collide in one column.

the account row's primary key is `(tx_id, account_id)` — the row is
about one transaction. the `queue_id` is kept as a column anyway, so a
query for "every account on this queue" is one indexed read with no join
through the transactions table.

## THE API KEYS TABLE

one row per ACCOUNT and not per person. an account has exactly one api
key, so the account id IS the row's identity — there is no linking
between accounts and no list to keep in agreement.

the key is stored as it is, and its sha-256 is stored next to it. a
request is looked up by the hash, so the read path never compares
against every row's key.

the `api_key_hash` index is UNIQUE. every authenticated request looks a
key up by its hash, and two rows with the same hash would mean two
accounts behind one credential. the uniqueness is the security property,
and the index is what makes the lookup cheap.

## THE SCHEMA

the schema lives in `src/backend/logic/db/`, one file per table, and
`db_schema_run_fun.rs` runs them all in order in one transaction.

**there is no migration system.** a migration system exists so a db's
schema can be CHANGED over time: each change is a numbered step, the db
records which steps it has run, and a fresh db runs them all in order.
this db has six tables and six settled shapes for them, so there is
nothing to step through and nothing to remember.

a changed shape means changing the table file. the db is ours and it
holds nothing that cannot be made again.

**the bins are where the CHANGING happens.** the app never has to
handle a migration, because a change to the db's contents is a job for
a bin and not for the running app. the app opens the db, reads it and
writes to it, and it knows the shape it expects. when the data needs
moving — the rows are old, the queue is too big, a vault has to move
from one machine to another — that is a bin's job, run by a person, on
a db that is not being written to at the time.

the tables are created in DEPENDENCY ORDER: `tx_queue` before
`tx_queue_transactions` before `tx_queue_accounts`.

## IN DOCKER

the db lives in the named volume `shit-1170-rex-engine-production-vault`,
mounted at `/home/shit-1170/.shit-1170-rex-engine` — the app's default path
inside the image, since `HOME` is `/home/shit-1170` there.

the volume is declared in `docker-compose.yml` with an explicit `name:`,
so it resolves to ONE fixed name on the host no matter what the compose
project is called. the pin is what keeps every api key alive across a
rename of the project, the service or the app.

`docker compose down` keeps the volume. `docker compose down -v`
DELETES it, and with it every api key and every queued batch.

## TESTING AGAINST A LOCAL VAULT

**never test a feature against the production vault.** the production
vault holds real api keys and real queued batches for real people. a
test that queues a swap, signs it, opens a websocket and watches the
feed move will write rows into it, and a test that prunes will take
them away.

so every test — a queueing test, a websocket test, a feed test, a
prune test, anything — runs against a LOCAL vault. a local vault is
just a db file somewhere on the machine, and the app can be pointed at
it with the env var the app already reads:

```sh
SHIT_1170_VAULT_DB_LOCATION=/tmp/shit-1170-test/vault.db cargo run
```

the app creates the file and the schema on first start. the local
vault is throwaway: delete it, and the next start makes a new one.

**a local vault can be a COPY of the production one.** a copy is the
right thing to test against when the thing being tested depends on the
SHAPE of the data — a queue with three transactions, an account that
has already sent, a swap whose quote is stale. `vault-export` writes
the production vault out as json and `vault-import` brings it into a
local db. the api keys are deliberately left out of that round trip:
the accounts come back and the user signs in again.

**the bins take the db path as a flag, and that is the point.** the
production db is on a docker volume and the local one is in a temp
folder, and the bins do not care which because they are told:

```sh
# the local one
./target/debug/vault_list --db /tmp/shit-1170-test/vault.db

# the production one, inside the container
docker compose -f DOCKER/docker-compose.yml exec \
  shit-1170-rex-engine-backend \
  vault-prune --db /home/shit-1170/.shit-1170-rex-engine/shit-1170-rex-engine.db --older-than 30
```

**always take a backup before a prune on the production vault.** the
backup is safe to take while the app is running, because it is taken
with sqlite's own backup and not with a file copy.

## THE VAULT BINS

the bins are the DEFAULT way to do db work on a vault. the app has no
migration system and no admin routes that wipe the queue, so anything
that changes the db from outside the app goes through a bin. that way
the rules of the db live in one place, the app never has to know that
the bins exist, and the app never has to handle migration logic.

the bins live in `src/bin/` and their logic lives in
`src/backend/logic/db/bin/`.

| bin            | what it does                                                                       |
| -------------- | ---------------------------------------------------------------------------------- |
| `vault-list`   | shows what is in a vault: the queues, the swaps and the row counts. it only reads. |
| `vault-prune`  | takes old queued batches and swaps off a vault, by age, by never-sent, or by id.   |
| `vault-backup` | takes a safe copy of a vault, with sqlite's own backup.                            |
| `vault-export` | writes a vault out as json.                                                        |
| `vault-import` | brings an export back into a vault.                                                |

every bin takes `--db <path>`. the flag wins over
`SHIT_1170_VAULT_DB_LOCATION`, and that wins over the default path —
the same order the app uses. so a bin with no flag on the same machine
as the app reads the app's db, and a bin with `--db` reads exactly the
file it was told to.

### THE BINS ARE SAFE BY DEFAULT

**`vault-prune` and `vault-import` are a DRY RUN until `--confirm` is
given.** they print the plan — which rows, how many, which tables' rows
go with them — and then stop. there is no undo, so the plan is the
whole point.

**a prune leaves the history alone.** a row that carries a `tx_hash` is
HISTORY: it shows who did the transaction and it stays. so a queue that
any account has sent does not match a prune at all. `--include-sent`
says "take that history too", and it is the only way to do it.

**the bins do not create a db.** a bin pointed at a path with no db
there says so and stops. it does not make an empty db where the real
one should have been, and it does not run the schema creates against a
vault it was asked to look at.

**the api keys do not travel.** `vault-export` leaves them out unless
`--with-api-keys` is given, and `vault-import` never brings them back
in — a key that has been in a file has been in the clear, so the
accounts come back and the user signs in again.

### THE PRUNE FILTERS

```
--older-than <days>   only the rows queued more than <days> ago
--before <epoch-ms>   only the rows queued before that moment
--after <epoch-ms>    only the rows queued after that moment
--never-sent          only the rows no account ever sent
--queue-ids <a,b,c>   only these queue ids
--swap-ids <a,b,c>    only these swap ids
--account <account>   only the rows this account is on
--include-sent        also take the rows that were sent
--swaps               also take the queued swaps
--confirm             actually delete
```

the swaps are only part of the plan when `--swaps` or `--swap-ids` is
given. a prune that names a queue id is about that queue, and taking
the swaps too would mean the prune took something it was not asked
for.

deleting a queue takes its transactions and their account rows with it,
in one statement — the schema's own `ON DELETE CASCADE` does that
work. the same goes for a swap and its account rows.

=====================
<br/>
copyright 2026 by sleet.near
