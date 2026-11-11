// ========================================
// TX_QUEUE_ROW_TO_JSON_FUN
// the three db rows become ONE wire shape.
//
// a queued batch is three levels deep — a QUEUE, the TRANSACTIONS inside
// it, and the ACCOUNT ROWS under those — and the wire shape is one flat
// json object holding all three. this file is where they are joined.
//
// --- WHY --- this is its own file: the wire shape is written down ONCE.
// the queue route, the one-batch route and the list route all answer
// with the same shape, and if each of them built it by hand one of them
// would eventually build it differently. the db rows are snake_case and
// the wire is camelCase, and the rename happens here.
//
// --- WHY --- the queue's `accounts` list is the WHO across the WHOLE
// BATCH — the union of the transactions' account rows: a batch of three
// calls has three account rows for alice, and the queue's list holds
// ONE entry for alice, not three. the transaction's own list holds that
// transaction's rows, with that transaction's status and hash.
//
// --- WHY --- the WHO list is DEDUPED by account id: the queue's list is
// one entry per account. a row that carries a hash and a row that does
// not are the same account, and the one WITH the hash is the newer
// truth — so the row with the hash wins.
use crate::backend::logic::endpoint_shit_1170_tx_queue::tx_queue_accounts_store::TX_QUEUE_ACCOUNT_ROW;
use crate::backend::logic::endpoint_shit_1170_tx_queue::tx_queue_args_fun::tx_queue_args_to_json_fun;
use crate::backend::logic::endpoint_shit_1170_tx_queue::tx_queue_key_method_names_fun::tx_queue_key_method_names_to_json_fun;
use crate::backend::logic::endpoint_shit_1170_tx_queue::tx_queue_store::TX_QUEUE_ROW;
use crate::backend::logic::endpoint_shit_1170_tx_queue::tx_queue_transactions_store::TX_QUEUE_TRANSACTION_ROW;
use crate::backend::types::tx_queue_request::{
    TX_QUEUE_ACCOUNT, TX_QUEUE_ITEM, TX_QUEUE_TRANSACTION,
};
// ========================================
// --- TO JSON ---------------------------
// the wire shape of one queued batch.
//
// --- WHY --- the args blob is parsed and NOT passed through as a string:
// the wire shape of the args is a json object, and the column holds it
// as a string. the conversion lives in `tx_queue_args_fun`, so the two
// directions can never disagree.
//
// --- WHY --- a blob that will not parse is an ERROR and not an empty
// object: a call whose arguments are gone must not be answered with
// arguments that were never there. the caller turns the error into a 500.
//
// --- WHY --- THE BINARY ARGS RIDE ALONGSIDE AND NOT INSTEAD OF THE JSON:
// the json object is still there — it is the empty object for a
// binary-args call — and the base64 goes beside it. a client that only
// reads `args` sees a call with no named arguments, which is the truth:
// the bytes have no names.
pub fn tx_queue_row_to_json_fun(
    queue: &TX_QUEUE_ROW,
    transactions: &[TX_QUEUE_TRANSACTION_ROW],
    accounts: &[TX_QUEUE_ACCOUNT_ROW],
) -> Result<TX_QUEUE_ITEM, serde_json::Error> {
    let mut wire_transactions: Vec<TX_QUEUE_TRANSACTION> = Vec::with_capacity(transactions.len());

    for transaction in transactions {
        let args = tx_queue_args_to_json_fun(&transaction.args)?;

        // the method names of a key action. they ride as a list on the
        // wire, and the column holds them as a json array string.
        let key_method_names =
            tx_queue_key_method_names_to_json_fun(transaction.key_method_names.as_deref())?;

        // the accounts under THIS transaction.
        let own_accounts: Vec<&TX_QUEUE_ACCOUNT_ROW> = accounts
            .iter()
            .filter(|account| account.tx_id == transaction.tx_id)
            .collect();

        wire_transactions.push(TX_QUEUE_TRANSACTION {
            tx_id: transaction.tx_id.clone(),
            position: transaction.position,
            contract_id: transaction.contract_id.clone(),
            method: transaction.method.clone(),
            args,
            args_base64: transaction.args_base64.clone(),
            gas: transaction.gas.clone(),
            deposit: transaction.deposit.clone(),
            key_action: transaction.key_action.clone(),
            key_public_key: transaction.key_public_key.clone(),
            key_receiver_id: transaction.key_receiver_id.clone(),
            key_method_names,
            key_allowance: transaction.key_allowance.clone(),
            accounts: own_accounts
                .iter()
                .map(|account| account_to_json_fun(account))
                .collect(),
        });
    }

    // the WHO across the whole batch — one entry per account.
    let wire_accounts = queue_accounts_to_json_fun(accounts);

    Ok(TX_QUEUE_ITEM {
        queue_id: queue.queue_id.clone(),
        what: queue.what.clone(),
        when: queue.when_text.clone(),
        why: queue.why.clone(),
        when_at: queue.when_at,
        transactions: wire_transactions,
        accounts: wire_accounts,
        created_at: queue.created_at,
        updated_at: queue.updated_at,
    })
}

// ========================================
// --- HELPERS ----------------------------
// one account row as the wire shape.
fn account_to_json_fun(account: &TX_QUEUE_ACCOUNT_ROW) -> TX_QUEUE_ACCOUNT {
    TX_QUEUE_ACCOUNT {
        account_id: account.account_id.clone(),
        status: account.status.clone(),
        tx_hash: account.tx_hash.clone(),
    }
}

// the WHO across a whole batch — one entry per account.
//
// --- WHY --- the rows are DEDUPED: the same account has one row under
// every transaction of the batch, and the queue's list is one entry for
// the account, not one per transaction.
//
// --- WHY --- the row that CARRIES A HASH wins: the hash is the fact — it
// means the account actually sent the transaction. a pending row under
// another transaction of the same batch is still the same account, and
// the sent one is the newer truth.
fn queue_accounts_to_json_fun(accounts: &[TX_QUEUE_ACCOUNT_ROW]) -> Vec<TX_QUEUE_ACCOUNT> {
    let mut out: Vec<TX_QUEUE_ACCOUNT> = Vec::new();

    for account in accounts {
        let existing = out
            .iter_mut()
            .find(|one| one.account_id == account.account_id);

        match existing {
            Some(one) => {
                // the sent row wins over the pending one.
                if one.tx_hash.is_none() && account.tx_hash.is_some() {
                    one.status = account.status.clone();
                    one.tx_hash = account.tx_hash.clone();
                }
            }
            None => out.push(account_to_json_fun(account)),
        }
    }

    out
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
