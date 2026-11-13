// ========================================
// the db admin bins, tested against a real sqlite file.
//
// the db is a temp file, not an in-memory one, because the production db
// is a file and the two can differ: WAL, the journal mode, and the way
// sqlite writes pages are all file behaviours.
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
        path.push(format!(
            "shit-1170-test-vault-bin-{name}-{:?}.db",
            std::process::id()
        ));
        // a leftover from a previous run would make the test read old rows.
        let _ = std::fs::remove_file(&path);
        TEMP_DB { path }
    }

    // a db with the schema in it.
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

// the day in ms.
const DAY_MS: i64 = 24 * 60 * 60 * 1000;

// put a queue with one transaction and one account row in the db.
//
// --- WHY --- THE HASH IS A PARAMETER: a queue with a hash is HISTORY
// and a queue without one is not, and the rule about history is the
// thing most of these tests are about.
async fn seed_queue_fun(
    pool: &sqlx::SqlitePool,
    queue_id: &str,
    tx_id: &str,
    account_id: &str,
    created_at: i64,
    tx_hash: Option<&str>,
) {
    sqlx::query(
        "INSERT INTO tx_queue (queue_id, what, \"when\", why, when_at, created_at, updated_at)
         VALUES (?1, 'a swap', NULL, NULL, NULL, ?2, ?2)",
    )
    .bind(queue_id)
    .bind(created_at)
    .execute(pool)
    .await
    .expect("the queue should go in");

    sqlx::query(
        "INSERT INTO tx_queue_transactions
         (tx_id, queue_id, position, contract_id, method, args, gas, deposit, created_at, updated_at)
         VALUES (?1, ?2, 0, 'ref.finance.near', 'swap', '{}', NULL, NULL, ?3, ?3)",
    )
    .bind(tx_id)
    .bind(queue_id)
    .bind(created_at)
    .execute(pool)
    .await
    .expect("the transaction should go in");

    sqlx::query(
        "INSERT INTO tx_queue_accounts
         (queue_id, tx_id, account_id, status, tx_hash, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'pending', ?4, ?5, ?5)",
    )
    .bind(queue_id)
    .bind(tx_id)
    .bind(account_id)
    .bind(tx_hash)
    .bind(created_at)
    .execute(pool)
    .await
    .expect("the account row should go in");
}

// how many rows a table holds.
async fn count_fun(pool: &sqlx::SqlitePool, table: &str) -> i64 {
    let statement = match table {
        "tx_queue" => "SELECT COUNT(*) FROM tx_queue",
        "tx_queue_transactions" => "SELECT COUNT(*) FROM tx_queue_transactions",
        "tx_queue_accounts" => "SELECT COUNT(*) FROM tx_queue_accounts",
        "swap_queue" => "SELECT COUNT(*) FROM swap_queue",
        "swap_queue_accounts" => "SELECT COUNT(*) FROM swap_queue_accounts",
        _ => panic!("no such table: {table}"),
    };
    let row: (i64,) = sqlx::query_as(statement)
        .fetch_one(pool)
        .await
        .expect("the count should read");
    row.0
}

// ========================================
// --- THE ARGS --------------------------
// the parser reads the flags the bins take.
#[tokio::test]
async fn db_admin_args_reads_the_flags_fun() {
    use crate::backend::logic::db::bin::db_admin_args_fun::{
        db_admin_args_fun, db_admin_flag_fun, db_admin_id_list_fun, db_admin_switch_fun,
    };

    let words: Vec<String> = [
        "--db",
        "/tmp/vault.db",
        "--older-than",
        "30",
        "--never-sent",
        "--queue-ids",
        "a,b , c",
    ]
    .iter()
    .map(|word| word.to_string())
    .collect();

    let args = db_admin_args_fun(&words);

    assert_eq!(
        args.db_path.as_ref().map(|path| path.display().to_string()),
        Some(String::from("/tmp/vault.db"))
    );
    assert_eq!(db_admin_flag_fun(&args, "older-than"), Some("30"));
    assert!(db_admin_switch_fun(&args, "never-sent"));
    assert!(!db_admin_switch_fun(&args, "include-sent"));
    assert_eq!(
        db_admin_id_list_fun(&args, "queue-ids"),
        vec![String::from("a"), String::from("b"), String::from("c")]
    );
}

// the parser takes the value in the flag too.
#[tokio::test]
async fn db_admin_args_takes_the_value_in_the_flag_fun() {
    use crate::backend::logic::db::bin::db_admin_args_fun::{db_admin_args_fun, db_admin_flag_fun};

    let words: Vec<String> = ["--db=/tmp/vault.db", "--older-than=30"]
        .iter()
        .map(|word| word.to_string())
        .collect();

    let args = db_admin_args_fun(&words);

    assert_eq!(
        args.db_path.as_ref().map(|path| path.display().to_string()),
        Some(String::from("/tmp/vault.db"))
    );
    assert_eq!(db_admin_flag_fun(&args, "older-than"), Some("30"));
}

// a flag with nothing after it is a switch and not a flag with an
// empty value.
#[tokio::test]
async fn db_admin_args_reads_a_bare_flag_as_a_switch_fun() {
    use crate::backend::logic::db::bin::db_admin_args_fun::{
        db_admin_args_fun, db_admin_flag_fun, db_admin_switch_fun,
    };

    let words: Vec<String> = ["--confirm"].iter().map(|word| word.to_string()).collect();

    let args = db_admin_args_fun(&words);

    assert!(db_admin_switch_fun(&args, "confirm"));
    assert_eq!(db_admin_flag_fun(&args, "confirm"), None);
}

// ========================================
// --- THE PRUNE -------------------------
// a prune by age takes the old queue and leaves the new one.
#[tokio::test]
async fn db_admin_prune_takes_the_old_queue_fun() {
    use crate::backend::logic::db::bin::db_admin_prune_fun::{
        DB_ADMIN_PRUNE_FILTERS, db_admin_prune_delete_fun, db_admin_prune_queue_plan_fun,
    };

    let db = TEMP_DB::new_fun("prune-old");
    let pool = db.pool_fun().await;
    let now = crate::backend::logic::db::bin::db_admin_now_fun::now_ms_fun();

    seed_queue_fun(
        &pool,
        "old",
        "tx-old",
        "alice.near",
        now - 40 * DAY_MS,
        None,
    )
    .await;
    seed_queue_fun(&pool, "new", "tx-new", "alice.near", now, None).await;

    let filters = DB_ADMIN_PRUNE_FILTERS {
        older_than_days: Some(30),
        ..DB_ADMIN_PRUNE_FILTERS::default()
    };

    let plan = db_admin_prune_queue_plan_fun(&pool, &filters)
        .await
        .expect("the plan should read");
    assert_eq!(plan, vec![String::from("old")]);

    let plan = crate::backend::logic::db::bin::db_admin_prune_fun::DB_ADMIN_PRUNE_PLAN {
        queue_ids: plan,
        ..Default::default()
    };
    db_admin_prune_delete_fun(&pool, &plan)
        .await
        .expect("the delete should run");

    // the queue went, and so did its transaction and its account row.
    assert_eq!(count_fun(&pool, "tx_queue").await, 1);
    assert_eq!(count_fun(&pool, "tx_queue_transactions").await, 1);
    assert_eq!(count_fun(&pool, "tx_queue_accounts").await, 1);
}

// a prune leaves a queue that was sent alone.
#[tokio::test]
async fn db_admin_prune_leaves_the_history_alone_fun() {
    use crate::backend::logic::db::bin::db_admin_prune_fun::{
        DB_ADMIN_PRUNE_FILTERS, db_admin_prune_delete_fun, db_admin_prune_queue_plan_fun,
    };

    let db = TEMP_DB::new_fun("prune-history");
    let pool = db.pool_fun().await;
    let now = crate::backend::logic::db::bin::db_admin_now_fun::now_ms_fun();

    seed_queue_fun(
        &pool,
        "sent",
        "tx-sent",
        "bob.near",
        now - 40 * DAY_MS,
        Some("HASH"),
    )
    .await;

    let filters = DB_ADMIN_PRUNE_FILTERS {
        older_than_days: Some(30),
        ..DB_ADMIN_PRUNE_FILTERS::default()
    };

    let plan = db_admin_prune_queue_plan_fun(&pool, &filters)
        .await
        .expect("the plan should read");
    assert!(plan.is_empty(), "a sent queue should not match");

    let plan = crate::backend::logic::db::bin::db_admin_prune_fun::DB_ADMIN_PRUNE_PLAN {
        queue_ids: plan,
        ..Default::default()
    };
    db_admin_prune_delete_fun(&pool, &plan)
        .await
        .expect("the delete should run");

    assert_eq!(count_fun(&pool, "tx_queue").await, 1);
    assert_eq!(count_fun(&pool, "tx_queue_accounts").await, 1);
}

// the include-sent flag says to take the history too.
#[tokio::test]
async fn db_admin_prune_takes_the_history_with_the_flag_fun() {
    use crate::backend::logic::db::bin::db_admin_prune_fun::{
        DB_ADMIN_PRUNE_FILTERS, db_admin_prune_delete_fun, db_admin_prune_queue_plan_fun,
    };

    let db = TEMP_DB::new_fun("prune-include-sent");
    let pool = db.pool_fun().await;
    let now = crate::backend::logic::db::bin::db_admin_now_fun::now_ms_fun();

    seed_queue_fun(
        &pool,
        "sent",
        "tx-sent",
        "bob.near",
        now - 40 * DAY_MS,
        Some("HASH"),
    )
    .await;

    let filters = DB_ADMIN_PRUNE_FILTERS {
        older_than_days: Some(30),
        include_sent: true,
        ..DB_ADMIN_PRUNE_FILTERS::default()
    };

    let plan = db_admin_prune_queue_plan_fun(&pool, &filters)
        .await
        .expect("the plan should read");
    assert_eq!(plan, vec![String::from("sent")]);

    let plan = crate::backend::logic::db::bin::db_admin_prune_fun::DB_ADMIN_PRUNE_PLAN {
        queue_ids: plan,
        ..Default::default()
    };
    db_admin_prune_delete_fun(&pool, &plan)
        .await
        .expect("the delete should run");

    assert_eq!(count_fun(&pool, "tx_queue").await, 0);
    assert_eq!(count_fun(&pool, "tx_queue_accounts").await, 0);
}

// a prune by id takes that one queue and no other.
#[tokio::test]
async fn db_admin_prune_takes_the_queue_it_was_named_fun() {
    use crate::backend::logic::db::bin::db_admin_prune_fun::{
        DB_ADMIN_PRUNE_FILTERS, db_admin_prune_queue_plan_fun,
    };

    let db = TEMP_DB::new_fun("prune-by-id");
    let pool = db.pool_fun().await;
    let now = crate::backend::logic::db::bin::db_admin_now_fun::now_ms_fun();

    seed_queue_fun(&pool, "one", "tx-one", "alice.near", now, None).await;
    seed_queue_fun(&pool, "two", "tx-two", "alice.near", now, None).await;

    let filters = DB_ADMIN_PRUNE_FILTERS {
        queue_ids: vec![String::from("two")],
        ..DB_ADMIN_PRUNE_FILTERS::default()
    };

    let plan = db_admin_prune_queue_plan_fun(&pool, &filters)
        .await
        .expect("the plan should read");
    assert_eq!(plan, vec![String::from("two")]);
}

// the swaps are only part of the plan when they were asked for.
#[tokio::test]
async fn db_admin_prune_leaves_the_swaps_alone_unless_asked_fun() {
    use crate::backend::logic::db::bin::db_admin_prune_fun::{
        DB_ADMIN_PRUNE_FILTERS, db_admin_prune_queue_plan_fun, db_admin_prune_swap_plan_fun,
    };

    let db = TEMP_DB::new_fun("prune-swaps");
    let pool = db.pool_fun().await;
    let now = crate::backend::logic::db::bin::db_admin_now_fun::now_ms_fun();

    seed_queue_fun(
        &pool,
        "old",
        "tx-old",
        "alice.near",
        now - 40 * DAY_MS,
        None,
    )
    .await;

    sqlx::query(
        "INSERT INTO swap_queue
         (swap_id, what, \"when\", why, when_at, token_in, amount_in, token_out,
          amount_out, dex_id, created_at, updated_at)
         VALUES ('swap-old', NULL, NULL, NULL, NULL, 'usdc.near', '1', 'wrap.near',
                 NULL, NULL, ?1, ?1)",
    )
    .bind(now - 40 * DAY_MS)
    .execute(&pool)
    .await
    .expect("the swap should go in");

    // the queue is asked for by its age. the swap is not in the plan.
    let filters = DB_ADMIN_PRUNE_FILTERS {
        older_than_days: Some(30),
        ..DB_ADMIN_PRUNE_FILTERS::default()
    };
    assert_eq!(
        db_admin_prune_queue_plan_fun(&pool, &filters)
            .await
            .expect("the plan should read"),
        vec![String::from("old")]
    );
    assert!(
        db_admin_prune_swap_plan_fun(&pool, &filters)
            .await
            .expect("the plan should read")
            .is_empty(),
        "the swaps should not be in the plan"
    );

    // with the flag, the swap is in the plan too.
    let filters = DB_ADMIN_PRUNE_FILTERS {
        older_than_days: Some(30),
        include_swaps: true,
        ..DB_ADMIN_PRUNE_FILTERS::default()
    };
    assert_eq!(
        db_admin_prune_swap_plan_fun(&pool, &filters)
            .await
            .expect("the plan should read"),
        vec![String::from("swap-old")]
    );
}

// the never-sent filter takes the queues nobody sent and leaves the
// ones somebody did.
#[tokio::test]
async fn db_admin_prune_never_sent_leaves_the_sent_ones_fun() {
    use crate::backend::logic::db::bin::db_admin_prune_fun::{
        DB_ADMIN_PRUNE_FILTERS, db_admin_prune_queue_plan_fun,
    };

    let db = TEMP_DB::new_fun("prune-never-sent");
    let pool = db.pool_fun().await;
    let now = crate::backend::logic::db::bin::db_admin_now_fun::now_ms_fun();

    seed_queue_fun(&pool, "never", "tx-never", "alice.near", now, None).await;
    seed_queue_fun(&pool, "sent", "tx-sent", "bob.near", now, Some("HASH")).await;

    let filters = DB_ADMIN_PRUNE_FILTERS {
        never_sent: true,
        ..DB_ADMIN_PRUNE_FILTERS::default()
    };

    let plan = db_admin_prune_queue_plan_fun(&pool, &filters)
        .await
        .expect("the plan should read");
    assert_eq!(plan, vec![String::from("never")]);
}

// the account filter takes the queues that account is on.
#[tokio::test]
async fn db_admin_prune_takes_the_accounts_queues_fun() {
    use crate::backend::logic::db::bin::db_admin_prune_fun::{
        DB_ADMIN_PRUNE_FILTERS, db_admin_prune_queue_plan_fun,
    };

    let db = TEMP_DB::new_fun("prune-account");
    let pool = db.pool_fun().await;
    let now = crate::backend::logic::db::bin::db_admin_now_fun::now_ms_fun();

    seed_queue_fun(&pool, "alice-one", "tx-a1", "alice.near", now, None).await;
    seed_queue_fun(&pool, "bob-one", "tx-b1", "bob.near", now, None).await;

    let filters = DB_ADMIN_PRUNE_FILTERS {
        account_id: Some(String::from("alice.near")),
        ..DB_ADMIN_PRUNE_FILTERS::default()
    };

    let plan = db_admin_prune_queue_plan_fun(&pool, &filters)
        .await
        .expect("the plan should read");
    assert_eq!(plan, vec![String::from("alice-one")]);
}

// ========================================
// --- THE EXPORT AND THE IMPORT --------
// the export comes back into a vault as the same rows.
#[tokio::test]
async fn db_admin_export_and_import_round_trip_fun() {
    use crate::backend::logic::db::bin::db_admin_export_fun::{
        db_admin_export_fun, db_admin_import_fun,
    };

    let db = TEMP_DB::new_fun("export");
    let pool = db.pool_fun().await;
    let now = crate::backend::logic::db::bin::db_admin_now_fun::now_ms_fun();

    seed_queue_fun(&pool, "one", "tx-one", "alice.near", now, None).await;
    seed_queue_fun(&pool, "two", "tx-two", "bob.near", now, Some("HASH")).await;

    let export = db_admin_export_fun(&pool, false)
        .await
        .expect("the export should read");

    // a fresh db, with the same schema.
    let other = TEMP_DB::new_fun("import");
    let other_pool = other.pool_fun().await;

    let (queues, transactions, accounts, swaps, swap_accounts) =
        db_admin_import_fun(&other_pool, &export)
            .await
            .expect("the import should run");
    assert_eq!(queues, 2);
    assert_eq!(transactions, 2);
    assert_eq!(accounts, 2);
    assert_eq!(swaps, 0);
    assert_eq!(swap_accounts, 0);

    assert_eq!(count_fun(&other_pool, "tx_queue").await, 2);
    assert_eq!(count_fun(&other_pool, "tx_queue_transactions").await, 2);
    assert_eq!(count_fun(&other_pool, "tx_queue_accounts").await, 2);

    // the hash came with it.
    let row = sqlx::query("SELECT tx_hash FROM tx_queue_accounts WHERE account_id = 'bob.near'")
        .fetch_one(&other_pool)
        .await
        .expect("the row should be there");
    assert_eq!(row.get::<String, _>("tx_hash"), String::from("HASH"));
}

// the api keys are left out of the export unless they are asked for.
#[tokio::test]
async fn db_admin_export_leaves_the_api_keys_out_fun() {
    use crate::backend::logic::db::bin::db_admin_export_fun::db_admin_export_fun;

    let db = TEMP_DB::new_fun("export-keys");
    let pool = db.pool_fun().await;

    sqlx::query(
        "INSERT INTO api_keys (account_id, api_key, api_key_hash, created_at, updated_at)
         VALUES ('alice.near', 'the-key', 'the-hash', 1, 1)",
    )
    .execute(&pool)
    .await
    .expect("the key should go in");

    let export = db_admin_export_fun(&pool, false)
        .await
        .expect("the export should read");
    assert!(export.get("api_keys").is_none());

    let export = db_admin_export_fun(&pool, true)
        .await
        .expect("the export should read");
    assert_eq!(
        export
            .get("api_keys")
            .and_then(|keys| keys.as_array())
            .map(|keys| keys.len()),
        Some(1)
    );
}

// the import never brings an api key back.
#[tokio::test]
async fn db_admin_import_does_not_bring_the_api_keys_back_fun() {
    use crate::backend::logic::db::bin::db_admin_export_fun::{
        db_admin_export_fun, db_admin_import_fun,
    };

    let db = TEMP_DB::new_fun("export-with-keys");
    let pool = db.pool_fun().await;

    sqlx::query(
        "INSERT INTO api_keys (account_id, api_key, api_key_hash, created_at, updated_at)
         VALUES ('alice.near', 'the-key', 'the-hash', 1, 1)",
    )
    .execute(&pool)
    .await
    .expect("the key should go in");

    let export = db_admin_export_fun(&pool, true)
        .await
        .expect("the export should read");

    let other = TEMP_DB::new_fun("import-keys");
    let other_pool = other.pool_fun().await;
    db_admin_import_fun(&other_pool, &export)
        .await
        .expect("the import should run");

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM api_keys")
        .fetch_one(&other_pool)
        .await
        .expect("the count should read");
    assert_eq!(count.0, 0, "the api keys should not come back");
}

// ========================================
// --- THE BACKUP ------------------------
// the backup is a whole db that opens on its own.
#[tokio::test]
async fn db_admin_backup_makes_a_db_that_opens_fun() {
    use crate::backend::logic::db::bin::db_admin_backup_fun::db_admin_backup_fun;
    use crate::backend::logic::db::bin::db_admin_db_fun::db_admin_db_has_tables_fun;

    let db = TEMP_DB::new_fun("backup");
    let pool = db.pool_fun().await;
    let now = crate::backend::logic::db::bin::db_admin_now_fun::now_ms_fun();

    seed_queue_fun(&pool, "one", "tx-one", "alice.near", now, None).await;

    let destination = db_admin_backup_fun(&pool, &db.path)
        .await
        .expect("the backup should be taken");
    assert!(destination.exists(), "the backup file should be there");

    // the copy opens on its own and holds the same rows.
    let copy = crate::backend::logic::db::bin::db_admin_db_fun::db_admin_db_open_fun(&destination)
        .await
        .expect("the copy should open");
    assert!(
        db_admin_db_has_tables_fun(&copy)
            .await
            .expect("the tables should read"),
        "the copy should have the tables"
    );
    assert_eq!(count_fun(&copy, "tx_queue").await, 1);

    let _ = std::fs::remove_file(&destination);
}

// ========================================
// --- THE DB OPEN -----------------------
// a bin does not make a db where there is none.
#[tokio::test]
async fn db_admin_db_open_does_not_make_a_db_fun() {
    use crate::backend::logic::db::bin::db_admin_db_fun::db_admin_db_open_fun;

    let db = TEMP_DB::new_fun("no-db");
    let result = db_admin_db_open_fun(&db.path).await;
    assert!(result.is_err(), "a missing db should be an error");
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
