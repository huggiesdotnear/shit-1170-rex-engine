// ========================================
// DB
// everything that touches the sqlite database.
//
// --- WHY --- these live together: the pool, the schema that creates
// the tables and the path the db file sits at are three halves of one
// thing — the storage. a change to any of them is a change to how the
// app stores its data, so they belong in one folder.
//
// --- WHY --- one file per table: a table's shape is read in one place.
// `db_schema_run_fun` holds the order and runs them, and it is the only
// file that knows how many tables there are.
pub mod bin;
pub mod db_history_fun;
pub mod db_open_fun;
pub mod db_schema_run_fun;
#[cfg(test)]
pub mod db_schema_test;
pub mod db_table_api_keys_fun;
pub mod db_table_swap_queue_accounts_fun;
pub mod db_table_swap_queue_fun;
pub mod db_table_tx_queue_accounts_fun;
pub mod db_table_tx_queue_fun;
pub mod db_table_tx_queue_transactions_fun;
pub mod vault_db_path_fun;
// ========================================
// copyright 2026 by sleet.near
// ========================================
