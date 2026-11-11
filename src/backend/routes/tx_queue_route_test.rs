// ========================================
// the tx queue ROUTES, tested against a real sqlite file and a real
// server.
//
// the router is served on a real listener and reached over http, so the
// tests go through the real routing, the real path params, the real
// query params and the real body extraction.
//
// --- WHY --- the db is a temp FILE and not an in-memory one: the
// production db is a file, and the two can differ — WAL, the journal
// mode, and the way sqlite writes pages are all file behaviours.
//
// --- WHY --- the api keys are put into the db DIRECTLY: the point of
// these tests is the queue routes, and minting a key through the
// api-keys route would only add a way for the test to fail for a reason
// that has nothing to do with the queue.
use serde_json::{Value, json};
use sqlx::SqlitePool;
// ========================================
// --- HELPERS ----------------------------
// a temp db file, removed when the guard is dropped.
struct TEMP_DB {
    path: std::path::PathBuf,
}

impl TEMP_DB {
    fn new_fun(name: &str) -> Self {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "shit-1170-tx-route-{name}-{:?}.db",
            std::process::id()
        ));
        // a leftover from a previous run would make the test read old rows.
        let _ = std::fs::remove_file(&path);
        TEMP_DB { path }
    }

    async fn pool_fun(&self) -> SqlitePool {
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

// an api key for one account, put straight into the db.
async fn put_api_key_fun(pool: &SqlitePool, account_id: &str) -> String {
    use crate::backend::logic::endpoint_shit_1170_api_keys::api_key_fun::api_key_hash_fun;
    use crate::backend::logic::endpoint_shit_1170_api_keys::api_keys_store::api_key_upsert_fun;

    let api_key = format!("shit_1170_{account_id}");
    let api_key_hash = api_key_hash_fun(&api_key);
    api_key_upsert_fun(pool, account_id, &api_key, &api_key_hash)
        .await
        .expect("the api key should save");
    api_key
}

// a running server on a free port, and the url it answers on.
//
// --- WHY --- port 0: the OS picks a free port, so the tests do not
// fight over one fixed port and cannot collide with the live app.
async fn serve_fun(pool: SqlitePool) -> String {
    use crate::backend::routes::app_router_fun::app_router_fun;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("the listener should bind");
    let addr = listener.local_addr().expect("the addr should read");
    let app = app_router_fun(pool);

    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    format!("http://{addr}")
}

// one answered request: the status and the parsed body.
struct THE_REPLY {
    status: u16,
    body: Value,
}

// a POST to the queue with the given api key (or none) and the given
// body.
async fn post_queue_fun(base_url: &str, api_key: Option<&str>, body: &Value) -> THE_REPLY {
    let mut request = reqwest::Client::new()
        .post(format!("{base_url}/api/shit-1170/tx-queue"))
        .json(body);
    if let Some(api_key) = api_key {
        request = request.header("x-api-key", api_key);
    }
    answer_fun(request.send().await.expect("the route should answer")).await
}

// a POST to the sent route of one queue.
async fn post_sent_fun(base_url: &str, api_key: &str, queue_id: &str, body: &Value) -> THE_REPLY {
    let request = reqwest::Client::new()
        .post(format!("{base_url}/api/shit-1170/tx-queue/{queue_id}/sent"))
        .header("x-api-key", api_key)
        .json(body);
    answer_fun(request.send().await.expect("the route should answer")).await
}

// a POST to the accounts route of one queue.
async fn post_account_fun(
    base_url: &str,
    api_key: &str,
    queue_id: &str,
    account_id: &str,
) -> THE_REPLY {
    let request = reqwest::Client::new()
        .post(format!(
            "{base_url}/api/shit-1170/tx-queue/{queue_id}/accounts"
        ))
        .header("x-api-key", api_key)
        .json(&json!({ "accountId": account_id }));
    answer_fun(request.send().await.expect("the route should answer")).await
}

// a DELETE of one account from one queue.
async fn delete_account_fun(
    base_url: &str,
    api_key: &str,
    queue_id: &str,
    account_id: &str,
) -> THE_REPLY {
    let request = reqwest::Client::new()
        .delete(format!(
            "{base_url}/api/shit-1170/tx-queue/{queue_id}/accounts/{account_id}"
        ))
        .header("x-api-key", api_key);
    answer_fun(request.send().await.expect("the route should answer")).await
}

// a GET of the queue list, with the given query string.
async fn get_list_fun(base_url: &str, api_key: Option<&str>, query: &str) -> THE_REPLY {
    let mut request =
        reqwest::Client::new().get(format!("{base_url}/api/shit-1170/tx-queue?{query}"));
    if let Some(api_key) = api_key {
        request = request.header("x-api-key", api_key);
    }
    answer_fun(request.send().await.expect("the route should answer")).await
}

// a GET of one queue.
async fn get_one_fun(base_url: &str, api_key: &str, queue_id: &str) -> THE_REPLY {
    let request = reqwest::Client::new()
        .get(format!("{base_url}/api/shit-1170/tx-queue/{queue_id}"))
        .header("x-api-key", api_key);
    answer_fun(request.send().await.expect("the route should answer")).await
}

// the status and the parsed body of an answered request.
//
// --- WHY --- the body is PARSED and not kept as text: the tests assert
// on the shape of the answer, and a json value says the shape.
async fn answer_fun(response: reqwest::Response) -> THE_REPLY {
    let status = response.status().as_u16();
    let text = response.text().await.unwrap_or_default();
    let body = serde_json::from_str(&text).unwrap_or(Value::Null);
    THE_REPLY { status, body }
}

// a body with one transaction in it.
fn one_tx_body_fun() -> Value {
    json!({
        "what": "swap 100 USDC for NEAR",
        "when": "when the price is right",
        "why": "you said to move half your usdc",
        "whenAt": 1_772_666_400_000i64,
        "transactions": [
            {
                "contractId": "ref.finance.near",
                "method": "ft_swap",
                "args": { "receiver_id": "alice.near", "amount": "100000000" },
                "gas": "30000000000000",
                "deposit": "1"
            }
        ]
    })
}

// a body that names the account the batch is for.
//
// --- WHY --- THE NAMED ACCOUNT IS THE WHO: a queued batch is a list of
// calls FOR somebody, and the `accounts` list says who. a batch with
// nobody on it is a batch nobody is waiting for.
fn one_tx_body_for_fun(account_id: &str) -> Value {
    let mut body = one_tx_body_fun();
    body["accounts"] = json!([account_id]);
    body
}

// a body with two transactions in it.
fn two_tx_body_fun() -> Value {
    json!({
        "what": "a batch of two",
        "transactions": [
            { "contractId": "ref.finance.near", "method": "ft_swap", "args": { "a": "1" } },
            { "contractId": "ref.finance.near", "method": "ft_resolve_transfer", "args": { "b": "2" } }
        ]
    })
}
// ========================================
// --- TESTS -----------------------------
#[tokio::test]
async fn queueing_with_no_api_key_is_refused() {
    let db = TEMP_DB::new_fun("no-key");
    let pool = db.pool_fun().await;
    let base_url = serve_fun(pool).await;

    let answer = post_queue_fun(&base_url, None, &one_tx_body_fun()).await;

    assert_eq!(answer.status, 401, "body was: {}", answer.body);
    assert_eq!(answer.body["error"]["code"], "no_api_key");
}

#[tokio::test]
async fn queueing_with_an_unknown_api_key_is_refused() {
    let db = TEMP_DB::new_fun("unknown-key");
    let pool = db.pool_fun().await;
    let base_url = serve_fun(pool).await;

    let answer = post_queue_fun(
        &base_url,
        Some("shit_1170_nobody_has_this"),
        &one_tx_body_fun(),
    )
    .await;

    assert_eq!(answer.status, 401, "body was: {}", answer.body);
    assert_eq!(answer.body["error"]["code"], "unknown_api_key");
}

#[tokio::test]
async fn a_batch_of_two_is_queued_and_the_transactions_keep_their_order() {
    let db = TEMP_DB::new_fun("batch");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let answer = post_queue_fun(&base_url, Some(&alice_key), &two_tx_body_fun()).await;

    assert_eq!(answer.status, 201, "body was: {}", answer.body);

    let transactions = answer.body["transactions"]
        .as_array()
        .expect("transactions");
    assert_eq!(transactions.len(), 2, "the batch holds two transactions");
    assert_eq!(transactions[0]["position"], 0);
    assert_eq!(transactions[0]["method"], "ft_swap");
    assert_eq!(transactions[1]["position"], 1);
    assert_eq!(transactions[1]["method"], "ft_resolve_transfer");

    // the queue's own id is there, and the what came back.
    assert!(
        answer.body["queueId"].as_str().is_some(),
        "the answer must carry the queue id"
    );
    assert_eq!(answer.body["what"], "a batch of two");

    // and the args come back as a json OBJECT, not a string.
    assert_eq!(transactions[0]["args"]["a"], "1");
    assert!(transactions[0]["args"].is_object());
}

#[tokio::test]
async fn the_answers_carry_the_what_the_when_and_the_why() {
    let db = TEMP_DB::new_fun("what-when-why");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let answer = post_queue_fun(&base_url, Some(&alice_key), &one_tx_body_fun()).await;

    assert_eq!(answer.status, 201, "body was: {}", answer.body);
    assert_eq!(answer.body["what"], "swap 100 USDC for NEAR");
    assert_eq!(answer.body["when"], "when the price is right");
    assert_eq!(answer.body["why"], "you said to move half your usdc");
    assert_eq!(answer.body["whenAt"], 1_772_666_400_000i64);
}

#[tokio::test]
async fn more_than_fifty_transactions_is_refused() {
    let db = TEMP_DB::new_fun("too-many");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let transactions: Vec<Value> = (0..51)
        .map(|_| json!({ "contractId": "ref.finance.near", "method": "ft_swap" }))
        .collect();
    let body = json!({ "transactions": transactions });

    let answer = post_queue_fun(&base_url, Some(&alice_key), &body).await;

    assert_eq!(answer.status, 400, "body was: {}", answer.body);
    assert_eq!(answer.body["error"]["code"], "too_many_transactions");
}

#[tokio::test]
async fn zero_transactions_is_refused() {
    let db = TEMP_DB::new_fun("no-transactions");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let answer = post_queue_fun(&base_url, Some(&alice_key), &json!({ "transactions": [] })).await;

    assert_eq!(answer.status, 400, "body was: {}", answer.body);
    assert_eq!(answer.body["error"]["code"], "no_transactions");
}

#[tokio::test]
async fn a_transaction_with_no_contract_id_is_refused() {
    let db = TEMP_DB::new_fun("no-contract");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let body = json!({ "transactions": [ { "method": "ft_swap" } ] });
    let answer = post_queue_fun(&base_url, Some(&alice_key), &body).await;

    assert_eq!(answer.status, 400, "body was: {}", answer.body);
    assert_eq!(answer.body["error"]["code"], "no_contract_id");
}

#[tokio::test]
async fn a_transaction_with_no_method_is_refused() {
    let db = TEMP_DB::new_fun("no-method");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let body = json!({ "transactions": [ { "contractId": "ref.finance.near" } ] });
    let answer = post_queue_fun(&base_url, Some(&alice_key), &body).await;

    assert_eq!(answer.status, 400, "body was: {}", answer.body);
    assert_eq!(answer.body["error"]["code"], "no_method");
}

#[tokio::test]
async fn args_that_are_not_an_object_are_refused() {
    let db = TEMP_DB::new_fun("bad-args");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let body = json!({
        "transactions": [ { "contractId": "ref.finance.near", "method": "ft_swap", "args": [ "a" ] } ]
    });
    let answer = post_queue_fun(&base_url, Some(&alice_key), &body).await;

    assert_eq!(answer.status, 400, "body was: {}", answer.body);
    assert_eq!(answer.body["error"]["code"], "args_not_an_object");
}

// --- WHY --- THE BINARY ARGS ---
// some contracts take their arguments as raw bytes and not as json —
// the FastFS protocol (`__fastdata_fastfs`) wants borsh bytes. so a
// transaction may carry `argsBase64` instead, and these tests are about
// that second way of carrying the arguments.
#[tokio::test]
async fn a_transaction_with_binary_args_is_queued_and_comes_back_with_the_same_base64() {
    let db = TEMP_DB::new_fun("binary-args");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    // "AAwAAABmYXN0bmVhci5wbmc=" — the bytes of a borsh call.
    let body = json!({
        "what": "a fastfs write",
        "transactions": [ {
            "contractId": "fastfs.near",
            "method": "__fastdata_fastfs",
            "argsBase64": "AAwAAABmYXN0bmVhci5wbmc=",
            "gas": "30000000000000",
            "deposit": "0"
        } ]
    });

    let answer = post_queue_fun(&base_url, Some(&alice_key), &body).await;
    assert_eq!(answer.status, 201, "body was: {}", answer.body);

    let transactions = answer.body["transactions"]
        .as_array()
        .expect("transactions");
    assert_eq!(transactions.len(), 1);
    assert_eq!(
        transactions[0]["argsBase64"], "AAwAAABmYXN0bmVhci5wbmc=",
        "the base64 must come back exactly as it went in"
    );
    // and the json args are the empty object — the bytes have no names.
    assert!(transactions[0]["args"].is_object());
    assert_eq!(transactions[0]["args"].as_object().expect("args").len(), 0);
}

#[tokio::test]
async fn a_transaction_with_both_kinds_of_args_is_refused() {
    let db = TEMP_DB::new_fun("both-args");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let body = json!({
        "transactions": [ {
            "contractId": "fastfs.near",
            "method": "__fastdata_fastfs",
            "args": { "a": "1" },
            "argsBase64": "AAwAAABmYXN0bmVhci5wbmc="
        } ]
    });

    let answer = post_queue_fun(&base_url, Some(&alice_key), &body).await;

    assert_eq!(answer.status, 400, "body was: {}", answer.body);
    assert_eq!(answer.body["error"]["code"], "args_and_args_base64");
}

#[tokio::test]
async fn an_empty_args_with_binary_args_is_the_binary_shape_and_not_both() {
    // --- WHY --- THE EMPTY OBJECT DOES NOT COUNT AS "BOTH": the rule is
    // about what the call MEANS, and the empty object is the shape a
    // binary-args call takes on its way through the db. the middleman
    // reads it the same way, so the two agree.
    let db = TEMP_DB::new_fun("empty-args-and-binary");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let body = json!({
        "transactions": [ {
            "contractId": "fastfs.near",
            "method": "__fastdata_fastfs",
            "args": {},
            "argsBase64": "AAwAAABmYXN0bmVhci5wbmc="
        } ]
    });

    let answer = post_queue_fun(&base_url, Some(&alice_key), &body).await;

    assert_eq!(answer.status, 201, "body was: {}", answer.body);
    assert_eq!(
        answer.body["transactions"][0]["argsBase64"],
        "AAwAAABmYXN0bmVhci5wbmc="
    );
    assert_eq!(
        answer.body["transactions"][0]["args"]
            .as_object()
            .expect("args")
            .len(),
        0
    );
}

#[tokio::test]
async fn a_transaction_with_args_base64_that_is_not_base64_is_refused() {
    let db = TEMP_DB::new_fun("bad-base64");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let body = json!({
        "transactions": [ {
            "contractId": "fastfs.near",
            "method": "__fastdata_fastfs",
            "argsBase64": "not base64 at all!!"
        } ]
    });

    let answer = post_queue_fun(&base_url, Some(&alice_key), &body).await;

    assert_eq!(answer.status, 400, "body was: {}", answer.body);
    assert_eq!(answer.body["error"]["code"], "args_base64_not_base64");
}

#[tokio::test]
async fn the_args_base64_the_engine_accepts_is_the_args_base64_it_answers_with() {
    // --- WHY --- THE SET IS THE STANDARD DECODER'S SET: the middleman
    // refuses a string this decoder would not read, so the two must
    // agree on every one of them. a payload the builder accepted and
    // the engine refused would be a payload that only fails on the wire.
    let db = TEMP_DB::new_fun("base64-set");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    // these are the ones the standard decoder takes.
    for good in ["AAA=", "abc=", "aQ==", "AQI=", "AQID", "AAAA"] {
        let body = json!({
            "transactions": [ {
                "contractId": "fastfs.near",
                "method": "__fastdata_fastfs",
                "argsBase64": good
            } ]
        });
        let answer = post_queue_fun(&base_url, Some(&alice_key), &body).await;
        assert_eq!(
            answer.status, 201,
            "{good} should be taken: {}",
            answer.body
        );
        assert_eq!(answer.body["transactions"][0]["argsBase64"], good);
    }

    // and these are the ones it does not.
    for bad in ["AAA", "abc", "ab==", "a-b_c==", "a+b/c"] {
        let body = json!({
            "transactions": [ {
                "contractId": "fastfs.near",
                "method": "__fastdata_fastfs",
                "argsBase64": bad
            } ]
        });
        let answer = post_queue_fun(&base_url, Some(&alice_key), &body).await;
        assert_eq!(
            answer.status, 400,
            "{bad} should be refused: {}",
            answer.body
        );
        assert_eq!(answer.body["error"]["code"], "args_base64_not_base64");
    }
}

#[tokio::test]
async fn a_transaction_with_no_args_at_all_is_still_an_empty_object() {
    let db = TEMP_DB::new_fun("no-args");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let body = json!({
        "transactions": [ { "contractId": "ref.finance.near", "method": "ft_swap" } ]
    });

    let answer = post_queue_fun(&base_url, Some(&alice_key), &body).await;

    assert_eq!(answer.status, 201, "body was: {}", answer.body);
    let transactions = answer.body["transactions"]
        .as_array()
        .expect("transactions");
    assert!(transactions[0]["args"].is_object());
    assert_eq!(transactions[0]["args"].as_object().expect("args").len(), 0);
    // and there is no binary args on it.
    assert!(transactions[0]["argsBase64"].is_null());
}

#[tokio::test]
async fn a_named_account_the_agent_holds_no_key_for_is_refused() {
    // --- WHY --- this is the NAMING RULE and the anti-spam wall: the
    // agent may only name its OWN users. an agent that could name
    // anybody would put a tx in another agent's user's feed.
    let db = TEMP_DB::new_fun("naming-refused");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    // carol has no api key at all — she is somebody else's user.
    let base_url = serve_fun(pool).await;

    let mut body = one_tx_body_fun();
    body["accounts"] = json!(["carol.near"]);

    let answer = post_queue_fun(&base_url, Some(&alice_key), &body).await;

    assert_eq!(answer.status, 400, "body was: {}", answer.body);
    assert_eq!(answer.body["error"]["code"], "account_not_named_by_agent");
}

#[tokio::test]
async fn a_named_account_the_agent_holds_a_key_for_is_accepted() {
    let db = TEMP_DB::new_fun("naming-ok");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let bob_key = put_api_key_fun(&pool, "bob.near").await;
    let _ = &bob_key;
    let base_url = serve_fun(pool).await;

    let mut body = one_tx_body_fun();
    body["accounts"] = json!(["alice.near", "bob.near"]);

    let answer = post_queue_fun(&base_url, Some(&alice_key), &body).await;

    assert_eq!(answer.status, 201, "body was: {}", answer.body);

    // the WHO of the queue holds both accounts, pending.
    let accounts = answer.body["accounts"].as_array().expect("accounts");
    assert_eq!(accounts.len(), 2, "the WHO holds both accounts");
    assert_eq!(accounts[0]["accountId"], "alice.near");
    assert_eq!(accounts[0]["status"], "pending");
    assert!(accounts[0]["txHash"].is_null());
    assert_eq!(accounts[1]["accountId"], "bob.near");

    // and the transaction's own accounts hold them too.
    let transaction_accounts = answer.body["transactions"][0]["accounts"]
        .as_array()
        .expect("the transaction accounts");
    assert_eq!(transaction_accounts.len(), 2);
}

#[tokio::test]
async fn the_who_is_empty_when_no_account_is_named() {
    let db = TEMP_DB::new_fun("no-who");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let answer = post_queue_fun(&base_url, Some(&alice_key), &one_tx_body_fun()).await;

    assert_eq!(answer.status, 201, "body was: {}", answer.body);
    let accounts = answer.body["accounts"].as_array().expect("accounts");
    assert!(accounts.is_empty(), "the WHO is empty when nobody is named");
}

#[tokio::test]
async fn adding_an_account_with_another_accounts_key_is_refused() {
    let db = TEMP_DB::new_fun("add-not-yours");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let bob_key = put_api_key_fun(&pool, "bob.near").await;
    let base_url = serve_fun(pool).await;

    let queued = post_queue_fun(&base_url, Some(&alice_key), &one_tx_body_fun()).await;
    let queue_id = queued.body["queueId"]
        .as_str()
        .expect("a queue id")
        .to_string();

    // bob's key, and bob's account — that is bob adding HIMSELF, which
    // is fine. so it is ALICE's key and ALICE's account that is tested
    // here: alice tries to add bob.
    let answer = post_account_fun(&base_url, &bob_key, &queue_id, "alice.near").await;

    assert_eq!(answer.status, 403, "body was: {}", answer.body);
    assert_eq!(answer.body["error"]["code"], "not_your_account");
}

#[tokio::test]
async fn adding_your_own_account_works_and_creates_a_row_under_every_transaction() {
    let db = TEMP_DB::new_fun("add-yours");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    // the agent queues a batch with NOBODY named on it.
    let queued = post_queue_fun(&base_url, Some(&alice_key), &two_tx_body_fun()).await;
    assert_eq!(queued.status, 201, "body was: {}", queued.body);
    let queue_id = queued.body["queueId"]
        .as_str()
        .expect("a queue id")
        .to_string();

    // alice adds HERSELF.
    let answer = post_account_fun(&base_url, &alice_key, &queue_id, "alice.near").await;

    assert_eq!(answer.status, 200, "body was: {}", answer.body);

    // the WHO of the queue holds her.
    let accounts = answer.body["accounts"].as_array().expect("accounts");
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0]["accountId"], "alice.near");
    assert_eq!(accounts[0]["status"], "pending");

    // and there is a row under EVERY transaction of the batch.
    let transactions = answer.body["transactions"]
        .as_array()
        .expect("transactions");
    assert_eq!(transactions.len(), 2);
    for transaction in transactions {
        let own = transaction["accounts"]
            .as_array()
            .expect("the transaction accounts");
        assert_eq!(own.len(), 1, "one row under every transaction");
        assert_eq!(own[0]["accountId"], "alice.near");
    }
}

#[tokio::test]
async fn adding_an_account_twice_is_refused() {
    let db = TEMP_DB::new_fun("add-twice");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let queued = post_queue_fun(&base_url, Some(&alice_key), &one_tx_body_fun()).await;
    let queue_id = queued.body["queueId"]
        .as_str()
        .expect("a queue id")
        .to_string();

    let first = post_account_fun(&base_url, &alice_key, &queue_id, "alice.near").await;
    assert_eq!(first.status, 200, "body was: {}", first.body);

    let second = post_account_fun(&base_url, &alice_key, &queue_id, "alice.near").await;
    assert_eq!(second.status, 409, "body was: {}", second.body);
    assert_eq!(second.body["error"]["code"], "already_on_the_queue");
}

#[tokio::test]
async fn adding_an_account_to_a_queue_that_is_not_there_is_a_404() {
    let db = TEMP_DB::new_fun("add-missing");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let answer = post_account_fun(&base_url, &alice_key, "no-such-queue", "alice.near").await;

    assert_eq!(answer.status, 404, "body was: {}", answer.body);
    assert_eq!(answer.body["error"]["code"], "no_such_queue");
}

#[tokio::test]
async fn the_sent_route_writes_the_hash_and_the_status_on_the_right_transaction() {
    let db = TEMP_DB::new_fun("sent");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    // alice is named on the batch, so she has a row under BOTH of its
    // transactions.
    let mut body = two_tx_body_fun();
    body["accounts"] = json!(["alice.near"]);

    let queued = post_queue_fun(&base_url, Some(&alice_key), &body).await;
    assert_eq!(queued.status, 201, "body was: {}", queued.body);
    let queue_id = queued.body["queueId"]
        .as_str()
        .expect("a queue id")
        .to_string();
    let first_tx_id = queued.body["transactions"][0]["txId"]
        .as_str()
        .expect("a tx id")
        .to_string();

    let body = json!({
        "txHashes": [ { "txId": first_tx_id, "txHash": "9hN3lq", "status": "succeeded" } ]
    });
    let answer = post_sent_fun(&base_url, &alice_key, &queue_id, &body).await;

    assert_eq!(answer.status, 200, "body was: {}", answer.body);

    // the FIRST transaction carries the hash, and the second does not.
    let transactions = answer.body["transactions"]
        .as_array()
        .expect("transactions");
    let first_accounts = transactions[0]["accounts"].as_array().expect("accounts");
    assert_eq!(first_accounts.len(), 1, "body was: {}", answer.body);
    assert_eq!(first_accounts[0]["status"], "succeeded");
    assert_eq!(first_accounts[0]["txHash"], "9hN3lq");

    let second_accounts = transactions[1]["accounts"].as_array().expect("accounts");
    assert_eq!(second_accounts.len(), 1, "body was: {}", answer.body);
    assert_eq!(second_accounts[0]["status"], "pending");
    assert!(second_accounts[0]["txHash"].is_null());
}

#[tokio::test]
async fn the_sent_route_creates_the_row_for_an_account_that_was_never_named() {
    // --- WHY --- this is the AUTO-JOIN: the queue is an open offer, and
    // an account that signs a queue it was not named on joins the WHO
    // list. the send has to be recorded somewhere.
    let db = TEMP_DB::new_fun("auto-join");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let bob_key = put_api_key_fun(&pool, "bob.near").await;
    let base_url = serve_fun(pool).await;

    // alice queues a batch for herself.
    let queued = post_queue_fun(&base_url, Some(&alice_key), &one_tx_body_fun()).await;
    let queue_id = queued.body["queueId"]
        .as_str()
        .expect("a queue id")
        .to_string();
    let tx_id = queued.body["transactions"][0]["txId"]
        .as_str()
        .expect("a tx id")
        .to_string();

    // bob signs it, and nobody ever named him.
    let body = json!({ "txHashes": [ { "txId": tx_id, "txHash": "AbCdEf" } ] });
    let answer = post_sent_fun(&base_url, &bob_key, &queue_id, &body).await;

    assert_eq!(answer.status, 200, "body was: {}", answer.body);

    let accounts = answer.body["accounts"].as_array().expect("accounts");
    assert_eq!(accounts.len(), 1, "bob joined the WHO list");
    assert_eq!(accounts[0]["accountId"], "bob.near");
    assert_eq!(accounts[0]["status"], "sent");
    assert_eq!(accounts[0]["txHash"], "AbCdEf");
}

#[tokio::test]
async fn the_sent_route_with_a_tx_id_from_another_queue_is_refused() {
    let db = TEMP_DB::new_fun("sent-other-queue");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    // two queues.
    let first = post_queue_fun(&base_url, Some(&alice_key), &one_tx_body_fun()).await;
    let second = post_queue_fun(&base_url, Some(&alice_key), &one_tx_body_fun()).await;
    let second_id = second.body["queueId"]
        .as_str()
        .expect("a queue id")
        .to_string();
    let other_tx_id = first.body["transactions"][0]["txId"]
        .as_str()
        .expect("a tx id")
        .to_string();

    // the tx id of the FIRST queue is reported to the SECOND one.
    let body = json!({ "txHashes": [ { "txId": other_tx_id, "txHash": "9hN3lq" } ] });
    let answer = post_sent_fun(&base_url, &alice_key, &second_id, &body).await;

    assert_eq!(answer.status, 400, "body was: {}", answer.body);
    assert_eq!(answer.body["error"]["code"], "tx_not_on_this_queue");
}

#[tokio::test]
async fn the_sent_route_with_no_hashes_is_refused() {
    let db = TEMP_DB::new_fun("sent-no-hashes");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let queued = post_queue_fun(&base_url, Some(&alice_key), &one_tx_body_fun()).await;
    let queue_id = queued.body["queueId"]
        .as_str()
        .expect("a queue id")
        .to_string();

    let answer = post_sent_fun(&base_url, &alice_key, &queue_id, &json!({ "txHashes": [] })).await;

    assert_eq!(answer.status, 400, "body was: {}", answer.body);
    assert_eq!(answer.body["error"]["code"], "no_tx_hashes");
}

#[tokio::test]
async fn the_sent_route_with_a_bad_status_is_refused() {
    let db = TEMP_DB::new_fun("sent-bad-status");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let queued = post_queue_fun(&base_url, Some(&alice_key), &one_tx_body_fun()).await;
    let queue_id = queued.body["queueId"]
        .as_str()
        .expect("a queue id")
        .to_string();
    let tx_id = queued.body["transactions"][0]["txId"]
        .as_str()
        .expect("a tx id")
        .to_string();

    let body = json!({
        "txHashes": [ { "txId": tx_id, "txHash": "9hN3lq", "status": "maybe" } ]
    });
    let answer = post_sent_fun(&base_url, &alice_key, &queue_id, &body).await;

    assert_eq!(answer.status, 400, "body was: {}", answer.body);
    assert_eq!(answer.body["error"]["code"], "bad_status");
}

#[tokio::test]
async fn the_delete_route_removes_a_pending_row_but_not_a_row_with_a_hash() {
    let db = TEMP_DB::new_fun("delete");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let bob_key = put_api_key_fun(&pool, "bob.near").await;
    let base_url = serve_fun(pool).await;

    // a batch with BOTH alice and bob named on it.
    let mut body = two_tx_body_fun();
    body["accounts"] = json!(["alice.near", "bob.near"]);
    let queued = post_queue_fun(&base_url, Some(&alice_key), &body).await;
    assert_eq!(queued.status, 201, "body was: {}", queued.body);
    let queue_id = queued.body["queueId"]
        .as_str()
        .expect("a queue id")
        .to_string();
    let first_tx_id = queued.body["transactions"][0]["txId"]
        .as_str()
        .expect("a tx id")
        .to_string();

    // bob signs the FIRST transaction only. his row under it carries a
    // hash now.
    let sent = json!({ "txHashes": [ { "txId": first_tx_id, "txHash": "AbCdEf" } ] });
    let sent_answer = post_sent_fun(&base_url, &bob_key, &queue_id, &sent).await;
    assert_eq!(sent_answer.status, 200, "body was: {}", sent_answer.body);

    // bob steps off the queue.
    let answer = delete_account_fun(&base_url, &bob_key, &queue_id, "bob.near").await;

    assert_eq!(answer.status, 200, "body was: {}", answer.body);

    let transactions = answer.body["transactions"]
        .as_array()
        .expect("transactions");

    // the row that carried a hash is STILL there — it is history.
    let first_accounts = transactions[0]["accounts"].as_array().expect("accounts");
    assert_eq!(first_accounts.len(), 2, "body was: {}", answer.body);
    let bob_on_first = first_accounts
        .iter()
        .find(|account| account["accountId"] == "bob.near")
        .expect("bob's row under the sent transaction");
    assert_eq!(bob_on_first["txHash"], "AbCdEf");
    assert_eq!(bob_on_first["status"], "sent");

    // the row with no hash is gone.
    let second_accounts = transactions[1]["accounts"].as_array().expect("accounts");
    assert_eq!(second_accounts.len(), 1, "body was: {}", answer.body);
    assert_eq!(second_accounts[0]["accountId"], "alice.near");
}

#[tokio::test]
async fn the_delete_route_with_another_accounts_key_is_refused() {
    let db = TEMP_DB::new_fun("delete-not-yours");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let bob_key = put_api_key_fun(&pool, "bob.near").await;
    let base_url = serve_fun(pool).await;

    let queued = post_queue_fun(&base_url, Some(&alice_key), &one_tx_body_fun()).await;
    let queue_id = queued.body["queueId"]
        .as_str()
        .expect("a queue id")
        .to_string();

    // bob tries to remove ALICE.
    let answer = delete_account_fun(&base_url, &bob_key, &queue_id, "alice.near").await;

    assert_eq!(answer.status, 403, "body was: {}", answer.body);
    assert_eq!(answer.body["error"]["code"], "not_your_account");
}

#[tokio::test]
async fn the_delete_route_for_an_account_that_is_not_on_the_queue_is_a_404() {
    let db = TEMP_DB::new_fun("delete-not-there");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let queued = post_queue_fun(&base_url, Some(&alice_key), &one_tx_body_fun()).await;
    let queue_id = queued.body["queueId"]
        .as_str()
        .expect("a queue id")
        .to_string();

    let answer = delete_account_fun(&base_url, &alice_key, &queue_id, "alice.near").await;

    assert_eq!(answer.status, 404, "body was: {}", answer.body);
    assert_eq!(answer.body["error"]["code"], "not_on_the_queue");
}

#[tokio::test]
async fn the_list_with_hide_sent_and_no_account_id_is_refused_for_a_reader_with_no_account() {
    let db = TEMP_DB::new_fun("list-hide-sent");
    let pool = db.pool_fun().await;
    let base_url = serve_fun(pool).await;

    // --- WHY --- A READER WITH NO ACCOUNT IS REFUSED: the filter says
    // "hide what THIS account sent", and a reader who has no account
    // and named none has nothing for it to be about.
    //
    // --- WHY --- A SIGNED-IN READER IS NOT REFUSED: they have an
    // account — their own — and the filter is about that one.
    let answer = get_list_fun(&base_url, None, "hide_sent=true").await;
    assert_eq!(answer.status, 400, "body was: {}", answer.body);
    assert_eq!(answer.body["error"]["code"], "hide_sent_without_account");
}

#[tokio::test]
async fn the_list_with_hide_sent_as_the_number_one_and_no_account_id_is_refused_for_a_reader_with_no_account()
 {
    let db = TEMP_DB::new_fun("list-hide-sent-one");
    let pool = db.pool_fun().await;
    let base_url = serve_fun(pool).await;

    // the plan spells the filter `hide_sent=1`, so the DIGIT is read
    // the same way the word is.
    let answer = get_list_fun(&base_url, None, "hide_sent=1").await;
    assert_eq!(answer.status, 400, "body was: {}", answer.body);
    assert_eq!(answer.body["error"]["code"], "hide_sent_without_account");
}

#[tokio::test]
async fn the_list_with_hide_sent_as_the_number_one_and_an_account_drops_the_sent_ones() {
    let db = TEMP_DB::new_fun("hide-sent-one");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let mut body = one_tx_body_fun();
    body["accounts"] = json!(["alice.near"]);
    let queued = post_queue_fun(&base_url, Some(&alice_key), &body).await;
    let queue_id = queued.body["queueId"]
        .as_str()
        .expect("a queue id")
        .to_string();
    let tx_id = queued.body["transactions"][0]["txId"]
        .as_str()
        .expect("a tx id")
        .to_string();

    let sent = json!({ "txHashes": [ { "txId": tx_id, "txHash": "9hN3lq" } ] });
    let sent_answer = post_sent_fun(&base_url, &alice_key, &queue_id, &sent).await;
    assert_eq!(sent_answer.status, 200, "body was: {}", sent_answer.body);

    let answer = get_list_fun(
        &base_url,
        Some(&alice_key),
        "account_id=alice.near&hide_sent=1",
    )
    .await;

    assert_eq!(answer.status, 200, "body was: {}", answer.body);
    assert_eq!(answer.body["total"], 0, "the sent one is hidden");
}

#[tokio::test]
async fn the_list_with_an_account_and_hide_sent_drops_the_sent_ones() {
    let db = TEMP_DB::new_fun("hide-sent");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let bob_key = put_api_key_fun(&pool, "bob.near").await;
    let base_url = serve_fun(pool).await;

    // two queues, both named for alice.
    let mut first_body = one_tx_body_fun();
    first_body["accounts"] = json!(["alice.near"]);
    let mut second_body = one_tx_body_fun();
    second_body["accounts"] = json!(["alice.near"]);

    let first = post_queue_fun(&base_url, Some(&alice_key), &first_body).await;
    let second = post_queue_fun(&base_url, Some(&alice_key), &second_body).await;
    let first_id = first.body["queueId"]
        .as_str()
        .expect("a queue id")
        .to_string();
    let second_id = second.body["queueId"]
        .as_str()
        .expect("a queue id")
        .to_string();
    let first_tx_id = first.body["transactions"][0]["txId"]
        .as_str()
        .expect("a tx id")
        .to_string();

    // alice sends the FIRST one.
    let sent = json!({ "txHashes": [ { "txId": first_tx_id, "txHash": "9hN3lq" } ] });
    let sent_answer = post_sent_fun(&base_url, &alice_key, &first_id, &sent).await;
    assert_eq!(sent_answer.status, 200, "body was: {}", sent_answer.body);

    // the whole list holds both.
    let all = get_list_fun(&base_url, Some(&alice_key), "").await;
    assert_eq!(all.status, 200, "body was: {}", all.body);
    assert_eq!(all.body["total"], 2);

    // and alice's hide-sent view holds only the one she has NOT sent.
    let mine = get_list_fun(
        &base_url,
        Some(&alice_key),
        "account_id=alice.near&hide_sent=true",
    )
    .await;
    assert_eq!(mine.status, 200, "body was: {}", mine.body);
    assert_eq!(mine.body["total"], 1);
    let items = mine.body["items"].as_array().expect("items");
    assert_eq!(items[0]["queueId"], second_id);

    // --- WHY --- bob's view is NOT alice's view: the filter is about
    // ONE account, and bob never sent anything.
    let bob_view = get_list_fun(
        &base_url,
        Some(&bob_key),
        "account_id=bob.near&hide_sent=true",
    )
    .await;
    assert_eq!(bob_view.status, 200, "body was: {}", bob_view.body);
    assert_eq!(bob_view.body["total"], 0);
}

#[tokio::test]
async fn the_list_filters_by_account_and_by_contract() {
    let db = TEMP_DB::new_fun("list-filters");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let mut body = one_tx_body_fun();
    body["accounts"] = json!(["alice.near"]);
    let queued = post_queue_fun(&base_url, Some(&alice_key), &body).await;
    let queue_id = queued.body["queueId"]
        .as_str()
        .expect("a queue id")
        .to_string();

    let by_account = get_list_fun(&base_url, Some(&alice_key), "account_id=alice.near").await;
    assert_eq!(by_account.status, 200, "body was: {}", by_account.body);
    assert_eq!(by_account.body["total"], 1);
    assert_eq!(by_account.body["items"][0]["queueId"], queue_id);

    let by_contract =
        get_list_fun(&base_url, Some(&alice_key), "contract_id=ref.finance.near").await;
    assert_eq!(by_contract.status, 200, "body was: {}", by_contract.body);
    assert_eq!(by_contract.body["total"], 1);

    let by_method = get_list_fun(&base_url, Some(&alice_key), "method=ft_swap").await;
    assert_eq!(by_method.status, 200, "body was: {}", by_method.body);
    assert_eq!(by_method.body["total"], 1);

    let by_nothing = get_list_fun(&base_url, Some(&alice_key), "contract_id=nothing.near").await;
    assert_eq!(by_nothing.status, 200, "body was: {}", by_nothing.body);
    assert_eq!(by_nothing.body["total"], 0);
}

#[tokio::test]
async fn the_list_is_open_and_needs_no_api_key() {
    let db = TEMP_DB::new_fun("list-open");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    // a batch is queued, so the list has something in it.
    post_queue_fun(&base_url, Some(&alice_key), &one_tx_body_fun()).await;

    // --- WHY --- NO HEADER AT ALL: the queue is an OPEN offer. a reader
    // who is not signed in can still read the feed.
    let request = reqwest::Client::new()
        .get(format!("{base_url}/api/shit-1170/tx-queue"))
        .send()
        .await
        .expect("the route should answer");
    let status = request.status().as_u16();
    let text = request.text().await.unwrap_or_default();
    assert_eq!(status, 200, "body was: {text}");
    let body: serde_json::Value = serde_json::from_str(&text).expect("json");
    assert_eq!(body["total"], 1, "the open list must show the batch");
}

#[tokio::test]
async fn the_list_of_one_queue_is_open_and_needs_no_api_key() {
    let db = TEMP_DB::new_fun("one-open");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let queued = post_queue_fun(&base_url, Some(&alice_key), &one_tx_body_fun()).await;
    let queue_id = queued.body["queueId"]
        .as_str()
        .expect("a queue id")
        .to_string();

    let request = reqwest::Client::new()
        .get(format!("{base_url}/api/shit-1170/tx-queue/{queue_id}"))
        .send()
        .await
        .expect("the route should answer");
    let status = request.status().as_u16();
    let text = request.text().await.unwrap_or_default();
    assert_eq!(status, 200, "body was: {text}");
    let body: serde_json::Value = serde_json::from_str(&text).expect("json");
    assert_eq!(body["queueId"], queue_id);
}

#[tokio::test]
async fn the_list_filtered_by_an_account_needs_no_api_key() {
    let db = TEMP_DB::new_fun("list-account-key");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    post_queue_fun(
        &base_url,
        Some(&alice_key),
        &one_tx_body_for_fun("alice.near"),
    )
    .await;

    // --- WHY --- NO KEY IS NEEDED: the queue list is an OPEN offer. a
    // reader who is not signed in can already read every batch and see
    // every account on it, so filtering that public list by a name is
    // not a leak — it is the filter the reader asked for.
    let answer = get_list_fun(&base_url, None, "account_id=alice.near").await;
    assert_eq!(answer.status, 200, "body was: {}", answer.body);
    assert_eq!(answer.body["total"], 1);
}

#[tokio::test]
async fn the_list_filtered_by_an_account_filters_by_the_account_that_was_named() {
    let db = TEMP_DB::new_fun("list-account-named");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let bob_key = put_api_key_fun(&pool, "bob.near").await;
    let base_url = serve_fun(pool).await;

    // bob queues a batch for himself.
    post_queue_fun(&base_url, Some(&bob_key), &one_tx_body_for_fun("bob.near")).await;

    // alice asks for bob's queues by name.
    //
    // --- WHY --- SHE GETS THEM AND NOT HER OWN: the account filter is
    // a filter BY account, and the reader picks the account. the
    // queue is an OPEN offer — alice can already read bob's batch and
    // see bob on it — so filtering the public list by a name is not a
    // leak, it is the filter the reader asked for.
    let answer = get_list_fun(&base_url, Some(&alice_key), "account_id=bob.near").await;
    assert_eq!(answer.status, 200, "body was: {}", answer.body);
    assert_eq!(
        answer.body["total"], 1,
        "the filter is by the account that was named"
    );

    // and alice's OWN queues — she has none — is still empty.
    let own = get_list_fun(&base_url, Some(&alice_key), "account_id=alice.near").await;
    assert_eq!(own.body["total"], 0);
}

#[tokio::test]
async fn a_signed_in_readers_own_account_is_the_account_the_hide_sent_filter_is_about() {
    let db = TEMP_DB::new_fun("list-hide-sent-own");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    // alice has two queues and has sent one of them.
    let sent = post_queue_fun(
        &base_url,
        Some(&alice_key),
        &one_tx_body_for_fun("alice.near"),
    )
    .await;
    let sent_id = sent.body["queueId"]
        .as_str()
        .expect("a queue id")
        .to_string();
    let sent_tx_id = sent.body["transactions"][0]["txId"]
        .as_str()
        .expect("a tx id")
        .to_string();
    post_queue_fun(
        &base_url,
        Some(&alice_key),
        &one_tx_body_for_fun("alice.near"),
    )
    .await;

    // alice says she sent it.
    post_sent_fun(
        &base_url,
        &alice_key,
        &sent_id,
        &json!({ "txHashes": [ { "txId": sent_tx_id, "txHash": "a-hash" } ] }),
    )
    .await;

    // the reader is signed in, so the filter has an account to be
    // about — HER account — and it drops the one she sent.
    let answer = get_list_fun(&base_url, Some(&alice_key), "hide_sent=1").await;
    assert_eq!(answer.status, 200, "body was: {}", answer.body);
    assert_eq!(answer.body["total"], 1, "the one she sent is hidden");
}

#[tokio::test]
async fn the_list_pages_with_a_cursor_and_the_cursor_is_handed_back() {
    let db = TEMP_DB::new_fun("list-cursor");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    // five batches for alice.
    for _ in 0..5 {
        post_queue_fun(&base_url, Some(&alice_key), &one_tx_body_fun()).await;
    }

    // the first page holds two, and it says there is another one.
    let first = get_list_fun(&base_url, Some(&alice_key), "limit=2").await;
    assert_eq!(first.status, 200, "body was: {}", first.body);
    assert_eq!(first.body["total"], 5, "the total is the WHOLE match");
    assert_eq!(first.body["items"].as_array().map(|a| a.len()), Some(2));
    let cursor = first.body["nextCursor"]
        .as_str()
        .expect("a cursor")
        .to_string();

    // the second page holds two more, and it does NOT repeat the first.
    let second = get_list_fun(
        &base_url,
        Some(&alice_key),
        &format!("limit=2&cursor={cursor}"),
    )
    .await;
    assert_eq!(second.status, 200, "body was: {}", second.body);
    assert_eq!(second.body["total"], 5, "the total does not change");
    assert_eq!(second.body["items"].as_array().map(|a| a.len()), Some(2));

    let first_ids: Vec<&str> = first.body["items"]
        .as_array()
        .expect("items")
        .iter()
        .map(|item| item["queueId"].as_str().unwrap_or(""))
        .collect();
    let second_ids: Vec<&str> = second.body["items"]
        .as_array()
        .expect("items")
        .iter()
        .map(|item| item["queueId"].as_str().unwrap_or(""))
        .collect();
    for id in &second_ids {
        assert!(
            !first_ids.contains(id),
            "the cursor must not repeat a queue the reader already saw"
        );
    }

    // the last page holds what is left, and it has NO cursor.
    let last_cursor = second.body["nextCursor"]
        .as_str()
        .expect("a cursor")
        .to_string();
    let third = get_list_fun(
        &base_url,
        Some(&alice_key),
        &format!("limit=2&cursor={last_cursor}"),
    )
    .await;
    assert_eq!(third.body["items"].as_array().map(|a| a.len()), Some(1));
    assert!(
        third.body["nextCursor"].is_null(),
        "the last page has no cursor — there is nothing after it"
    );
}

#[tokio::test]
async fn a_cursor_that_the_api_did_not_make_is_refused() {
    let db = TEMP_DB::new_fun("list-bad-cursor");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let answer = get_list_fun(&base_url, Some(&alice_key), "cursor=not-a-cursor").await;
    assert_eq!(answer.status, 400, "body was: {}", answer.body);
    assert_eq!(answer.body["error"]["code"], "bad_cursor");
}

#[tokio::test]
async fn the_list_filters_by_several_accounts_at_once() {
    let db = TEMP_DB::new_fun("list-many-accounts");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let bob_key = put_api_key_fun(&pool, "bob.near").await;
    let carol_key = put_api_key_fun(&pool, "carol.near").await;
    let base_url = serve_fun(pool).await;

    post_queue_fun(
        &base_url,
        Some(&alice_key),
        &one_tx_body_for_fun("alice.near"),
    )
    .await;
    post_queue_fun(&base_url, Some(&bob_key), &one_tx_body_for_fun("bob.near")).await;
    post_queue_fun(
        &base_url,
        Some(&carol_key),
        &one_tx_body_for_fun("carol.near"),
    )
    .await;

    // alice OR bob is two queues, and carol's is not one of them.
    let answer = get_list_fun(
        &base_url,
        Some(&alice_key),
        "account_id=alice.near,bob.near",
    )
    .await;
    assert_eq!(answer.status, 200, "body was: {}", answer.body);
    assert_eq!(answer.body["total"], 2);

    // and all three is three.
    let all = get_list_fun(
        &base_url,
        Some(&alice_key),
        "account_id=alice.near,bob.near,carol.near",
    )
    .await;
    assert_eq!(all.body["total"], 3);
}

#[tokio::test]
async fn the_list_filters_by_several_contracts_at_once() {
    let db = TEMP_DB::new_fun("list-many-contracts");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    post_queue_fun(&base_url, Some(&alice_key), &one_tx_body_fun()).await;

    // the queue's contract is in the body. let us read it.
    let one = one_tx_body_fun();
    let contract_id = one["transactions"][0]["contractId"]
        .as_str()
        .expect("a contract")
        .to_string();

    // that contract is in the list, and a made-up one is not.
    let hit = get_list_fun(
        &base_url,
        Some(&alice_key),
        &format!("contract_id={contract_id}"),
    )
    .await;
    assert_eq!(hit.body["total"], 1);

    let miss = get_list_fun(
        &base_url,
        Some(&alice_key),
        "contract_id=nothing.near,other.near",
    )
    .await;
    assert_eq!(miss.body["total"], 0);
}

#[tokio::test]
async fn the_facets_route_lists_the_contracts_and_the_accounts() {
    let db = TEMP_DB::new_fun("facets");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let one = one_tx_body_fun();
    let contract_id = one["transactions"][0]["contractId"]
        .as_str()
        .expect("a contract")
        .to_string();
    post_queue_fun(
        &base_url,
        Some(&alice_key),
        &one_tx_body_for_fun("alice.near"),
    )
    .await;

    // the route is OPEN — it needs no api key.
    let request = reqwest::Client::new().get(format!("{base_url}/api/shit-1170/tx-queue/facets"));
    let answer = answer_fun(request.send().await.expect("the route should answer")).await;
    assert_eq!(answer.status, 200, "body was: {}", answer.body);

    // alice is on the queue, and the contract is called.
    let accounts = answer.body["accounts"].as_array().expect("accounts");
    assert!(
        accounts
            .iter()
            .any(|one| one["accountId"] == "alice.near" && one["count"] == 1),
        "alice should be listed with one queue"
    );
    let contracts = answer.body["contracts"].as_array().expect("contracts");
    assert!(
        contracts
            .iter()
            .any(|one| one["contractId"] == contract_id && one["count"] == 1),
        "the contract should be listed with one call"
    );
}

#[tokio::test]
async fn one_queue_is_read_back_and_a_queue_that_is_not_there_is_a_404() {
    let db = TEMP_DB::new_fun("one");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let queued = post_queue_fun(&base_url, Some(&alice_key), &one_tx_body_fun()).await;
    let queue_id = queued.body["queueId"]
        .as_str()
        .expect("a queue id")
        .to_string();

    let answer = get_one_fun(&base_url, &alice_key, &queue_id).await;
    assert_eq!(answer.status, 200, "body was: {}", answer.body);
    assert_eq!(answer.body["queueId"], queue_id);
    assert_eq!(answer.body["transactions"][0]["method"], "ft_swap");

    let missing = get_one_fun(&base_url, &alice_key, "no-such-queue").await;
    assert_eq!(missing.status, 404, "body was: {}", missing.body);
    assert_eq!(missing.body["error"]["code"], "no_such_queue");
}

#[tokio::test]
async fn the_list_clamps_a_wild_limit_and_a_negative_offset() {
    // --- WHY --- a NEGATIVE limit means "no limit" in sqlite, so it has
    // to be clamped — otherwise `limit=-1` answers with every queue.
    let db = TEMP_DB::new_fun("list-limits");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let mut body = one_tx_body_fun();
    body["accounts"] = json!(["alice.near"]);
    post_queue_fun(&base_url, Some(&alice_key), &body).await;

    let negative = get_list_fun(&base_url, Some(&alice_key), "limit=-1&offset=-5").await;
    assert_eq!(negative.status, 200, "body was: {}", negative.body);
    assert_eq!(negative.body["total"], 1, "the clamp holds");

    let wild = get_list_fun(&base_url, Some(&alice_key), "limit=100000").await;
    assert_eq!(wild.status, 200, "body was: {}", wild.body);
    assert_eq!(wild.body["total"], 1);
}

#[tokio::test]
async fn the_stats_paths_are_not_swallowed_by_the_queue_id_path() {
    // --- WHY --- `/tx-queue/{queue_id}` and `/tx-queue/stats/contracts`
    // are the same shape to a naive matcher. axum matches the LITERAL
    // path first, so the stats routes have to be registered before the
    // one-batch route. this test proves it.
    let db = TEMP_DB::new_fun("stats-not-swallowed");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    for path in ["stats/contracts", "stats/methods"] {
        let request = reqwest::Client::new()
            .get(format!("{base_url}/api/shit-1170/tx-queue/{path}"))
            .header("x-api-key", &alice_key);
        let answer = answer_fun(request.send().await.expect("the route should answer")).await;
        assert_eq!(
            answer.status, 200,
            "path was {path}, body was: {}",
            answer.body
        );
        assert!(
            answer.body.get("items").is_some(),
            "path {path} must answer the stats shape, body was: {}",
            answer.body
        );
    }
}

#[tokio::test]
async fn the_contract_stats_count_the_queued_transactions() {
    let db = TEMP_DB::new_fun("stats-contracts");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    // two swaps on ref finance and one mint on an nft contract.
    let body = json!({
        "transactions": [
            { "contractId": "ref.finance.near", "method": "ft_swap" },
            { "contractId": "ref.finance.near", "method": "ft_swap" },
            { "contractId": "nft.near", "method": "nft_mint" }
        ]
    });
    let queued = post_queue_fun(&base_url, Some(&alice_key), &body).await;
    assert_eq!(queued.status, 201, "body was: {}", queued.body);

    let request = reqwest::Client::new()
        .get(format!("{base_url}/api/shit-1170/tx-queue/stats/contracts"))
        .header("x-api-key", &alice_key);
    let answer = answer_fun(request.send().await.expect("the route should answer")).await;

    assert_eq!(answer.status, 200, "body was: {}", answer.body);
    let items = answer.body["items"].as_array().expect("items");
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["contractId"], "ref.finance.near");
    assert_eq!(items[0]["count"], 2);
    assert_eq!(items[1]["contractId"], "nft.near");
    assert_eq!(items[1]["count"], 1);
}

#[tokio::test]
async fn the_method_stats_carry_the_contract_id() {
    let db = TEMP_DB::new_fun("stats-methods");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    // the SAME method on TWO contracts. they are two different
    // transactions, and the count must not blend them.
    let body = json!({
        "transactions": [
            { "contractId": "ref.finance.near", "method": "ft_swap" },
            { "contractId": "nft.near", "method": "ft_swap" }
        ]
    });
    let queued = post_queue_fun(&base_url, Some(&alice_key), &body).await;
    assert_eq!(queued.status, 201, "body was: {}", queued.body);

    let request = reqwest::Client::new()
        .get(format!("{base_url}/api/shit-1170/tx-queue/stats/methods"))
        .header("x-api-key", &alice_key);
    let answer = answer_fun(request.send().await.expect("the route should answer")).await;

    assert_eq!(answer.status, 200, "body was: {}", answer.body);
    let items = answer.body["items"].as_array().expect("items");
    assert_eq!(
        items.len(),
        2,
        "the same method on two contracts is two rows"
    );
    for item in items {
        assert_eq!(item["method"], "ft_swap");
        assert_eq!(item["count"], 1);
        assert!(
            item["contractId"].is_string(),
            "the contract id rides along"
        );
    }
}
// ========================================
// copyright 2026 by sleet.near
// ========================================

// --- WHY --- THE SKILL'S OWN PAYLOAD, POSTED TO THE REAL ROUTE: the
// FastFS skill builds its payload with its own encoder, and this route
// is what receives it. the two are separate code, so the only way to
// know they fit is to take the skill's bytes and post them here.
#[tokio::test]
async fn the_fastfs_skills_own_payload_is_queued() {
    let db = TEMP_DB::new_fun("skill-payload");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    // the exact base64 the `shit-1170-rex-fastfs` skill produces for a
    // 13-byte text file at "notes/small.txt" with the mime "text/plain".
    let args_base64 = "AA8AAABub3Rlcy9zbWFsbC50eHQBCgAAAHRleHQvcGxhaW4NAAAAaGVsbG8gZmFzdGZzCg==";

    let body = json!({
        "what": "Upload notes/small.txt to FastFS",
        "transactions": [ {
            "contractId": "fastfs.near",
            "method": "__fastdata_fastfs",
            "argsBase64": args_base64,
            "gas": "1",
            "deposit": "0"
        } ]
    });

    let answer = post_queue_fun(&base_url, Some(&alice_key), &body).await;
    assert_eq!(answer.status, 201, "body was: {}", answer.body);

    let transactions = answer.body["transactions"]
        .as_array()
        .expect("transactions");
    assert_eq!(transactions.len(), 1);
    assert_eq!(transactions[0]["argsBase64"], args_base64);
    assert_eq!(transactions[0]["method"], "__fastdata_fastfs");
    assert_eq!(transactions[0]["contractId"], "fastfs.near");
    assert_eq!(transactions[0]["gas"], "1");
    assert_eq!(transactions[0]["deposit"], "0");
}

// --- WHY --- THE KEY ACTIONS ---
// the queue carries two kinds of thing: a contract call and a KEY
// ACTION. these tests are about the second one.
//
// --- WHY --- THE `add_key` TESTS NEED NO CHAIN: an add names a key
// that does not exist yet, so there is nothing on chain to check. the
// delete DOES need the chain, and the chain is stubbed out here by
// pointing the rpc url at a server that answers nothing — the point is
// that the queue refuses the delete rather than writing it.
fn add_key_body_fun() -> Value {
    json!({
        "what": "give you a limited key",
        "transactions": [ {
            "keyAction": "add_key",
            "keyPublicKey": "ed25519:FrtUfbY74kXpBiWmqeQMtn7jKCX3zJhmwyg32ZqANj3c",
            "keyReceiverId": "ref.finance.near",
            "keyMethodNames": [ "ft_swap" ],
            "keyAllowance": "1000000000000000000000000"
        } ]
    })
}

#[tokio::test]
async fn an_add_key_is_queued_and_comes_back_with_the_key_fields() {
    let db = TEMP_DB::new_fun("add-key");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let answer = post_queue_fun(&base_url, Some(&alice_key), &add_key_body_fun()).await;

    assert_eq!(answer.status, 201, "body was: {}", answer.body);

    let transaction = &answer.body["transactions"][0];
    assert_eq!(transaction["keyAction"], "add_key");
    assert_eq!(
        transaction["keyPublicKey"],
        "ed25519:FrtUfbY74kXpBiWmqeQMtn7jKCX3zJhmwyg32ZqANj3c"
    );
    assert_eq!(transaction["keyReceiverId"], "ref.finance.near");
    assert_eq!(transaction["keyMethodNames"][0], "ft_swap");
    assert_eq!(transaction["keyAllowance"], "1000000000000000000000000");

    // --- WHY --- THE CONTRACT FIELDS ARE EMPTY AND NOT MISSING: the
    // columns are NOT NULL and cannot be nulled without a table
    // rebuild, so a key action writes the empty string and the
    // `keyAction` is what says what the transaction is.
    assert_eq!(transaction["contractId"], "");
    assert_eq!(transaction["method"], "");
}

#[tokio::test]
async fn a_contract_call_carries_the_key_fields_as_null() {
    // --- WHY --- THE KEY FIELDS ARE ALWAYS ON THE WIRE: a client reads
    // one shape for every transaction, and a field that is sometimes
    // missing is a field every client has to check for.
    let db = TEMP_DB::new_fun("call-key-null");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let answer = post_queue_fun(&base_url, Some(&alice_key), &one_tx_body_fun()).await;
    assert_eq!(answer.status, 201, "body was: {}", answer.body);

    let transaction = &answer.body["transactions"][0];
    assert!(transaction["keyAction"].is_null());
    assert!(transaction["keyPublicKey"].is_null());
    assert!(transaction["keyReceiverId"].is_null());
    assert!(transaction["keyMethodNames"].is_null());
    assert!(transaction["keyAllowance"].is_null());
}

#[tokio::test]
async fn a_key_action_that_is_also_a_contract_call_is_refused() {
    let db = TEMP_DB::new_fun("key-and-call");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let body = json!({
        "transactions": [ {
            "keyAction": "add_key",
            "keyPublicKey": "ed25519:FrtUfbY74kXpBiWmqeQMtn7jKCX3zJhmwyg32ZqANj3c",
            "keyReceiverId": "ref.finance.near",
            "contractId": "ref.finance.near",
            "method": "ft_swap"
        } ]
    });

    let answer = post_queue_fun(&base_url, Some(&alice_key), &body).await;

    assert_eq!(answer.status, 400, "body was: {}", answer.body);
    assert_eq!(answer.body["error"]["code"], "key_action_and_contract_call");
}

#[tokio::test]
async fn a_key_action_with_no_public_key_is_refused() {
    let db = TEMP_DB::new_fun("key-no-public-key");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let body = json!({
        "transactions": [ {
            "keyAction": "delete_key"
        } ]
    });

    let answer = post_queue_fun(&base_url, Some(&alice_key), &body).await;

    assert_eq!(answer.status, 400, "body was: {}", answer.body);
    assert_eq!(answer.body["error"]["code"], "no_key_public_key");
}

#[tokio::test]
async fn an_add_key_with_no_receiver_is_refused() {
    let db = TEMP_DB::new_fun("key-no-receiver");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let body = json!({
        "transactions": [ {
            "keyAction": "add_key",
            "keyPublicKey": "ed25519:FrtUfbY74kXpBiWmqeQMtn7jKCX3zJhmwyg32ZqANj3c"
        } ]
    });

    let answer = post_queue_fun(&base_url, Some(&alice_key), &body).await;

    assert_eq!(answer.status, 400, "body was: {}", answer.body);
    assert_eq!(answer.body["error"]["code"], "no_key_receiver_id");
}

#[tokio::test]
async fn a_key_action_that_is_neither_add_nor_delete_is_refused() {
    let db = TEMP_DB::new_fun("key-bad-action");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let body = json!({
        "transactions": [ {
            "keyAction": "burn_the_keys",
            "keyPublicKey": "ed25519:FrtUfbY74kXpBiWmqeQMtn7jKCX3zJhmwyg32ZqANj3c"
        } ]
    });

    let answer = post_queue_fun(&base_url, Some(&alice_key), &body).await;

    assert_eq!(answer.status, 400, "body was: {}", answer.body);
    assert_eq!(answer.body["error"]["code"], "bad_key_action");
}

#[tokio::test]
async fn a_delete_key_with_a_public_key_that_is_not_a_key_is_refused() {
    // --- WHY --- THIS TEST NEVER TOUCHES THE CHAIN: the public key is
    // parsed BEFORE the chain is asked, and a string that is not a NEAR
    // public key is refused right there. that is the right order — a
    // request that cannot be true is refused without a round trip.
    let db = TEMP_DB::new_fun("delete-key-bad-key");
    let pool = db.pool_fun().await;
    let alice_key = put_api_key_fun(&pool, "alice.near").await;
    let base_url = serve_fun(pool).await;

    let body = json!({
        "transactions": [ {
            "keyAction": "delete_key",
            "keyPublicKey": "not-a-public-key"
        } ]
    });

    let answer = post_queue_fun(&base_url, Some(&alice_key), &body).await;

    assert_eq!(answer.status, 400, "body was: {}", answer.body);
    assert_eq!(answer.body["error"]["code"], "bad_key_public_key");

    // --- WHY --- NOTHING WAS WRITTEN: the check runs before the write,
    // so a refused batch leaves no queue behind.
    let list = get_list_fun(&base_url, Some(&alice_key), "").await;
    assert_eq!(list.body["total"], 0, "the refused batch left nothing");
}
