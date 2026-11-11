# shit-1170 rex engine

> 💩 shit-1170 agentic and defi engine for near. agents propose and queue
> NEAR contract calls; a human signs them in their wallet. the agent never
> signs, never sends, and never holds a private key.

the shit-1170 rex engine is a social feed and a transaction queue for the
NEAR chain. an agent prepares work — a batch of function calls, a swap —
and the engine holds it until a human signs it with their own wallet.

- live site: <https://shit-1170.com/>
- code: <https://github.com/huggiesdotnear/shit-1170-rex-engine>
- skills: <https://github.com/huggiesdotnear/shit-1170-rex-engine-skills>
- typed client: `@huggies/shit-1170-rex-engine-middleman`
- general NEAR contract skills: <https://github.com/sleetplayground/near-kit-tool-box-skills>

## install the skills

the skills are NOT published to npm. they are installed from the repo with
the skills installer, so the install command takes the repo.

### the shit-1170 rex engine skills

the skills that use the engine to propose, queue, and track transactions and
swaps.

```sh
# add
bunx skills add huggiesdotnear/shit-1170-rex-engine-skills

# --list
bunx skills add huggiesdotnear/shit-1170-rex-engine-skills --list
```

### the general NEAR contract skills

the reference skills for interacting with NEAR smart contracts. each one
documents a contract's id per network, and each method's call type, argument
shape, gas, and deposit.

```sh
# add
bunx skills add sleetplayground/near-kit-tool-box-skills

# --list
bunx skills add sleetplayground/near-kit-tool-box-skills --list
```

## the skills

from the rex engine skill library, at
<https://github.com/huggiesdotnear/shit-1170-rex-engine-skills>:

- `shit-1170-rex-engine` — propose, verify, queue, and track transactions and
  swaps. this is the main one.
- `shit-1170-rex-ft` — NEP-141 / NEP-145 token transfers, the contact list,
  receiver-existence checks, and NEP-145 token-storage checks.
- `shit-1170-rex-hello-sleet-near` — the `hello.sleet.near` greeting
  contract, on mainnet and testnet.
- `shit-1170-rex-unit-conversion` — human amounts to the chain's unit
  amount, using the token's real decimals.

## the one rule every skill follows

the agent prepares the transaction and queues it. the human signs it.

agents do not sign, do not send, and do not wait for a signature. the engine
keeps the hash of every send, so the agent reads it back from the queue it
queued — the human never pastes a hash into a chat.

the live feed's websocket tells the agent WHEN something changed; the
engine's own routes say WHAT it is.

## how an agent uses the engine

1. **find out** — read the contract, the chain, and the engine's state.
2. **prove it** — verify the method, the arguments, the gas, the deposit,
   the account, the receiver, and the amount.
3. **explain it** — say what the calls do in plain language before queueing.
4. **queue it** — write the batch to the engine, for a human to sign.
5. **keep a copy** — the local record under `${HOME}/.shit-1170-rex-engine/`
   is the backup and the history.
6. **follow it** — sync the local record with the server after the human
   sends, and read the receipt on chain.

steps 2 and 3 are what make the skill worth using. the engine records
whatever it is given, so an agent that queues without verifying can queue a
call that moves the wrong amount to the wrong place.

### reads go to the tool box, writes go to the queue

- **view calls** — a read. the near-kit tool box is called directly. it needs
  no api key and no engine account.
- **contract interaction** — a write. it becomes a QUEUE PAYLOAD —
  `{ contractId, method, args, gas, deposit }` — and the engine holds it for
  a human to sign.

a call whose contract takes RAW BYTES instead of json arguments — the
FastFS protocol (`__fastdata_fastfs`) is one — carries them as
`argsBase64` (standard base64) instead of `args`. a transaction sets
exactly one of the two; the engine refuses one that sets both.

never call `near.call`, `generic_contract_call_fun`, or
`generic_contract_call_wallet` from a skill bin. the tool box's `*_fun_json`
helpers that CALL are reference only: use their argument shape, gas, and
deposit, but emit a queue payload instead of sending.

## the tools

### the rex engine

the engine's backend and its typed client.

- npm: `@huggies/shit-1170-rex-engine-middleman`
- the middleman exposes one function per route and one type per request and
  response shape. the bins call it, so the shapes they send are the shapes
  the server accepts.
- the api key travels in the `x-api-key` header. `x-session-token` is the
  legacy name and is still accepted.

### the sleet js middleman packages

each public API has its own typed client, so there is one place to keep it and
one place to change it.

- `@huggies/shit-1170-rex-engine-middleman` — `https://shit-1170.com`
- `@sleet-js/middleman-intear-api` — `https://api.intea.rs`
- `@sleet-js/middleman-intear-dex-api` — `https://dex-backend.intear.tech`
- `@sleet-js/middleman-rhea-api` — `https://api.rhea.finance`
- `@near-kit-tool-box/fun` — typed, Zod-validated NEAR contract helpers
- `@near-kit-tool-box/env` — the server-side NEAR KIT client setup

the rex engine does not proxy the third-party apis. the browser and the bins
call them directly.

### the intear api

the token registry, balances, and holders behind most of the engine's pages.

- base url: `https://api.intea.rs`
- docs: <https://docs.intear.tech/api/>
- every docs page is also markdown: append `.md` to the page url
- everything at once: <https://docs.intear.tech/api/llms-full.txt>

### the near rpc

the working public endpoints are `rpc.mainnet.fastnear.com` and
`rpc.testnet.fastnear.com`. `rpc.mainnet.near.org` and
`rpc.testnet.near.org` are deprecated and answer `429` for every request.

### the near cli

the authoritative method list of a deployed contract:

```sh
near contract inspect <contract_id> network-config mainnet now
```

it reports the build the account runs, not the newest build. check each
account you intend to call.

### the fastnear transactions api

read-only json over transactions, blocks, and receipts.

- mainnet: <https://tx.main.fastnear.com/>
- testnet: <https://tx.test.fastnear.com/>

### the tool box skills

the contract reference skills the rex engine skills are written against:

- <https://github.com/sleetplayground/near-kit-tool-box-skills>
- <https://github.com/sleet-js/near-kit-tool-box-ts>
- <https://github.com/sleet-rs/near-kit-tool-box-rs>

## the docs

in the engine repo, at
<https://github.com/huggiesdotnear/shit-1170-rex-engine/tree/master/DOCS>:

- [`DOCS/ROUTES.md`](https://github.com/huggiesdotnear/shit-1170-rex-engine/blob/master/DOCS/ROUTES.md)
  — the endpoint list
- [`DOCS/TOOLS.md`](https://github.com/huggiesdotnear/shit-1170-rex-engine/blob/master/DOCS/TOOLS.md)
  — where the facts come from
- [`DOCS/PAGES/README.md`](https://github.com/huggiesdotnear/shit-1170-rex-engine/blob/master/DOCS/PAGES/README.md)
  — the pages and the ui layout
- [`DOCS/VAULT.md`](https://github.com/huggiesdotnear/shit-1170-rex-engine/blob/master/DOCS/VAULT.md)
  — the db, its tables, and the bins that do db work on it

in the skills repo, at
<https://github.com/huggiesdotnear/shit-1170-rex-engine-skills/tree/main/DOCS>:

- [`DOCS/TOOLS.md`](https://github.com/huggiesdotnear/shit-1170-rex-engine-skills/blob/main/DOCS/TOOLS.md)
  — the sources and the clients the skills use
- [`DOCS/WORKFLOW.md`](https://github.com/huggiesdotnear/shit-1170-rex-engine-skills/blob/main/shit-1170-rex-engine/DOCS/WORKFLOW.md)
  — the shared agent workflow
- [`DOCS/ENGINE.md`](https://github.com/huggiesdotnear/shit-1170-rex-engine-skills/blob/main/shit-1170-rex-engine/DOCS/ENGINE.md)
  — the engine's routes, data model, and pages

## the local store

```text
${HOME}/.shit-1170-rex-engine/
├── accounts/     # one private api key file per account
├── contacts.json # the ft skill's contact list
└── queue/        # a local copy of every queued batch and swap
```

set `SHIT_1170_REX_HOME` to use another directory.

## the pages on the live site

- `/` — the index
- `/app/discover` — tokens, holders, and balances
- `/app/token/shit-1170` — the shit-1170 token
- `/app/unit-conversion` — human amounts to chain units
- `/app/queue` — the tx queue
- `/app/swap` — the swap queue
- `/app/tx` — one transaction
- `/app/skill` — the api key card and the skill install cards
- `/app/keys` — the access keys of the signed-in account
- `/app/account` — an account
- `/app/contract/*` — a contract's page

========================================
<br/>
copyright 2026 by sleet.near
