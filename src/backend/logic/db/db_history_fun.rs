// ========================================
// DB_HISTORY_FUN
// what counts as history: a row that has been sent.
//
// --- WHY --- THIS IS IN THE DB AND NOT IN AN ENDPOINT: the rule "a row
// that carries a `tx_hash` is history and it stays" is the rule the whole
// app turns on. the feed hides sent rows with it, the list counts sent
// rows with it, and the prune refuses to take history with it. when that
// rule is written three times it is three rules, and the day one of them
// changes the app and the bin disagree about what history is.
//
// --- WHY --- THE ACCOUNT ROW IS WHERE THE STATUS LIVES: the status is
// on the account row and nowhere else. a queue is never "done" — a queue
// is a batch of transactions, and each transaction has accounts on it,
// and each account row carries the hash once it has been sent. so "was
// this ever sent" is a question about the account rows.
//
// --- WHY --- "ANY ACCOUNT" AND NOT "THIS ACCOUNT": the rule here is the
// unscoped one — a queue is history if ANY account on it sent it. the
// account-scoped question ("has ALICE sent it") is a different question
// and lives with the feed, which is about one reader's view.
//
// --- WHY --- THE PREDICATE IS A STRING AND NOT A BUILT STATEMENT: the
// callers are two different worlds. the feed builds its statement with
// `SqliteListBuilder` and needs a numbered placeholder; the bin builds a
// plain where clause with its own bind list. what they share is the SHAPE
// of the question, not the way the sql gets put together, so the shape
// is one function and each caller puts it into its own builder.
// ========================================
// --- THE ACCOUNT ROWS -------------------
// the account rows that belong to a queue, and the id that links them.
#[derive(Debug, Clone, Copy)]
pub enum DB_HISTORY_TABLE {
    // the account rows of a queued batch.
    TxQueue,
    // the account rows of a queued swap.
    SwapQueue,
}

impl DB_HISTORY_TABLE {
    // the account rows table.
    pub fn accounts_table_fun(&self) -> &'static str {
        match self {
            DB_HISTORY_TABLE::TxQueue => "tx_queue_accounts",
            DB_HISTORY_TABLE::SwapQueue => "swap_queue_accounts",
        }
    }

    // the column on the account rows that points at its parent.
    //
    // --- WHY --- THE COLUMN IS NOT THE SAME ON BOTH: a queue's account
    // rows hang off the transaction and carry the queue's id as well, and
    // a swap's account rows hang off the swap. so the two point at their
    // parent with different column names.
    pub fn parent_column_fun(&self) -> &'static str {
        match self {
            DB_HISTORY_TABLE::TxQueue => "queue_id",
            DB_HISTORY_TABLE::SwapQueue => "swap_id",
        }
    }

    // the id column on the parent table.
    pub fn parent_id_column_fun(&self) -> &'static str {
        match self {
            DB_HISTORY_TABLE::TxQueue => "queue_id",
            DB_HISTORY_TABLE::SwapQueue => "swap_id",
        }
    }

    // the table the parent lives in.
    pub fn parent_table_fun(&self) -> &'static str {
        match self {
            DB_HISTORY_TABLE::TxQueue => "tx_queue",
            DB_HISTORY_TABLE::SwapQueue => "swap_queue",
        }
    }
}
// ========================================
// --- THE PREDICATE ----------------------
// the sql that matches the rows that have NOT been sent.
//
// --- WHY --- IT IS `NOT EXISTS` AND NOT A JOIN: the question is whether
// there is NO account row with a hash, and that is a question about the
// absence of a row. a join would find the rows that have one, which is
// the other question.
//
// --- WHY --- THE PARENT IS NAMED EXPLICITLY and not left to the outer
// query: the clause names the parent table so it can be dropped into any
// statement without the caller having to know what the caller aliased it
// to.
pub fn db_history_not_sent_sql_fun(table: DB_HISTORY_TABLE) -> String {
    let accounts = table.accounts_table_fun();
    let parent_column = table.parent_column_fun();
    let parent_table = table.parent_table_fun();
    format!(
        "NOT EXISTS (SELECT 1 FROM {accounts} h
                  WHERE h.{parent_column} = {parent_table}.{parent_column}
                    AND h.tx_hash IS NOT NULL)"
    )
}

// ========================================
// --- THE ACCOUNT PREDICATE, SPLIT ------
// the sql that matches the rows that THIS account has not sent.
//
// --- WHY --- IT IS A SEPARATE THING and not the unscoped predicate with
// an account added: the unscoped question is "has anybody sent it" and
// this one is "has THIS account sent it". they are different questions
// and a queue can be sent by alice and not by bob, so one of them cannot
// be made out of the other without giving up the difference.
// the sql that matches the rows that THIS account has not sent, with the
// account's bind in the middle.
//
// --- WHY --- IT IS SPLIT IN TWO and not one string with a hole in it: the
// caller has a builder that OWNS the placeholder number — it writes `?N`
// itself when a value is bound — so a hole in the sql would mean the
// caller had to know the number before it built the clause. two halves —
// the sql before the bind and the sql after it — let the builder put the
// bind in the middle and keep the numbering to itself.
//
// --- WHY --- THE PLACEHOLDER IS A MARKER and not a `?N`: the builder
// writes the placeholder when the value is bound, so the sql written here
// must NOT write one too — two placeholders for one value is a syntax
// error. the marker is replaced by the builder's own placeholder.
pub fn db_history_account_not_sent_before_fun(
    table: DB_HISTORY_TABLE,
    account_placeholder: &str,
) -> String {
    let accounts = table.accounts_table_fun();
    let parent_column = table.parent_column_fun();
    let parent_table = table.parent_table_fun();
    format!(
        "NOT EXISTS (SELECT 1 FROM {accounts} h
                  WHERE h.{parent_column} = {parent_table}.{parent_column}
                    AND h.account_id = {account_placeholder}"
    )
}

// the sql that closes the account-scoped predicate.
pub fn db_history_account_not_sent_after_fun() -> String {
    String::from(" AND h.tx_hash IS NOT NULL)")
}

// ========================================
// --- THE ACCOUNT MARKER ----------------
// the marker that stands for the account's placeholder.
//
// --- WHY --- IT IS A MARKER AND NOT A PLACEHOLDER: the builder writes
// the placeholder itself when it binds the value, so the sql this module
// hands over must not carry one. the marker is a piece of text that is
// not sql, and the builder replaces it with the placeholder it is about
// to write.
pub const DB_HISTORY_ACCOUNT_MARKER: &str = "DB_HISTORY_ACCOUNT_MARKER";
// ========================================
// --- THE COUNTED SENT ROWS --------------
// the sql that counts the account rows of a parent that carry a hash.
//
// --- WHY --- IT IS ITS OWN FUNCTION: the list counts the sent rows, and
// a count is a different shape of question from a match. writing it here
// keeps the "sent" test in one place — the same test, three questions.
pub fn db_history_sent_count_sql_fun(table: DB_HISTORY_TABLE, parent_alias: &str) -> String {
    let accounts = table.accounts_table_fun();
    let parent_column = table.parent_column_fun();
    format!(
        "(SELECT COUNT(*) FROM {accounts} h
          WHERE h.{parent_column} = {parent_alias}.{parent_column}
            AND h.tx_hash IS NOT NULL)"
    )
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
