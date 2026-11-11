// ========================================
// the feed socket's EVENT, tested.
//
// the json on the wire is the contract with the client, so it is checked
// here — the whole message, key by key, and not just a part of it.
use serde_json::{Value, json};

use crate::backend::logic::feed_ws::feed_ws_event::FEED_WS_EVENT;
use crate::backend::types::swap_queue_request::{SWAP_QUEUE_ACCOUNT, SWAP_QUEUE_ITEM};
use crate::backend::types::tx_queue_request::{
    TX_QUEUE_ACCOUNT, TX_QUEUE_ITEM, TX_QUEUE_TRANSACTION,
};
// ========================================
// --- HELPERS ----------------------------
// a queued batch, in the wire shape.
fn one_queue_item_fun() -> TX_QUEUE_ITEM {
    TX_QUEUE_ITEM {
        queue_id: "11111111-1111-4111-8111-111111111111".to_string(),
        what: Some("swap 100 USDC for NEAR".to_string()),
        when: Some("when the price is right".to_string()),
        why: Some("you said to move half your usdc".to_string()),
        when_at: Some(1_772_666_400_000),
        transactions: vec![TX_QUEUE_TRANSACTION {
            tx_id: "22222222-2222-4222-8222-222222222222".to_string(),
            position: 0,
            contract_id: "ref.finance.near".to_string(),
            method: "ft_swap".to_string(),
            args: json!({ "receiver_id": "alice.near" }),
            args_base64: None,
            gas: Some("30000000000000".to_string()),
            deposit: Some("1".to_string()),
            key_action: None,
            key_public_key: None,
            key_receiver_id: None,
            key_method_names: None,
            key_allowance: None,
            accounts: vec![TX_QUEUE_ACCOUNT {
                account_id: "alice.near".to_string(),
                status: "pending".to_string(),
                tx_hash: None,
            }],
        }],
        accounts: vec![TX_QUEUE_ACCOUNT {
            account_id: "alice.near".to_string(),
            status: "pending".to_string(),
            tx_hash: None,
        }],
        created_at: 1_772_666_400_000,
        updated_at: 1_772_666_400_001,
    }
}

// a queued swap, in the wire shape.
fn one_swap_item_fun() -> SWAP_QUEUE_ITEM {
    SWAP_QUEUE_ITEM {
        swap_id: "33333333-3333-4333-8333-333333333333".to_string(),
        what: Some("sell usdc".to_string()),
        when: None,
        why: None,
        when_at: None,
        token_in: "usdt.tether-token.near".to_string(),
        amount_in: "100000000".to_string(),
        token_out: "wrap.near".to_string(),
        amount_out: None,
        dex_id: None,
        accounts: vec![SWAP_QUEUE_ACCOUNT {
            account_id: "alice.near".to_string(),
            status: "pending".to_string(),
            tx_hash: None,
            amount_out: None,
        }],
        created_at: 1_772_666_400_000,
        updated_at: 1_772_666_400_001,
    }
}
// ========================================
// --- TESTS -----------------------------
#[test]
fn a_queued_batch_is_sent_as_a_queue_event() {
    let event = FEED_WS_EVENT::Queue {
        queue: one_queue_item_fun(),
    };

    let text = serde_json::to_string(&event).expect("the event should serialize");
    let value: Value = serde_json::from_str(&text).expect("the event should be json");

    // --- WHY --- THE KEYS ARE EXACTLY TWO: the type and the payload.
    // a third key would be a message the client does not know how to
    // read, and the contract says the server sends nothing else.
    let keys: Vec<&String> = value.as_object().expect("an object").keys().collect();
    assert_eq!(
        keys.len(),
        2,
        "the message carries the type and the payload"
    );
    assert!(value.get("type").is_some());
    assert!(value.get("queue").is_some());
    assert!(value.get("swap").is_none());

    assert_eq!(value["type"], "queue");

    // and the payload is the SAME shape the list route answers with.
    let queue = &value["queue"];
    assert_eq!(queue["queueId"], "11111111-1111-4111-8111-111111111111");
    assert_eq!(queue["what"], "swap 100 USDC for NEAR");
    assert_eq!(queue["when"], "when the price is right");
    assert_eq!(queue["why"], "you said to move half your usdc");
    assert_eq!(queue["whenAt"], 1_772_666_400_000i64);
    assert_eq!(queue["createdAt"], 1_772_666_400_000i64);
    assert_eq!(queue["updatedAt"], 1_772_666_400_001i64);

    // the transactions are there, with their accounts.
    let transactions = queue["transactions"].as_array().expect("transactions");
    assert_eq!(transactions.len(), 1);
    assert_eq!(
        transactions[0]["txId"],
        "22222222-2222-4222-8222-222222222222"
    );
    assert_eq!(transactions[0]["position"], 0);
    assert_eq!(transactions[0]["contractId"], "ref.finance.near");
    assert_eq!(transactions[0]["method"], "ft_swap");
    assert_eq!(transactions[0]["args"]["receiver_id"], "alice.near");
    assert_eq!(transactions[0]["gas"], "30000000000000");
    assert_eq!(transactions[0]["deposit"], "1");

    // and the WHO.
    let accounts = queue["accounts"].as_array().expect("accounts");
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0]["accountId"], "alice.near");
    assert_eq!(accounts[0]["status"], "pending");
    assert!(accounts[0]["txHash"].is_null());
}

#[test]
fn a_queued_swap_is_sent_as_a_swap_event() {
    let event = FEED_WS_EVENT::Swap {
        swap: one_swap_item_fun(),
    };

    let text = serde_json::to_string(&event).expect("the event should serialize");
    let value: Value = serde_json::from_str(&text).expect("the event should be json");

    let keys: Vec<&String> = value.as_object().expect("an object").keys().collect();
    assert_eq!(
        keys.len(),
        2,
        "the message carries the type and the payload"
    );
    assert!(value.get("type").is_some());
    assert!(value.get("swap").is_some());
    assert!(value.get("queue").is_none());

    assert_eq!(value["type"], "swap");

    // and the payload is the SAME shape the swap list route answers with.
    let swap = &value["swap"];
    assert_eq!(swap["swapId"], "33333333-3333-4333-8333-333333333333");
    assert_eq!(swap["what"], "sell usdc");
    assert!(swap["when"].is_null());
    assert!(swap["why"].is_null());
    assert!(swap["whenAt"].is_null());
    assert_eq!(swap["tokenIn"], "usdt.tether-token.near");
    assert_eq!(swap["amountIn"], "100000000");
    assert_eq!(swap["tokenOut"], "wrap.near");
    assert!(swap["amountOut"].is_null());
    assert!(swap["dexId"].is_null());
    assert_eq!(swap["createdAt"], 1_772_666_400_000i64);
    assert_eq!(swap["updatedAt"], 1_772_666_400_001i64);

    let accounts = swap["accounts"].as_array().expect("accounts");
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0]["accountId"], "alice.near");
    assert_eq!(accounts[0]["status"], "pending");
    assert!(accounts[0]["txHash"].is_null());
    assert!(accounts[0]["amountOut"].is_null());
}

#[test]
fn the_two_events_say_what_they_are() {
    // --- WHY --- the type is the FIRST thing the client reads, and it
    // has to be one of exactly two words. a third kind would be a
    // message the client cannot handle.
    let queue = serde_json::to_value(FEED_WS_EVENT::Queue {
        queue: one_queue_item_fun(),
    })
    .expect("the event should serialize");
    let swap = serde_json::to_value(FEED_WS_EVENT::Swap {
        swap: one_swap_item_fun(),
    })
    .expect("the event should serialize");

    assert_eq!(queue["type"], "queue");
    assert_eq!(swap["type"], "swap");
    assert_ne!(queue["type"], swap["type"]);
}
