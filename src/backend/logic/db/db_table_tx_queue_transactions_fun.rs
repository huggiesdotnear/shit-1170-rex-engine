// ========================================
// DB_TABLE_TX_QUEUE_TRANSACTIONS_FUN
// the tx_queue_transactions table: one row per transaction inside a
// queued batch.
//
// --- WHY --- a row is EITHER a function call OR a key action: a
// function call carries the near-kit toolbox `generic_contract_call_fun`
// shape — contract id, method, args, gas, deposit — so the frontend can
// hand the transactions straight to `generic_contract_call_wallet` without
// reshaping them. a KEY ACTION carries the access key it adds or
// removes, and the five `key_*` columns are what it needs. there is
// still no transfer and no account creation.
//
// --- WHY --- `contract_id` and `method` STAY `NOT NULL` AND A KEY
// ACTION WRITES EMPTY STRINGS INTO THEM: making them nullable would
// need a table rebuild — sqlite cannot drop a `NOT NULL` — and a
// rebuild on a live vault is a risk the queue does not need. so a key
// action row carries `contract_id = ''` and `method = ''`, and the
// EMPTY STRING MEANS "this is not a contract call". the `key_action`
// column is what says which it is.
//
// --- WHY --- `contract_id`, `method` and `args` have their OWN columns
// and not one json blob: the plan is to later show "the most common
// contracts and methods" in the ui. a `contract_id` column with an
// index answers "how many queued txs hit ref.finance.near" with a
// `GROUP BY`. inside a json blob it is a json scan on every row, every
// time. `args` stays a json column because it is a bag of arbitrary keys
// with no shape — there is nothing to group by.
//
// --- WHY --- `position` is the order of the transactions: a batch of
// three calls has an order — the deposit comes first, the swap second,
// the withdrawal third — and `position` is that order, `0, 1, 2` within
// the queue.
//
// --- WHY --- `position` is a column and not the rowid: sqlite hands out
// rowids that depend on the insert order and the vacuum state.
// `position` is explicit, and it is what the wallet popup needs — the
// wallet gets the transactions in the order the agent wrote them.
//
// --- WHY --- `tx_id` is a uuid and not `queue_id` + `position`: the
// position is renumbered when a transaction is removed from the middle
// of the batch. a url that named a transaction by its position would then
// point at a different transaction. the uuid does not move.
//
// --- WHY --- `gas` and `deposit` are TEXT and nullable: the toolbox
// takes `number | string` for gas and `number | string | bigint` for
// deposit, and a big deposit does not fit a json number safely. as
// strings both fit, and a call that omits them has no gas of its own.
//
// --- WHY --- `args_base64` is a column of its OWN and not a second
// encoding inside `args`: the two carry different things. `args` is a
// json object with named keys, and `args_base64` is an opaque run of
// bytes that only the contract can read — the FastFS protocol's borsh
// arguments are the case. a column of its own keeps the json object
// readable as json, so the feed's json reads are not asked to look at
// base64.
use sqlx::{Connection, Executor, Row, SqliteConnection};
// ========================================
// --- SCHEMA ----------------------------
// the statement that creates the table and its indexes.
//
// --- WHY --- `CREATE TABLE IF NOT EXISTS`: it creates the table on a
// fresh db and does nothing on one that already has it.
//
// --- WHY --- the foreign key ON DELETE CASCADE: a transaction without a
// queue is nothing. deleting the queue takes its transactions with it,
// and the transactions take their account rows with them.
//
// --- WHY --- the four indexes, one each for the endpoint that needs it:
// `contract_id` serves "how many txs for this contract" and "the most
// common contracts", `method` serves "the most common methods",
// `queue_id` serves "the transactions of this queue, in order", and
// `created_at` serves the feed list, newest first.
pub const SHIT_1170_DB_TABLE_TX_QUEUE_TRANSACTIONS: &str = r#"
  CREATE TABLE IF NOT EXISTS tx_queue_transactions (
    tx_id              TEXT    NOT NULL PRIMARY KEY,
    queue_id           TEXT    NOT NULL,
    position           INTEGER NOT NULL,
    contract_id        TEXT    NOT NULL,
    method             TEXT    NOT NULL,
    args               TEXT    NOT NULL,
    args_base64        TEXT,
    gas                TEXT,
    deposit            TEXT,
    key_action         TEXT,
    key_public_key     TEXT,
    key_receiver_id    TEXT,
    key_method_names   TEXT,
    key_allowance      TEXT,
    created_at         INTEGER NOT NULL,
    updated_at         INTEGER NOT NULL,
    FOREIGN KEY (queue_id) REFERENCES tx_queue (queue_id) ON DELETE CASCADE
  );
  CREATE INDEX IF NOT EXISTS tx_queue_transactions_contract_id_idx
    ON tx_queue_transactions (contract_id);
  CREATE INDEX IF NOT EXISTS tx_queue_transactions_method_idx
    ON tx_queue_transactions (method);
  CREATE INDEX IF NOT EXISTS tx_queue_transactions_queue_id_idx
    ON tx_queue_transactions (queue_id);
  CREATE INDEX IF NOT EXISTS tx_queue_transactions_created_at_idx
    ON tx_queue_transactions (created_at);
"#;

// the columns the `ALTER TABLE` adds, named once so the check and the
// statement cannot drift apart.
//
// --- WHY --- EACH ONE HAS ITS OWN CONSTANT AND NOT ONE LIST: the
// pattern this file already uses is one named column per `ALTER TABLE`,
// and a list would be a second way of writing the same thing.
const SHIT_1170_TX_QUEUE_TRANSACTIONS_ARGS_BASE64_COLUMN: &str = "args_base64";
const SHIT_1170_TX_QUEUE_TRANSACTIONS_KEY_ACTION_COLUMN: &str = "key_action";
const SHIT_1170_TX_QUEUE_TRANSACTIONS_KEY_PUBLIC_KEY_COLUMN: &str = "key_public_key";
const SHIT_1170_TX_QUEUE_TRANSACTIONS_KEY_RECEIVER_ID_COLUMN: &str = "key_receiver_id";
const SHIT_1170_TX_QUEUE_TRANSACTIONS_KEY_METHOD_NAMES_COLUMN: &str = "key_method_names";
const SHIT_1170_TX_QUEUE_TRANSACTIONS_KEY_ALLOWANCE_COLUMN: &str = "key_allowance";
// ========================================
// --- CREATE ----------------------------
// create the tx_queue_transactions table if it is not there yet.
//
// --- WHY --- it runs in a transaction: a half-created table would fail
// the first query against it.
//
// --- WHY --- it runs on every start: the statement does nothing when the
// table already exists, so there is no bookkeeping to say "already
// done" and no state to get out of step with the schema.
//
// --- WHY --- IT ALSO ADDS THE MISSING COLUMNS: this db has no
// migration system, so a db that was created BEFORE a column was added
// would never get it — `CREATE TABLE IF NOT EXISTS` does nothing to a
// table that already exists. so the columns that were added later are
// added here, one `ALTER TABLE` each, and each one is checked first so
// it does not run on a db that already has it.
pub async fn db_table_tx_queue_transactions_create_fun(
    connection: &mut SqliteConnection,
) -> Result<(), sqlx::Error> {
    let mut transaction = connection.begin().await?;
    transaction
        .execute(SHIT_1170_DB_TABLE_TX_QUEUE_TRANSACTIONS)
        .await?;
    transaction.commit().await?;

    // the column that came after the table was first written.
    db_table_tx_queue_transactions_add_args_base64_fun(connection).await?;

    // the columns that carry the KEY ACTION. they came after the
    // `args_base64` one, and they follow the same pattern.
    db_table_tx_queue_transactions_add_key_action_fun(connection).await?;
    db_table_tx_queue_transactions_add_key_public_key_fun(connection).await?;
    db_table_tx_queue_transactions_add_key_receiver_id_fun(connection).await?;
    db_table_tx_queue_transactions_add_key_method_names_fun(connection).await?;
    db_table_tx_queue_transactions_add_key_allowance_fun(connection).await?;

    Ok(())
}

// ========================================
// --- ADD ARGS BASE64 ------------------
// add the `args_base64` column to a db that was made before it existed.
//
// --- WHY --- THE COLUMN IS CHECKED FIRST AND NOT CAUGHT AS AN ERROR:
// sqlite has no `ADD COLUMN IF NOT EXISTS`, and an `ALTER TABLE` that
// adds a column that is already there fails with "duplicate column
// name". so the table's columns are read first and the statement only
// runs when the column is missing. that way the same code brings a
// fresh db and an old db to the same shape, and neither of them fails.
//
// --- WHY --- THE CHECK IS ON `pragma_table_info` AND NOT ON A TRY: a
// try would swallow a real failure — a locked db, a full disk — and
// answer "the column is there" when it is not. the pragma says what the
// table's shape IS.
//
// --- WHY --- IT IS CALLED AFTER THE TABLE'S OWN TRANSACTION IS
// COMMITTED: the transaction is already closed by the time this runs,
// and a fresh db has the column from the `CREATE TABLE` above, so this
// statement does nothing there.
pub async fn db_table_tx_queue_transactions_add_args_base64_fun(
    connection: &mut SqliteConnection,
) -> Result<(), sqlx::Error> {
    let has_column = sqlx::query("SELECT name FROM pragma_table_info('tx_queue_transactions')")
        .fetch_all(&mut *connection)
        .await?
        .iter()
        .any(|row| {
            row.get::<String, _>("name") == SHIT_1170_TX_QUEUE_TRANSACTIONS_ARGS_BASE64_COLUMN
        });

    if has_column {
        return Ok(());
    }

    connection
        .execute("ALTER TABLE tx_queue_transactions ADD COLUMN args_base64 TEXT")
        .await?;

    Ok(())
}

// ========================================
// --- ADD KEY ACTION --------------------
// add the `key_action` column to a db that was made before it existed.
//
// --- WHY --- THE PATTERN IS THE ONE THE `args_base64` COLUMN USED: the
// column is checked first, and the `ALTER TABLE` only runs when the
// column is missing. sqlite has no `ADD COLUMN IF NOT EXISTS`, so the
// pragma is what keeps the statement from failing on a db that already
// has the column.
//
// --- WHY --- IT IS ITS OWN FILE'S WORTH OF FUNCTION: the rule is one
// function per file, and five columns means five of them. a loop over
// a list would be one function doing five jobs, and a job list in a
// const is a shape that hides which column is which.
pub async fn db_table_tx_queue_transactions_add_key_action_fun(
    connection: &mut SqliteConnection,
) -> Result<(), sqlx::Error> {
    let has_column = sqlx::query("SELECT name FROM pragma_table_info('tx_queue_transactions')")
        .fetch_all(&mut *connection)
        .await?
        .iter()
        .any(|row| row.get::<String, _>("name") == SHIT_1170_TX_QUEUE_TRANSACTIONS_KEY_ACTION_COLUMN);

    if has_column {
        return Ok(());
    }

    connection
        .execute("ALTER TABLE tx_queue_transactions ADD COLUMN key_action TEXT")
        .await?;

    Ok(())
}

// ========================================
// --- ADD KEY PUBLIC KEY -----------------
// add the `key_public_key` column to a db that was made before it
// existed.
//
// --- WHY --- THE PATTERN IS THE ONE THE `args_base64` COLUMN USED: the
// column is checked first, and the `ALTER TABLE` only runs when the
// column is missing.
pub async fn db_table_tx_queue_transactions_add_key_public_key_fun(
    connection: &mut SqliteConnection,
) -> Result<(), sqlx::Error> {
    let has_column = sqlx::query("SELECT name FROM pragma_table_info('tx_queue_transactions')")
        .fetch_all(&mut *connection)
        .await?
        .iter()
        .any(|row| {
            row.get::<String, _>("name") == SHIT_1170_TX_QUEUE_TRANSACTIONS_KEY_PUBLIC_KEY_COLUMN
        });

    if has_column {
        return Ok(());
    }

    connection
        .execute("ALTER TABLE tx_queue_transactions ADD COLUMN key_public_key TEXT")
        .await?;

    Ok(())
}

// ========================================
// --- ADD KEY RECEIVER ID ----------------
// add the `key_receiver_id` column to a db that was made before it
// existed.
//
// --- WHY --- THE PATTERN IS THE ONE THE `args_base64` COLUMN USED: the
// column is checked first, and the `ALTER TABLE` only runs when the
// column is missing.
pub async fn db_table_tx_queue_transactions_add_key_receiver_id_fun(
    connection: &mut SqliteConnection,
) -> Result<(), sqlx::Error> {
    let has_column = sqlx::query("SELECT name FROM pragma_table_info('tx_queue_transactions')")
        .fetch_all(&mut *connection)
        .await?
        .iter()
        .any(|row| {
            row.get::<String, _>("name") == SHIT_1170_TX_QUEUE_TRANSACTIONS_KEY_RECEIVER_ID_COLUMN
        });

    if has_column {
        return Ok(());
    }

    connection
        .execute("ALTER TABLE tx_queue_transactions ADD COLUMN key_receiver_id TEXT")
        .await?;

    Ok(())
}

// ========================================
// --- ADD KEY METHOD NAMES ---------------
// add the `key_method_names` column to a db that was made before it
// existed.
//
// --- WHY --- THE PATTERN IS THE ONE THE `args_base64` COLUMN USED: the
// column is checked first, and the `ALTER TABLE` only runs when the
// column is missing.
pub async fn db_table_tx_queue_transactions_add_key_method_names_fun(
    connection: &mut SqliteConnection,
) -> Result<(), sqlx::Error> {
    let has_column = sqlx::query("SELECT name FROM pragma_table_info('tx_queue_transactions')")
        .fetch_all(&mut *connection)
        .await?
        .iter()
        .any(|row| {
            row.get::<String, _>("name") == SHIT_1170_TX_QUEUE_TRANSACTIONS_KEY_METHOD_NAMES_COLUMN
        });

    if has_column {
        return Ok(());
    }

    connection
        .execute("ALTER TABLE tx_queue_transactions ADD COLUMN key_method_names TEXT")
        .await?;

    Ok(())
}

// ========================================
// --- ADD KEY ALLOWANCE ------------------
// add the `key_allowance` column to a db that was made before it
// existed.
//
// --- WHY --- THE PATTERN IS THE ONE THE `args_base64` COLUMN USED: the
// column is checked first, and the `ALTER TABLE` only runs when the
// column is missing.
pub async fn db_table_tx_queue_transactions_add_key_allowance_fun(
    connection: &mut SqliteConnection,
) -> Result<(), sqlx::Error> {
    let has_column = sqlx::query("SELECT name FROM pragma_table_info('tx_queue_transactions')")
        .fetch_all(&mut *connection)
        .await?
        .iter()
        .any(|row| {
            row.get::<String, _>("name") == SHIT_1170_TX_QUEUE_TRANSACTIONS_KEY_ALLOWANCE_COLUMN
        });

    if has_column {
        return Ok(());
    }

    connection
        .execute("ALTER TABLE tx_queue_transactions ADD COLUMN key_allowance TEXT")
        .await?;

    Ok(())
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
