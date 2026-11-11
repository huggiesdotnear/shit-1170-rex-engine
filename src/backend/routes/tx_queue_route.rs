// ========================================
// TX_QUEUE_ROUTE
// POST /api/shit-1170/tx-queue — queue a batch of transactions.
// GET  /api/shit-1170/tx-queue — the queue list, filterable.
//
// the POST is the agent's route: the agent hands over a batch of calls
// and the accounts they are for, and the backend writes the queue, one
// transaction per call, and one account row per transaction per named
// account.
//
// --- WHY --- A TRANSACTION MAY ALSO BE A KEY ACTION: the queue carries
// two kinds of thing — a CONTRACT CALL, and an ACCESS KEY ACTION
// (`add_key` or `delete_key`). an agent that may only queue calls can
// never offer the user "give me a limited key" or "take that limited
// key away", and those are ordinary things a user asks for.
//
// --- WHY --- A `delete_key` IS CHECKED AGAINST THE CHAIN BEFORE IT IS
// QUEUED: the key's permission lives on chain, and only the chain knows
// it. a full access key is the user's wallet — deleting it locks them
// out — so the queue refuses to carry a delete for a key that is not a
// LIMITED one. the check happens at QUEUE TIME, before anything is
// written, so a refused request never leaves half of itself in the db.
//
// --- WHY --- THE TARGET ACCOUNT IS THE WHO LIST, OR THE CALLER'S OWN
// ACCOUNT WHEN THE WHO LIST IS EMPTY: the queue is FOR somebody, and
// the WHO list says who. a public key alone does not say whose key it
// is — the same key can be on ten accounts.
//
// the GET is the feed: it is OPEN. anyone can list the queues, because a
// queue is an open offer and hiding it would defeat the point.
//
// 400 = the body was empty, or held more than fifty transactions, or a
// transaction had no contract id, no method, or args that were not a
// json object, or args that were not base64, or both kinds of args at
// once, or the `accounts` list named an account the agent holds
// NO api key for — the NAMING rule — or a key action was not shaped
// right, or a `delete_key` named a key that is not a limited key on the
// account it is about. on the GET, 400 is also a cursor the api did not
// make, and a `hide_sent` with no account to be about.
// 401 = the x-api-key header is missing, or the key is not one of ours.
// 503 = the chain could not be read, so a key action could not be
// checked.
// 500 = the db could not be reached.
//
// --- WHY --- THE GET NEVER ANSWERS A 401: the list is open, and so are
// the FILTERS over it. a reader who is not signed in can read every
// batch and see every account on it, so filtering that public list by a
// name is not a leak — it is the filter the reader asked for. the key
// is read by the GET only so the reader's OWN account is known for the
// `hide_sent` filter, and a reader who has no key names the account in
// the query instead.
//
// --- WHY --- ONE file holds TWO handlers: the repo's rule is one
// FUNCTION per file, and this file holds the two functions of ONE url —
// the batch you queue and the batches you can see. they are two halves
// of the same resource.
//
// --- WHY --- the NAMING RULE is the anti-spam wall: the WHO list is what
// puts a tx in somebody's feed. if an agent could name any account id
// it liked, agent A could fill agent B's users' feeds with txs they
// never asked for — and the ui filters by account, so those users would
// keep seeing them. so the agent's WHO list is its OWN users, and only
// its own users. one agent spamming another agent's users through the
// queue is exactly what this check stops.
//
// --- WHY --- the SIGNING side stays open: a user who sees a tx in their
// feed may send it, and their account joins the WHO list. that is a
// user acting for themselves with their own key — no agent is putting it
// in front of them.
//
// --- WHY --- `hide_sent` WITHOUT an account is a 400: "has this account
// sent it" is a question about ONE account. a queue can be sent by alice
// and not by bob, so there is no single answer to "has this queue been
// sent". the filter therefore names the account it is about, and
// saying so is better than quietly ignoring it and returning the whole
// list.
use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use base64::Engine;
use sqlx::SqlitePool;

use crate::backend::logic::endpoint_shit_1170_tx_queue::tx_queue_accounts_store::{
    TX_QUEUE_ACCOUNT_ROW, tx_queue_accounts_insert_fun, tx_queue_accounts_list_by_queue_fun,
};
use crate::backend::logic::endpoint_shit_1170_tx_queue::tx_queue_args_fun::tx_queue_args_from_json_fun;
use crate::backend::logic::endpoint_shit_1170_tx_queue::tx_queue_caller_fun::tx_queue_caller_fun;
use crate::backend::logic::endpoint_shit_1170_tx_queue::tx_queue_cursor_fun::{
    tx_queue_cursor_decode_fun, tx_queue_cursor_encode_fun,
};
use crate::backend::logic::endpoint_shit_1170_tx_queue::tx_queue_key_action_fun::{
    TX_QUEUE_KEY_ACTION, TX_QUEUE_KEY_ACTION_SHAPE, tx_queue_key_action_read_fun,
};
use crate::backend::logic::endpoint_shit_1170_tx_queue::tx_queue_key_method_names_fun::tx_queue_key_method_names_from_json_fun;
use crate::backend::logic::endpoint_shit_1170_tx_queue::tx_queue_key_verify_fun::{
    TX_QUEUE_KEY_VERIFY_ERROR, tx_queue_key_verify_fun,
};
use crate::backend::logic::endpoint_shit_1170_tx_queue::tx_queue_named_accounts_fun::tx_queue_named_accounts_fun;
use crate::backend::logic::endpoint_shit_1170_tx_queue::tx_queue_row_to_json_fun::tx_queue_row_to_json_fun;
use crate::backend::logic::endpoint_shit_1170_tx_queue::tx_queue_status_fun::SHIT_1170_TX_QUEUE_STATUS_PENDING;
use crate::backend::logic::endpoint_shit_1170_tx_queue::tx_queue_store::{
    TX_QUEUE_CURSOR_POINT, TX_QUEUE_LIST_QUERY, TX_QUEUE_ROW, tx_queue_get_fun,
    tx_queue_insert_fun, tx_queue_list_fun, tx_queue_page_children_fun,
};
use crate::backend::logic::endpoint_shit_1170_tx_queue::tx_queue_transactions_store::{
    TX_QUEUE_TRANSACTION_ROW, tx_queue_transactions_insert_fun,
    tx_queue_transactions_list_by_queue_fun,
};
use crate::backend::logic::endpoint_shit_1170_tx_queue::tx_queue_uuid_fun::tx_queue_uuid_fun;
use crate::backend::logic::feed_ws::app_state::APP_STATE;
use crate::backend::logic::feed_ws::feed_ws_event::FEED_WS_EVENT;
use crate::backend::logic::feed_ws::feed_ws_publish_fun::feed_ws_publish_fun;
use crate::backend::types::shit_1170_query_list_fun::shit_1170_query_list_fun;
use crate::backend::types::tx_queue_query::TX_QUEUE_QUERY;
use crate::backend::types::tx_queue_request::{
    TX_QUEUE_ITEM, TX_QUEUE_LIST_RESPONSE, TX_QUEUE_REQUEST, TX_QUEUE_TRANSACTION_REQUEST,
};
// ========================================
// --- CONFIG -----------------------------
// how many transactions one batch may hold.
//
// --- WHY --- fifty: the wallet signs the whole batch in ONE popup, and a
// popup with two hundred transactions is unreadable and slow.
const SHIT_1170_TX_QUEUE_MAX_TRANSACTIONS: usize = 50;

// how many batches the list answers with when no limit is given.
const SHIT_1170_TX_QUEUE_LIST_LIMIT: i64 = 50;

// the most batches one page may hold, whatever the caller asked for.
//
// --- WHY --- a limit is CLAMPED and not refused: a caller who asks for
// a thousand is not a caller to be turned away — they want a big page,
// and a big page is fine. and a NEGATIVE limit means "no limit" in
// sqlite, so it has to be clamped too, or `limit=-1` would answer with
// every queue in the db.
const SHIT_1170_TX_QUEUE_LIST_MAX_LIMIT: i64 = 500;
// ========================================
// --- POST -------------------------------
// queue a batch of transactions.
//
// --- WHY --- the api key comes from the HEADER: a url ends up in logs
// and in the referer of the next page, and a bearer token in a url is a
// bearer token in everybody's browser history.
pub async fn tx_queue_post_route(
    State(state): State<APP_STATE>,
    headers: HeaderMap,
    Json(body): Json<TX_QUEUE_REQUEST>,
) -> Response {
    // --- WHY --- THE POOL IS TAKEN OUT OF THE STATE: the router's
    // state holds the pool and the feed bus, and this route needs
    // the pool. the bus is not needed here.
    let pool = state.pool.clone();
    let account_id = match tx_queue_caller_fun(&pool, &headers).await {
        Ok(account_id) => account_id,
        Err(failure) => {
            return shit_1170_error_fun(failure.status, &failure.code, &failure.message)
                .into_response();
        }
    };

    // --- THE BATCH ------------------------
    // it must hold at least one transaction and at most fifty.
    if body.transactions.is_empty() {
        return shit_1170_error_fun(
            StatusCode::BAD_REQUEST,
            "no_transactions",
            "a queue must hold at least one transaction",
        )
        .into_response();
    }
    if body.transactions.len() > SHIT_1170_TX_QUEUE_MAX_TRANSACTIONS {
        return shit_1170_error_fun(
            StatusCode::BAD_REQUEST,
            "too_many_transactions",
            &format!(
                "a queue holds at most {} transactions",
                SHIT_1170_TX_QUEUE_MAX_TRANSACTIONS
            ),
        )
        .into_response();
    }

    // --- THE CALLS ------------------------
    // every call is checked BEFORE anything is written, so a batch that
    // is going to be refused never leaves half of itself in the db.
    let mut prepared: Vec<PREPARED_TRANSACTION> = Vec::with_capacity(body.transactions.len());
    for (position, transaction) in body.transactions.iter().enumerate() {
        match prepared_transaction_fun(transaction, position as i64) {
            Ok(prepared_one) => prepared.push(prepared_one),
            Err((code, message)) => {
                return shit_1170_error_fun(StatusCode::BAD_REQUEST, &code, &message)
                    .into_response();
            }
        }
    }

    // --- THE WHO --------------------------
    // the NAMING RULE. every account the agent names must be one the
    // agent holds a key for.
    let named_accounts = body.accounts.clone().unwrap_or_default();
    if !named_accounts.is_empty() {
        let allowed = match tx_queue_named_accounts_fun(&pool, &account_id, &named_accounts).await {
            Ok(allowed) => allowed,
            Err(err) => {
                eprintln!("[tx-queue] could not check the named accounts: {err}");
                return shit_1170_error_fun(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "db_unavailable",
                    "the named accounts could not be checked",
                )
                .into_response();
            }
        };

        // --- WHY --- a SHORTER answer means one of them was refused, and
        // the WHOLE request is refused: naming one account you may not
        // name would still put a tx in somebody's feed. so the batch is
        // not written at all.
        if allowed.len() != named_accounts.len() {
            let refused = named_accounts
                .iter()
                .find(|account| !allowed.contains(account))
                .cloned()
                .unwrap_or_default();
            return shit_1170_error_fun(
                StatusCode::BAD_REQUEST,
                "account_not_named_by_agent",
                &format!("the agent holds no api key for {refused}, so it may not name it"),
            )
            .into_response();
        }
    }

    // --- THE KEY ACTIONS ------------------
    // a `delete_key` is checked AGAINST THE CHAIN before anything is
    // written. the check is here and not later because the whole point
    // is that a refused batch never leaves half of itself in the db.
    //
    // --- WHY --- THE TARGET ACCOUNT IS THE WHO LIST, OR THE CALLER'S
    // OWN ACCOUNT WHEN THE WHO LIST IS EMPTY: the queue is FOR somebody,
    // and the WHO list is the only thing that says who. an agent that
    // queues a key action for itself names nobody, and then the account
    // the key belongs to is the account the api key belongs to.
    //
    // --- WHY --- THE NAMING RULE HAS ALREADY RUN: the accounts have
    // been checked to be the agent's own users, so the chain check
    // cannot be pointed at somebody else's account.
    let target_accounts: Vec<String> = if named_accounts.is_empty() {
        vec![account_id.clone()]
    } else {
        named_accounts.clone()
    };

    // --- WHY --- THE TARGETS ARE DEDUPED: the WHO list may name the same
    // account twice — the agent wrote it twice — and checking it twice
    // would cost two round trips to the chain for the same answer.
    let mut targets: Vec<&String> = Vec::with_capacity(target_accounts.len());
    for target in &target_accounts {
        if !targets.contains(&target) {
            targets.push(target);
        }
    }

    for one in &prepared {
        // --- WHY --- ONLY A DELETE IS CHECKED: an `add_key` names a key
        // that does not exist yet, so there is nothing on chain to
        // check, and it cannot take anything away.
        if one.key_action != Some(TX_QUEUE_KEY_ACTION::DeleteKey) {
            continue;
        }

        let public_key = one.key_public_key.clone().unwrap_or_default();
        for target in &targets {
            match tx_queue_key_verify_fun(target, &public_key).await {
                Ok(()) => {}
                Err(failure) => {
                    let (status, code, message) =
                        key_verify_failure_fun(failure, target, &public_key);
                    return shit_1170_error_fun(status, &code, &message).into_response();
                }
            }
        }
    }

    // --- THE WRITE ------------------------
    let queue_id = tx_queue_uuid_fun();
    if let Err(err) = tx_queue_insert_fun(
        &pool,
        body.what.as_deref(),
        body.when.as_deref(),
        body.why.as_deref(),
        body.when_at,
        &queue_id,
    )
    .await
    {
        eprintln!("[tx-queue] could not write the queue: {err}");
        return shit_1170_error_fun(
            StatusCode::INTERNAL_SERVER_ERROR,
            "db_unavailable",
            "the queue could not be written",
        )
        .into_response();
    }

    let mut tx_ids: Vec<String> = Vec::with_capacity(prepared.len());
    for one in &prepared {
        let tx_id = tx_queue_uuid_fun();
        if let Err(err) = tx_queue_transactions_insert_fun(
            &pool,
            &tx_id,
            &queue_id,
            one.position,
            &one.contract_id,
            &one.method,
            &one.args,
            one.args_base64.as_deref(),
            one.gas.as_deref(),
            one.deposit.as_deref(),
            one.key_action.map(|action| match action {
                TX_QUEUE_KEY_ACTION::AddKey => "add_key",
                TX_QUEUE_KEY_ACTION::DeleteKey => "delete_key",
            }),
            one.key_public_key.as_deref(),
            one.key_receiver_id.as_deref(),
            one.key_method_names.as_deref(),
            one.key_allowance.as_deref(),
        )
        .await
        {
            eprintln!("[tx-queue] could not write the transaction: {err}");
            return shit_1170_error_fun(
                StatusCode::INTERNAL_SERVER_ERROR,
                "db_unavailable",
                "the transaction could not be written",
            )
            .into_response();
        }
        tx_ids.push(tx_id);
    }

    // --- WHY --- one account row PER TRANSACTION per named account: the
    // account row hangs off the transaction, because each transaction
    // produces its OWN hash and the hash cannot live on the queue.
    for tx_id in &tx_ids {
        for account in &named_accounts {
            if let Err(err) = tx_queue_accounts_insert_fun(
                &pool,
                &queue_id,
                tx_id,
                account,
                SHIT_1170_TX_QUEUE_STATUS_PENDING,
            )
            .await
            {
                eprintln!("[tx-queue] could not write the account row: {err}");
                return shit_1170_error_fun(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "db_unavailable",
                    "the account could not be written on the queue",
                )
                .into_response();
            }
        }
    }

    // --- THE ANSWER -----------------------
    let item = match queue_item_fun(&pool, &queue_id).await {
        Ok(item) => item,
        Err(failure) => return *failure,
    };

    // --- THE LIVE FEED --------------------
    // the open feed pages are told about the new batch.
    //
    // --- WHY --- THE SAME ITEM THAT IS ANSWERED WITH: the client's live
    // post has to be byte-for-byte what a refetch would give it, and
    // reading the row again could read a different one.
    feed_ws_publish_fun(
        &state.feed_ws,
        FEED_WS_EVENT::Queue {
            queue: item.clone(),
        },
    );

    (StatusCode::CREATED, Json(item)).into_response()
}

// ========================================
// --- GET --------------------------------
// the queue list.
//
// --- WHY --- the list is OPEN: anyone with an api key can list the
// queues. a queue is an open offer, and hiding it would defeat the
// point.
pub async fn tx_queue_get_route(
    State(state): State<APP_STATE>,
    headers: HeaderMap,
    Query(query): Query<TX_QUEUE_QUERY>,
) -> Response {
    // --- WHY --- THE POOL IS TAKEN OUT OF THE STATE: the router's
    // state holds the pool and the feed bus, and this route needs
    // the pool. the bus is not needed here.
    let pool = state.pool.clone();
    // --- WHY --- THE LIST DOES NOT ASK FOR A KEY: the queue is an OPEN
    // offer. a reader who is not signed in can still read the feed — they
    // see every batch, and they can open one. what they cannot do is act
    // on it, and the acting is what the other routes ask the key for.
    //
    // so the key is read here ONLY so the reader's OWN account is known
    // — the `hide_sent` filter is about the reader's own history.
    let caller_account_id = match tx_queue_caller_fun(&pool, &headers).await {
        Ok(account_id) => Some(account_id),
        // --- WHY --- a missing or unknown key is NOT an error here: the
        // list is open. the key is only needed so the reader's own
        // account can be used for the hide-sent filter, and that filter
        // is optional.
        Err(failure) if failure.status == StatusCode::UNAUTHORIZED => None,
        Err(failure) => {
            return shit_1170_error_fun(failure.status, &failure.code, &failure.message)
                .into_response();
        }
    };

    // --- THE ACCOUNT LIST ----------------
    // the accounts the filter bar picked.
    //
    // --- WHY --- THE PARAM IS COMMA SEPARATED AND NOT REPEATED: the
    // query extractor reads one value per param, and the filter bar
    // builds its query string out of a map. one comma separated string
    // carries many accounts and reads the same in a hand-written url.
    let picked_accounts = shit_1170_query_list_fun(query.account_id.as_deref());

    // --- WHY --- `hide_sent` needs an account: "has this account sent it"
    // is a question about ONE account. a queue can be sent by alice and
    // not by bob, so there is no such question about all the accounts at
    // once. without an account the filter has nothing to be about, and
    // saying so is better than quietly ignoring it.
    //
    // --- WHY --- EITHER THE NAMED ACCOUNT OR THE READER'S OWN COUNTS:
    // the filter says "hide what THIS account sent", and the account is
    // the one the reader named in the query — or, when the reader named
    // nobody, the reader's own account. a signed-in reader always has
    // an account to be about, so they are never refused.
    let hide_sent = hide_sent_flag_fun(query.hide_sent.as_deref());
    let hide_sent_account: Option<String> = match &caller_account_id {
        Some(caller) => Some(caller.clone()),
        None => picked_accounts.first().cloned(),
    };
    if hide_sent == Some(true) && hide_sent_account.is_none() {
        return shit_1170_error_fun(
            StatusCode::BAD_REQUEST,
            "hide_sent_without_account",
            "hide_sent needs an account_id — it is a filter on one account's view",
        )
        .into_response();
    }

    // --- WHY --- THE ACCOUNT FILTER IS THE ACCOUNT THE READER NAMED:
    // the filter says "the queues THIS account is on", and the reader
    // picks the accounts. a reader who is signed in and picks alice
    // sees alice's queues — that is the whole point of a filter by
    // account.
    //
    // --- WHY --- THE CALLER'S ACCOUNT IS NOT PREPENDED TO THE LIST: the
    // reader's own account going first would make the filter answer
    // "my queues OR the ones I picked", and the reader asked for the
    // ones they picked. the reader's OWN view is the front end's job —
    // it asks for the account it wants.
    //
    // --- WHY --- NO KEY IS NEEDED FOR THE ACCOUNT FILTER: the queue list
    // is an OPEN offer. a reader who is not signed in can already read
    // every batch and see every account on it — the WHO is right there
    // on the card. so filtering that public list by a name is not a
    // leak, it is the filter the reader asked for, and the reader does
    // not have to be signed in to use it.
    let account_ids: Vec<String> = picked_accounts.clone();

    // --- THE CONTRACT LIST ---------------
    // the contracts the filter bar picked.
    let contract_ids = shit_1170_query_list_fun(query.contract_id.as_deref());

    // --- THE CURSOR ----------------------
    // where the page starts.
    //
    // --- WHY --- A BAD CURSOR IS A 400 AND NOT THE FIRST PAGE: the
    // caller sent something the api does not understand, and answering
    // with the first page would show the reader the same rows twice and
    // hide the fact that their paging was broken.
    let cursor = match query.cursor.as_deref() {
        Some(raw) => match tx_queue_cursor_decode_fun(raw) {
            Ok(cursor) => Some(TX_QUEUE_CURSOR_POINT {
                created_at: cursor.created_at,
                queue_id: cursor.queue_id,
            }),
            Err(message) => {
                return shit_1170_error_fun(
                    StatusCode::BAD_REQUEST,
                    "bad_cursor",
                    &format!("the cursor could not be read: {message}"),
                )
                .into_response();
            }
        },
        None => None,
    };

    // --- THE HIDE-SENT ACCOUNT ------------
    // the account the `hide_sent` filter is about — worked out above,
    // so the refusal and the filter always agree.

    let list_query = TX_QUEUE_LIST_QUERY {
        account_ids,
        hide_sent,
        hide_sent_account,
        contract_ids,
        method: query.method.clone(),
        cursor,
        limit: limit_fun(
            query.limit,
            SHIT_1170_TX_QUEUE_LIST_LIMIT,
            SHIT_1170_TX_QUEUE_LIST_MAX_LIMIT,
        ),
    };

    let (rows, total) = match tx_queue_list_fun(&pool, &list_query).await {
        Ok(found) => found,
        Err(err) => {
            eprintln!("[tx-queue] could not read the queue list: {err}");
            return shit_1170_error_fun(
                StatusCode::INTERNAL_SERVER_ERROR,
                "db_unavailable",
                "the queue list could not be read",
            )
            .into_response();
        }
    };

    // --- THE CHILDREN --------------------
    // the transactions and the WHO of every batch on the page, in TWO
    // reads.
    //
    // --- WHY --- THIS IS ONE READ AND NOT ONE READ PER BATCH: reading
    // them per batch is two queries per batch, and a page of a hundred
    // is two hundred queries. that is the single biggest cost in
    // serving the feed and it grows with the page. reading them for the
    // WHOLE page at once is two queries, whatever the page holds.
    let queue_ids: Vec<String> = rows.iter().map(|row| row.queue_id.clone()).collect();
    let children = match tx_queue_page_children_fun(&pool, &queue_ids).await {
        Ok(children) => children,
        Err(err) => {
            eprintln!("[tx-queue] could not read the page children: {err}");
            return shit_1170_error_fun(
                StatusCode::INTERNAL_SERVER_ERROR,
                "db_unavailable",
                "the queue list could not be read",
            )
            .into_response();
        }
    };

    let mut items: Vec<TX_QUEUE_ITEM> = Vec::with_capacity(rows.len());
    for row in &rows {
        let empty_transactions: Vec<TX_QUEUE_TRANSACTION_ROW> = Vec::new();
        let empty_accounts: Vec<TX_QUEUE_ACCOUNT_ROW> = Vec::new();
        let transactions = children
            .transactions
            .get(&row.queue_id)
            .unwrap_or(&empty_transactions);
        let accounts = children
            .accounts
            .get(&row.queue_id)
            .unwrap_or(&empty_accounts);
        match tx_queue_row_to_json_fun(row, transactions, accounts) {
            Ok(item) => items.push(item),
            Err(err) => {
                eprintln!("[tx-queue] could not read the args: {err}");
                return shit_1170_error_fun(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "args_broken",
                    "the arguments of the queue could not be read",
                )
                .into_response();
            }
        }
    }

    // --- THE NEXT CURSOR -----------------
    // the cursor of the page after this one.
    //
    // --- WHY --- IT IS ONLY HANDED OUT WHEN THE PAGE IS FULL: a page
    // that came back short is the LAST page, and handing out a cursor
    // for it would send the reader off to look for rows that are not
    // there.
    let next_cursor = match rows.last() {
        Some(last) if rows.len() as i64 >= list_query.limit => {
            Some(tx_queue_cursor_encode_fun(last.created_at, &last.queue_id))
        }
        _ => None,
    };

    (
        StatusCode::OK,
        Json(TX_QUEUE_LIST_RESPONSE {
            items,
            total,
            next_cursor,
        }),
    )
        .into_response()
}
// ========================================
// --- HELPERS ----------------------------
// why a key action was refused, as the status and the code and the
// words.
//
// --- WHY --- THE CODE SAYS WHICH THING WENT WRONG: a key that is not
// there and a key that is full access are different facts, and the user
// reads them differently. one code for both would leave them guessing.
//
// --- WHY --- THE CHAIN BEING UNREACHABLE IS A 503 AND NOT A 400: the
// request was fine, the chain was not. and it is NOT a 500: nothing
// broke, the app just could not answer right now, and the client should
// come back.
fn key_verify_failure_fun(
    failure: TX_QUEUE_KEY_VERIFY_ERROR,
    account_id: &str,
    public_key: &str,
) -> (StatusCode, String, String) {
    match failure {
        TX_QUEUE_KEY_VERIFY_ERROR::NoSuchKey => (
            StatusCode::BAD_REQUEST,
            "key_action_not_allowed".to_string(),
            format!("{account_id} has no access key {public_key}"),
        ),
        TX_QUEUE_KEY_VERIFY_ERROR::NotALimitedKey => (
            StatusCode::BAD_REQUEST,
            "key_action_not_allowed".to_string(),
            format!(
                "the access key {public_key} of {account_id} is a full access key, not a limited one, so it may not be removed by the queue"
            ),
        ),
        TX_QUEUE_KEY_VERIFY_ERROR::BadAccountId => (
            StatusCode::BAD_REQUEST,
            "bad_account_id".to_string(),
            format!("{account_id} is not a NEAR account id"),
        ),
        TX_QUEUE_KEY_VERIFY_ERROR::BadPublicKey => (
            StatusCode::BAD_REQUEST,
            "bad_key_public_key".to_string(),
            format!("{public_key} is not a NEAR public key"),
        ),
        TX_QUEUE_KEY_VERIFY_ERROR::ChainUnreachable => (
            StatusCode::SERVICE_UNAVAILABLE,
            "chain_unreachable".to_string(),
            "the chain could not be read, so the key could not be checked".to_string(),
        ),
    }
}

// ========================================
// --- HELPERS ----------------------------
// one transaction of the batch, checked and ready to write.
//
// --- WHY --- A TRANSACTION IS EITHER A CONTRACT CALL OR A KEY ACTION:
// the two are different things the user approves in the same popup. a
// key action has no contract and no method, and a contract call has no
// key. a transaction that claims to be both is refused — it does not
// say what it is.
//
// --- WHY --- THE KEY ACTION IS READ FIRST: it is the one that says
// "this is not a contract call", and reading it first is what lets the
// contract id and the method stay required for the calls that ARE
// calls.
fn prepared_transaction_fun(
    transaction: &TX_QUEUE_TRANSACTION_REQUEST,
    position: i64,
) -> Result<PREPARED_TRANSACTION, (String, String)> {
    let key_action = tx_queue_key_action_read_fun(transaction).map_err(|(code, message)| {
        (code, format!("the transaction at position {position}: {message}"))
    })?;

    match key_action {
        Some(shape) => prepared_key_transaction_fun(transaction, position, shape),
        None => prepared_call_transaction_fun(transaction, position),
    }
}

// ========================================
// --- PREPARED CALL ---------------------
// a CONTRACT CALL, checked and ready to write.
//
// --- WHY --- the args must be a JSON OBJECT: the near-kit toolbox takes
// a json object of arguments, and the frontend hands the transactions
// straight to `generic_contract_call_wallet`. an array or a bare string
// is not a set of named arguments, and the call would fail on chain.
//
// --- WHY --- a missing args is an EMPTY OBJECT and not an error: a call
// with no arguments is an ordinary thing.
//
// --- WHY --- THERE IS A SECOND WAY TO CARRY THE ARGUMENTS: some
// contracts take their arguments as RAW BYTES and not as json — the
// FastFS protocol (`__fastdata_fastfs`) wants borsh bytes, and a json
// object cannot be borsh bytes. so a transaction may carry `argsBase64`
// instead, and the bytes ride the wire as standard base64.
//
// --- WHY --- EXACTLY ONE OF THE TWO: they are different things, and a
// transaction that carries both does not say which one the contract
// gets. refusing it is better than picking one.
//
// --- WHY --- THE BASE64 IS CHECKED HERE AND NOT ON THE WAY OUT: the
// bytes have to survive the db and come back the same, and a string
// that is not base64 is a string that will not decode into bytes. the
// check happens where the mistake is still visible — at the request.
fn prepared_call_transaction_fun(
    transaction: &TX_QUEUE_TRANSACTION_REQUEST,
    position: i64,
) -> Result<PREPARED_TRANSACTION, (String, String)> {
    let contract_id = transaction.contract_id.trim();
    if contract_id.is_empty() {
        return Err((
            "no_contract_id".to_string(),
            format!("the transaction at position {position} has no contract id"),
        ));
    }

    let method = transaction.method.trim();
    if method.is_empty() {
        return Err((
            "no_method".to_string(),
            format!("the transaction at position {position} has no method"),
        ));
    }

    // --- THE BINARY ARGS ----------------
    // the base64, when the transaction carries one.
    let args_base64 = match transaction.args_base64.as_deref() {
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Err((
                    "args_base64_not_base64".to_string(),
                    format!("the argsBase64 of the transaction at position {position} is empty"),
                ));
            }
            // --- WHY --- THE DECODE IS THE CHECK: a base64 string that
            // decodes is a base64 string, and one that does not is not.
            // the bytes are thrown away — they are read back out of the
            // base64 by the browser — but decoding them proves the
            // string is base64.
            if base64::engine::general_purpose::STANDARD
                .decode(trimmed.as_bytes())
                .is_err()
            {
                return Err((
                    "args_base64_not_base64".to_string(),
                    format!(
                        "the argsBase64 of the transaction at position {position} is not base64"
                    ),
                ));
            }
            Some(trimmed.to_string())
        }
        None => None,
    };

    // --- WHY --- AN EMPTY `args` DOES NOT COUNT AS "BOTH": the rule is
    // about what the transaction MEANS, and an empty json object is the
    // shape a binary-args call takes on the way through the db. so a
    // transaction with an empty object and a base64 is a binary-args
    // call, not a call that carries two kinds of arguments.
    let args_value = match &transaction.args {
        Some(value) => value.clone(),
        None => serde_json::json!({}),
    };
    let args_is_empty_object = args_value
        .as_object()
        .map(|object| object.is_empty())
        .unwrap_or(false);

    if args_base64.is_some() && !args_is_empty_object {
        return Err((
            "args_and_args_base64".to_string(),
            format!(
                "the transaction at position {position} carries both args and argsBase64 — it must carry only one"
            ),
        ));
    }

    // --- WHY --- THE JSON ARGS ARE STILL CHECKED FOR BEING AN OBJECT:
    // a json object is the shape the toolbox takes, and an array or a
    // bare string is not a set of named arguments.
    if !args_value.is_object() {
        return Err((
            "args_not_an_object".to_string(),
            format!("the args of the transaction at position {position} are not a json object"),
        ));
    }

    let args = match tx_queue_args_from_json_fun(&args_value) {
        Ok(args) => args,
        Err(err) => {
            return Err((
                "args_not_an_object".to_string(),
                format!("the args of the transaction at position {position} are not json: {err}"),
            ));
        }
    };

    Ok(PREPARED_TRANSACTION {
        position,
        contract_id: contract_id.to_string(),
        method: method.to_string(),
        args,
        args_base64,
        gas: transaction.gas.clone(),
        deposit: transaction.deposit.clone(),
        key_action: None,
        key_public_key: None,
        key_receiver_id: None,
        key_method_names: None,
        key_allowance: None,
    })
}

// ========================================
// --- PREPARED KEY ----------------------
// a KEY ACTION, checked and ready to write.
//
// --- WHY --- THE CONTRACT ID AND THE METHOD ARE WRITTEN AS EMPTY
// STRINGS: their columns are `NOT NULL` and making them nullable would
// need a table rebuild. the empty string means "this is not a contract
// call", and the `key_action` column is what says what it IS.
//
// --- WHY --- THE ARGS ARE THE EMPTY OBJECT: a key action has no
// arguments, and the `args` column is `NOT NULL` too.
//
// --- WHY --- A KEY ACTION MAY NOT CARRY A CONTRACT: it has no contract
// to call, so a contract id on it is a mistake in the request. and it
// may not carry args either — a key action is not a call.
fn prepared_key_transaction_fun(
    transaction: &TX_QUEUE_TRANSACTION_REQUEST,
    position: i64,
    shape: TX_QUEUE_KEY_ACTION_SHAPE,
) -> Result<PREPARED_TRANSACTION, (String, String)> {
    // --- WHY --- THE BOTH RULE IS CHECKED HERE AND NOT ON THE WAY IN:
    // the key action was read first because it is what says "this is
    // not a contract call", so this is where the "and it is not a
    // contract call either" half of the rule lives.
    if !transaction.contract_id.trim().is_empty() || !transaction.method.trim().is_empty() {
        return Err((
            "key_action_and_contract_call".to_string(),
            format!(
                "the transaction at position {position} is a key action and a contract call at once — it must be one or the other"
            ),
        ));
    }

    // --- THE METHOD NAMES ---------------
    // they ride as a json array string in the column.
    let key_method_names = match shape.method_names.as_deref() {
        Some(names) => match tx_queue_key_method_names_from_json_fun(names) {
            Ok(json) => Some(json),
            Err(err) => {
                return Err((
                    "bad_key_method_names".to_string(),
                    format!(
                        "the keyMethodNames of the transaction at position {position} are not json: {err}"
                    ),
                ));
            }
        },
        None => None,
    };

    Ok(PREPARED_TRANSACTION {
        position,
        contract_id: String::new(),
        method: String::new(),
        args: tx_queue_args_from_json_fun(&serde_json::json!({})).unwrap_or_else(|_| "{}".to_string()),
        args_base64: None,
        gas: None,
        deposit: None,
        key_action: Some(shape.action),
        key_public_key: Some(shape.public_key),
        key_receiver_id: shape.receiver_id,
        key_method_names,
        key_allowance: shape.allowance,
    })
}

// the page size, clamped.
//
// --- WHY --- a limit is CLAMPED and not refused: a caller who asks for
// a thousand is not a caller to be turned away — they want a big page,
// and a big page is fine. and a NEGATIVE limit means "no limit" in
// sqlite, so it has to be clamped too, or `limit=-1` would answer with
// every queue in the db.
fn limit_fun(asked: Option<i64>, default: i64, max: i64) -> i64 {
    match asked {
        None => default,
        Some(limit) if limit < 1 => default,
        Some(limit) => limit.min(max),
    }
}

// the `hide_sent` flag, as the bool the store wants.
//
// --- WHY --- the words AND the digits are both accepted: the plan
// spells the filter `hide_sent=1`, and a url is a url — a caller who
// writes `hide_sent=1` means it. refusing it with a 422 would be a
// surprise for a filter that everybody understands.
//
// --- WHY --- anything else is NOT an error and is read as "off": the
// flag is a filter, and a filter that is switched on by accident hides
// the rows the reader came for. a value nobody understands is treated
// as "not set".
fn hide_sent_flag_fun(raw: Option<&str>) -> Option<bool> {
    let raw = raw?.trim().to_lowercase();
    match raw.as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

// the wire shape of one queue, read back from the db.
//
// --- WHY --- it is read back and not built from what was just written:
// the answer is then the SAME shape whether it comes from the queue
// route, the one-batch route or the list route.
async fn queue_item_fun(pool: &SqlitePool, queue_id: &str) -> Result<TX_QUEUE_ITEM, Box<Response>> {
    let queue = match tx_queue_get_fun(pool, queue_id).await {
        Ok(Some(queue)) => queue,
        Ok(None) => {
            return Err(Box::new(
                shit_1170_error_fun(
                    StatusCode::NOT_FOUND,
                    "no_such_queue",
                    "there is no queue with that id",
                )
                .into_response(),
            ));
        }
        Err(err) => {
            eprintln!("[tx-queue] could not read the queue: {err}");
            return Err(Box::new(
                shit_1170_error_fun(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "db_unavailable",
                    "the queue could not be read",
                )
                .into_response(),
            ));
        }
    };

    queue_item_from_row_fun(pool, &queue).await
}

// the wire shape of one queue that has already been read.
async fn queue_item_from_row_fun(
    pool: &SqlitePool,
    queue: &TX_QUEUE_ROW,
) -> Result<TX_QUEUE_ITEM, Box<Response>> {
    let transactions = match tx_queue_transactions_list_by_queue_fun(pool, &queue.queue_id).await {
        Ok(transactions) => transactions,
        Err(err) => {
            eprintln!("[tx-queue] could not read the transactions: {err}");
            return Err(Box::new(
                shit_1170_error_fun(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "db_unavailable",
                    "the transactions could not be read",
                )
                .into_response(),
            ));
        }
    };

    let accounts = match tx_queue_accounts_list_by_queue_fun(pool, &queue.queue_id).await {
        Ok(accounts) => accounts,
        Err(err) => {
            eprintln!("[tx-queue] could not read the accounts: {err}");
            return Err(Box::new(
                shit_1170_error_fun(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "db_unavailable",
                    "the accounts could not be read",
                )
                .into_response(),
            ));
        }
    };

    match tx_queue_row_to_json_fun(queue, &transactions, &accounts) {
        Ok(item) => Ok(item),
        Err(err) => {
            eprintln!("[tx-queue] could not read the args: {err}");
            Err(Box::new(
                shit_1170_error_fun(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "args_broken",
                    "the arguments of the queue could not be read",
                )
                .into_response(),
            ))
        }
    }
}

// one transaction of the batch, ready to write.
//
// --- WHY --- this is its own struct and not the request type: the
// request carries the args as a json VALUE, and the column carries them
// as a STRING. the conversion happens once, here, so the write path
// never has to think about it.
//
// --- WHY --- THE BASE64 IS A FIELD OF ITS OWN: it is not part of the
// json args, it is the OTHER way to carry them. a row has either a json
// object or a run of bytes, and the struct says which.
//
// --- WHY --- THE KEY FIELDS ARE HERE AND NOT IN A SECOND STRUCT: a row
// is EITHER a contract call OR a key action, and one row carries both
// shapes' columns. the `key_action` field says which one it is, and a
// contract call leaves the rest at none.
struct PREPARED_TRANSACTION {
    position: i64,
    contract_id: String,
    method: String,
    args: String,
    args_base64: Option<String>,
    gas: Option<String>,
    deposit: Option<String>,
    // the key action, when this transaction is one.
    //
    // --- WHY --- IT IS THE ENUM AND NOT A STRING: the route asks "is
    // this a delete" and an enum answers that without a string
    // comparison that could be wrong.
    key_action: Option<TX_QUEUE_KEY_ACTION>,
    key_public_key: Option<String>,
    key_receiver_id: Option<String>,
    // the method names, already as the json array string the column
    // holds.
    key_method_names: Option<String>,
    key_allowance: Option<String>,
}

// the one json error shape every failure of this route answers with.
//
// --- WHY --- the same shape the trezu proxy uses, and not a new one: the
// client parses one error envelope for the whole api.
fn shit_1170_error_fun(status: StatusCode, code: &str, message: &str) -> impl IntoResponse {
    (
        status,
        Json(serde_json::json!({
          "error": {
            "code": code,
            "message": message,
          }
        })),
    )
}

// ========================================
// copyright 2026 by sleet.near
// ========================================
