# @huggies/shit-1170-rex-engine-middleman

clean ts library for interacting with the shit-1170-rex-engine backend routes.

```sh
bun i
# bun run build
bun run build
# publish
bun publish --dry-run
bunx npm login
bun publish --access public
# other
bun run typecheck
bunx prettier . --write
```

---

shit-1170-rex-engine
<br/>
https://shit-1170.com

the base url is a parameter on every call: the frontend passes the
origin of the page it is served from, a bin passes whatever backend it
was told to talk to (`--base-url`).

the routes covered here are the ones this backend owns:

- `/api/shit-1170/*` - our own routes, nothing is proxied
- `/api/trezu/*` - our proxy for the trezu api

## THE LIVE FEED

`/api/shit-1170/feed/ws` is a WEBSOCKET UPGRADE, not a request and a
response, so it does not go through `api_call`. it has its own client:

```ts
import { api_shit_1170_feed_ws } from "@huggies/shit-1170-rex-engine-middleman";

const handle = api_shit_1170_feed_ws(
  "http://127.0.0.1:54733",
  (event) => {
    // event.type is "queue" or "swap"
    if (event.type === "queue") console.log(event.queue.queueId);
    if (event.type === "swap") console.log(event.swap.swapId);
  },
  (message) => console.error(message),
);

// later — close the socket and stop the reconnects
handle.close();
```

the base url is turned into a socket url — `http` becomes `ws`, `https`
becomes `wss` — and the route is appended. the socket is OPEN: it needs
no api key.

the handle reconnects on its own when the socket drops, waiting one
second, then two, then four, up to fifteen. `close()` stops that for
good.

the third-party apis the app talks to are NOT here. the intear api, the
intear dex backend and the rhea indexer each ship their own published
middleman package, so there is one client per api and one place to keep
it:

- `@sleet-js/middleman-intear-api` - `https://api.intea.rs`
- `@sleet-js/middleman-intear-dex-api` - `https://dex-backend.intear.tech`
- `@sleet-js/middleman-rhea-api` - `https://api.rhea.finance`

## THE TOOL PRIMITIVES

the middleman logs every request and response with `console.log`, which
lands on STDOUT. a tool that promises pure JSON on stdout has to move
that banner, so the parsing and the printing live here too — next to
the logging that makes them necessary.

```ts
import {
  parse_args_fun,
  args_string_fun,
  args_require_string_fun,
  args_bool_fun,
  args_number_fun,
  quiet_fun,
  print_json_fun,
  print_error_fun,
  queue_call_fun,
} from "@huggies/shit-1170-rex-engine-middleman";

const args = parse_args_fun(process.argv.slice(2), {
  boolean_flags: ["hide-sent"],
});

const queue = args_require_string_fun(args, "queue");
const limit = args_number_fun(args, "limit", 20);
const hide_sent = args_bool_fun(args, "hide-sent");

const answer = await quiet_fun(() =>
  api_shit_1170_tx_queue_get(base_url, { limit }),
);
print_json_fun({ ok: true, answer });
```

a flag that takes a value and does not get one is an ERROR, not an
empty string — `--limit` with nothing after it is a typo, and a bin
that read it as "no limit" would quietly do the wrong thing. a boolean
flag is declared per bin, so it never swallows the next positional.

`quiet_fun` moves the banner to stderr for the length of the call and
puts `console.log` back afterwards, even when the call throws.

## THE QUEUE PAYLOAD

`queue_call_fun` builds the shape the engine's tx-queue accepts. it
builds DATA — the engine holds it for a human to sign, and nothing is
sent.

```ts
const call = queue_call_fun({
  contractId: "usdt.tether-token.near",
  method: "ft_transfer",
  args: { receiver_id: "bob.near", amount: "25000000" },
  gas: "30000000000000",
  deposit: "1",
});
```

the gas and the deposit are raw digit strings — gas units and
yoctoNEAR. the web app also accepts the human forms ("30 Tgas"), but
raw digits are the unambiguous form.

a call whose contract takes RAW BYTES instead of json arguments — the
FastFS protocol (`__fastdata_fastfs`) is one — carries them as
`args_base64` (standard base64) instead of `args`:

```ts
const call = queue_call_fun({
  contractId: "fastfs.near",
  method: "__fastdata_fastfs",
  args_base64: "AAwAAABmYXN0bmVhci5wbmc=",
});
```

a call carries one or the other, never both.

the builder refuses what it cannot trust: an empty contract id, an
empty method, args that are not a plain object, an args_base64 that is
not base64, both kinds of args at once, and a gas or deposit that is
not raw digits. the engine records whatever it is given, so the check
belongs where the payload is built.

=====================
<br/>
copyright 2026 by sleet.near
