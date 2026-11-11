// ========================================
// ENDPOINT_SHIT_1170_TX_QUEUE
// everything that keeps the tx queue: the batch, the transactions inside
// it, and the accounts on them.
//
// it fronts the queue routes: `/api/shit-1170/tx-queue` and everything
// under it.
//
// --- WHY --- these live together: a queued batch is THREE levels deep —
// a QUEUE, the TRANSACTIONS inside it, and the ACCOUNT ROWS under those.
// splitting them across folders would break one object into three pieces
// that have to be found separately.
//
// --- WHY --- the WHO is a table and not a json array on the queue: each
// object in the array is one row, and the three fields of the object are
// three columns — `account_id`, `status` and `tx_hash`. the array read on
// the wire is those rows serialized back to a list. nothing is added and
// nothing is lost.
//
// --- WHY --- the status lives ONLY on the account row: a transaction is
// not "sent" or "failed" on its own — it is sent BY an account, and each
// account's send is its own on-chain transaction with its own hash and its
// own outcome. two accounts sending the same queued transaction produce
// two different on-chain transactions, and either can succeed while the
// other fails.
// ========================================
pub mod tx_queue_accounts_store;
pub mod tx_queue_args_fun;
pub mod tx_queue_auth_fun;
pub mod tx_queue_caller_fun;
pub mod tx_queue_cursor_fun;
pub mod tx_queue_facets_store;
pub mod tx_queue_key_action_fun;
#[cfg(test)]
pub mod tx_queue_key_action_fun_test;
pub mod tx_queue_key_method_names_fun;
pub mod tx_queue_key_verify_fun;
#[cfg(test)]
pub mod tx_queue_key_verify_fun_test;
pub mod tx_queue_named_accounts_fun;
pub mod tx_queue_row_to_json_fun;
pub mod tx_queue_stats_store;
pub mod tx_queue_status_fun;
pub mod tx_queue_store;
#[cfg(test)]
pub mod tx_queue_store_test;
pub mod tx_queue_transactions_store;
pub mod tx_queue_uuid_fun;
// ========================================
// copyright 2026 by sleet.near
// ========================================
