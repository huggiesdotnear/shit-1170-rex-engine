// ========================================
// TX_QUEUE_STORE
// the db access for the tx_queue table. every read and every write of a
// queued batch goes through this file, so the sql lives in one place.
//
// --- WHY --- one row per BATCH and not per transaction: the agent
// queues a swap, and that swap needs three function calls — a deposit,
// the swap itself, and a withdrawal — so they go in together under one
// `queue_id`, and the user signs them as one batch, in one wallet popup.
//
// --- WHY --- the `what`, `when` and `why` live on the QUEUE and not on
// the transaction rows: they describe the batch, not one call. "swap 100
// USDC for NEAR" is the what of three calls taken together.
//
// --- WHY --- there is NO status column on this table: a queue is never
// done. at any moment another account can decide they want to send it,
// and they join. the status lives on the account row and nowhere else.
//
// --- WHY --- the `when` column is QUOTED in every statement that reads
// or writes it: `when` is a reserved word in sqlite, and an unquoted
// `when` in a column list is a syntax error. `why` and `when_at` are
// ordinary words and are left alone.
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;

use crate::backend::logic::endpoint_shit_1170_tx_queue::tx_queue_accounts_store::{
    TX_QUEUE_ACCOUNT_ROW, row_to_tx_queue_account_fun,
};
use crate::backend::logic::endpoint_shit_1170_tx_queue::tx_queue_transactions_store::{
    TX_QUEUE_TRANSACTION_ROW, row_to_tx_queue_transaction_fun,
};
use crate::backend::types::shit_1170_sql_list_fun::{SqliteBind, SqliteListBuilder};
// ========================================
// --- TYPES ------------------------------
// one queued batch, as it comes out of the db.
#[derive(Debug, Clone)]
pub struct TX_QUEUE_ROW {
    // the batch's id. it is the row's primary key, and it is the uuid
    // that goes in the url.
    pub queue_id: String,
    // what the transactions do, in words. it may be nothing.
    pub what: Option<String>,
    // when they should happen, in words. it may be nothing.
    pub when_text: Option<String>,
    // why the user wants them. it may be nothing.
    pub why: Option<String>,
    // the machine-readable version of `when`, epoch ms. it is null when
    // there is no moment.
    pub when_at: Option<i64>,
    // when the batch was queued, in ms since the epoch.
    pub created_at: i64,
    // when the batch was last touched, in ms since the epoch.
    pub updated_at: i64,
}

// the filters on the queue list.
//
// --- WHY --- the filters are all OPTIONAL and carried in one struct: the
// list query is one statement with optional binds, and the caller fills
// in the ones it was given. a struct keeps the filters named, so the
// route cannot swap two of them by accident.
//
// --- WHY --- the account and the contract filters are LISTS and not
// single values: the feed's filter bar lets a reader pick several
// accounts and several contracts, and "the queues of alice OR bob" is a
// question a single id cannot answer.
#[derive(Debug, Clone, Default)]
pub struct TX_QUEUE_LIST_QUERY {
    // only the queues these accounts are on. an empty list means "any
    // account".
    pub account_ids: Vec<String>,
    // with the account, drop the queues this account has already sent.
    pub hide_sent: Option<bool>,
    // the account the `hide_sent` filter is about.
    //
    // --- WHY --- IT IS ITS OWN FIELD AND NOT THE FIRST OF THE ACCOUNT
    // LIST: "hide what I have sent" is a question about the READER, and
    // the account list is what the reader is filtering BY. they are two
    // different questions and one field cannot answer both.
    pub hide_sent_account: Option<String>,
    // only the queues holding a transaction for one of these contracts.
    // an empty list means "any contract".
    pub contract_ids: Vec<String>,
    // only the queues holding a transaction for this method.
    pub method: Option<String>,
    // where the page starts. it is the row the page starts after.
    pub cursor: Option<TX_QUEUE_CURSOR_POINT>,
    // how many to return.
    pub limit: i64,
}

// the row a page starts after.
//
// --- WHY --- IT IS ITS OWN STRUCT AND NOT THE CURSOR TOKEN: the cursor
// token is base64 and is meant for the wire. this is what it turns into
// once it has been read, and the sql binds it as two numbers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TX_QUEUE_CURSOR_POINT {
    // the `created_at` of the row the page starts after.
    pub created_at: i64,
    // the id of that same row.
    pub queue_id: String,
}
// ========================================
// --- INSERT ----------------------------
// create the queue row for a batch.
//
// --- WHY --- the row is written BEFORE the transactions: the
// transactions point at the queue, and a foreign key that points at
// nothing is not allowed.
//
// --- WHY --- `what`, `when` and `why` are bound as OPTIONALS and not as
// empty strings: an empty string is a sentence that says nothing, and
// null says "there was no what". the ui can tell them apart.
pub async fn tx_queue_insert_fun(
    pool: &SqlitePool,
    what: Option<&str>,
    when_text: Option<&str>,
    why: Option<&str>,
    when_at: Option<i64>,
    queue_id: &str,
) -> Result<(), sqlx::Error> {
    let now = now_ms_fun();

    sqlx::query(
        "INSERT INTO tx_queue
        (queue_id, what, \"when\", why, when_at, created_at, updated_at)
       VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
    )
    .bind(queue_id)
    .bind(what)
    .bind(when_text)
    .bind(why)
    .bind(when_at)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}

// ========================================
// --- GET -------------------------------
// one queued batch by its id, or None.
//
// --- WHY --- None and not an error: a queue id nobody has is not a broken
// db, it is a 404.
pub async fn tx_queue_get_fun(
    pool: &SqlitePool,
    queue_id: &str,
) -> Result<Option<TX_QUEUE_ROW>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT queue_id, what, \"when\", why, when_at, created_at, updated_at
       FROM tx_queue
      WHERE queue_id = ?1",
    )
    .bind(queue_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| row_to_tx_queue_fun(&row)))
}

// ========================================
// --- LIST ------------------------------
// the queue list, and how many queues matched before the limit took its
// cut.
//
// --- WHY --- the total is a SECOND statement and not a window function
// over the page: the total is the count of the WHOLE match, so it has to
// be counted before the limit cut it. a window function would count only
// the rows that survived.
//
// --- WHY --- the filters are OPTIONAL binds and the same `?1` is written
// THREE times: the account id is wanted in the `EXISTS` and again in the
// `NOT EXISTS`. sqlx's sqlite driver reads the bind parameter NAMES off
// the statement, and when a name is `?NNN` it binds argument NNN to that
// slot — so `?1` means "argument one" and it can be written as many times
// as the query wants. a plain `?` would be positional and could not be
// reused.
//
// --- WHY --- the optional filters bind an `Option` and are tested with
// `?1 IS NULL`: `None` means "this filter was not given" and the clause
// is skipped. that is why one statement serves every shape of the list
// and there is no second sql to keep in step with the first.
//
// --- WHY --- `hide_sent` needs the account: "has this account sent it"
// is a question about ONE account. a queue can be sent by alice and not
// by bob, so there is no such question about all the accounts at once.
// when the caller gives a `hide_sent` with no account, the `NOT EXISTS`
// never matches anything that account did and the filter falls away —
// the ROUTE is where that is refused, because the route knows it was a
// mistake and the store only knows the shape of the data.
//
// --- WHY --- the contract and the method filters use `EXISTS` over the
// TRANSACTIONS and not over the account rows: a queue holds a
// transaction for a contract even when nobody has sent it yet, and the
// queue is what the filter is about.
//
// --- WHY --- the order is `created_at DESC`: the list is a feed, and the
// newest batch is the one the reader wants.
//
// --- WHY --- THE ORDER IS `(created_at DESC, queue_id DESC)` AND NOT
// JUST `created_at DESC`: two batches can be queued in the same
// millisecond, and the cursor that walks this order needs a total one.
// the id breaks the tie in the same direction as the time, so "the rows
// after this one" is one comparison and not two.
//
// --- WHY --- THE CURSOR IS `(created_at, queue_id) < (?, ?)` AND NOT A
// ROWID: the rowid moves when the db is vacuumed, and the cursor has
// to survive that. the pair is what the row is identified by.
pub async fn tx_queue_list_fun(
    pool: &SqlitePool,
    query: &TX_QUEUE_LIST_QUERY,
) -> Result<(Vec<TX_QUEUE_ROW>, i64), sqlx::Error> {
    // --- THE HIDE-SENT ACCOUNT ------------
    // the account the `hide_sent` filter is about.
    //
    // --- WHY --- IT IS THE FIRST ONE OF THE LIST: the filter is "has
    // THIS account sent it", and the list is a set of accounts the
    // reader is looking at. when there is one account it is that one,
    // and when there are several the first one is the reader's own —
    // the route always puts the caller's account first.
    let hide_sent_account: Option<String> = query.hide_sent_account.clone();

    // --- THE PAGE -----------------------
    let mut builder = SqliteListBuilder::new_fun(
        "SELECT queue_id, what, \"when\", why, when_at, created_at, updated_at
       FROM tx_queue
      WHERE ( ",
    );

    // --- THE CALLER'S OWN VIEW -----------
    // "is the caller on this batch" — the reader's own view of the feed.
    //
    // --- WHY --- THE CLAUSE IS ONLY WRITTEN FOR A SINGLE ACCOUNT: the
    // caller's own view is about the ONE account the caller is. when
    // the reader has picked SEVERAL accounts in the filter bar, the
    // question is "is any of THESE on this batch", and asking "is the
    // CALLER on this batch" as well would narrow the list down to the
    // caller's own batches and hide the ones the reader asked for.
    //
    // so the clause is written only when there is exactly one account,
    // and then it is that account's own view.
    if query.account_ids.len() == 1 {
        builder.push_text_fun(
            "EXISTS (SELECT 1 FROM tx_queue_accounts a
                       WHERE a.queue_id = tx_queue.queue_id
                         AND a.account_id = ",
        );
        builder.push_bind_fun(SqliteBind::Text(query.account_ids[0].clone()));
        builder.push_text_fun(" ) ");
    } else {
        // no single account to ask about, so the clause is always true.
        builder.push_text_fun("1 ");
    }
    builder.push_text_fun(") ");

    // --- THE ACCOUNT FILTER -------------
    // "is any of the picked accounts on this batch" — the filter bar's
    // multi-select.
    //
    // --- WHY --- IT IS A SEPARATE CLAUSE AND NOT PART OF THE `EXISTS`:
    // the `EXISTS` above asks "is the CALLER on this batch", and this
    // one asks "is ANY of these accounts on this batch". they are two
    // different questions — the first is the reader's own view and the
    // second is the filter bar — and one subquery cannot ask both.
    //
    // so the two are ANDed: the batch must be one the caller can see
    // AND one that carries an account from the list.
    if !query.account_ids.is_empty() {
        builder.push_text_fun(
            " AND EXISTS (SELECT 1 FROM tx_queue_accounts f
                           WHERE f.queue_id = tx_queue.queue_id
                             AND f.account_id ",
        );
        builder.push_in_fun(&query.account_ids);
        builder.push_text_fun(")");
    }

    // --- THE HIDE-SENT FILTER ------------
    //
    // --- WHY --- THE FLAG IS BOUND AND NOT WRITTEN AS A LITERAL: the
    // clause is "the flag is zero OR the account has not sent it". the
    // flag is ONE when the reader wants the sent ones hidden, and then
    // the clause asks the real question. the flag is ZERO when the
    // reader does not, and then the whole clause is true and nothing
    // is hidden.
    if hide_sent_account.is_some() {
        builder.push_text_fun(" AND ( ");
        builder.push_bind_fun(SqliteBind::Int(if query.hide_sent.unwrap_or(false) {
            1
        } else {
            0
        }));
        builder.push_text_fun(
            " = 0
           OR NOT EXISTS (SELECT 1 FROM tx_queue_accounts s
                           WHERE s.queue_id = tx_queue.queue_id
                             AND s.account_id = ",
        );
        builder.push_bind_fun(SqliteBind::Text(
            hide_sent_account.clone().unwrap_or_default(),
        ));
        builder.push_text_fun(" AND s.tx_hash IS NOT NULL) )");
    }

    // --- THE CONTRACT FILTER -------------
    // --- WHY --- IT IS AN `EXISTS` OVER THE TRANSACTIONS and not over
    // the account rows: a queue holds a transaction for a contract even
    // when nobody has sent it yet, and the queue is what the filter is
    // about.
    if !query.contract_ids.is_empty() {
        builder.push_text_fun(
            " AND EXISTS (SELECT 1 FROM tx_queue_transactions t
                           WHERE t.queue_id = tx_queue.queue_id
                             AND t.contract_id ",
        );
        builder.push_in_fun(&query.contract_ids);
        builder.push_text_fun(")");
    }

    // --- THE METHOD FILTER --------------
    if let Some(method) = query.method.as_deref() {
        builder.push_text_fun(
            " AND EXISTS (SELECT 1 FROM tx_queue_transactions t
                           WHERE t.queue_id = tx_queue.queue_id
                             AND t.method = ",
        );
        builder.push_bind_fun(SqliteBind::Text(method.to_string()));
        builder.push_text_fun(")");
    }

    // --- THE CURSOR ---------------------
    // the rows after the one the cursor names.
    if let Some(cursor) = query.cursor.as_ref() {
        builder.push_text_fun(" AND ( created_at < ");
        builder.push_bind_fun(SqliteBind::Int(cursor.created_at));
        builder.push_text_fun(" OR ( created_at = ");
        builder.push_bind_fun(SqliteBind::Int(cursor.created_at));
        builder.push_text_fun(" AND queue_id < ");
        builder.push_bind_fun(SqliteBind::Text(cursor.queue_id.clone()));
        builder.push_text_fun(" ) )");
    }

    builder.push_text_fun(" ORDER BY created_at DESC, queue_id DESC LIMIT ");
    builder.push_bind_fun(SqliteBind::Int(query.limit));

    let rows = builder.build_fun().bind_all_fun().fetch_all(pool).await?;
    let items: Vec<TX_QUEUE_ROW> = rows.iter().map(row_to_tx_queue_fun).collect();

    // --- THE TOTAL ----------------------
    // how many matched before the limit and the cursor took their cut.
    //
    // --- WHY --- THE TOTAL COUNTS THE WHOLE MATCH AND NOT THE PAGE: the
    // reader is told how many queued txs there are in total, and that
    // number has to be the whole match and not the rows that survived
    // the limit. the cursor is NOT part of the count — the total is how
    // many match the FILTERS, and it does not change as the reader
    // pages through them.
    //
    // --- WHY --- IT IS THE SAME CLAUSES BUILT AGAIN AND NOT A SECOND
    // FORM OF THE LIST: the count and the list have to agree about what
    // matched, and two hand-written forms of the same filter are two
    // chances to disagree.
    let mut total_builder = SqliteListBuilder::new_fun("SELECT COUNT(1) FROM tx_queue WHERE ( ");
    if query.account_ids.len() == 1 {
        total_builder.push_text_fun(
            "EXISTS (SELECT 1 FROM tx_queue_accounts a
                       WHERE a.queue_id = tx_queue.queue_id
                         AND a.account_id = ",
        );
        total_builder.push_bind_fun(SqliteBind::Text(query.account_ids[0].clone()));
        total_builder.push_text_fun(" ) ");
    } else {
        total_builder.push_text_fun("1 ");
    }
    total_builder.push_text_fun(") ");
    if !query.account_ids.is_empty() {
        total_builder.push_text_fun(
            " AND EXISTS (SELECT 1 FROM tx_queue_accounts f
                           WHERE f.queue_id = tx_queue.queue_id
                             AND f.account_id ",
        );
        total_builder.push_in_fun(&query.account_ids);
        total_builder.push_text_fun(")");
    }
    if hide_sent_account.is_some() {
        total_builder.push_text_fun(" AND ( ");
        total_builder.push_bind_fun(SqliteBind::Int(if query.hide_sent.unwrap_or(false) {
            1
        } else {
            0
        }));
        total_builder.push_text_fun(
            " = 0
           OR NOT EXISTS (SELECT 1 FROM tx_queue_accounts s
                           WHERE s.queue_id = tx_queue.queue_id
                             AND s.account_id = ",
        );
        total_builder.push_bind_fun(SqliteBind::Text(
            hide_sent_account.clone().unwrap_or_default(),
        ));
        total_builder.push_text_fun(" AND s.tx_hash IS NOT NULL) )");
    }
    if !query.contract_ids.is_empty() {
        total_builder.push_text_fun(
            " AND EXISTS (SELECT 1 FROM tx_queue_transactions t
                           WHERE t.queue_id = tx_queue.queue_id
                             AND t.contract_id ",
        );
        total_builder.push_in_fun(&query.contract_ids);
        total_builder.push_text_fun(")");
    }
    if let Some(method) = query.method.as_deref() {
        total_builder.push_text_fun(
            " AND EXISTS (SELECT 1 FROM tx_queue_transactions t
                           WHERE t.queue_id = tx_queue.queue_id
                             AND t.method = ",
        );
        total_builder.push_bind_fun(SqliteBind::Text(method.to_string()));
        total_builder.push_text_fun(")");
    }

    let total: i64 = total_builder
        .build_fun()
        .bind_all_fun()
        .fetch_one(pool)
        .await?
        .get(0);

    Ok((items, total))
}

// ========================================
// --- CHILDREN --------------------------
// the transactions and the account rows of a whole page of batches, in
// TWO reads.
//
// --- WHY --- THIS EXISTS AND NOT A LOOP OVER THE PAGE: the answer to
// the list route is one json object per batch, and each one holds its
// transactions and its WHO. reading them per batch is TWO queries per
// batch — a page of fifty is a hundred queries, and a thousand queued
// batches is two thousand. that is the single biggest cost in serving
// the feed, and it grows with the page. reading them for the WHOLE page
// at once is two queries, whatever the page holds.
//
// --- WHY --- THE IDS GO IN AS A LIST AND NOT AS A JOIN: a join over the
// transactions and the account rows would multiply the rows together —
// three transactions times three accounts is nine rows for one batch —
// and the reading would have to un-multiply them again. a list of ids
// keeps every row exactly as it is.
//
// --- WHY --- THE ANSWER IS GROUPED BY ID AND NOT BY POSITION: the
// batches come back in one order and the children come back in another,
// and matching them up by position would break the moment the db
// answered in a different order. the id is the join.
pub struct TX_QUEUE_PAGE_CHILDREN {
    // the transactions of every batch on the page, keyed by the batch's
    // id.
    pub transactions: HashMap<String, Vec<TX_QUEUE_TRANSACTION_ROW>>,
    // the account rows of every batch on the page, keyed by the batch's
    // id.
    pub accounts: HashMap<String, Vec<TX_QUEUE_ACCOUNT_ROW>>,
}

// the transactions and the account rows of a whole page of batches.
//
// --- WHY --- AN EMPTY PAGE IS AN EMPTY ANSWER AND NOT A QUERY: a page
// with no batches has no children, and asking the db for the children
// of nothing is a query that cannot answer anything.
pub async fn tx_queue_page_children_fun(
    pool: &SqlitePool,
    queue_ids: &[String],
) -> Result<TX_QUEUE_PAGE_CHILDREN, sqlx::Error> {
    let mut transactions: HashMap<String, Vec<TX_QUEUE_TRANSACTION_ROW>> = HashMap::new();
    let mut accounts: HashMap<String, Vec<TX_QUEUE_ACCOUNT_ROW>> = HashMap::new();

    if queue_ids.is_empty() {
        return Ok(TX_QUEUE_PAGE_CHILDREN {
            transactions,
            accounts,
        });
    }

    // --- WHY --- THE IDS ARE BOUND ONE BY ONE AND NOT AS A LIST: sqlx
    // does not bind a `Vec` as an `IN` list. the ids are bound one
    // after the other and the placeholders are the same count, so the
    // two still line up.
    let mut transaction_builder = sqlx::QueryBuilder::<sqlx::Sqlite>::new(
        "SELECT tx_id, queue_id, position, contract_id, method, args, args_base64, gas, deposit,
                key_action, key_public_key, key_receiver_id, key_method_names, key_allowance,
                created_at, updated_at
       FROM tx_queue_transactions
      WHERE queue_id IN (",
    );
    {
        let mut separated = transaction_builder.separated(", ");
        for id in queue_ids {
            separated.push_bind(id.as_str());
        }
    }
    transaction_builder.push(") ORDER BY queue_id ASC, position ASC");
    let transaction_rows = transaction_builder.build().fetch_all(pool).await?;

    for row in &transaction_rows {
        let transaction = row_to_tx_queue_transaction_fun(row);
        transactions
            .entry(transaction.queue_id.clone())
            .or_default()
            .push(transaction);
    }

    let mut account_builder = sqlx::QueryBuilder::<sqlx::Sqlite>::new(
        "SELECT queue_id, tx_id, account_id, status, tx_hash, created_at, updated_at
       FROM tx_queue_accounts
      WHERE queue_id IN (",
    );
    {
        let mut separated = account_builder.separated(", ");
        for id in queue_ids {
            separated.push_bind(id.as_str());
        }
    }
    account_builder.push(") ORDER BY queue_id ASC, created_at ASC, tx_id ASC, account_id ASC");
    let account_rows = account_builder.build().fetch_all(pool).await?;

    for row in &account_rows {
        let account = row_to_tx_queue_account_fun(row);
        accounts
            .entry(account.queue_id.clone())
            .or_default()
            .push(account);
    }

    Ok(TX_QUEUE_PAGE_CHILDREN {
        transactions,
        accounts,
    })
}

// ========================================
// --- DELETE ----------------------------
// delete a queue. its transactions and their account rows go with it.
//
// --- WHY --- it is a hard delete and not a soft one: a queued batch is
// immutable once queued, there is no edit, and there is no expiry. a
// soft delete would leave rows nobody can read behind a flag that
// nothing looks at.
pub async fn tx_queue_delete_fun(pool: &SqlitePool, queue_id: &str) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM tx_queue WHERE queue_id = ?1")
        .bind(queue_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}
// ========================================
// --- HELPERS ----------------------------
// turn a db row into a queued batch.
fn row_to_tx_queue_fun(row: &sqlx::sqlite::SqliteRow) -> TX_QUEUE_ROW {
    TX_QUEUE_ROW {
        queue_id: row.get("queue_id"),
        what: row.get("what"),
        when_text: row.get("when"),
        why: row.get("why"),
        when_at: row.get("when_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

// the current time in ms since the epoch.
pub fn now_ms_fun() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
