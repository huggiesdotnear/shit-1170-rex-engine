// ========================================
// SWAP_QUEUE_STORE
// the db access for the swap_queue table. every read and every write of a
// queued swap goes through this file, so the sql lives in one place.
//
// --- WHY --- one row per SWAP and not per transaction: the agent does
// not know the route. it knows the token in, the amount in and the token
// out, and the route is worked out at the moment somebody executes it.
//
// --- WHY --- the `what`, `when` and `why` live on the SWAP: they
// describe the swap, and they are the same three fields the tx queue
// carries, so a reader sees one kind of post whether it is a swap or a
// batch of calls.
//
// --- WHY --- there is NO status column on this table: a queued swap is
// never done. at any moment another account can decide they want it,
// and they join. the status lives on the account row and nowhere else.
//
// --- WHY --- the `when` column is QUOTED in every statement that reads
// or writes it: `when` is a reserved word in sqlite, and an unquoted
// `when` in a column list is a syntax error. `why` and `when_at` are
// ordinary words and are left alone.
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;

use crate::backend::logic::endpoint_shit_1170_swap_queue::swap_queue_accounts_store::{
    SWAP_QUEUE_ACCOUNT_ROW, row_to_swap_queue_account_fun,
};
use crate::backend::types::shit_1170_sql_list_fun::{SqliteBind, SqliteListBuilder};
// ========================================
// --- TYPES ------------------------------
// one queued swap, as it comes out of the db.
#[derive(Debug, Clone)]
pub struct SWAP_QUEUE_ROW {
    // the swap's id. it is the row's primary key, and it is the uuid
    // that goes in the url.
    pub swap_id: String,
    // what the swap does, in words. it may be nothing.
    pub what: Option<String>,
    // when it should happen, in words. it may be nothing.
    pub when_text: Option<String>,
    // why the user wants it. it may be nothing.
    pub why: Option<String>,
    // the machine-readable version of `when`, epoch ms. it is null when
    // there is no moment.
    pub when_at: Option<i64>,
    // the token being sold.
    pub token_in: String,
    // the raw amount of the token in that is sold, in the token's
    // smallest unit.
    pub amount_in: String,
    // the token being bought.
    pub token_out: String,
    // the quoted amount out, in the output token's smallest unit. it is
    // null when nothing was quoted yet.
    pub amount_out: Option<String>,
    // the dex the quote came from. it is null when no dex was named.
    pub dex_id: Option<String>,
    // when the swap was queued, in ms since the epoch.
    pub created_at: i64,
    // when the swap was last touched, in ms since the epoch.
    pub updated_at: i64,
}

// the filters on the swap list.
//
// --- WHY --- the filters are all OPTIONAL and carried in one struct: the
// list query is one statement with optional binds, and the caller fills
// in the ones it was given. a struct keeps the filters named, so the
// route cannot swap two of them by accident.
//
// --- WHY --- the account filter is a LIST and not a single value: the
// feed's filter bar lets a reader pick several accounts, and "the swaps
// of alice OR bob" is a question one id cannot answer.
//
// --- WHY --- the contract filter is a LIST too: the same bar lets a
// reader pick several contracts, and "the swaps of usdt OR wrap" is a
// question one id cannot answer either.
#[derive(Debug, Clone, Default)]
pub struct SWAP_QUEUE_LIST_QUERY {
    // only the swaps these accounts are on. an empty list means "any
    // account".
    pub account_ids: Vec<String>,
    // with the account, drop the swaps this account has already sent.
    pub hide_sent: Option<bool>,
    // the account the `hide_sent` filter is about.
    //
    // --- WHY --- IT IS ITS OWN FIELD AND NOT THE FIRST OF THE ACCOUNT
    // LIST: "hide what I have sent" is a question about the READER, and
    // the account list is what the reader is filtering BY. they are two
    // different questions and one field cannot answer both.
    pub hide_sent_account: Option<String>,
    // only the swaps that sell this token.
    pub token_in: Option<String>,
    // only the swaps that buy this token.
    pub token_out: Option<String>,
    // only the swaps that touch one of these contracts — the token they
    // sell OR the token they buy. an empty list means "any contract".
    pub contract_ids: Vec<String>,
    // where the page starts. it is the row the page starts after.
    pub cursor: Option<SWAP_QUEUE_CURSOR_POINT>,
    // how many to return.
    pub limit: i64,
}

// the row a page starts after.
//
// --- WHY --- IT IS ITS OWN STRUCT AND NOT THE CURSOR TOKEN: the cursor
// token is base64 and is meant for the wire. this is what it turns into
// once it has been read, and the sql binds it as two numbers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SWAP_QUEUE_CURSOR_POINT {
    // the `created_at` of the row the page starts after.
    pub created_at: i64,
    // the id of that same row.
    pub swap_id: String,
}
// ========================================
// --- INSERT ----------------------------
// create the swap row.
//
// --- WHY --- the row is written BEFORE the account rows: they point at
// it, and a foreign key that points at nothing is not allowed.
//
// --- WHY --- `what`, `when` and `why` are bound as OPTIONALS and not as
// empty strings: an empty string is a sentence that says nothing, and
// null says "there was no what". the ui can tell them apart.
//
// --- WHY --- `amount_out` and `dex_id` are OPTIONALS: the agent may
// queue a swap without a quote, and the quote is worked out live anyway.
pub async fn swap_queue_insert_fun(
    pool: &SqlitePool,
    swap_id: &str,
    what: Option<&str>,
    when_text: Option<&str>,
    why: Option<&str>,
    when_at: Option<i64>,
    token_in: &str,
    amount_in: &str,
    token_out: &str,
    amount_out: Option<&str>,
    dex_id: Option<&str>,
) -> Result<(), sqlx::Error> {
    let now = now_ms_fun();

    sqlx::query(
        "INSERT INTO swap_queue
        (swap_id, what, \"when\", why, when_at, token_in, amount_in, token_out,
         amount_out, dex_id, created_at, updated_at)
       VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)",
    )
    .bind(swap_id)
    .bind(what)
    .bind(when_text)
    .bind(why)
    .bind(when_at)
    .bind(token_in)
    .bind(amount_in)
    .bind(token_out)
    .bind(amount_out)
    .bind(dex_id)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}

// ========================================
// --- GET -------------------------------
// one queued swap by its id, or None.
//
// --- WHY --- None and not an error: a swap id nobody has is not a broken
// db, it is a 404.
pub async fn swap_queue_get_fun(
    pool: &SqlitePool,
    swap_id: &str,
) -> Result<Option<SWAP_QUEUE_ROW>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT swap_id, what, \"when\", why, when_at, token_in, amount_in,
                token_out, amount_out, dex_id, created_at, updated_at
       FROM swap_queue
      WHERE swap_id = ?1",
    )
    .bind(swap_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| row_to_swap_queue_fun(&row)))
}

// ========================================
// --- LIST ------------------------------
// the swap list, and how many swaps matched before the limit and the
// offset took their cut.
//
// --- WHY --- the total is a SECOND statement and not a window function
// over the page: the total is the count of the WHOLE match, so it has to
// be counted before the limit and the offset cut it. a window function
// would count only the rows that survived.
//
// --- WHY --- the optional filters bind an `Option` and are tested with
// `?1 IS NULL`: `None` means "this filter was not given" and the clause
// is skipped. that is why one statement serves every shape of the list
// and there is no second sql to keep in step with the first.
//
// --- WHY --- `hide_sent` needs the account: "has this account sent it"
// is a question about ONE account. a swap can be sent by alice and not
// by bob, so there is no such question about all the accounts at once.
// when the caller gives a `hide_sent` with no account, the `NOT EXISTS`
// never matches anything that account did and the filter falls away —
// the ROUTE is where that is refused, because the route knows it was a
// mistake and the store only knows the shape of the data.
//
// --- WHY --- the token filters are plain comparisons and not `EXISTS`
// over another table: the token in and the token out are COLUMNS on the
// swap, not rows on a transaction table. a queued swap has no
// transactions until somebody signs it.
//
// --- WHY --- the order is `created_at DESC`: the list is a feed, and the
// newest swap is the one the reader wants.
//
// --- WHY --- THE ORDER IS `(created_at DESC, swap_id DESC)` AND NOT
// JUST `created_at DESC`: two swaps can be queued in the same
// millisecond, and the cursor that walks this order needs a total one.
// the id breaks the tie in the same direction as the time, so "the rows
// after this one" is one comparison and not two.
pub async fn swap_queue_list_fun(
    pool: &SqlitePool,
    query: &SWAP_QUEUE_LIST_QUERY,
) -> Result<(Vec<SWAP_QUEUE_ROW>, i64), sqlx::Error> {
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
        "SELECT swap_id, what, \"when\", why, when_at, token_in, amount_in,
                token_out, amount_out, dex_id, created_at, updated_at
       FROM swap_queue
      WHERE ( ",
    );

    // --- THE CALLER'S OWN VIEW -----------
    // "is the caller on this swap" — the reader's own view of the feed.
    //
    // --- WHY --- THE CLAUSE IS ONLY WRITTEN FOR A SINGLE ACCOUNT: the
    // caller's own view is about the ONE account the caller is. when
    // the reader has picked SEVERAL accounts in the filter bar, the
    // question is "is any of THESE on this swap", and asking "is the
    // CALLER on this swap" as well would narrow the list down to the
    // caller's own swaps and hide the ones the reader asked for.
    if query.account_ids.len() == 1 {
        builder.push_text_fun(
            "EXISTS (SELECT 1 FROM swap_queue_accounts a
                       WHERE a.swap_id = swap_queue.swap_id
                         AND a.account_id = ",
        );
        builder.push_bind_fun(SqliteBind::Text(query.account_ids[0].clone()));
        builder.push_text_fun(" ) ");
    } else {
        builder.push_text_fun("1 ");
    }
    builder.push_text_fun(") ");

    // --- THE ACCOUNT FILTER -------------
    // "is any of the picked accounts on this swap" — the filter bar's
    // multi-select.
    //
    // --- WHY --- IT IS A SEPARATE CLAUSE AND NOT PART OF THE `EXISTS`:
    // the `EXISTS` above asks "is the CALLER on this swap", and this
    // one asks "is ANY of these accounts on this swap". they are two
    // different questions, and one subquery cannot ask both.
    if !query.account_ids.is_empty() {
        builder.push_text_fun(
            " AND EXISTS (SELECT 1 FROM swap_queue_accounts f
                           WHERE f.swap_id = swap_queue.swap_id
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
        builder.push_text_fun(" = 0 OR ");
        // --- WHY --- THE HISTORY PREDICATE IS THE SHARED ONE: "a row
        // that carries a hash is history" is the rule the whole app turns
        // on — the feed hides it, the list counts it, and the prune
        // refuses to take it. one definition of it, or three answers to
        // the same question.
        builder.push_text_fun(
            &crate::backend::logic::db::db_history_fun::db_history_account_not_sent_before_fun(
                crate::backend::logic::db::db_history_fun::DB_HISTORY_TABLE::SwapQueue,
                crate::backend::logic::db::db_history_fun::DB_HISTORY_ACCOUNT_MARKER,
            ),
        );
        builder.push_bind_at_marker_fun(
            crate::backend::logic::db::db_history_fun::DB_HISTORY_ACCOUNT_MARKER,
            SqliteBind::Text(hide_sent_account.clone().unwrap_or_default()),
        );
        builder.push_text_fun(
            &crate::backend::logic::db::db_history_fun::db_history_account_not_sent_after_fun(),
        );
        builder.push_text_fun(" )");
    }

    // --- THE TOKEN FILTERS ---------------
    // --- WHY --- THEY ARE PLAIN COMPARISONS AND NOT `EXISTS` OVER
    // ANOTHER TABLE: the token in and the token out are COLUMNS on the
    // swap, not rows on a transaction table. a queued swap has no
    // transactions until somebody signs it.
    if let Some(token_in) = query.token_in.as_deref() {
        builder.push_text_fun(" AND token_in = ");
        builder.push_bind_fun(SqliteBind::Text(token_in.to_string()));
    }
    if let Some(token_out) = query.token_out.as_deref() {
        builder.push_text_fun(" AND token_out = ");
        builder.push_bind_fun(SqliteBind::Text(token_out.to_string()));
    }

    // --- THE CONTRACT FILTER -------------
    // "is one of the picked contracts on this swap" — the filter bar's
    // multi-select of contracts.
    //
    // --- WHY --- IT MATCHES THE TOKEN IN OR THE TOKEN OUT: a queued
    // swap NAMES two contracts — the one it sells and the one it buys —
    // and the reader picked a contract, not a side of the trade. so
    // picking usdt shows the swaps that SELL usdt AND the swaps that BUY
    // usdt. a filter that only looked at one side would hide half of
    // what the reader asked for.
    //
    // --- WHY --- THE SAME LIST IS BOUND TWICE AND NOT WRITTEN TWICE:
    // the builder numbers its placeholders by the order the values were
    // bound, so the second `push_in_fun` writes `?N+1 ...` and binds the
    // list again. one list, two bindings, and the two sides of the `OR`
    // cannot drift apart — they are the same values in the same order.
    if !query.contract_ids.is_empty() {
        builder.push_text_fun(" AND ( token_in ");
        builder.push_in_fun(&query.contract_ids);
        builder.push_text_fun(" OR token_out ");
        builder.push_in_fun(&query.contract_ids);
        builder.push_text_fun(" )");
    }

    // --- THE CURSOR ---------------------
    // the rows after the one the cursor names.
    if let Some(cursor) = query.cursor.as_ref() {
        builder.push_text_fun(" AND ( created_at < ");
        builder.push_bind_fun(SqliteBind::Int(cursor.created_at));
        builder.push_text_fun(" OR ( created_at = ");
        builder.push_bind_fun(SqliteBind::Int(cursor.created_at));
        builder.push_text_fun(" AND swap_id < ");
        builder.push_bind_fun(SqliteBind::Text(cursor.swap_id.clone()));
        builder.push_text_fun(" ) )");
    }

    builder.push_text_fun(" ORDER BY created_at DESC, swap_id DESC LIMIT ");
    builder.push_bind_fun(SqliteBind::Int(query.limit));

    let rows = builder.build_fun().bind_all_fun().fetch_all(pool).await?;
    let items: Vec<SWAP_QUEUE_ROW> = rows.iter().map(row_to_swap_queue_fun).collect();

    // --- THE TOTAL ----------------------
    // how many matched before the limit and the cursor took their cut.
    //
    // --- WHY --- THE TOTAL COUNTS THE WHOLE MATCH AND NOT THE PAGE: the
    // reader is told how many queued swaps there are in total, and that
    // number has to be the whole match and not the rows that survived
    // the limit. the cursor is NOT part of the count.
    //
    // --- WHY --- IT IS THE SAME CLAUSES BUILT AGAIN AND NOT A SECOND
    // FORM OF THE LIST: the count and the list have to agree about what
    // matched, and two hand-written forms of the same filter are two
    // chances to disagree.
    let mut total_builder = SqliteListBuilder::new_fun("SELECT COUNT(1) FROM swap_queue WHERE ( ");
    if query.account_ids.len() == 1 {
        total_builder.push_text_fun(
            "EXISTS (SELECT 1 FROM swap_queue_accounts a
                       WHERE a.swap_id = swap_queue.swap_id
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
            " AND EXISTS (SELECT 1 FROM swap_queue_accounts f
                           WHERE f.swap_id = swap_queue.swap_id
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
        total_builder.push_text_fun(" = 0 OR ");
        // --- WHY --- THE HISTORY PREDICATE IS THE SHARED ONE: see the
        // note on the list query above.
        total_builder.push_text_fun(
            &crate::backend::logic::db::db_history_fun::db_history_account_not_sent_before_fun(
                crate::backend::logic::db::db_history_fun::DB_HISTORY_TABLE::SwapQueue,
                crate::backend::logic::db::db_history_fun::DB_HISTORY_ACCOUNT_MARKER,
            ),
        );
        total_builder.push_bind_at_marker_fun(
            crate::backend::logic::db::db_history_fun::DB_HISTORY_ACCOUNT_MARKER,
            SqliteBind::Text(hide_sent_account.clone().unwrap_or_default()),
        );
        total_builder.push_text_fun(
            &crate::backend::logic::db::db_history_fun::db_history_account_not_sent_after_fun(),
        );
        total_builder.push_text_fun(" )");
    }
    if let Some(token_in) = query.token_in.as_deref() {
        total_builder.push_text_fun(" AND token_in = ");
        total_builder.push_bind_fun(SqliteBind::Text(token_in.to_string()));
    }
    if let Some(token_out) = query.token_out.as_deref() {
        total_builder.push_text_fun(" AND token_out = ");
        total_builder.push_bind_fun(SqliteBind::Text(token_out.to_string()));
    }
    // --- WHY --- THE CONTRACT CLAUSE IS HERE TOO AND NOT ONLY IN THE
    // LIST: the total counts the WHOLE match, and a total that did not
    // carry the contract clause would count every swap on the feed and
    // tell the reader there are more pages than there are. the count and
    // the list have to agree about what matched.
    if !query.contract_ids.is_empty() {
        total_builder.push_text_fun(" AND ( token_in ");
        total_builder.push_in_fun(&query.contract_ids);
        total_builder.push_text_fun(" OR token_out ");
        total_builder.push_in_fun(&query.contract_ids);
        total_builder.push_text_fun(" )");
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
// --- PAGE CHILDREN ---------------------
// the account rows of a whole page of swaps, in ONE read.
//
// --- WHY --- THIS EXISTS AND NOT A LOOP OVER THE PAGE: the answer to
// the list route is one json object per swap, and each one holds its
// WHO. reading them per swap is one query per swap — a page of fifty is
// fifty queries, and a thousand queued swaps is a thousand. that is the
// single biggest cost in serving the feed, and it grows with the page.
// reading them for the WHOLE page at once is one query, whatever the
// page holds.
//
// --- WHY --- THE ANSWER IS GROUPED BY ID AND NOT BY POSITION: the swaps
// come back in one order and the children come back in another, and
// matching them up by position would break the moment the db answered
// in a different order. the id is the join.
pub async fn swap_queue_page_accounts_fun(
    pool: &SqlitePool,
    swap_ids: &[String],
) -> Result<HashMap<String, Vec<SWAP_QUEUE_ACCOUNT_ROW>>, sqlx::Error> {
    let mut accounts: HashMap<String, Vec<SWAP_QUEUE_ACCOUNT_ROW>> = HashMap::new();

    // --- WHY --- AN EMPTY PAGE IS AN EMPTY ANSWER AND NOT A QUERY: a page
    // with no swaps has no children, and asking the db for the children
    // of nothing is a query that cannot answer anything.
    if swap_ids.is_empty() {
        return Ok(accounts);
    }

    let mut builder = SqliteListBuilder::new_fun(
        "SELECT swap_id, account_id, status, tx_hash, amount_out, created_at, updated_at
       FROM swap_queue_accounts
      WHERE swap_id ",
    );
    builder.push_in_fun(swap_ids);
    builder.push_text_fun(" ORDER BY swap_id ASC, created_at ASC, account_id ASC");

    let rows = builder.build_fun().bind_all_fun().fetch_all(pool).await?;

    for row in &rows {
        let account = row_to_swap_queue_account_fun(row);
        accounts
            .entry(account.swap_id.clone())
            .or_default()
            .push(account);
    }

    Ok(accounts)
}

// ========================================
// --- UPDATE QUOTE ----------------------
// write the quote that was taken for a swap.
//
// --- WHY --- THE QUOTE IS WRITTEN BACK AND NOT ONLY READ: the feed asks
// the router for a live quote, and the live quote is a fact about this
// swap at this moment. keeping it means the next reader sees the last
// known quote even when the router is down.
//
// --- WHY --- THE QUOTE DOES NOT TOUCH `created_at`: the swap was queued
// when it was queued, and a later quote does not make it newer.
pub async fn swap_queue_set_quote_fun(
    pool: &SqlitePool,
    swap_id: &str,
    amount_out: &str,
    dex_id: Option<&str>,
) -> Result<u64, sqlx::Error> {
    let now = now_ms_fun();

    let result = sqlx::query(
        "UPDATE swap_queue
        SET amount_out = ?2,
            dex_id = ?3,
            updated_at = ?4
      WHERE swap_id = ?1",
    )
    .bind(swap_id)
    .bind(amount_out)
    .bind(dex_id)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

// ========================================
// --- HELPERS ----------------------------
// turn a db row into a queued swap.
fn row_to_swap_queue_fun(row: &sqlx::sqlite::SqliteRow) -> SWAP_QUEUE_ROW {
    SWAP_QUEUE_ROW {
        swap_id: row.get("swap_id"),
        what: row.get("what"),
        when_text: row.get("when"),
        why: row.get("why"),
        when_at: row.get("when_at"),
        token_in: row.get("token_in"),
        amount_in: row.get("amount_in"),
        token_out: row.get("token_out"),
        amount_out: row.get("amount_out"),
        dex_id: row.get("dex_id"),
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
