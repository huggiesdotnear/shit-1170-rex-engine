// ========================================
// TX_QUEUE_TRANSACTIONS_STORE
// the db access for the tx_queue_transactions table: one row per
// transaction inside a queued batch.
//
// --- WHY --- a row is EITHER a FUNCTION CALL OR a KEY ACTION: no
// transfers, no account creation. a function call carries the near-kit
// toolbox `generic_contract_call_fun` shape — contract id, method, args,
// gas, deposit — so the frontend can hand the transactions straight to
// `generic_contract_call_wallet` without reshaping them. a KEY ACTION
// carries the access key it adds or removes, in the five `key_*`
// columns.
//
// --- WHY --- A KEY ACTION WRITES EMPTY STRINGS FOR `contract_id` AND
// `method`: the two columns are `NOT NULL` and making them nullable
// would need a table rebuild. the empty string means "this is not a
// contract call", and the `key_action` column is what says which it is.
//
// --- WHY --- `contract_id`, `method` and `args` have their OWN columns
// and not one json blob: the plan is to later show "the most common
// contracts and methods" in the ui. a `contract_id` column with an
// index answers "how many queued txs hit ref.finance.near" with a
// `GROUP BY`. inside a json blob it is a json scan on every row, every
// time. `args` stays a json string because it is a bag of arbitrary keys
// with no shape — there is nothing to group by.
//
// --- WHY --- `position` is the order of the transactions: a batch of
// three calls has an order — the deposit comes first, the swap second,
// the withdrawal third — and `position` is that order, `0, 1, 2` within
// the queue.
//
// --- WHY --- `args_base64` is a column of its OWN: a call's arguments
// are EITHER a json object with named keys OR an opaque run of bytes —
// the FastFS protocol's borsh arguments are the case. one column for
// both would make every json read of the feed walk over base64.
use sqlx::{Row, SqlitePool};

use crate::backend::logic::endpoint_shit_1170_tx_queue::tx_queue_store::now_ms_fun;
// ========================================
// --- TYPES ------------------------------
// one queued transaction, as it comes out of the db.
#[derive(Debug, Clone)]
pub struct TX_QUEUE_TRANSACTION_ROW {
    // the transaction's id. it is the row's primary key, and it is a uuid
    // so it does not move when the batch is renumbered.
    pub tx_id: String,
    // the batch this transaction belongs to.
    pub queue_id: String,
    // the order of this transaction inside the batch, from zero.
    pub position: i64,
    // the contract being called.
    pub contract_id: String,
    // the method name.
    pub method: String,
    // the arguments, as a json string. the wire shape is a json object.
    //
    // --- WHY --- IT IS NOT NULL AND IS NOT OPTIONAL: a call always has
    // arguments, and a call with binary arguments has the EMPTY object
    // here — the bytes are in the column below. so this column is never
    // null and never missing.
    pub args: String,
    // the arguments, as raw bytes, base64 encoded. it is null when the
    // arguments are the json object above.
    pub args_base64: Option<String>,
    // the gas, as a string. it may be nothing.
    pub gas: Option<String>,
    // the deposit, as a string. it may be nothing.
    pub deposit: Option<String>,
    // the KEY ACTION — `add_key` or `delete_key`. it is null for an
    // ordinary contract call.
    //
    // --- WHY --- IT IS THE COLUMN THAT SAYS WHICH KIND OF ROW THIS IS:
    // a key action row has an empty `contract_id` and an empty
    // `method`, and the empty strings mean "not a contract call". this
    // column is what says what it is instead.
    pub key_action: Option<String>,
    // the public key the key action names. it is null for a contract
    // call.
    pub key_public_key: Option<String>,
    // the contract the new key may call. it is null for a contract call
    // and for a `delete_key`.
    pub key_receiver_id: Option<String>,
    // the methods the new key may call, as a json array string. it is
    // null for a contract call and for a `delete_key`.
    pub key_method_names: Option<String>,
    // the yoctoNEAR ceiling on the new key. it is null for a contract
    // call and for a `delete_key`.
    pub key_allowance: Option<String>,
    // when the transaction was queued, in ms since the epoch.
    pub created_at: i64,
    // when the transaction was last touched, in ms since the epoch.
    pub updated_at: i64,
}
// ========================================
// --- INSERT ----------------------------
// create the row for one transaction of a batch.
//
// --- WHY --- the args are stored as the STRING they were given: the
// column is text, and the json <-> column conversion lives in
// `tx_queue_args_fun` so the two directions can never disagree.
//
// --- WHY --- there are FIFTEEN arguments and not a struct: every one of
// them is a COLUMN of the row, and a struct would only move the columns
// into a struct and give the caller another thing to fill in. the
// columns are the shape.
#[allow(clippy::too_many_arguments)]
pub async fn tx_queue_transactions_insert_fun(
    pool: &SqlitePool,
    tx_id: &str,
    queue_id: &str,
    position: i64,
    contract_id: &str,
    method: &str,
    args: &str,
    args_base64: Option<&str>,
    gas: Option<&str>,
    deposit: Option<&str>,
    key_action: Option<&str>,
    key_public_key: Option<&str>,
    key_receiver_id: Option<&str>,
    key_method_names: Option<&str>,
    key_allowance: Option<&str>,
) -> Result<(), sqlx::Error> {
    let now = now_ms_fun();

    sqlx::query(
        "INSERT INTO tx_queue_transactions
        (tx_id, queue_id, position, contract_id, method, args, args_base64, gas, deposit,
         key_action, key_public_key, key_receiver_id, key_method_names, key_allowance,
         created_at, updated_at)
       VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?15)",
    )
    .bind(tx_id)
    .bind(queue_id)
    .bind(position)
    .bind(contract_id)
    .bind(method)
    .bind(args)
    .bind(args_base64)
    .bind(gas)
    .bind(deposit)
    .bind(key_action)
    .bind(key_public_key)
    .bind(key_receiver_id)
    .bind(key_method_names)
    .bind(key_allowance)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}

// ========================================
// --- LIST BY QUEUE ---------------------
// the transactions of one batch, in `position` order.
//
// --- WHY --- the order is `position ASC` and not the rowid: the position
// is the order the wallet popup has to get them in, and it is the order
// the agent wrote them in.
pub async fn tx_queue_transactions_list_by_queue_fun(
    pool: &SqlitePool,
    queue_id: &str,
) -> Result<Vec<TX_QUEUE_TRANSACTION_ROW>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT tx_id, queue_id, position, contract_id, method, args, args_base64, gas, deposit,
                key_action, key_public_key, key_receiver_id, key_method_names, key_allowance,
                created_at, updated_at
       FROM tx_queue_transactions
      WHERE queue_id = ?1
      ORDER BY position ASC",
    )
    .bind(queue_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.iter().map(row_to_tx_queue_transaction_fun).collect())
}

// ========================================
// --- GET -------------------------------
// one queued transaction by its id, or None.
pub async fn tx_queue_transactions_get_fun(
    pool: &SqlitePool,
    tx_id: &str,
) -> Result<Option<TX_QUEUE_TRANSACTION_ROW>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT tx_id, queue_id, position, contract_id, method, args, args_base64, gas, deposit,
                key_action, key_public_key, key_receiver_id, key_method_names, key_allowance,
                created_at, updated_at
       FROM tx_queue_transactions
      WHERE tx_id = ?1",
    )
    .bind(tx_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| row_to_tx_queue_transaction_fun(&row)))
}
// ========================================
// --- HELPERS ----------------------------
// turn a db row into a queued transaction.
//
// --- WHY --- IT IS PUBLIC: the store reads the transactions of a WHOLE
// PAGE in one query — that is the batched read that keeps the feed's
// answer at two queries instead of two per batch — and that read lives
// in `tx_queue_store`. so the row reader has to be reachable from there.
pub fn row_to_tx_queue_transaction_fun(row: &sqlx::sqlite::SqliteRow) -> TX_QUEUE_TRANSACTION_ROW {
    TX_QUEUE_TRANSACTION_ROW {
        tx_id: row.get("tx_id"),
        queue_id: row.get("queue_id"),
        position: row.get("position"),
        contract_id: row.get("contract_id"),
        method: row.get("method"),
        args: row.get("args"),
        args_base64: row.get("args_base64"),
        gas: row.get("gas"),
        deposit: row.get("deposit"),
        key_action: row.get("key_action"),
        key_public_key: row.get("key_public_key"),
        key_receiver_id: row.get("key_receiver_id"),
        key_method_names: row.get("key_method_names"),
        key_allowance: row.get("key_allowance"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
