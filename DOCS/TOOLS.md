# TOOLS

Intear API
<br/>
https://docs.intear.tech/api/

the base url is `https://api.intea.rs`, no api key. every page of the
docs is also markdown, at its url with `.md` in place of the trailing
slash — <https://docs.intear.tech/api/llms-full.txt> has all of them in
one file.

NEAR RPC
<br/>

the backend reads the chain over json-rpc, and it needs it for ONE
thing: checking that a key an agent wants to remove really IS a limited
key on the account it names. the key's permission lives on chain, and
only the chain knows it.

the url is the `SHIT_1170_NEAR_RPC_URL` env var, and it defaults to
`https://rpc.mainnet.fastnear.com`. a deployment that serves testnet
sets `https://rpc.testnet.fastnear.com`.

the deprecated `rpc.mainnet.near.org` and `rpc.testnet.near.org` answer
`429` for every request, and they are not the default.

=====================
<br/>
copyright 2026 by sleet.near
