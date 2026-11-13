// ========================================
// DB_ADMIN_LIST_FUN
// what is in a vault: the queues, the swaps and the api keys, with the
// counts that say how big each one is.
//
// --- WHY --- IT IS A SEPARATE BIN and not a flag on the prune: the
// question "what is in here" is asked on its own, all the time, and it
// is the question to ask BEFORE a prune. a prune that prints its plan
// answers it, but only for the rows the filters matched.
//
// --- WHY --- THE COUNTS ARE READ IN ONE QUERY each and not one query
// per row: a vault with a thousand queues and a query per queue is a
// thousand queries, and the whole point of the list is to be quick
// enough to run while you think.
use sqlx::AssertSqlSafe;
use sqlx::{Row, SqlitePool};
// ========================================
// --- TYPES ------------------------------
// one queue as the list shows it.
#[derive(Debug, Clone)]
pub struct DB_ADMIN_LIST_QUEUE {
    // the batch's id.
    pub queue_id: String,
    // what the transactions do, in words.
    pub what: Option<String>,
    // when the batch was queued, in ms since the epoch.
    pub created_at: i64,
    // how many transactions it holds.
    pub transactions: u64,
    // how many accounts are on it.
    pub accounts: u64,
    // how many of those accounts have sent it.
    pub sent: u64,
}

// one swap as the list shows it.
#[derive(Debug, Clone)]
pub struct DB_ADMIN_LIST_SWAP {
    // the swap's id.
    pub swap_id: String,
    // what the swap does, in words.
    pub what: Option<String>,
    // when the swap was queued, in ms since the epoch.
    pub created_at: i64,
    // how many accounts are on it.
    pub accounts: u64,
    // how many of those accounts have sent it.
    pub sent: u64,
}
// ========================================
// --- LIST THE QUEUES -------------------
// every queue with its counts.
//
// --- WHY --- THE COUNTS COME FROM SUBQUERIES and not from a join: a
// join over the account rows would multiply the queue rows by the
// number of accounts, and the counts would have to be undone with a
// DISTINCT. a subquery counts the rows it is asked about and nothing
// else.
//
// --- WHY --- THE COUNTED ROWS ARE THE ACCOUNT ROWS of the queue: the
// account row is the one place the status lives, so "how many sent it"
// is a count over the rows that carry a hash.
pub async fn db_admin_list_queues_fun(
    pool: &SqlitePool,
    limit: Option<i64>,
) -> Result<Vec<DB_ADMIN_LIST_QUEUE>, sqlx::Error> {
    let mut query = String::from(
        "SELECT q.queue_id, q.what, q.created_at,
                (SELECT COUNT(*) FROM tx_queue_transactions t
                  WHERE t.queue_id = q.queue_id) AS transactions,
                (SELECT COUNT(*) FROM tx_queue_accounts a
                  WHERE a.queue_id = q.queue_id) AS accounts,
                (SELECT COUNT(*) FROM tx_queue_accounts a
                  WHERE a.queue_id = q.queue_id
                    AND a.tx_hash IS NOT NULL) AS sent
           FROM tx_queue q",
    );
    // --- WHY --- THE ORDER COMES BEFORE THE LIMIT: that is the order
    // sqlite wants them in. a limit before the order is a syntax error.
    query.push_str(" ORDER BY q.created_at DESC, q.queue_id DESC");
    if let Some(limit) = limit {
        query.push_str(&format!(" LIMIT {limit}"));
    }

    let rows = sqlx::query(AssertSqlSafe(query.clone()))
        .fetch_all(pool)
        .await?;

    Ok(rows
        .iter()
        .map(|row| DB_ADMIN_LIST_QUEUE {
            queue_id: row.get("queue_id"),
            what: row.get("what"),
            created_at: row.get("created_at"),
            transactions: row.get::<i64, _>("transactions") as u64,
            accounts: row.get::<i64, _>("accounts") as u64,
            sent: row.get::<i64, _>("sent") as u64,
        })
        .collect())
}

// ========================================
// --- LIST THE SWAPS --------------------
// every swap with its counts.
pub async fn db_admin_list_swaps_fun(
    pool: &SqlitePool,
    limit: Option<i64>,
) -> Result<Vec<DB_ADMIN_LIST_SWAP>, sqlx::Error> {
    let mut query = String::from(
        "SELECT s.swap_id, s.what, s.created_at,
                (SELECT COUNT(*) FROM swap_queue_accounts a
                  WHERE a.swap_id = s.swap_id) AS accounts,
                (SELECT COUNT(*) FROM swap_queue_accounts a
                  WHERE a.swap_id = s.swap_id
                    AND a.tx_hash IS NOT NULL) AS sent
           FROM swap_queue s",
    );
    // --- WHY --- THE ORDER COMES BEFORE THE LIMIT: that is the order
    // sqlite wants them in. a limit before the order is a syntax error.
    query.push_str(" ORDER BY s.created_at DESC, s.swap_id DESC");
    if let Some(limit) = limit {
        query.push_str(&format!(" LIMIT {limit}"));
    }

    let rows = sqlx::query(AssertSqlSafe(query.clone()))
        .fetch_all(pool)
        .await?;

    Ok(rows
        .iter()
        .map(|row| DB_ADMIN_LIST_SWAP {
            swap_id: row.get("swap_id"),
            what: row.get("what"),
            created_at: row.get("created_at"),
            accounts: row.get::<i64, _>("accounts") as u64,
            sent: row.get::<i64, _>("sent") as u64,
        })
        .collect())
}

// ========================================
// --- COUNT EVERYTHING ------------------
// how many rows each table holds.
//
// --- WHY --- THE COUNTS ARE READ AT ONCE and not one at a time: the
// numbers are the summary of the vault, and a summary that takes six
// round trips is not a summary.
pub async fn db_admin_list_counts_fun(
    pool: &SqlitePool,
) -> Result<Vec<(String, u64)>, sqlx::Error> {
    let tables = crate::backend::logic::db::bin::db_admin_db_fun::DB_ADMIN_TABLES;
    let mut counts = Vec::new();

    for table in tables {
        let row: (i64,) =
            sqlx::query_as::<_, (i64,)>(AssertSqlSafe(format!("SELECT COUNT(*) FROM {table}")))
                .fetch_one(pool)
                .await?;
        counts.push((table.to_string(), row.0 as u64));
    }

    Ok(counts)
}

// ========================================
// --- WHEN -------------------------------
// a timestamp in ms, as a person reads it.
//
// --- WHY --- THE TIME IS PRINTED AS A UTC STRING and not as a number:
// the list is read by a person, and a number of milliseconds since the
// epoch is not a time. the number is still there in the export.
pub fn db_admin_when_fun(ms: i64) -> String {
    if ms < 0 {
        return String::from("unknown");
    }
    let total_seconds = ms / 1000;
    let days = total_seconds / 86_400;
    let rem = total_seconds % 86_400;
    format!("{}d {:02}:{:02}", days, rem / 3_600, (rem % 3_600) / 60)
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
