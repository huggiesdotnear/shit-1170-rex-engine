// ========================================
// DB_ADMIN_PRUNE_FUN
// what to delete from a vault: the filters, the plan, and the delete.
//
// --- WHY --- the plan and the delete are TWO STEPS: the first says what
// would go and the second takes it. a bin that deletes rows must be
// able to show the plan before it acts, and the plan must be built by
// the same code that does the deleting, so the two cannot disagree.
//
// --- WHY --- THE QUEUE IS DELETED AND NOT THE ACCOUNT ROWS: the
// account rows hang off the transactions and the transactions hang off
// the queue, all with ON DELETE CASCADE. deleting the queue takes its
// transactions and their account rows with it, in one statement, and the
// db's own rules do the work.
//
// --- WHY --- "NEVER SENT" IS ABOUT THE ACCOUNT ROWS and not about the
// queue: the status lives on the account row and nowhere else. a queue
// is never done, so "was this queue ever sent" is really "did any
// account on it ever send". the filter looks for queues where NO
// account row carries a hash.
//
// --- WHY --- A QUEUE THAT WAS SENT IS LEFT WHOLE unless `--include-sent`
// is given: a row that carries a hash is HISTORY — it shows who did the
// transaction and it stays. so the default prune skips a queue that any
// account has sent, and the flag says "take that history too". there is
// no half measure that keeps the account row and drops its transaction,
// because that would leave the row pointing at nothing.
use sqlx::AssertSqlSafe;
use sqlx::{Row, SqlitePool};
// ========================================
// --- TYPES ------------------------------
// the filters that say what to delete.
#[derive(Debug, Clone, Default)]
pub struct DB_ADMIN_PRUNE_FILTERS {
    // only the queues and swaps older than this many days.
    pub older_than_days: Option<u32>,
    // only the queues and swaps created before this moment, in epoch ms.
    pub before_ms: Option<i64>,
    // only the queues and swaps created after this moment, in epoch ms.
    pub after_ms: Option<i64>,
    // only the queues and swaps where NO account ever sent.
    pub never_sent: bool,
    // only these queue ids.
    pub queue_ids: Vec<String>,
    // only these swap ids.
    pub swap_ids: Vec<String>,
    // take the queues and swaps that were sent too.
    pub include_sent: bool,
    // only the queues and swaps of this account.
    pub account_id: Option<String>,
    // take the queued swaps as well as the queued batches.
    //
    // --- WHY --- THE SWAPS ARE OPTIONAL: a prune that names a queue id
    // is about that queue. taking the swaps too would mean a prune that
    // asked for one thing took another, and the swap would have matched
    // only by its age, which is not what was asked for.
    pub include_swaps: bool,
}
// ========================================
// --- THE OPPOSITE FLAGS ----------------
// do these two filters ask opposite questions?
//
// --- WHY --- THE RULE IS HERE and not in the bin: the question is about
// the FILTERS and not about the command line, and the bin asks it before
// it plans anything. a rule in the bin would be a rule the tests could
// not reach.
//
// --- WHY --- THE PAIR IS REFUSED and not resolved: `--never-sent` says
// "only the rows nobody sent" and `--include-sent` says "the rows that
// were sent too". a prune cannot do both — one of them has to lose, and a
// tool that deletes rows should not pick which one silently. the person
// who typed both knows what they want.
pub fn db_admin_prune_filters_are_opposite_fun(filters: &DB_ADMIN_PRUNE_FILTERS) -> bool {
    filters.never_sent && filters.include_sent
}

// ========================================
// --- PLAN ------------------------------
// what would be deleted.
#[derive(Debug, Clone, Default)]
pub struct DB_ADMIN_PRUNE_PLAN {
    // the queue ids that would go.
    pub queue_ids: Vec<String>,
    // the swap ids that would go.
    pub swap_ids: Vec<String>,
    // the transaction rows that would go with the queues.
    pub transaction_rows: u64,
    // the account rows that would go with the queues.
    pub account_rows: u64,
    // the swap account rows that would go with the swaps.
    pub swap_account_rows: u64,
}
// ========================================
// --- BIND ------------------------------
// a value to bind into a where clause.
//
// --- WHY --- THE BIND IS ITS OWN TYPE and not a plain string: the filter
// values come from a command line, and a value that goes into a query as
// text is a way to run a query the caller did not ask for. the bind says
// what kind of value it is so sqlite is told the truth about it.
#[derive(Debug, Clone)]
pub enum SqlxBind {
    // an integer.
    Int(i64),
    // a string.
    Text(String),
}

// ========================================
// --- WHERE -----------------------------
// the `where` clause that matches the queues, and the binds it needs.
//
// --- WHY --- the clause is BUILT as a string and the values are BOUND:
// the filter values come from a command line, and a value that goes into
// a query as text is a way to run a query the caller did not ask for.
//
// --- WHY --- `EXISTS` for the never-sent filter and not a join: the
// filter asks whether there is NO account row with a hash, and that is
// a question about the absence of a row.
//
// --- WHY --- THE SENT ROWS ARE EXCLUDED UNLESS `--include-sent` IS
// GIVEN: the same history rule as the delete routes. the `NOT EXISTS`
// is the whole rule, and without the flag a queue that any account has
// sent does not match at all.
//
// --- WHY --- the account filter is an `EXISTS` over the account rows of
// the queue: the account may be on ONE transaction of the batch and not
// on the others, and "the queues this account is on" is the question.
fn db_admin_prune_queue_where_fun(
    filters: &DB_ADMIN_PRUNE_FILTERS,
    binds: &mut Vec<SqlxBind>,
) -> String {
    let mut clauses: Vec<String> = Vec::new();

    if let Some(days) = filters.older_than_days {
        let cutoff = crate::backend::logic::db::bin::db_admin_now_fun::now_ms_fun()
            - (days as i64) * 24 * 60 * 60 * 1000;
        clauses.push(format!("created_at < ?{}", binds.len() + 1));
        binds.push(SqlxBind::Int(cutoff));
    }
    if let Some(before_ms) = filters.before_ms {
        clauses.push(format!("created_at < ?{}", binds.len() + 1));
        binds.push(SqlxBind::Int(before_ms));
    }
    if let Some(after_ms) = filters.after_ms {
        clauses.push(format!("created_at > ?{}", binds.len() + 1));
        binds.push(SqlxBind::Int(after_ms));
    }
    if !filters.queue_ids.is_empty() {
        let names: Vec<String> = filters
            .queue_ids
            .iter()
            .enumerate()
            .map(|(offset, _)| format!("?{}", binds.len() + 1 + offset))
            .collect();
        clauses.push(format!("queue_id IN ({})", names.join(", ")));
        for id in &filters.queue_ids {
            binds.push(SqlxBind::Text(id.clone()));
        }
    }
    // --- WHY --- THE HISTORY RULE IS THE SHARED ONE and not a copy of
    // it: "a row that carries a hash is history" is the rule the whole app
    // turns on, and the feed and the list ask it with the same predicate.
    // a second copy of it here is a second answer to the same question.
    //
    // --- WHY --- `never_sent` AND THE DEFAULT ARE THE SAME PREDICATE:
    // `--never-sent` asks for the rows nobody sent, and NOT sending
    // history is the default. so they are the same clause, and the
    // default is written once and the flag adds nothing to it.
    //
    // --- WHY --- THE FLAG CANNOT BE COMBINED WITH `--include-sent`: they
    // ask opposite questions — "only the ones nobody sent" and "the ones
    // that were sent too" — and a prune that took both at once would have
    // to answer both. the bin refuses the pair instead of picking one.
    if filters.never_sent || !filters.include_sent {
        clauses.push(
            crate::backend::logic::db::db_history_fun::db_history_not_sent_sql_fun(
                crate::backend::logic::db::db_history_fun::DB_HISTORY_TABLE::TxQueue,
            ),
        );
    }
    if let Some(account_id) = &filters.account_id {
        clauses.push(format!(
            "EXISTS (
               SELECT 1 FROM tx_queue_accounts a
                WHERE a.queue_id = tx_queue.queue_id
                  AND a.account_id = ?{})",
            binds.len() + 1
        ));
        binds.push(SqlxBind::Text(account_id.clone()));
    }

    if clauses.is_empty() {
        return String::new();
    }
    format!(" WHERE {}", clauses.join(" AND "))
}

// ========================================
// --- WHERE FOR THE SWAPS ----------------
// the `where` clause that matches the swaps.
//
// --- WHY --- THE SWAPS HAVE THEIR OWN WHERE BUILDER and not a shared
// one: the swap table has no `tx_queue_accounts` under it, so the
// never-sent and the account filters read a different table for them.
fn db_admin_prune_swap_where_fun(
    filters: &DB_ADMIN_PRUNE_FILTERS,
    binds: &mut Vec<SqlxBind>,
) -> String {
    let mut clauses: Vec<String> = Vec::new();

    if let Some(days) = filters.older_than_days {
        let cutoff = crate::backend::logic::db::bin::db_admin_now_fun::now_ms_fun()
            - (days as i64) * 24 * 60 * 60 * 1000;
        clauses.push(format!("created_at < ?{}", binds.len() + 1));
        binds.push(SqlxBind::Int(cutoff));
    }
    if let Some(before_ms) = filters.before_ms {
        clauses.push(format!("created_at < ?{}", binds.len() + 1));
        binds.push(SqlxBind::Int(before_ms));
    }
    if let Some(after_ms) = filters.after_ms {
        clauses.push(format!("created_at > ?{}", binds.len() + 1));
        binds.push(SqlxBind::Int(after_ms));
    }
    if !filters.swap_ids.is_empty() {
        let names: Vec<String> = filters
            .swap_ids
            .iter()
            .enumerate()
            .map(|(offset, _)| format!("?{}", binds.len() + 1 + offset))
            .collect();
        clauses.push(format!("swap_id IN ({})", names.join(", ")));
        for id in &filters.swap_ids {
            binds.push(SqlxBind::Text(id.clone()));
        }
    }
    if filters.never_sent || !filters.include_sent {
        clauses.push(
            crate::backend::logic::db::db_history_fun::db_history_not_sent_sql_fun(
                crate::backend::logic::db::db_history_fun::DB_HISTORY_TABLE::SwapQueue,
            ),
        );
    }
    if let Some(account_id) = &filters.account_id {
        clauses.push(format!(
            "EXISTS (
               SELECT 1 FROM swap_queue_accounts a
                WHERE a.swap_id = swap_queue.swap_id
                  AND a.account_id = ?{})",
            binds.len() + 1
        ));
        binds.push(SqlxBind::Text(account_id.clone()));
    }

    if clauses.is_empty() {
        return String::new();
    }
    format!(" WHERE {}", clauses.join(" AND "))
}

// ========================================
// --- RUN A QUERY -----------------------
// run a query with the binds, in one place.
//
// --- WHY --- the two plan queries are the same shape and the same bind
// list, and a place that binds them once means a mistake in the bind
// list cannot happen twice.
//
// --- WHY --- THE COLUMN NAME IS A PARAMETER and not read out of the
// query text: a query that happens to contain the word "swap_id" in it
// would make the reader pick the wrong column, and the check would be
// reading the sql to work out what the sql means.
async fn db_admin_prune_run_id_query_fun(
    pool: &SqlitePool,
    query: &str,
    column: &str,
    binds: &[SqlxBind],
) -> Result<Vec<String>, sqlx::Error> {
    let mut statement = sqlx::query(AssertSqlSafe(query.to_string()));
    for bind in binds {
        statement = match bind {
            SqlxBind::Int(value) => statement.bind(*value),
            SqlxBind::Text(value) => statement.bind(value.clone()),
        };
    }
    let rows = statement.fetch_all(pool).await?;
    Ok(rows
        .iter()
        .map(|row| row.get::<String, _>(column))
        .collect())
}

// ========================================
// --- PLAN THE QUEUES -------------------
// the queue ids the filters match.
//
// --- WHY --- THE QUEUES ARE ONLY PLANNED WHEN THEY WERE ASKED FOR: a
// prune that names a swap id is about that swap. the queues are not part
// of the plan unless one of the queue filters matched them.
//
// --- WHY --- A QUEUE ID ON ITS OWN IS ENOUGH: naming a queue id says
// "this queue", and it does not need anything else on top of it.
pub async fn db_admin_prune_queue_plan_fun(
    pool: &SqlitePool,
    filters: &DB_ADMIN_PRUNE_FILTERS,
) -> Result<Vec<String>, sqlx::Error> {
    let asks_for_queues = filters.older_than_days.is_some()
        || filters.before_ms.is_some()
        || filters.after_ms.is_some()
        || filters.never_sent
        || !filters.queue_ids.is_empty()
        || filters.account_id.is_some();

    if !asks_for_queues {
        return Ok(Vec::new());
    }

    let mut binds: Vec<SqlxBind> = Vec::new();
    let where_clause = db_admin_prune_queue_where_fun(filters, &mut binds);

    let mut query = String::from("SELECT queue_id FROM tx_queue");
    query.push_str(&where_clause);
    query.push_str(" ORDER BY created_at ASC, queue_id ASC");

    db_admin_prune_run_id_query_fun(pool, &query, "queue_id", &binds).await
}

// ========================================
// --- PLAN THE SWAPS --------------------
// the swap ids the filters match.
//
// --- WHY --- THE SWAPS ARE ONLY PLANNED WHEN THEY WERE ASKED FOR: a
// prune that names a queue id is about that queue. without `--swaps` or
// `--swap-ids` the swaps are not part of the plan, even when their age
// would have matched them.
//
// --- WHY --- A SWAP ID ON ITS OWN IS ENOUGH: naming a swap id says
// "this swap", and it does not need `--swaps` on top of it.
pub async fn db_admin_prune_swap_plan_fun(
    pool: &SqlitePool,
    filters: &DB_ADMIN_PRUNE_FILTERS,
) -> Result<Vec<String>, sqlx::Error> {
    if !filters.include_swaps && filters.swap_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut binds: Vec<SqlxBind> = Vec::new();
    let where_clause = db_admin_prune_swap_where_fun(filters, &mut binds);

    let mut query = String::from("SELECT swap_id FROM swap_queue");
    query.push_str(&where_clause);
    query.push_str(" ORDER BY created_at ASC, swap_id ASC");

    db_admin_prune_run_id_query_fun(pool, &query, "swap_id", &binds).await
}

// ========================================
// --- COUNT THE REST --------------------
// how many transaction and account rows go with the queues.
//
// --- WHY --- THE COUNT IS TAKEN BEFORE THE DELETE: the numbers are
// what the plan prints, and after the delete there is nothing left to
// count.
pub async fn db_admin_prune_count_children_fun(
    pool: &SqlitePool,
    queue_ids: &[String],
) -> Result<(u64, u64), sqlx::Error> {
    if queue_ids.is_empty() {
        return Ok((0, 0));
    }

    let names: Vec<String> = (0..queue_ids.len())
        .map(|offset| format!("?{}", offset + 1))
        .collect();
    let list = names.join(", ");

    let mut transactions_statement = sqlx::query_as::<_, (i64,)>(AssertSqlSafe(format!(
        "SELECT COUNT(*) FROM tx_queue_transactions WHERE queue_id IN ({list})"
    )));
    for id in queue_ids {
        transactions_statement = transactions_statement.bind(id.clone());
    }
    let transactions: (i64,) = transactions_statement.fetch_one(pool).await?;

    let mut accounts_statement = sqlx::query_as::<_, (i64,)>(AssertSqlSafe(format!(
        "SELECT COUNT(*) FROM tx_queue_accounts WHERE queue_id IN ({list})"
    )));
    for id in queue_ids {
        accounts_statement = accounts_statement.bind(id.clone());
    }
    let accounts: (i64,) = accounts_statement.fetch_one(pool).await?;

    Ok((transactions.0 as u64, accounts.0 as u64))
}

// ========================================
// --- COUNT THE SWAP ACCOUNTS -----------
// how many swap account rows go with the swaps.
pub async fn db_admin_prune_count_swap_accounts_fun(
    pool: &SqlitePool,
    swap_ids: &[String],
) -> Result<u64, sqlx::Error> {
    if swap_ids.is_empty() {
        return Ok(0);
    }
    let names: Vec<String> = (0..swap_ids.len())
        .map(|offset| format!("?{}", offset + 1))
        .collect();
    let list = names.join(", ");

    let mut statement = sqlx::query_as::<_, (i64,)>(AssertSqlSafe(format!(
        "SELECT COUNT(*) FROM swap_queue_accounts WHERE swap_id IN ({list})"
    )));
    for id in swap_ids {
        statement = statement.bind(id.clone());
    }
    let count: (i64,) = statement.fetch_one(pool).await?;
    Ok(count.0 as u64)
}

// ========================================
// --- THE DELETE -----------------------
// take the queues and the swaps off the db.
//
// --- WHY --- THE DELETE IS IN A TRANSACTION: either the whole plan goes
// or none of it does. a delete that stopped half way would leave the
// db in a state neither the plan nor the app expected.
//
// --- WHY --- THE SWAPS GO FIRST: their account rows point at them, and
// the cascade takes them. the tx queues go after, and their
// transactions and account rows go with them.
pub async fn db_admin_prune_delete_fun(
    pool: &SqlitePool,
    plan: &DB_ADMIN_PRUNE_PLAN,
) -> Result<(u64, u64), sqlx::Error> {
    let mut transaction = pool.begin().await?;

    let mut swaps_removed = 0u64;
    for swap_id in &plan.swap_ids {
        let result = sqlx::query("DELETE FROM swap_queue WHERE swap_id = ?1")
            .bind(swap_id)
            .execute(&mut *transaction)
            .await?;
        swaps_removed += result.rows_affected();
    }

    let mut queues_removed = 0u64;
    for queue_id in &plan.queue_ids {
        let result = sqlx::query("DELETE FROM tx_queue WHERE queue_id = ?1")
            .bind(queue_id)
            .execute(&mut *transaction)
            .await?;
        queues_removed += result.rows_affected();
    }

    transaction.commit().await?;

    Ok((queues_removed, swaps_removed))
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
