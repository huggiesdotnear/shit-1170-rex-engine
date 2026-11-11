// SHIT_1170_RPC_URL_FUN
// the near rpc url the backend reads the chain from.
//
// the backend needs the chain for ONE thing: checking that a key an
// agent wants to remove really IS a limited key on the account it names.
// the key's permission is not something the api can decide on its own —
// it lives on chain, and only the chain knows it.
//
// WHY THE DEFAULT IS THE MAINNET ENDPOINT AND NOT TESTNET: the keys
// the queue talks about are real NEAR accounts with real money on them,
// and a limited key on testnet is worth nothing. a deployment that
// serves testnet sets the variable.
//
// WHY THE DEPRECATED rpc.mainnet.near.org IS NOT THE DEFAULT: it
// answers 429 for every request, and a backend that cannot reach the
// chain cannot check a key.
//
// WHY THE URL IS READ EVERY TIME AND NOT CACHED: this is read on a
// request path that is rare, and reading it late means a container that
// is handed a new url on restart picks it up without a rebuild.
pub fn shit_1170_rpc_url_fun() -> String {
    std::env::var("SHIT_1170_NEAR_RPC_URL")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "https://rpc.mainnet.fastnear.com".to_string())
}
// ====================================
// copyright 2026 by sleet.near
// ====================================
