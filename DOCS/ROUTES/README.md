# ROUTES

one file per route that needs more than a bullet point.

the parent [`ROUTES.md`](../ROUTES.md) is the index — every route is
listed there, and a route that needs its own detail gets a file here.

## what is where

a route is described by two things:

- `shit-1170-tx-queue.md` | one route that needs more than a bullet point
- [`routes.csv`](./routes.csv) | EVERY route at once — the method, the
  path, what it does, its auth, its success and its errors

the split is by granularity. the md file is the route. the csv is every
route at once, so a question like "which routes need an api key?" has
one answer, in one place, and no table is ever written into a markdown
file.

## THE FILES

- [`routes.csv`](./routes.csv) — every route: its method, its path, its
  auth, its success and its errors
- [`trezu-treasuries.md`](./trezu-treasuries.md) — the treasuries
  (sputnik daos) an account belongs to
- [`shit-1170-api-keys.md`](./shit-1170-api-keys.md) — turn signed
  messages into the user's one api key per account
- [`shit-1170-api-key-me.md`](./shit-1170-api-key-me.md) — who does
  this api key belong to, and the header it travels in
- [`shit-1170-api-key-rotate.md`](./shit-1170-api-key-rotate.md) — give
  an account a new api key
- [`shit-1170-feed-ws.md`](./shit-1170-feed-ws.md) — the live feed, a
  websocket that pushes every newly queued batch and swap
- [`shit-1170-tx-queue.md`](./shit-1170-tx-queue.md) — queue a batch of
  transactions, and the filterable list of them
- [`shit-1170-tx-queue-facets.md`](./shit-1170-tx-queue-facets.md) — the
  contracts and the accounts that have queued work
- [`shit-1170-tx-queue-accounts.md`](./shit-1170-tx-queue-accounts.md) —
  add or remove YOUR OWN account on a queued batch
- [`shit-1170-tx-queue-sent.md`](./shit-1170-tx-queue-sent.md) — record
  the hashes of the transactions that went out
- [`shit-1170-tx-queue-stats.md`](./shit-1170-tx-queue-stats.md) — how
  many queued txs per contract and per method
- [`shit-1170-swap-queue.md`](./shit-1170-swap-queue.md) — queue a swap,
  and the filterable list of them
- [`shit-1170-swap-queue-accounts.md`](./shit-1170-swap-queue-accounts.md) —
  add or remove YOUR OWN account on a queued swap
- [`shit-1170-swap-queue-sent.md`](./shit-1170-swap-queue-sent.md) —
  record that the swap went out — the hash, the outcome and the amount
  that came out

=====================
<br/>
copyright 2026 by sleet.near
