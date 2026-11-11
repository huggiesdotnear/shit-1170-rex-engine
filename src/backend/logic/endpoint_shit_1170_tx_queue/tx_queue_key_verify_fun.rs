// ========================================
// TX_QUEUE_KEY_VERIFY_FUN
// check on the CHAIN that a key really is a LIMITED key on the account
// the key action names.
//
// this is the check that stops the queue from being a way to delete
// somebody's full access key. an agent that could queue "delete this
// key" for a key it named wrongly would take the user's wallet out of
// their hands — the key that signs their transactions is a FULL access
// key, and deleting it locks them out.
//
// --- WHY --- the check is against the chain and not against what the
// agent says: the agent says which key it wants gone, and the chain
// says what that key IS. the whole point is that the two have to agree.
//
// --- WHY --- NEAR-KIT DOES THE CALL AND NOT THIS FILE: the rpc call is
// a json-rpc round trip, and hand-rolling one would be a copy of
// near-kit's own that could drift. near-kit is already a dependency
// and its `view_access_key_list` is the reference read of an account's
// keys.
//
// --- WHY --- NEAR-KIT-Rs IS THE RUST TOOL BOX: the typescript app reads
// the chain through `near_kit_client()` from `@near-kit-tool-box/web`,
// which is a thin wrapper over the `near-kit` javascript package. the
// rust side has the same shape: there is a `near-kit-tool-box-rs` over
// on github, and it is a library of contract helpers over `near-kit`
// itself. the toolbox pins an older `near-kit`, and carrying two copies
// of the same library in one binary is a cost the one call this file
// makes does not justify — so the backend uses `near-kit` directly, the
// same library both tool boxes sit on.
//
// --- WHY --- THE CLIENT IS NEAR-KIT'S AND NOT A BARE RPC CLIENT: near-kit
// ships the rpc url for each network with the network itself
// (`Near::mainnet()`, `Near::testnet()`), so the backend does not carry
// an rpc url of its own and cannot be pointed at the wrong chain by a
// stale environment variable. the browser picks its network from the
// app's network toggle and near-kit supplies the url for it; the
// backend asks near-kit for the mainnet client and near-kit supplies
// the url for that.
//
// --- WHY --- ONLY THE DELETE IS CHECKED: `add_key` creates a key that
// does not exist yet, so there is nothing on chain to check — the key
// it names is the key the user is about to sign for. and an `add_key`
// cannot take anything away: the worst it can do is add a key the user
// then does not want, and the user approves it.
//
// --- WHY --- THE LATEST FINAL BLOCK IS READ: the answer has to be about
// the chain as it is NOW. an older block would answer about a key that
// has since been changed.
use near_kit::AccountId;
use near_kit::Near;
use near_kit::rpc::{AccessKeyListView, AccessKeyPermissionView, BlockReference};
use near_kit::signer::PublicKey;
// ========================================
// --- TYPES ------------------------------
// why a key action was refused.
//
// --- WHY --- it is an enum and not a string: the route has to say
// WHICH thing went wrong — a key that is not there and a key that is
// full access are different facts, and the user reads them differently.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TX_QUEUE_KEY_VERIFY_ERROR {
    // the account id is not a NEAR account id.
    BadAccountId,
    // the public key is not a NEAR public key.
    BadPublicKey,
    // the account could not be read from the chain.
    ChainUnreachable,
    // the account has no key with that public key.
    NoSuchKey,
    // the key is there, and it is a FULL access key.
    NotALimitedKey,
}
// ========================================
// --- VERIFY ----------------------------
// is this public key a LIMITED key on this account?
//
// --- WHY --- THE CHECK IS OVER EVERY NAMED ACCOUNT AND NOT ONE: a
// delete that names several accounts has to hold on every one of them.
// a key that is limited on alice and full access on bob is not a
// limited key — it is a full access key on bob — and the batch is
// refused. the route calls this once per account and refuses on the
// first refusal.
//
// --- WHY --- THE ACCOUNT LIST IS DEDUPED BY THE CALLER: the same
// account twice is one account, and checking it twice would cost two
// round trips for the same answer.
pub async fn tx_queue_key_verify_fun(
    account_id: &str,
    public_key: &str,
) -> Result<(), TX_QUEUE_KEY_VERIFY_ERROR> {
    // --- THE ACCOUNT ID ------------------
    let account = match account_id.trim().parse::<AccountId>() {
        Ok(account) => account,
        Err(_) => return Err(TX_QUEUE_KEY_VERIFY_ERROR::BadAccountId),
    };

    // --- THE PUBLIC KEY ------------------
    // --- WHY --- IT IS PARSED AND NOT COMPARED AS A STRING: the chain
    // may hand the key back in a form that is not byte-for-byte the
    // string the agent wrote, and the parsed key is the one thing both
    // sides agree on.
    let key = match public_key.trim().parse::<PublicKey>() {
        Ok(key) => key,
        Err(_) => return Err(TX_QUEUE_KEY_VERIFY_ERROR::BadPublicKey),
    };

    // --- THE CHAIN ----------------------
    // --- WHY --- NEAR-KIT'S OWN CLIENT AND NOT A RAW RPC CLIENT: near-kit
    // carries the rpc url for each network with the network, so the
    // backend has no url of its own to get wrong. and the client is
    // cheap to build — it holds a url and a transport, not a connection.
    let near = Near::mainnet().build();
    let list = match near
        .rpc()
        .view_access_key_list(&account, BlockReference::default())
        .await
    {
        Ok(list) => list,
        Err(_) => return Err(TX_QUEUE_KEY_VERIFY_ERROR::ChainUnreachable),
    };

    tx_queue_key_is_limited_fun(&list, &key)
}

// ========================================
// --- IS LIMITED ------------------------
// is this key in this list a LIMITED key?
//
// --- WHY --- THE ANSWER IS COMPUTED AND NOT FETCHED: the list is the
// chain's answer and the permission is in it, so the decision is a
// lookup and a match. splitting it out is what lets the decision be
// tested without a node.
//
// --- WHY --- ONLY THE TWO LIMITED PERMISSIONS PASS: a `FunctionCall`
// key and a `GasKeyFunctionCall` key are both limited — they can only
// call the named receiver, and only the named methods. a `FullAccess`
// key and a `GasKeyFullAccess` key can do anything the account can do,
// and deleting one of those is taking the user's wallet away.
pub fn tx_queue_key_is_limited_fun(
    list: &AccessKeyListView,
    key: &PublicKey,
) -> Result<(), TX_QUEUE_KEY_VERIFY_ERROR> {
    // --- THE KEY -----------------------
    let found = list.keys.iter().find(|one| one.public_key.refers_to(key));
    let found = match found {
        Some(found) => found,
        // --- WHY --- A KEY THAT IS NOT THERE IS REFUSED AND NOT
        // TREATED AS "ALREADY GONE": the queue is an open offer and the
        // user reads it before acting on it, so a delete of a key that
        // does not exist is a question about the chain that the chain
        // has already answered.
        None => return Err(TX_QUEUE_KEY_VERIFY_ERROR::NoSuchKey),
    };

    match &found.access_key.permission {
        AccessKeyPermissionView::FunctionCall { .. } => Ok(()),
        AccessKeyPermissionView::GasKeyFunctionCall { .. } => Ok(()),
        AccessKeyPermissionView::FullAccess => Err(TX_QUEUE_KEY_VERIFY_ERROR::NotALimitedKey),
        AccessKeyPermissionView::GasKeyFullAccess { .. } => {
            Err(TX_QUEUE_KEY_VERIFY_ERROR::NotALimitedKey)
        }
    }
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
