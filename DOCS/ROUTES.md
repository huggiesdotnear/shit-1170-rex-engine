# ROUTES

routes and pages

- `/` - serve the static folder

every route under `/api/` is prefixed with the api it fronts, so the
url alone says which backend the route belongs to:

- `/api/shit-1170/*` - our own routes, nothing is proxied
- `/api/trezu/*` - our proxy for the trezu api

a route with more than one bullet point gets its own file in
[`ROUTES/`](./ROUTES/).

### `/api/shit-1170/` - shit-1170 specifics

user info, api key managmnets and the tx queue

- `/api/shit-1170/ping` - liveness check, returns `{ "ok": true, "app": "shit-1170" }`

  the typed client is `api_shit_1170_ping_get(base_url)`. it is named
  after us because it is OUR route — nothing is proxied through it.

- `/api/shit-1170/api-keys` - turn signed messages into api keys.
  `POST`. the browser sends every account it holds a signed message for,
  the backend verifies each one, and the answer is the ONE api key the
  user has per account.

  see [`ROUTES/shit-1170-api-keys.md`](./ROUTES/shit-1170-api-keys.md)

- `/api/shit-1170/api-key/me` - who does this api key belong to.
  `GET`. the key travels in the `x-api-key` header.

  see [`ROUTES/shit-1170-api-key-me.md`](./ROUTES/shit-1170-api-key-me.md)

- `/api/shit-1170/api-key/rotate` - give an account a NEW api key.
  `POST`. the old key stops working at once.

  see [`ROUTES/shit-1170-api-key-rotate.md`](./ROUTES/shit-1170-api-key-rotate.md)

- `/api/shit-1170/feed/ws` - the LIVE feed. `GET`, a websocket upgrade.
  the server pushes every newly queued batch and every newly queued swap
  to every connected client, so the feed page does not have to poll.
  the message is `{ "type": "queue", "queue": {…} }` or
  `{ "type": "swap", "swap": {…} }`, and the payload is the SAME item
  the list route answers with. it is OPEN and asks for NO api key.

  see [`ROUTES/shit-1170-feed-ws.md`](./ROUTES/shit-1170-feed-ws.md)

- `/api/shit-1170/tx-queue` - the tx queue. `POST` queues a BATCH of
  transactions, `GET` is the filterable list of them. the list pages
  with an opaque cursor and filters by many accounts and many contracts
  at once. a queue is never done — the status lives on the account row.
  the `GET` list is OPEN and asks for NO api key.
  a queued transaction is EITHER a contract call OR a KEY ACTION —
  `add_key` or `delete_key` — so an agent can offer the user a limited
  key or take one away. a `delete_key` is checked AGAINST THE CHAIN at
  queue time: the queue refuses a delete for a key that is not a LIMITED
  key on the account it is about, because a full access key is the
  user's wallet.

  see [`ROUTES/shit-1170-tx-queue.md`](./ROUTES/shit-1170-tx-queue.md)

- `/api/shit-1170/tx-queue/facets` - the contracts and the accounts
  that have queued work. `GET`. it is what the feed's filter bar builds
  its buttons from, and it is OPEN.

  see [`ROUTES/shit-1170-tx-queue-facets.md`](./ROUTES/shit-1170-tx-queue-facets.md)

- `/api/shit-1170/tx-queue/{queue_id}` - one queued batch, with its
  transactions and its accounts. `GET`.

  see [`ROUTES/shit-1170-tx-queue.md`](./ROUTES/shit-1170-tx-queue.md)

- `/api/shit-1170/tx-queue/{queue_id}/accounts` - add YOUR OWN account
  to a queued batch. `POST`.

  see [`ROUTES/shit-1170-tx-queue-accounts.md`](./ROUTES/shit-1170-tx-queue-accounts.md)

- `/api/shit-1170/tx-queue/{queue_id}/accounts/{account_id}` - remove
  YOUR OWN account from a queued batch. `DELETE`.

  see [`ROUTES/shit-1170-tx-queue-accounts.md`](./ROUTES/shit-1170-tx-queue-accounts.md)

- `/api/shit-1170/tx-queue/{queue_id}/sent` - record that the
  transactions were sent — the hashes and the outcome. `POST`.

  see [`ROUTES/shit-1170-tx-queue-sent.md`](./ROUTES/shit-1170-tx-queue-sent.md)

- `/api/shit-1170/tx-queue/stats/contracts` - how many queued txs per
  contract. `GET`.

  see [`ROUTES/shit-1170-tx-queue-stats.md`](./ROUTES/shit-1170-tx-queue-stats.md)

- `/api/shit-1170/tx-queue/stats/methods` - how many queued txs per
  method. `GET`.

  see [`ROUTES/shit-1170-tx-queue-stats.md`](./ROUTES/shit-1170-tx-queue-stats.md)

- `/api/shit-1170/swap-queue` - the swap queue. `POST` queues a SWAP -
  the token in, the amount in and the token out, and NOT the route -
  and `GET` is the filterable list of them. the list pages with an
  opaque cursor and filters by many accounts at once and by many
  contracts at once - a swap matches the contract list when EITHER of
  its two tokens is one of them. a swap is never done, so the status
  lives on the account row. the `GET` list is OPEN and asks for NO api
  key.

  see [`ROUTES/shit-1170-swap-queue.md`](./ROUTES/shit-1170-swap-queue.md)

- `/api/shit-1170/swap-queue/{swap_id}` - one queued swap, with its
  accounts. `GET`.

  see [`ROUTES/shit-1170-swap-queue.md`](./ROUTES/shit-1170-swap-queue.md)

- `/api/shit-1170/swap-queue/{swap_id}/accounts` - add YOUR OWN account
  to a queued swap. `POST`.

  see [`ROUTES/shit-1170-swap-queue-accounts.md`](./ROUTES/shit-1170-swap-queue-accounts.md)

- `/api/shit-1170/swap-queue/{swap_id}/accounts/{account_id}` - remove
  YOUR OWN account from a queued swap. `DELETE`.

  see [`ROUTES/shit-1170-swap-queue-accounts.md`](./ROUTES/shit-1170-swap-queue-accounts.md)

- `/api/shit-1170/swap-queue/{swap_id}/sent` - record that the swap was
  sent - the hash, the outcome and the amount that came out. `POST`.
  this is the route the swap page calls after it executes a queued swap.

  see [`ROUTES/shit-1170-swap-queue-sent.md`](./ROUTES/shit-1170-swap-queue-sent.md)

the api key travels in the `x-api-key` header. the old `x-session-token`
header is still accepted during the beta — `x-api-key` is read FIRST and
the old name is the fallback.

### `/api/trezu/` - trezu specifics

- `/api/trezu/user/treasuries` - the treasuries (sputnik daos) an account
  belongs to. proxies `https://api.trezu.app/api/user/treasuries`.

  see [`ROUTES/trezu-treasuries.md`](./ROUTES/trezu-treasuries.md)

### intear

the intear api is NOT proxied through our backend. the
`@sleet-js/middleman-intear-api` package calls `https://api.intea.rs`
directly from the browser, so there are no `/api/intear/*` routes here
and our backend is never rate limited by intear.

the same goes for the two other public apis the app talks to — the
intear dex backend and the rhea indexer — each with its own published
middleman package:

- `@sleet-js/middleman-intear-api` — `https://api.intea.rs`
- `@sleet-js/middleman-intear-dex-api` — `https://dex-backend.intear.tech`
- `@sleet-js/middleman-rhea-api` — `https://api.rhea.finance`

=====================
<br/>
copyright 2026 by sleet.near
