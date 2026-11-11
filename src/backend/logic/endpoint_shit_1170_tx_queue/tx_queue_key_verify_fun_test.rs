// ========================================
// the LIMITED KEY check, tested against the shape the chain answers.
//
// --- WHY --- THE LIST IS BUILT FROM JSON AND NOT FIELD BY FIELD: the
// chain delivers the access key list as json, and the type deserializes
// from it. building the fixture the same way means the test walks the
// same path a real answer walks, and a change in the wire shape breaks
// the test instead of silently making it meaningless.
//
// --- WHY --- THERE IS NO NODE IN THIS FILE: the decision — "is this key
// limited" — is a lookup and a match, and it is decided on what the
// chain said. testing the decision needs the chain's ANSWER, not the
// chain.
use near_kit::rpc::{AccessKeyListView, AccessKeyPermissionView};
use near_kit::signer::PublicKey;

use crate::backend::logic::endpoint_shit_1170_tx_queue::tx_queue_key_verify_fun::{
    TX_QUEUE_KEY_VERIFY_ERROR, tx_queue_key_is_limited_fun,
};
// ========================================
// --- FIXTURE ---------------------------
// the key the tests are about.
const FIXTURE_KEY: &str = "ed25519:FrtUfbY74kXpBiWmqeQMtn7jKCX3zJhmwyg32ZqANj3c";

// the chain's answer, with one key in it.
//
// --- WHY --- THE BLOCK FIELDS ARE FILLED IN: the type carries them and
// a real answer has them, so the fixture looks like the real thing.
//
// --- WHY --- THE BLOCK HASH IS BASE58: the chain sends the hash as
// base58, and the type reads it as base58. a hex string would not
// deserialize — and the fixture would fail for a reason that has
// nothing to do with the rule under test.
fn list_with_permission_fun(permission: serde_json::Value) -> AccessKeyListView {
    let json = serde_json::json!({
        "keys": [ {
            "public_key": FIXTURE_KEY,
            "access_key": { "nonce": 1, "permission": permission }
        } ],
        "last_key": null,
        "block_height": 100,
        "block_hash": "11111111111111111111111111111111"
    });

    serde_json::from_value(json).expect("the fixture should deserialize")
}

// the fixture's key, parsed.
fn fixture_key_fun() -> PublicKey {
    FIXTURE_KEY.parse().expect("the key should parse")
}
// ========================================
// --- TESTS -----------------------------
#[test]
fn a_function_call_key_is_a_limited_key() {
    // --- WHY --- A FUNCTION CALL KEY IS THE LIMITED ONE: it may only
    // call the named receiver, and only the named methods.
    let list = list_with_permission_fun(serde_json::json!({
        "FunctionCall": {
            "allowance": "1000000000000000000000000",
            "receiver_id": "ref.finance.near",
            "method_names": [ "ft_swap" ]
        }
    }));

    assert_eq!(tx_queue_key_is_limited_fun(&list, &fixture_key_fun()), Ok(()));
}

#[test]
fn a_function_call_key_with_no_allowance_is_still_a_limited_key() {
    // --- WHY --- NO ALLOWANCE IS NOT NO LIMIT: a function call key with
    // no allowance may still call its receiver, and the wallet covers
    // the cost. it is still limited.
    let list = list_with_permission_fun(serde_json::json!({
        "FunctionCall": {
            "allowance": null,
            "receiver_id": "ref.finance.near",
            "method_names": []
        }
    }));

    assert_eq!(tx_queue_key_is_limited_fun(&list, &fixture_key_fun()), Ok(()));
}

#[test]
fn a_gas_key_function_call_key_is_a_limited_key() {
    // --- WHY --- A GAS KEY WITH A FUNCTION CALL PERMISSION IS LIMITED
    // TOO: it may only call the named receiver and the named methods.
    let list = list_with_permission_fun(serde_json::json!({
        "GasKeyFunctionCall": {
            "balance": "1000000000000000000000000",
            "num_nonces": 10,
            "allowance": null,
            "receiver_id": "ref.finance.near",
            "method_names": [ "ft_swap" ]
        }
    }));

    assert_eq!(tx_queue_key_is_limited_fun(&list, &fixture_key_fun()), Ok(()));
}

#[test]
fn a_full_access_key_is_not_a_limited_key() {
    // --- WHY --- THIS IS THE WHOLE POINT OF THE CHECK: a full access
    // key is the user's wallet. deleting it takes the wallet away.
    let list = list_with_permission_fun(serde_json::json!("FullAccess"));

    assert_eq!(
        tx_queue_key_is_limited_fun(&list, &fixture_key_fun()),
        Err(TX_QUEUE_KEY_VERIFY_ERROR::NotALimitedKey)
    );
}

#[test]
fn a_gas_key_with_full_access_is_not_a_limited_key() {
    let list = list_with_permission_fun(serde_json::json!({
        "GasKeyFullAccess": {
            "balance": "1000000000000000000000000",
            "num_nonces": 10
        }
    }));

    assert_eq!(
        tx_queue_key_is_limited_fun(&list, &fixture_key_fun()),
        Err(TX_QUEUE_KEY_VERIFY_ERROR::NotALimitedKey)
    );
}

#[test]
fn a_key_that_is_not_on_the_account_is_refused() {
    // --- WHY --- A KEY THAT IS NOT THERE IS NOT A LIMITED KEY: there is
    // nothing to say about a key the account does not hold.
    let list = list_with_permission_fun(serde_json::json!({
        "FunctionCall": {
            "allowance": null,
            "receiver_id": "ref.finance.near",
            "method_names": []
        }
    }));

    // a DIFFERENT key — one the account does not hold.
    let other: PublicKey = "ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp"
        .parse()
        .expect("the key should parse");

    assert_eq!(
        tx_queue_key_is_limited_fun(&list, &other),
        Err(TX_QUEUE_KEY_VERIFY_ERROR::NoSuchKey)
    );
}

#[test]
fn an_empty_list_refuses_every_key() {
    let list: AccessKeyListView = serde_json::from_value(serde_json::json!({
        "keys": [],
        "last_key": null,
        "block_height": 100,
        "block_hash": "11111111111111111111111111111111"
    }))
    .expect("the fixture should deserialize");

    assert_eq!(
        tx_queue_key_is_limited_fun(&list, &fixture_key_fun()),
        Err(TX_QUEUE_KEY_VERIFY_ERROR::NoSuchKey)
    );
}

#[test]
fn the_permission_reads_the_way_the_chain_writes_it() {
    // --- WHY --- THE PERMISSION IS READ BACK OUT OF THE FIXTURE: this
    // asserts that the json the test wrote is the json the chain
    // writes, so a shape change in near-kit breaks here instead of
    // making the permission tests above quietly wrong.
    let list = list_with_permission_fun(serde_json::json!("FullAccess"));
    let permission = &list.keys[0].access_key.permission;
    assert!(matches!(permission, AccessKeyPermissionView::FullAccess));

    let list = list_with_permission_fun(serde_json::json!({
        "FunctionCall": {
            "allowance": null,
            "receiver_id": "ref.finance.near",
            "method_names": []
        }
    }));
    let permission = &list.keys[0].access_key.permission;
    assert!(matches!(
        permission,
        AccessKeyPermissionView::FunctionCall { .. }
    ));
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
