// ========================================
// DB_SCHEMA_RUN_FUN
// runs the table creates: the one file that knows how many tables there
// are.
//
// --- WHY --- there is NO migration system here. a migration system exists
// so a db's schema can be CHANGED over time: each change is a numbered
// step, the db records which steps it has run, and a fresh db runs them
// all in order.
//
// this db has four tables of its own and four settled shapes for them.
// so there is nothing to step through and nothing to remember. what is
// left is the schema itself and the statements that create it.
//
// --- WHY --- the tables are NOT versioned per row: a `format_version`
// column would let a new shape be written beside an old one and the old
// one ignored. that is worth doing when a shape changes often enough
// for the old rows to pile up, and it is not worth doing for tables whose
// shapes are settled. a changed shape means changing the table file.
//
// --- WHY --- the schema lives in code and not in a folder of .sql files:
// the schema has to travel with the binary. a file the docker image does
// not carry is a file the app cannot open.
//
// --- WHY --- the tables are created in DEPENDENCY ORDER: `tx_queue` has
// to exist before `tx_queue_transactions` points at it, and
// `tx_queue_transactions` has to exist before `tx_queue_accounts` points
// at it. sqlite does not check a foreign key's target when the table is
// created, but the order still has to be right for the rows that follow
// and for a reader of the code.
//
// --- WHY --- each table file holds its own `CREATE TABLE` and its own
// indexes, and this file holds nothing but the order: a table's shape is
// read in one place, and this is the only file that says how many tables
// there are.
//
// --- WHY --- A COLUMN THAT WAS ADDED LATER IS A DIFFERENT THING: a
// `CREATE TABLE IF NOT EXISTS` does nothing to a table that already
// exists, so a db that was made before a column was added would never
// get it. there is still no migration system — no numbered steps and no
// record of which steps have run — but a column is added by an
// `ALTER TABLE` that checks whether the column is already there, and it
// runs on every start. the check is what makes it safe to run every
// time: the statement runs only when the column is missing.
use sqlx::{Connection, Executor, SqliteConnection};
// ========================================
// --- RUN -------------------------------
// create every table if it is not there yet.
//
// --- WHY --- it runs on every start: the statements do nothing when the
// tables already exist, so there is no bookkeeping to say "already done"
// and no state to get out of step with the schema.
//
// --- WHY --- it runs in ONE transaction: a half-created schema would fail
// the first query against it. one transaction means either every table is
// there or none of them is.
//
// --- WHY --- the table creates are called with the SAME connection and
// NOT in a nested transaction of their own: a transaction cannot be
// started inside another transaction. so the table files expose their
// statement as a constant and the runner executes them itself.
pub async fn db_schema_run_fun(connection: &mut SqliteConnection) -> Result<(), sqlx::Error> {
    use crate::backend::logic::db::db_table_api_keys_fun::SHIT_1170_DB_TABLE_API_KEYS;
    use crate::backend::logic::db::db_table_swap_queue_accounts_fun::SHIT_1170_DB_TABLE_SWAP_QUEUE_ACCOUNTS;
    use crate::backend::logic::db::db_table_swap_queue_fun::SHIT_1170_DB_TABLE_SWAP_QUEUE;
    use crate::backend::logic::db::db_table_tx_queue_accounts_fun::SHIT_1170_DB_TABLE_TX_QUEUE_ACCOUNTS;
    use crate::backend::logic::db::db_table_tx_queue_fun::SHIT_1170_DB_TABLE_TX_QUEUE;
    use crate::backend::logic::db::db_table_tx_queue_transactions_fun::SHIT_1170_DB_TABLE_TX_QUEUE_TRANSACTIONS;

    let mut transaction = connection.begin().await?;

    // the api keys. no foreign keys point at it, so it goes first.
    transaction.execute(SHIT_1170_DB_TABLE_API_KEYS).await?;

    // the queue. the transactions point at it.
    transaction.execute(SHIT_1170_DB_TABLE_TX_QUEUE).await?;

    // the transactions of a queue. the account rows point at them.
    transaction
        .execute(SHIT_1170_DB_TABLE_TX_QUEUE_TRANSACTIONS)
        .await?;

    // the account rows. they point at the transactions.
    transaction
        .execute(SHIT_1170_DB_TABLE_TX_QUEUE_ACCOUNTS)
        .await?;

    // the swap queue. the account rows point at it.
    transaction.execute(SHIT_1170_DB_TABLE_SWAP_QUEUE).await?;

    // the account rows of a swap. they point at the swap.
    transaction
        .execute(SHIT_1170_DB_TABLE_SWAP_QUEUE_ACCOUNTS)
        .await?;

    transaction.commit().await?;

    // --- WHY --- THE COLUMNS THAT CAME LATER ARE ADDED AFTER THE
    // TRANSACTION: the table creates run in ONE transaction so a
    // half-created schema is impossible. the `ALTER TABLE`s that bring an
    // OLD db up to the current shape have to run outside it — they are
    // not part of the create, and they must not be able to roll the
    // creates back.
    //
    // --- WHY --- THIS IS WHERE THE "NO MIGRATION SYSTEM" NOTE IS KEPT
    // HONEST: a db that was made before a column was added does not get
    // it from `CREATE TABLE IF NOT EXISTS`, so the column is added here,
    // and the add is checked so it never runs twice.
    crate::backend::logic::db::db_table_tx_queue_transactions_fun::db_table_tx_queue_transactions_add_args_base64_fun(connection).await?;

    // --- WHY --- THE KEY COLUMNS ARE ADDED HERE AND NOT IN THE TABLE
    // FILE'S OWN CREATE: the runner is the one that knows what a db
    // that already exists needs, and it is the one that runs on every
    // start.
    crate::backend::logic::db::db_table_tx_queue_transactions_fun::db_table_tx_queue_transactions_add_key_action_fun(connection).await?;
    crate::backend::logic::db::db_table_tx_queue_transactions_fun::db_table_tx_queue_transactions_add_key_public_key_fun(connection).await?;
    crate::backend::logic::db::db_table_tx_queue_transactions_fun::db_table_tx_queue_transactions_add_key_receiver_id_fun(connection).await?;
    crate::backend::logic::db::db_table_tx_queue_transactions_fun::db_table_tx_queue_transactions_add_key_method_names_fun(connection).await?;
    crate::backend::logic::db::db_table_tx_queue_transactions_fun::db_table_tx_queue_transactions_add_key_allowance_fun(connection).await?;

    Ok(())
}
// ========================================
// --- RUN ON A POOL ----------------------
// the same schema, over a pool connection.
//
// --- WHY --- the pool's connection is a different type than a plain
// SqliteConnection, and this code should not care which one it was
// handed.
pub async fn db_schema_run_pool_fun(pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
    let mut connection = pool.acquire().await?;
    db_schema_run_fun(&mut connection).await
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
