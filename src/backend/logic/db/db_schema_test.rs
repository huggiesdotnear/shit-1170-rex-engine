// ========================================
// the db schema, tested against a real sqlite file.
//
// the db is a temp file, not an in-memory one, because the production db
// is a file and the two can differ: WAL, the journal mode, and the way
// sqlite writes pages are all file behaviours.
use sqlx::Row;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
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
            "shit-1170-test-schema-{name}-{:?}.db",
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

// the column names of a table, in the order the table declares them.
//
// --- WHY --- the table name is picked with a match and not a format!:
// sqlx 0.9 wants a `&'static str` for a statement, and a formatted
// string is not one. the match also means only the four known tables can
// be asked about.
async fn columns_of_fun(pool: &sqlx::SqlitePool, table: &str) -> Vec<String> {
    let statement = match table {
        "api_keys" => "SELECT name FROM pragma_table_info('api_keys')",
        "tx_queue" => "SELECT name FROM pragma_table_info('tx_queue')",
        "tx_queue_transactions" => "SELECT name FROM pragma_table_info('tx_queue_transactions')",
        "tx_queue_accounts" => "SELECT name FROM pragma_table_info('tx_queue_accounts')",
        "swap_queue" => "SELECT name FROM pragma_table_info('swap_queue')",
        "swap_queue_accounts" => "SELECT name FROM pragma_table_info('swap_queue_accounts')",
        _ => panic!("no such table: {table}"),
    };

    let rows = sqlx::query(statement)
        .fetch_all(pool)
        .await
        .expect("the columns should read");
    rows.iter()
        .map(|row| row.get::<String, _>("name"))
        .collect()
}

// every table name in the db.
async fn table_names_fun(pool: &sqlx::SqlitePool) -> Vec<String> {
    let rows = sqlx::query("SELECT name FROM sqlite_master WHERE type = 'table'")
        .fetch_all(pool)
        .await
        .expect("the tables should read");
    rows.iter()
        .map(|row| row.get::<String, _>("name"))
        .collect()
}

// ========================================
// --- TESTS -----------------------------
#[tokio::test]
async fn the_db_opens_and_every_table_is_there() {
    let db = TEMP_DB::new_fun("tables");
    let pool = db.pool_fun().await;

    let tables = table_names_fun(&pool).await;

    for name in [
        "api_keys",
        "tx_queue",
        "tx_queue_transactions",
        "tx_queue_accounts",
        "swap_queue",
        "swap_queue_accounts",
    ] {
        assert!(
            tables.contains(&name.to_string()),
            "the {name} table should exist"
        );
    }
}

#[tokio::test]
async fn the_api_keys_table_has_its_columns() {
    let db = TEMP_DB::new_fun("api-keys");
    let pool = db.pool_fun().await;

    assert_eq!(
        columns_of_fun(&pool, "api_keys").await,
        vec![
            "account_id",
            "api_key",
            "api_key_hash",
            "created_at",
            "updated_at"
        ]
    );
}

#[tokio::test]
async fn the_tx_queue_table_has_its_columns() {
    let db = TEMP_DB::new_fun("tx-queue");
    let pool = db.pool_fun().await;

    assert_eq!(
        columns_of_fun(&pool, "tx_queue").await,
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

#[tokio::test]
async fn the_tx_queue_transactions_table_has_its_columns() {
    let db = TEMP_DB::new_fun("tx-queue-transactions");
    let pool = db.pool_fun().await;

    assert_eq!(
        columns_of_fun(&pool, "tx_queue_transactions").await,
        vec![
            "tx_id",
            "queue_id",
            "position",
            "contract_id",
            "method",
            "args",
            "args_base64",
            "gas",
            "deposit",
            "key_action",
            "key_public_key",
            "key_receiver_id",
            "key_method_names",
            "key_allowance",
            "created_at",
            "updated_at"
        ]
    );
}

#[tokio::test]
async fn the_tx_queue_accounts_table_has_its_columns() {
    let db = TEMP_DB::new_fun("tx-queue-accounts");
    let pool = db.pool_fun().await;

    assert_eq!(
        columns_of_fun(&pool, "tx_queue_accounts").await,
        vec![
            "queue_id",
            "tx_id",
            "account_id",
            "status",
            "tx_hash",
            "created_at",
            "updated_at"
        ]
    );
}

#[tokio::test]
async fn the_indexes_are_there() {
    let db = TEMP_DB::new_fun("indexes");
    let pool = db.pool_fun().await;

    let rows = sqlx::query("SELECT name FROM sqlite_master WHERE type = 'index'")
        .fetch_all(&pool)
        .await
        .expect("the indexes should read");
    let indexes: Vec<String> = rows
        .iter()
        .map(|row| row.get::<String, _>("name"))
        .collect();

    for name in [
        "api_keys_api_key_hash_idx",
        "tx_queue_transactions_contract_id_idx",
        "tx_queue_transactions_method_idx",
        "tx_queue_transactions_queue_id_idx",
        "tx_queue_transactions_created_at_idx",
        "tx_queue_accounts_account_id_idx",
        "tx_queue_accounts_queue_id_idx",
        "tx_queue_accounts_status_idx",
        "swap_queue_accounts_account_id_idx",
        "swap_queue_accounts_swap_id_idx",
        "swap_queue_accounts_status_idx",
    ] {
        assert!(
            indexes.contains(&name.to_string()),
            "the {name} index should exist"
        );
    }
}

#[tokio::test]
async fn the_swap_queue_table_has_its_columns() {
    let db = TEMP_DB::new_fun("swap-queue");
    let pool = db.pool_fun().await;

    assert_eq!(
        columns_of_fun(&pool, "swap_queue").await,
        vec![
            "swap_id",
            "what",
            "when",
            "why",
            "when_at",
            "token_in",
            "amount_in",
            "token_out",
            "amount_out",
            "dex_id",
            "created_at",
            "updated_at"
        ]
    );
}

#[tokio::test]
async fn the_swap_queue_accounts_table_has_its_columns() {
    let db = TEMP_DB::new_fun("swap-queue-accounts");
    let pool = db.pool_fun().await;

    assert_eq!(
        columns_of_fun(&pool, "swap_queue_accounts").await,
        vec![
            "swap_id",
            "account_id",
            "status",
            "tx_hash",
            "amount_out",
            "created_at",
            "updated_at"
        ]
    );
}

#[tokio::test]
async fn a_swap_account_row_without_its_swap_is_refused() {
    let db = TEMP_DB::new_fun("swap-bad-fk");
    let pool = db.pool_fun().await;

    let result = sqlx::query(
        "INSERT INTO swap_queue_accounts
           (swap_id, account_id, status, created_at, updated_at)
         VALUES (?1, ?2, 'pending', 1, 1)",
    )
    .bind("swap-does-not-exist")
    .bind("alice.near")
    .execute(&pool)
    .await;

    assert!(
        result.is_err(),
        "an account row pointing at a missing swap must fail"
    );
}

#[tokio::test]
async fn the_same_account_on_the_same_swap_twice_is_refused() {
    let db = TEMP_DB::new_fun("swap-dup-account");
    let pool = db.pool_fun().await;

    sqlx::query(
        "INSERT INTO swap_queue
           (swap_id, token_in, amount_in, token_out, created_at, updated_at)
         VALUES (?1, 'near', '1', 'usdt.tether-token.near', 1, 1)",
    )
    .bind("swap-1")
    .execute(&pool)
    .await
    .expect("the swap should save");

    let insert = || {
        sqlx::query(
            "INSERT INTO swap_queue_accounts
               (swap_id, account_id, status, created_at, updated_at)
             VALUES (?1, ?2, 'pending', 1, 1)",
        )
        .bind("swap-1")
        .bind("alice.near")
        .execute(&pool)
    };

    insert().await.expect("the first account row should save");

    let second = insert().await;
    assert!(
        second.is_err(),
        "the same account on the same swap twice must fail"
    );
}

#[tokio::test]
async fn deleting_a_swap_takes_its_account_rows_with_it() {
    let db = TEMP_DB::new_fun("swap-cascade");
    let pool = db.pool_fun().await;

    sqlx::query(
        "INSERT INTO swap_queue
           (swap_id, token_in, amount_in, token_out, created_at, updated_at)
         VALUES (?1, 'near', '1', 'usdt.tether-token.near', 1, 1)",
    )
    .bind("swap-1")
    .execute(&pool)
    .await
    .expect("the swap should save");

    sqlx::query(
        "INSERT INTO swap_queue_accounts
           (swap_id, account_id, status, created_at, updated_at)
         VALUES (?1, ?2, 'pending', 1, 1)",
    )
    .bind("swap-1")
    .bind("alice.near")
    .execute(&pool)
    .await
    .expect("the account row should save");

    sqlx::query("DELETE FROM swap_queue WHERE swap_id = ?1")
        .bind("swap-1")
        .execute(&pool)
        .await
        .expect("the swap should delete");

    let accounts: (i64,) = sqlx::query_as("SELECT COUNT(1) FROM swap_queue_accounts")
        .fetch_one(&pool)
        .await
        .expect("the count should read");
    assert_eq!(accounts.0, 0, "the account row should be cascaded away");
}

#[tokio::test]
async fn a_swap_account_row_starts_out_pending() {
    let db = TEMP_DB::new_fun("swap-default-status");
    let pool = db.pool_fun().await;

    sqlx::query(
        "INSERT INTO swap_queue
           (swap_id, token_in, amount_in, token_out, created_at, updated_at)
         VALUES (?1, 'near', '1', 'usdt.tether-token.near', 1, 1)",
    )
    .bind("swap-1")
    .execute(&pool)
    .await
    .expect("the swap should save");

    // the status is left out. it has to default to pending.
    sqlx::query(
        "INSERT INTO swap_queue_accounts
           (swap_id, account_id, created_at, updated_at)
         VALUES (?1, ?2, 1, 1)",
    )
    .bind("swap-1")
    .bind("alice.near")
    .execute(&pool)
    .await
    .expect("the account row should save");

    let status: (String,) = sqlx::query_as(
        "SELECT status FROM swap_queue_accounts WHERE swap_id = ?1 AND account_id = ?2",
    )
    .bind("swap-1")
    .bind("alice.near")
    .fetch_one(&pool)
    .await
    .expect("the status should read");
    assert_eq!(status.0, "pending");
}

#[tokio::test]
async fn opening_twice_is_fine() {
    let db = TEMP_DB::new_fun("twice");
    // the db is opened twice: the second open finds the tables already
    // there and does nothing.
    let _pool = db.pool_fun().await;
    let pool = db.pool_fun().await;

    let tables = table_names_fun(&pool).await;
    assert!(tables.contains(&"api_keys".to_string()));
    assert!(tables.contains(&"tx_queue".to_string()));
}

#[tokio::test]
async fn there_is_no_migration_table() {
    let db = TEMP_DB::new_fun("no-migrations");
    let pool = db.pool_fun().await;

    let tables = table_names_fun(&pool).await;
    assert!(
        !tables.contains(&"_sqlx_migrations".to_string()),
        "there is no migration system, so there is no bookkeeping table"
    );
}

#[tokio::test]
async fn an_account_row_without_its_transaction_is_refused() {
    let db = TEMP_DB::new_fun("bad-fk");
    let pool = db.pool_fun().await;

    let result = sqlx::query(
        "INSERT INTO tx_queue_accounts
           (queue_id, tx_id, account_id, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'pending', 1, 1)",
    )
    .bind("queue-1")
    .bind("tx-does-not-exist")
    .bind("alice.near")
    .execute(&pool)
    .await;

    assert!(
        result.is_err(),
        "an account row pointing at a missing transaction must fail"
    );
}

#[tokio::test]
async fn a_transaction_without_its_queue_is_refused() {
    let db = TEMP_DB::new_fun("bad-fk-queue");
    let pool = db.pool_fun().await;

    let result = sqlx::query(
        "INSERT INTO tx_queue_transactions
           (tx_id, queue_id, position, contract_id, method, args, created_at, updated_at)
         VALUES (?1, ?2, 0, 'ref.finance.near', 'ft_swap', '{}', 1, 1)",
    )
    .bind("tx-1")
    .bind("queue-does-not-exist")
    .execute(&pool)
    .await;

    assert!(
        result.is_err(),
        "a transaction pointing at a missing queue must fail"
    );
}

#[tokio::test]
async fn the_same_account_on_the_same_transaction_twice_is_refused() {
    let db = TEMP_DB::new_fun("dup-account");
    let pool = db.pool_fun().await;

    sqlx::query("INSERT INTO tx_queue (queue_id, created_at, updated_at) VALUES (?1, 1, 1)")
        .bind("queue-1")
        .execute(&pool)
        .await
        .expect("the queue should save");

    sqlx::query(
        "INSERT INTO tx_queue_transactions
           (tx_id, queue_id, position, contract_id, method, args, created_at, updated_at)
         VALUES (?1, ?2, 0, 'ref.finance.near', 'ft_swap', '{}', 1, 1)",
    )
    .bind("tx-1")
    .bind("queue-1")
    .execute(&pool)
    .await
    .expect("the transaction should save");

    let insert = || {
        sqlx::query(
            "INSERT INTO tx_queue_accounts
               (queue_id, tx_id, account_id, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'pending', 1, 1)",
        )
        .bind("queue-1")
        .bind("tx-1")
        .bind("alice.near")
        .execute(&pool)
    };

    insert().await.expect("the first account row should save");

    let second = insert().await;
    assert!(
        second.is_err(),
        "the same account on the same transaction twice must fail"
    );
}

#[tokio::test]
async fn deleting_a_queue_takes_its_transactions_and_accounts_with_it() {
    let db = TEMP_DB::new_fun("cascade");
    let pool = db.pool_fun().await;

    sqlx::query("INSERT INTO tx_queue (queue_id, created_at, updated_at) VALUES (?1, 1, 1)")
        .bind("queue-1")
        .execute(&pool)
        .await
        .expect("the queue should save");

    sqlx::query(
        "INSERT INTO tx_queue_transactions
           (tx_id, queue_id, position, contract_id, method, args, created_at, updated_at)
         VALUES (?1, ?2, 0, 'ref.finance.near', 'ft_swap', '{}', 1, 1)",
    )
    .bind("tx-1")
    .bind("queue-1")
    .execute(&pool)
    .await
    .expect("the transaction should save");

    sqlx::query(
        "INSERT INTO tx_queue_accounts
           (queue_id, tx_id, account_id, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'pending', 1, 1)",
    )
    .bind("queue-1")
    .bind("tx-1")
    .bind("alice.near")
    .execute(&pool)
    .await
    .expect("the account row should save");

    // delete the queue. the transaction and the account row go with it.
    sqlx::query("DELETE FROM tx_queue WHERE queue_id = ?1")
        .bind("queue-1")
        .execute(&pool)
        .await
        .expect("the queue should delete");

    let transactions: (i64,) = sqlx::query_as("SELECT COUNT(1) FROM tx_queue_transactions")
        .fetch_one(&pool)
        .await
        .expect("the count should read");
    assert_eq!(transactions.0, 0, "the transaction should be cascaded away");

    let accounts: (i64,) = sqlx::query_as("SELECT COUNT(1) FROM tx_queue_accounts")
        .fetch_one(&pool)
        .await
        .expect("the count should read");
    assert_eq!(accounts.0, 0, "the account row should be cascaded away");
}

#[tokio::test]
async fn deleting_a_transaction_takes_its_account_rows_with_it() {
    let db = TEMP_DB::new_fun("cascade-tx");
    let pool = db.pool_fun().await;

    sqlx::query("INSERT INTO tx_queue (queue_id, created_at, updated_at) VALUES (?1, 1, 1)")
        .bind("queue-1")
        .execute(&pool)
        .await
        .expect("the queue should save");

    sqlx::query(
        "INSERT INTO tx_queue_transactions
           (tx_id, queue_id, position, contract_id, method, args, created_at, updated_at)
         VALUES (?1, ?2, 0, 'ref.finance.near', 'ft_swap', '{}', 1, 1)",
    )
    .bind("tx-1")
    .bind("queue-1")
    .execute(&pool)
    .await
    .expect("the transaction should save");

    sqlx::query(
        "INSERT INTO tx_queue_accounts
           (queue_id, tx_id, account_id, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'pending', 1, 1)",
    )
    .bind("queue-1")
    .bind("tx-1")
    .bind("alice.near")
    .execute(&pool)
    .await
    .expect("the account row should save");

    sqlx::query("DELETE FROM tx_queue_transactions WHERE tx_id = ?1")
        .bind("tx-1")
        .execute(&pool)
        .await
        .expect("the transaction should delete");

    let accounts: (i64,) = sqlx::query_as("SELECT COUNT(1) FROM tx_queue_accounts")
        .fetch_one(&pool)
        .await
        .expect("the count should read");
    assert_eq!(accounts.0, 0, "the account row should be cascaded away");
}

#[tokio::test]
async fn an_account_row_starts_out_pending() {
    let db = TEMP_DB::new_fun("default-status");
    let pool = db.pool_fun().await;

    sqlx::query("INSERT INTO tx_queue (queue_id, created_at, updated_at) VALUES (?1, 1, 1)")
        .bind("queue-1")
        .execute(&pool)
        .await
        .expect("the queue should save");

    sqlx::query(
        "INSERT INTO tx_queue_transactions
           (tx_id, queue_id, position, contract_id, method, args, created_at, updated_at)
         VALUES (?1, ?2, 0, 'ref.finance.near', 'ft_swap', '{}', 1, 1)",
    )
    .bind("tx-1")
    .bind("queue-1")
    .execute(&pool)
    .await
    .expect("the transaction should save");

    // the status is left out. it has to default to pending.
    sqlx::query(
        "INSERT INTO tx_queue_accounts
           (queue_id, tx_id, account_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, 1, 1)",
    )
    .bind("queue-1")
    .bind("tx-1")
    .bind("alice.near")
    .execute(&pool)
    .await
    .expect("the account row should save");

    let status: (String,) =
        sqlx::query_as("SELECT status FROM tx_queue_accounts WHERE tx_id = ?1 AND account_id = ?2")
            .bind("tx-1")
            .bind("alice.near")
            .fetch_one(&pool)
            .await
            .expect("the status should read");
    assert_eq!(status.0, "pending");
}

#[tokio::test]
async fn an_old_db_gets_the_args_base64_column_and_a_new_one_does_not_break() {
    // --- WHY --- THIS IS THE "NO MIGRATION SYSTEM" PROBLEM: a db that
    // was made BEFORE the `args_base64` column existed would never get
    // it from `CREATE TABLE IF NOT EXISTS`, so the column is added by an
    // `ALTER TABLE` that checks first. this test makes the old db, opens
    // it, and checks the column is there.
    let db = TEMP_DB::new_fun("old-db");
    {
        // the db as it was BEFORE the column existed.
        //
        // --- WHY --- A RAW POOL AND NOT `db_open_fun`: `db_open_fun`
        // runs the schema, and the schema is what this test is checking.
        // so the old table is made by hand here.
        let options = SqliteConnectOptions::new()
            .filename(&db.path)
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("the db should connect");
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS tx_queue_transactions (
                tx_id              TEXT    NOT NULL PRIMARY KEY,
                queue_id           TEXT    NOT NULL,
                position           INTEGER NOT NULL,
                contract_id        TEXT    NOT NULL,
                method             TEXT    NOT NULL,
                args               TEXT    NOT NULL,
                gas                TEXT,
                deposit            TEXT,
                created_at         INTEGER NOT NULL,
                updated_at         INTEGER NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .expect("the old table should be made");
        pool.close().await;
    }

    // opening it runs the schema, and the schema adds the column.
    let pool = db.pool_fun().await;

    let columns = columns_of_fun(&pool, "tx_queue_transactions").await;
    assert!(
        columns.contains(&"args_base64".to_string()),
        "the old db must have gained the column: {columns:?}"
    );

    // and opening it AGAIN must not fail with "duplicate column name".
    let _pool_again = db.pool_fun().await;
    let columns_again = columns_of_fun(&_pool_again, "tx_queue_transactions").await;
    assert_eq!(columns_again, columns);
}

#[tokio::test]
async fn an_old_db_gets_the_key_columns_and_a_new_one_does_not_break() {
    // --- WHY --- THIS IS THE SAME PROBLEM, FIVE TIMES OVER: a db that
    // was made BEFORE the key action existed would never get the five
    // `key_*` columns from `CREATE TABLE IF NOT EXISTS`, so they are
    // added by an `ALTER TABLE` that checks first. this test makes the
    // old db, opens it, and checks the columns are there.
    let db = TEMP_DB::new_fun("old-db-key-columns");
    {
        // the db as it was BEFORE the key columns existed.
        //
        // --- WHY --- A RAW POOL AND NOT `db_open_fun`: `db_open_fun`
        // runs the schema, and the schema is what this test is checking.
        // so the old table is made by hand here.
        let options = SqliteConnectOptions::new()
            .filename(&db.path)
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("the db should connect");
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS tx_queue_transactions (
                tx_id              TEXT    NOT NULL PRIMARY KEY,
                queue_id           TEXT    NOT NULL,
                position           INTEGER NOT NULL,
                contract_id        TEXT    NOT NULL,
                method             TEXT    NOT NULL,
                args               TEXT    NOT NULL,
                args_base64        TEXT,
                gas                TEXT,
                deposit            TEXT,
                created_at         INTEGER NOT NULL,
                updated_at         INTEGER NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .expect("the old table should be made");
        pool.close().await;
    }

    // opening it runs the schema, and the schema adds the columns.
    let pool = db.pool_fun().await;

    let columns = columns_of_fun(&pool, "tx_queue_transactions").await;
    for name in [
        "key_action",
        "key_public_key",
        "key_receiver_id",
        "key_method_names",
        "key_allowance",
    ] {
        assert!(
            columns.contains(&name.to_string()),
            "the old db must have gained the {name} column: {columns:?}"
        );
    }

    // and opening it AGAIN must not fail with "duplicate column name".
    let _pool_again = db.pool_fun().await;
    let columns_again = columns_of_fun(&_pool_again, "tx_queue_transactions").await;
    assert_eq!(columns_again, columns);
}

// ========================================
// copyright 2026 by sleet.near
// ========================================
