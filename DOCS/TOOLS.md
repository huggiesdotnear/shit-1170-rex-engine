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

the backend reads the chain for ONE thing: checking that a key an agent
wants to remove really IS a limited key on the account it names. the
key's permission lives on chain, and only the chain knows it.

it reads the chain through **near-kit**, the same toolkit the browser
reads it through, and near-kit carries the rpc url for each network
with the network itself. so there is no url to configure and no env var
to get wrong — the backend asks near-kit for the client it wants and
near-kit knows where to send it.

the frontend reads the chain through `near_kit_client()` from
`@near-kit-tool-box/web`, which is a thin wrapper over the `near-kit`
javascript package.

the rust side has the same shape: `near-kit-tool-box-rs` over on
github is a library of contract helpers over `near-kit`. it pins an
older `near-kit`, and carrying two copies of the same library in one
binary is a cost the backend's single chain call does not justify — so
the backend uses `near-kit` directly, the same library both tool boxes
sit on.

=====================
<br/>
copyright 2026 by sleet.near
