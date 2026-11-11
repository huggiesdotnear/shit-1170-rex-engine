// ========================================
// the tx queue store, tested against a real sqlite file.
//
// the db is a temp file, not an in-memory one, because the production db
// is a file and the two can differ: WAL, the journal mode, and the way
// sqlite writes pages are all file behaviours.
use crate::backend::logic::endpoint_shit_1170_api_keys::api_key_fun::{
    api_key_generate_fun, api_key_hash_fun,
};
use crate::backend::logic::endpoint_shit_1170_api_keys::api_keys_store::api_key_upsert_fun;
use crate::backend::logic::endpoint_shit_1170_tx_queue::tx_queue_accounts_store::{
    tx_queue_accounts_delete_fun, tx_queue_accounts_has_for_account_fun,
    tx_queue_accounts_insert_fun, tx_queue_accounts_list_by_account_fun,
    tx_queue_accounts_list_by_queue_fun, tx_queue_accounts_list_by_tx_fun,
    tx_queue_accounts_set_sent_fun,
};
use crate::backend::logic::endpoint_shit_1170_tx_queue::tx_queue_args_fun::{
    tx_queue_args_from_json_fun, tx_queue_args_to_json_fun,
};
use crate::backend::logic::endpoint_shit_1170_tx_queue::tx_queue_named_accounts_fun::tx_queue_named_accounts_fun;
use crate::backend::logic::endpoint_shit_1170_tx_queue::tx_queue_status_fun::SHIT_1170_TX_QUEUE_STATUS_PENDING;
use crate::backend::logic::endpoint_shit_1170_tx_queue::tx_queue_store::{
    TX_QUEUE_CURSOR_POINT, TX_QUEUE_LIST_QUERY, tx_queue_delete_fun, tx_queue_get_fun,
    tx_queue_insert_fun, tx_queue_list_fun,
};
use crate::backend::logic::endpoint_shit_1170_tx_queue::tx_queue_transactions_store::{
    tx_queue_transactions_insert_fun, tx_queue_transactions_list_by_queue_fun,
};
use crate::backend::logic::endpoint_shit_1170_tx_queue::tx_queue_uuid_fun::tx_queue_uuid_fun;
use sqlx::Row;
// ========================================
// --- HELPERS ----------------------------
// a temp db file, removed when the guard is dropped.
struct TEMP_DB {
    path: std::path::PathBuf,
}

impl TEMP_DB {
    fn new_fun(name: &str) -> Self {
        let mut path = std::env::temp_dir();
        // --- WHY --- the prefix says WHICH test file this db belongs to:
        // the api keys store test uses the same `name` values, and two
        // test files that pick the same file name delete each other's db
        // out from under each other.
        path.push(format!(
            "shit-1170-tx-store-{name}-{:?}.db",
            std::process::id()
        ));
        // a leftover from a previous run would make the test read old rows.
        let _ = std::fs::remove_file(&path);
        TEMP_DB { path }
    }

    async fn pool_fun(&self) -> sqlx::SqlitePool {
        crate::backend::logic::db::db_open_fun::db_open_fun(&self.path)
            .await
            .expect("the db should open")
    }
}

impl Drop for TEMP_DB {
    fn drop(&mut self) {
        // the sqlite sidecar files too, or the temp dir fills up.
        let _ = std::fs::remove_file(&self.path);
        let _ = std::fs::remove_file(format!("{}-wal", self.path.display()));
        let _ = std::fs::remove_file(format!("{}-shm", self.path.display()));
    }
}

// a queue with one transaction on it, and the ids of both.
struct MADE_QUEUE {
    queue_id: String,
    tx_id: String,
}

async fn make_queue_fun(
    pool: &sqlx::SqlitePool,
    what: &str,
    contract_id: &str,
    method: &str,
) -> MADE_QUEUE {
    let queue_id = tx_queue_uuid_fun();
    let tx_id = tx_queue_uuid_fun();

    tx_queue_insert_fun(
        pool,
        Some(what),
        Some("when the price is right"),
        Some("because"),
        None,
        &queue_id,
    )
    .await
    .expect("the queue should save");

    tx_queue_transactions_insert_fun(
        pool,
        &tx_id,
        &queue_id,
        0,
        contract_id,
        method,
        "{}",
        None,
        Some("30000000000000"),
        Some("1"),
        None,
        None,
        None,
        None,
        None,
    )
    .await
    .expect("the transaction should save");

    MADE_QUEUE { queue_id, tx_id }
}

// an api key for one account, so the naming rule has something to find.
async fn put_api_key_fun(pool: &sqlx::SqlitePool, account_id: &str) {
    let key = api_key_generate_fun();
    let key_hash = api_key_hash_fun(&key);
    api_key_upsert_fun(pool, account_id, &key, &key_hash)
        .await
        .expect("the api key should save");
}

// the list with no filters.
fn all_queues_query_fun() -> TX_QUEUE_LIST_QUERY {
    TX_QUEUE_LIST_QUERY {
        account_ids: Vec::new(),
        hide_sent: None,
        hide_sent_account: None,
        contract_ids: Vec::new(),
        method: None,
        cursor: None,
        limit: 50,
    }
}
// ========================================
// --- TESTS -----------------------------
#[tokio::test]
async fn a_queued_batch_is_stored_and_read_back() {
    let db = TEMP_DB::new_fun("store");
    let pool = db.pool_fun().await;

    let queue_id = tx_queue_uuid_fun();
    tx_queue_insert_fun(
        &pool,
        Some("swap 100 USDC for NEAR"),
        Some("when the price is right"),
        Some("you said to move half your usdc"),
        Some(1_772_666_400_000),
        &queue_id,
    )
    .await
    .expect("the queue should save");

    let stored = tx_queue_get_fun(&pool, &queue_id)
        .await
        .expect("the queue should read")
        .expect("the queue should exist");
    assert_eq!(stored.queue_id, queue_id);
    assert_eq!(stored.what.as_deref(), Some("swap 100 USDC for NEAR"));
    assert_eq!(stored.when_text.as_deref(), Some("when the price is right"));
    assert_eq!(
        stored.why.as_deref(),
        Some("you said to move half your usdc")
    );
    assert_eq!(stored.when_at, Some(1_772_666_400_000));
    assert!(stored.created_at > 0);
}

#[tokio::test]
async fn the_null_fields_stay_null() {
    let db = TEMP_DB::new_fun("nulls");
    let pool = db.pool_fun().await;

    let queue_id = tx_queue_uuid_fun();
    tx_queue_insert_fun(&pool, None, None, None, None, &queue_id)
        .await
        .expect("the queue should save");

    let stored = tx_queue_get_fun(&pool, &queue_id)
        .await
        .expect("the queue should read")
        .expect("the queue should exist");
    assert!(stored.what.is_none());
    assert!(stored.when_text.is_none());
    assert!(stored.why.is_none());
    assert!(stored.when_at.is_none());
}

#[tokio::test]
async fn a_queue_that_is_not_there_is_not_found() {
    let db = TEMP_DB::new_fun("missing");
    let pool = db.pool_fun().await;

    let found = tx_queue_get_fun(&pool, "no-such-queue")
        .await
        .expect("the queue should read");
    assert!(found.is_none());
}

#[tokio::test]
async fn the_transactions_of_a_batch_come_back_in_position_order() {
    let db = TEMP_DB::new_fun("position");
    let pool = db.pool_fun().await;

    let queue_id = tx_queue_uuid_fun();
    tx_queue_insert_fun(&pool, Some("a batch"), None, None, None, &queue_id)
        .await
        .expect("the queue should save");

    // inserted OUT of order on purpose — the position column is what
    // orders them, not the insert order.
    for (position, method) in [(2, "ft_withdraw"), (0, "ft_deposit"), (1, "ft_swap")] {
        tx_queue_transactions_insert_fun(
            &pool,
            &tx_queue_uuid_fun(),
            &queue_id,
            position,
            "ref.finance.near",
            method,
            "{}",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .expect("the transaction should save");
    }

    let rows = tx_queue_transactions_list_by_queue_fun(&pool, &queue_id)
        .await
        .expect("the transactions should read");
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].position, 0);
    assert_eq!(rows[0].method, "ft_deposit");
    assert_eq!(rows[1].position, 1);
    assert_eq!(rows[1].method, "ft_swap");
    assert_eq!(rows[2].position, 2);
    assert_eq!(rows[2].method, "ft_withdraw");
}

#[tokio::test]
async fn the_args_json_round_trips() {
    let db = TEMP_DB::new_fun("args");
    let pool = db.pool_fun().await;

    let value = serde_json::json!({
        "receiver_id": "alice.near",
        "amount": "100000000",
        "token_in": "usdt.tether-token.near",
        "token_out": "wrap.near",
        "slippage": 0.5,
        "flag": true,
    });

    let column = tx_queue_args_from_json_fun(&value).expect("the args should serialize");
    // the column holds COMPACT json.
    assert!(!column.contains('\n'));
    assert!(column.starts_with('{'));

    let queue_id = tx_queue_uuid_fun();
    let tx_id = tx_queue_uuid_fun();
    tx_queue_insert_fun(&pool, Some("a batch"), None, None, None, &queue_id)
        .await
        .expect("the queue should save");
    tx_queue_transactions_insert_fun(
        &pool,
        &tx_id,
        &queue_id,
        0,
        "ref.finance.near",
        "ft_swap",
        &column,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .await
    .expect("the transaction should save");

    let rows = tx_queue_transactions_list_by_queue_fun(&pool, &queue_id)
        .await
        .expect("the transactions should read");
    let read_back = tx_queue_args_to_json_fun(&rows[0].args).expect("the args should parse");
    assert_eq!(read_back, value);
}

#[tokio::test]
async fn a_bad_args_blob_is_an_error() {
    // a blob that is not json does not come back as a default — it comes
    // back as an error, so the caller knows the arguments are gone.
    let bad = tx_queue_args_to_json_fun("{not json").expect_err("the args should not parse");
    assert!(
        !bad.to_string().is_empty(),
        "a bad args blob must carry a message"
    );

    // and a blob that IS json comes back as that json.
    let good = tx_queue_args_to_json_fun("{\"a\":1}").expect("the args should parse");
    assert_eq!(good, serde_json::json!({"a": 1}));
}

#[tokio::test]
async fn the_account_rows_of_a_queue_are_read_back() {
    let db = TEMP_DB::new_fun("accounts");
    let pool = db.pool_fun().await;

    let made = make_queue_fun(&pool, "a batch", "ref.finance.near", "ft_swap").await;

    tx_queue_accounts_insert_fun(
        &pool,
        &made.queue_id,
        &made.tx_id,
        "alice.near",
        SHIT_1170_TX_QUEUE_STATUS_PENDING,
    )
    .await
    .expect("the account row should save");
    tx_queue_accounts_insert_fun(
        &pool,
        &made.queue_id,
        &made.tx_id,
        "bob.near",
        SHIT_1170_TX_QUEUE_STATUS_PENDING,
    )
    .await
    .expect("the account row should save");

    let rows = tx_queue_accounts_list_by_queue_fun(&pool, &made.queue_id)
        .await
        .expect("the account rows should read");
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].account_id, "alice.near");
    assert_eq!(rows[0].status, SHIT_1170_TX_QUEUE_STATUS_PENDING);
    assert!(rows[0].tx_hash.is_none());
    assert_eq!(rows[1].account_id, "bob.near");

    // and by the transaction.
    let by_tx = tx_queue_accounts_list_by_tx_fun(&pool, &made.tx_id)
        .await
        .expect("the account rows should read");
    assert_eq!(by_tx.len(), 2);

    // and by the account.
    let by_account = tx_queue_accounts_list_by_account_fun(&pool, "alice.near")
        .await
        .expect("the account rows should read");
    assert_eq!(by_account.len(), 1);
    assert_eq!(by_account[0].tx_id, made.tx_id);

    // and the account IS on the queue.
    let has = tx_queue_accounts_has_for_account_fun(&pool, &made.queue_id, "alice.near")
        .await
        .expect("the check should read");
    assert!(has);

    // and somebody who is not, is not.
    let has_not = tx_queue_accounts_has_for_account_fun(&pool, &made.queue_id, "carol.near")
        .await
        .expect("the check should read");
    assert!(!has_not);
}

#[tokio::test]
async fn adding_the_same_account_twice_is_refused() {
    let db = TEMP_DB::new_fun("twice");
    let pool = db.pool_fun().await;

    let made = make_queue_fun(&pool, "a batch", "ref.finance.near", "ft_swap").await;

    tx_queue_accounts_insert_fun(
        &pool,
        &made.queue_id,
        &made.tx_id,
        "alice.near",
        SHIT_1170_TX_QUEUE_STATUS_PENDING,
    )
    .await
    .expect("the account row should save");

    // the primary key is `(tx_id, account_id)`, so the second one fails.
    let second = tx_queue_accounts_insert_fun(
        &pool,
        &made.queue_id,
        &made.tx_id,
        "alice.near",
        SHIT_1170_TX_QUEUE_STATUS_PENDING,
    )
    .await;
    assert!(
        second.is_err(),
        "adding the same account twice must be refused"
    );

    // and there is still exactly ONE row.
    let rows = tx_queue_accounts_list_by_queue_fun(&pool, &made.queue_id)
        .await
        .expect("the account rows should read");
    assert_eq!(rows.len(), 1);
}

#[tokio::test]
async fn a_pending_row_is_deleted_but_a_row_with_a_hash_is_history() {
    let db = TEMP_DB::new_fun("delete");
    let pool = db.pool_fun().await;

    let made = make_queue_fun(&pool, "a batch", "ref.finance.near", "ft_swap").await;

    tx_queue_accounts_insert_fun(
        &pool,
        &made.queue_id,
        &made.tx_id,
        "alice.near",
        SHIT_1170_TX_QUEUE_STATUS_PENDING,
    )
    .await
    .expect("the account row should save");

    // the pending row goes.
    let deleted = tx_queue_accounts_delete_fun(&pool, &made.tx_id, "alice.near")
        .await
        .expect("the row should delete");
    assert_eq!(deleted, 1);

    let rows = tx_queue_accounts_list_by_queue_fun(&pool, &made.queue_id)
        .await
        .expect("the account rows should read");
    assert!(rows.is_empty());

    // now alice sends it, and the row comes back WITH a hash.
    tx_queue_accounts_set_sent_fun(&pool, &made.tx_id, "alice.near", "9hN3lq", "succeeded")
        .await
        .expect("the row should save");

    // the row with a hash does NOT go.
    let deleted_sent = tx_queue_accounts_delete_fun(&pool, &made.tx_id, "alice.near")
        .await
        .expect("the delete should run");
    assert_eq!(
        deleted_sent, 0,
        "a row that carries a hash is history and must not be deleted"
    );

    let rows = tx_queue_accounts_list_by_queue_fun(&pool, &made.queue_id)
        .await
        .expect("the account rows should read");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].tx_hash.as_deref(), Some("9hN3lq"));
    assert_eq!(rows[0].status, "succeeded");
}

#[tokio::test]
async fn set_sent_creates_the_row_when_it_is_not_there() {
    let db = TEMP_DB::new_fun("join");
    let pool = db.pool_fun().await;

    let made = make_queue_fun(&pool, "a batch", "ref.finance.near", "ft_swap").await;

    // alice was NEVER named on this queue. she signs it anyway and she
    // joins it.
    let affected =
        tx_queue_accounts_set_sent_fun(&pool, &made.tx_id, "alice.near", "AbCdEf", "sent")
            .await
            .expect("the row should save");
    assert_eq!(affected, 1);

    let rows = tx_queue_accounts_list_by_queue_fun(&pool, &made.queue_id)
        .await
        .expect("the account rows should read");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].account_id, "alice.near");
    assert_eq!(rows[0].status, "sent");
    assert_eq!(rows[0].tx_hash.as_deref(), Some("AbCdEf"));
    assert_eq!(rows[0].queue_id, made.queue_id);
}

#[tokio::test]
async fn set_sent_updates_the_row_that_is_already_there() {
    let db = TEMP_DB::new_fun("update-sent");
    let pool = db.pool_fun().await;

    let made = make_queue_fun(&pool, "a batch", "ref.finance.near", "ft_swap").await;

    tx_queue_accounts_insert_fun(
        &pool,
        &made.queue_id,
        &made.tx_id,
        "alice.near",
        SHIT_1170_TX_QUEUE_STATUS_PENDING,
    )
    .await
    .expect("the account row should save");

    tx_queue_accounts_set_sent_fun(&pool, &made.tx_id, "alice.near", "9hN3lq", "sent")
        .await
        .expect("the row should save");

    // and the outcome comes back later.
    tx_queue_accounts_set_sent_fun(&pool, &made.tx_id, "alice.near", "9hN3lq", "failed")
        .await
        .expect("the row should save");

    let rows = tx_queue_accounts_list_by_queue_fun(&pool, &made.queue_id)
        .await
        .expect("the account rows should read");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].status, "failed");
}

#[tokio::test]
async fn the_list_with_an_account_returns_only_that_accounts_queues() {
    let db = TEMP_DB::new_fun("list-account");
    let pool = db.pool_fun().await;

    let alice_queue = make_queue_fun(&pool, "for alice", "ref.finance.near", "ft_swap").await;
    let bob_queue = make_queue_fun(&pool, "for bob", "ref.finance.near", "ft_swap").await;

    tx_queue_accounts_insert_fun(
        &pool,
        &alice_queue.queue_id,
        &alice_queue.tx_id,
        "alice.near",
        SHIT_1170_TX_QUEUE_STATUS_PENDING,
    )
    .await
    .expect("the account row should save");
    tx_queue_accounts_insert_fun(
        &pool,
        &bob_queue.queue_id,
        &bob_queue.tx_id,
        "bob.near",
        SHIT_1170_TX_QUEUE_STATUS_PENDING,
    )
    .await
    .expect("the account row should save");

    let query = TX_QUEUE_LIST_QUERY {
        account_ids: vec!["alice.near".to_string()],
        ..all_queues_query_fun()
    };

    let (items, total) = tx_queue_list_fun(&pool, &query)
        .await
        .expect("the list should read");
    assert_eq!(total, 1);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].queue_id, alice_queue.queue_id);
}

#[tokio::test]
async fn the_list_with_hide_sent_drops_the_queues_that_account_has_sent() {
    let db = TEMP_DB::new_fun("hide-sent");
    let pool = db.pool_fun().await;

    let sent_queue = make_queue_fun(&pool, "already sent", "ref.finance.near", "ft_swap").await;
    let pending_queue = make_queue_fun(&pool, "not sent", "ref.finance.near", "ft_swap").await;

    for made in [&sent_queue, &pending_queue] {
        tx_queue_accounts_insert_fun(
            &pool,
            &made.queue_id,
            &made.tx_id,
            "alice.near",
            SHIT_1170_TX_QUEUE_STATUS_PENDING,
        )
        .await
        .expect("the account row should save");
    }

    // alice sent one of them.
    tx_queue_accounts_set_sent_fun(
        &pool,
        &sent_queue.tx_id,
        "alice.near",
        "9hN3lq",
        "succeeded",
    )
    .await
    .expect("the row should save");

    let query = TX_QUEUE_LIST_QUERY {
        account_ids: vec!["alice.near".to_string()],
        hide_sent: Some(true),
        hide_sent_account: Some("alice.near".to_string()),
        ..all_queues_query_fun()
    };

    let (items, total) = tx_queue_list_fun(&pool, &query)
        .await
        .expect("the list should read");
    assert_eq!(total, 1);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].queue_id, pending_queue.queue_id);
}

#[tokio::test]
async fn hide_sent_asks_about_the_account_not_about_everybody() {
    let db = TEMP_DB::new_fun("hide-sent-account");
    let pool = db.pool_fun().await;

    let made = make_queue_fun(&pool, "a batch", "ref.finance.near", "ft_swap").await;

    // alice and bob are both on it, and ALICE sent it.
    for account_id in ["alice.near", "bob.near"] {
        tx_queue_accounts_insert_fun(
            &pool,
            &made.queue_id,
            &made.tx_id,
            account_id,
            SHIT_1170_TX_QUEUE_STATUS_PENDING,
        )
        .await
        .expect("the account row should save");
    }
    tx_queue_accounts_set_sent_fun(&pool, &made.tx_id, "alice.near", "9hN3lq", "sent")
        .await
        .expect("the row should save");

    // alice's view drops it.
    let alice_query = TX_QUEUE_LIST_QUERY {
        account_ids: vec!["alice.near".to_string()],
        hide_sent: Some(true),
        hide_sent_account: Some("alice.near".to_string()),
        ..all_queues_query_fun()
    };
    let (alice_items, alice_total) = tx_queue_list_fun(&pool, &alice_query)
        .await
        .expect("the list should read");
    assert_eq!(alice_total, 0);
    assert!(alice_items.is_empty());

    // bob's view does NOT — bob never sent it.
    let bob_query = TX_QUEUE_LIST_QUERY {
        account_ids: vec!["bob.near".to_string()],
        hide_sent: Some(true),
        hide_sent_account: Some("bob.near".to_string()),
        ..all_queues_query_fun()
    };
    let (bob_items, bob_total) = tx_queue_list_fun(&pool, &bob_query)
        .await
        .expect("the list should read");
    assert_eq!(bob_total, 1);
    assert_eq!(bob_items.len(), 1);
}

#[tokio::test]
async fn hide_sent_without_an_account_is_not_rejected_by_the_store() {
    let db = TEMP_DB::new_fun("hide-sent-no-account");
    let pool = db.pool_fun().await;

    let made = make_queue_fun(&pool, "a batch", "ref.finance.near", "ft_swap").await;
    tx_queue_accounts_insert_fun(
        &pool,
        &made.queue_id,
        &made.tx_id,
        "alice.near",
        SHIT_1170_TX_QUEUE_STATUS_PENDING,
    )
    .await
    .expect("the account row should save");
    tx_queue_accounts_set_sent_fun(&pool, &made.tx_id, "alice.near", "9hN3lq", "sent")
        .await
        .expect("the row should save");

    // --- WHY --- the store does NOT refuse this. the store knows the
    // shape of the data, and "hide sent" without an account has no
    // account to be about — so the `NOT EXISTS` matches nothing that
    // anybody sent and the filter falls away. the list comes back whole.
    //
    // the ROUTE is where this is refused with a 400, because the route
    // knows the caller asked for something that cannot be answered.
    // saying it here too would put a rule about the request in the
    // storage layer, and the storage layer is not asked what the caller
    // wanted.
    let query = TX_QUEUE_LIST_QUERY {
        hide_sent: Some(true),
        hide_sent_account: None,
        ..all_queues_query_fun()
    };

    let (items, total) = tx_queue_list_fun(&pool, &query)
        .await
        .expect("the list should read");
    assert_eq!(total, 1);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].queue_id, made.queue_id);
}

#[tokio::test]
async fn the_list_filters_by_contract_and_by_method() {
    let db = TEMP_DB::new_fun("list-contract");
    let pool = db.pool_fun().await;

    let ref_queue = make_queue_fun(&pool, "a ref swap", "ref.finance.near", "ft_swap").await;
    let other_queue = make_queue_fun(&pool, "a nft mint", "nft.near", "nft_mint").await;

    let by_contract = TX_QUEUE_LIST_QUERY {
        contract_ids: vec!["ref.finance.near".to_string()],
        ..all_queues_query_fun()
    };
    let (items, total) = tx_queue_list_fun(&pool, &by_contract)
        .await
        .expect("the list should read");
    assert_eq!(total, 1);
    assert_eq!(items[0].queue_id, ref_queue.queue_id);

    let by_method = TX_QUEUE_LIST_QUERY {
        method: Some("ft_swap".to_string()),
        ..all_queues_query_fun()
    };
    let (items, total) = tx_queue_list_fun(&pool, &by_method)
        .await
        .expect("the list should read");
    assert_eq!(total, 1);
    assert_eq!(items[0].queue_id, ref_queue.queue_id);

    // and a method on a DIFFERENT contract does not match.
    let other_method = TX_QUEUE_LIST_QUERY {
        method: Some("nft_mint".to_string()),
        ..all_queues_query_fun()
    };
    let (items, total) = tx_queue_list_fun(&pool, &other_method)
        .await
        .expect("the list should read");
    assert_eq!(total, 1);
    assert_eq!(items[0].queue_id, other_queue.queue_id);
}

#[tokio::test]
async fn a_contract_filter_matches_a_queue_holding_any_transaction_for_it() {
    let db = TEMP_DB::new_fun("list-contract-any");
    let pool = db.pool_fun().await;

    let queue_id = tx_queue_uuid_fun();
    tx_queue_insert_fun(&pool, Some("a batch"), None, None, None, &queue_id)
        .await
        .expect("the queue should save");
    for (position, contract_id) in [(0, "usdt.tether-token.near"), (1, "ref.finance.near")] {
        tx_queue_transactions_insert_fun(
            &pool,
            &tx_queue_uuid_fun(),
            &queue_id,
            position,
            contract_id,
            "ft_transfer",
            "{}",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .expect("the transaction should save");
    }

    let query = TX_QUEUE_LIST_QUERY {
        contract_ids: vec!["ref.finance.near".to_string()],
        ..all_queues_query_fun()
    };
    let (items, total) = tx_queue_list_fun(&pool, &query)
        .await
        .expect("the list should read");
    assert_eq!(total, 1);
    assert_eq!(items[0].queue_id, queue_id);
}

#[tokio::test]
async fn the_total_is_the_count_before_the_limit() {
    let db = TEMP_DB::new_fun("total");
    let pool = db.pool_fun().await;

    for _ in 0..5 {
        make_queue_fun(&pool, "a batch", "ref.finance.near", "ft_swap").await;
    }

    let query = TX_QUEUE_LIST_QUERY {
        limit: 2,
        ..all_queues_query_fun()
    };

    let (items, total) = tx_queue_list_fun(&pool, &query)
        .await
        .expect("the list should read");
    assert_eq!(items.len(), 2, "the limit takes the cut");
    assert_eq!(total, 5, "the total is the count BEFORE the limit");
}

#[tokio::test]
async fn the_cursor_walks_the_list_without_repeating_or_skipping() {
    let db = TEMP_DB::new_fun("cursor");
    let pool = db.pool_fun().await;

    // five batches, and their ids in the order the list gives them.
    for _ in 0..5 {
        make_queue_fun(&pool, "a batch", "ref.finance.near", "ft_swap").await;
    }

    let (first, total) = tx_queue_list_fun(
        &pool,
        &TX_QUEUE_LIST_QUERY {
            limit: 2,
            ..all_queues_query_fun()
        },
    )
    .await
    .expect("the list should read");
    assert_eq!(first.len(), 2);
    assert_eq!(total, 5);

    // the cursor of the last row of the page.
    let last = first.last().expect("the page should have a last row");
    let cursor = TX_QUEUE_CURSOR_POINT {
        created_at: last.created_at,
        queue_id: last.queue_id.clone(),
    };

    let (second, total) = tx_queue_list_fun(
        &pool,
        &TX_QUEUE_LIST_QUERY {
            limit: 2,
            cursor: Some(cursor.clone()),
            ..all_queues_query_fun()
        },
    )
    .await
    .expect("the list should read");
    assert_eq!(second.len(), 2);
    assert_eq!(total, 5, "the total does not change as the reader pages");

    // the second page does NOT repeat the first page.
    for row in &second {
        assert!(
            !first.iter().any(|one| one.queue_id == row.queue_id),
            "the cursor must not repeat a row the reader already saw"
        );
    }

    // and the last page holds the rest.
    let last_of_second = second.last().expect("the page should have a last row");
    let (third, _total) = tx_queue_list_fun(
        &pool,
        &TX_QUEUE_LIST_QUERY {
            limit: 2,
            cursor: Some(TX_QUEUE_CURSOR_POINT {
                created_at: last_of_second.created_at,
                queue_id: last_of_second.queue_id.clone(),
            }),
            ..all_queues_query_fun()
        },
    )
    .await
    .expect("the list should read");
    assert_eq!(third.len(), 1, "the last page holds what is left");

    // and every row was seen exactly once.
    let mut seen: Vec<String> = Vec::new();
    for row in first.iter().chain(second.iter()).chain(third.iter()) {
        seen.push(row.queue_id.clone());
    }
    let mut unique = seen.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(seen.len(), 5);
    assert_eq!(unique.len(), 5, "every row is seen exactly once");
}

#[tokio::test]
async fn the_account_filter_takes_several_accounts_at_once() {
    let db = TEMP_DB::new_fun("list-accounts");
    let pool = db.pool_fun().await;

    let alice_queue = make_queue_fun(&pool, "for alice", "ref.finance.near", "ft_swap").await;
    let bob_queue = make_queue_fun(&pool, "for bob", "ref.finance.near", "ft_swap").await;
    let carol_queue = make_queue_fun(&pool, "for carol", "ref.finance.near", "ft_swap").await;

    for (made, account_id) in [
        (&alice_queue, "alice.near"),
        (&bob_queue, "bob.near"),
        (&carol_queue, "carol.near"),
    ] {
        tx_queue_accounts_insert_fun(
            &pool,
            &made.queue_id,
            &made.tx_id,
            account_id,
            SHIT_1170_TX_QUEUE_STATUS_PENDING,
        )
        .await
        .expect("the account row should save");
    }

    let query = TX_QUEUE_LIST_QUERY {
        account_ids: vec!["alice.near".to_string(), "bob.near".to_string()],
        ..all_queues_query_fun()
    };

    let (items, total) = tx_queue_list_fun(&pool, &query)
        .await
        .expect("the list should read");
    assert_eq!(total, 2, "alice OR bob is two batches");
    assert_eq!(items.len(), 2);
}

#[tokio::test]
async fn the_contract_filter_takes_several_contracts_at_once() {
    let db = TEMP_DB::new_fun("list-contracts");
    let pool = db.pool_fun().await;

    let ref_queue = make_queue_fun(&pool, "a ref swap", "ref.finance.near", "ft_swap").await;
    let nft_queue = make_queue_fun(&pool, "a nft mint", "nft.near", "nft_mint").await;
    let other_queue = make_queue_fun(&pool, "a token mint", "token.near", "mint").await;

    let query = TX_QUEUE_LIST_QUERY {
        contract_ids: vec!["ref.finance.near".to_string(), "nft.near".to_string()],
        ..all_queues_query_fun()
    };

    let (items, total) = tx_queue_list_fun(&pool, &query)
        .await
        .expect("the list should read");
    assert_eq!(total, 2, "ref OR nft is two batches");
    assert_eq!(items.len(), 2);
    assert!(items.iter().any(|one| one.queue_id == ref_queue.queue_id));
    assert!(items.iter().any(|one| one.queue_id == nft_queue.queue_id));
    assert!(!items.iter().any(|one| one.queue_id == other_queue.queue_id));
}

#[tokio::test]
async fn the_list_is_newest_first() {
    let db = TEMP_DB::new_fun("order");
    let pool = db.pool_fun().await;

    let first = make_queue_fun(&pool, "first", "ref.finance.near", "ft_swap").await;
    let second = make_queue_fun(&pool, "second", "ref.finance.near", "ft_swap").await;

    let (items, _total) = tx_queue_list_fun(&pool, &all_queues_query_fun())
        .await
        .expect("the list should read");
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].queue_id, second.queue_id);
    assert_eq!(items[1].queue_id, first.queue_id);
}

#[tokio::test]
async fn deleting_a_queue_takes_its_transactions_and_its_account_rows_with_it() {
    let db = TEMP_DB::new_fun("cascade");
    let pool = db.pool_fun().await;

    let made = make_queue_fun(&pool, "a batch", "ref.finance.near", "ft_swap").await;
    tx_queue_accounts_insert_fun(
        &pool,
        &made.queue_id,
        &made.tx_id,
        "alice.near",
        SHIT_1170_TX_QUEUE_STATUS_PENDING,
    )
    .await
    .expect("the account row should save");

    // the rows are there before the delete.
    assert_eq!(
        tx_queue_transactions_list_by_queue_fun(&pool, &made.queue_id)
            .await
            .expect("the transactions should read")
            .len(),
        1
    );
    assert_eq!(
        tx_queue_accounts_list_by_queue_fun(&pool, &made.queue_id)
            .await
            .expect("the account rows should read")
            .len(),
        1
    );

    let deleted = tx_queue_delete_fun(&pool, &made.queue_id)
        .await
        .expect("the queue should delete");
    assert_eq!(deleted, 1);

    // the queue is gone.
    assert!(
        tx_queue_get_fun(&pool, &made.queue_id)
            .await
            .expect("the queue should read")
            .is_none()
    );

    // and the transactions went with it.
    assert!(
        tx_queue_transactions_list_by_queue_fun(&pool, &made.queue_id)
            .await
            .expect("the transactions should read")
            .is_empty()
    );

    // and so did the account rows.
    assert!(
        tx_queue_accounts_list_by_queue_fun(&pool, &made.queue_id)
            .await
            .expect("the account rows should read")
            .is_empty()
    );
    assert!(
        tx_queue_accounts_list_by_tx_fun(&pool, &made.tx_id)
            .await
            .expect("the account rows should read")
            .is_empty()
    );
}

#[tokio::test]
async fn the_naming_rule_only_lets_the_agent_name_its_own_users() {
    let db = TEMP_DB::new_fun("naming");
    let pool = db.pool_fun().await;

    // the agent holds a key for alice, and for bob. carol is somebody
    // else's user.
    put_api_key_fun(&pool, "alice.near").await;
    put_api_key_fun(&pool, "bob.near").await;

    let wanted = vec![
        "alice.near".to_string(),
        "carol.near".to_string(),
        "bob.near".to_string(),
    ];

    let allowed = tx_queue_named_accounts_fun(&pool, "agent.near", &wanted)
        .await
        .expect("the check should read");

    assert_eq!(allowed.len(), 2, "carol is not the agent's user");
    assert!(allowed.contains(&"alice.near".to_string()));
    assert!(allowed.contains(&"bob.near".to_string()));
    assert!(!allowed.contains(&"carol.near".to_string()));
}

#[tokio::test]
async fn the_agent_may_always_name_its_own_account() {
    let db = TEMP_DB::new_fun("naming-self");
    let pool = db.pool_fun().await;

    let wanted = vec!["agent.near".to_string()];
    let allowed = tx_queue_named_accounts_fun(&pool, "agent.near", &wanted)
        .await
        .expect("the check should read");
    assert_eq!(allowed, vec!["agent.near".to_string()]);
}

#[test]
fn a_uuid_is_lowercase_hyphenated_and_unique() {
    let first = tx_queue_uuid_fun();
    let second = tx_queue_uuid_fun();

    assert_eq!(first.len(), 36);
    assert_eq!(first, first.to_lowercase());
    assert_ne!(first, second);

    // the hyphens are where a uuid puts them.
    let parts: Vec<&str> = first.split('-').collect();
    assert_eq!(
        parts.iter().map(|part| part.len()).collect::<Vec<usize>>(),
        vec![8, 4, 4, 4, 12]
    );

    // and the version nibble is a four.
    assert_eq!(parts[2].chars().next(), Some('4'));
}

#[tokio::test]
async fn the_schema_has_the_three_queue_tables() {
    let db = TEMP_DB::new_fun("schema");
    let _pool = db.pool_fun().await;
    let pool = db.pool_fun().await;

    let tables: Vec<String> = sqlx::query("SELECT name FROM sqlite_master WHERE type = 'table'")
        .fetch_all(&pool)
        .await
        .expect("the tables should read")
        .iter()
        .map(|row| row.get::<String, _>("name"))
        .collect();

    assert!(tables.contains(&"tx_queue".to_string()));
    assert!(tables.contains(&"tx_queue_transactions".to_string()));
    assert!(tables.contains(&"tx_queue_accounts".to_string()));

    // the `when` column is there, under its own name.
    let columns: Vec<String> = sqlx::query("SELECT name FROM pragma_table_info('tx_queue')")
        .fetch_all(&pool)
        .await
        .expect("the columns should read")
        .iter()
        .map(|row| row.get::<String, _>("name"))
        .collect();
    assert_eq!(
        columns,
        vec![
            "queue_id",
            "what",
            "when",
            "why",
            "when_at",
            "created_at",
            "updated_at"
        ]
    );
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
