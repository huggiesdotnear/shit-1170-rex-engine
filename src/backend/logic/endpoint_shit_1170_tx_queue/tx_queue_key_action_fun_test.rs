// ========================================
// the KEY ACTION shape, tested on its own.
//
// the shape is a RULE, and a rule that is only tested through the route
// is a rule whose failures are hard to read. these tests say what the
// rule refuses and why, without a db and without a chain.
//
// --- WHY --- THE CHAIN IS NOT IN THIS FILE: the chain check is a
// round trip to a node, and a test that needs a node is a test that
// fails when the network is down. the chain check has its own file and
// its own tests.
use crate::backend::logic::endpoint_shit_1170_tx_queue::tx_queue_key_action_fun::{
    TX_QUEUE_KEY_ACTION, tx_queue_key_action_read_fun,
};
use crate::backend::types::tx_queue_request::TX_QUEUE_TRANSACTION_REQUEST;
// ========================================
// --- HELPERS ----------------------------
// a transaction with no key action on it.
fn plain_call_fun() -> TX_QUEUE_TRANSACTION_REQUEST {
    TX_QUEUE_TRANSACTION_REQUEST {
        contract_id: "ref.finance.near".to_string(),
        method: "ft_swap".to_string(),
        args: None,
        args_base64: None,
        gas: None,
        deposit: None,
        key_action: None,
        key_public_key: None,
        key_receiver_id: None,
        key_method_names: None,
        key_allowance: None,
    }
}

// a transaction that adds a limited key.
fn add_key_fun() -> TX_QUEUE_TRANSACTION_REQUEST {
    TX_QUEUE_TRANSACTION_REQUEST {
        key_action: Some("add_key".to_string()),
        key_public_key: Some("ed25519:FrtUfbY74kXpBiWmqeQMtn7jKCX3zJhmwyg32ZqANj3c".to_string()),
        key_receiver_id: Some("ref.finance.near".to_string()),
        key_method_names: Some(vec!["ft_swap".to_string()]),
        key_allowance: Some("1000000000000000000000000".to_string()),
        ..plain_call_fun()
    }
}
// ========================================
// --- TESTS -----------------------------
#[test]
fn a_transaction_without_a_key_action_is_not_a_key_action() {
    // --- WHY --- THE CONTRACT CALL IS STILL READ AS A CALL: the shape
    // check answers "is this a key action", and a call is not one.
    assert_eq!(tx_queue_key_action_read_fun(&plain_call_fun()), Ok(None));
}

#[test]
fn an_empty_key_action_is_not_a_key_action() {
    // --- WHY --- AN EMPTY STRING SAYS NOTHING, and a transaction that
    // says nothing is a contract call.
    let mut transaction = plain_call_fun();
    transaction.key_action = Some("   ".to_string());
    assert_eq!(tx_queue_key_action_read_fun(&transaction), Ok(None));
}

#[test]
fn an_add_key_is_read_with_every_field() {
    let shape = tx_queue_key_action_read_fun(&add_key_fun())
        .expect("the add_key should be read")
        .expect("the add_key should be a key action");

    assert_eq!(shape.action, TX_QUEUE_KEY_ACTION::AddKey);
    assert_eq!(
        shape.public_key,
        "ed25519:FrtUfbY74kXpBiWmqeQMtn7jKCX3zJhmwyg32ZqANj3c"
    );
    assert_eq!(shape.receiver_id.as_deref(), Some("ref.finance.near"));
    assert_eq!(
        shape.method_names.as_deref(),
        Some(["ft_swap".to_string()].as_slice())
    );
    assert_eq!(
        shape.allowance.as_deref(),
        Some("1000000000000000000000000")
    );
}

#[test]
fn a_delete_key_drops_the_receiver_the_methods_and_the_allowance() {
    // --- WHY --- A DELETE CARRIES NO RECEIVER: a limited key with no
    // receiver can call nothing, and the chain rejects it. and the
    // fields describe the key that is BEING REMOVED — the chain is the
    // one that knows what it may call, so an agent that filled them in
    // was describing a different key.
    let mut transaction = plain_call_fun();
    transaction.key_action = Some("delete_key".to_string());
    transaction.key_public_key =
        Some("ed25519:FrtUfbY74kXpBiWmqeQMtn7jKCX3zJhmwyg32ZqANj3c".to_string());
    transaction.key_receiver_id = Some("ref.finance.near".to_string());
    transaction.key_method_names = Some(vec!["ft_swap".to_string()]);
    transaction.key_allowance = Some("1000000000000000000000000".to_string());

    let shape = tx_queue_key_action_read_fun(&transaction)
        .expect("the delete_key should be read")
        .expect("the delete_key should be a key action");

    assert_eq!(shape.action, TX_QUEUE_KEY_ACTION::DeleteKey);
    assert_eq!(shape.receiver_id, None);
    assert_eq!(shape.method_names, None);
    assert_eq!(shape.allowance, None);
}

#[test]
fn a_key_action_with_no_public_key_is_refused() {
    let mut transaction = plain_call_fun();
    transaction.key_action = Some("delete_key".to_string());

    let (code, _) = tx_queue_key_action_read_fun(&transaction).expect_err("it should be refused");
    assert_eq!(code, "no_key_public_key");
}

#[test]
fn an_add_key_with_no_receiver_is_refused() {
    let mut transaction = add_key_fun();
    transaction.key_receiver_id = None;

    let (code, _) = tx_queue_key_action_read_fun(&transaction).expect_err("it should be refused");
    assert_eq!(code, "no_key_receiver_id");
}

#[test]
fn a_key_action_that_is_neither_add_nor_delete_is_refused() {
    let mut transaction = plain_call_fun();
    transaction.key_action = Some("burn_the_keys".to_string());
    transaction.key_public_key = Some("ed25519:whatever".to_string());

    let (code, _) = tx_queue_key_action_read_fun(&transaction).expect_err("it should be refused");
    assert_eq!(code, "bad_key_action");
}

#[test]
fn an_empty_method_name_is_dropped_and_the_rest_stay() {
    let mut transaction = add_key_fun();
    transaction.key_method_names = Some(vec![
        "ft_swap".to_string(),
        "  ".to_string(),
        "ft_resolve_transfer".to_string(),
    ]);

    let shape = tx_queue_key_action_read_fun(&transaction)
        .expect("the add_key should be read")
        .expect("the add_key should be a key action");

    assert_eq!(
        shape.method_names.as_deref(),
        Some(["ft_swap".to_string(), "ft_resolve_transfer".to_string()].as_slice())
    );
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
