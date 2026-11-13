// ========================================
// DB_ADMIN_DB_FUN
// how a bin opens the db file it was pointed at.
//
// --- WHY --- the bins open the db DIRECTLY and not through the app's
// `db_open_fun`: the app's opener runs the schema creates, which is
// right for the app and wrong for a bin. a bin that reads a vault must
// NOT create tables in it, and a bin that must not touch a db at all
// must fail on a db with no schema instead of quietly making one.
//
// --- WHY --- the path resolution is THE SAME ORDER as the app's: the
// `--db` flag wins, the env var wins over the default, and the default
// is the home folder path. so a bin with no flag on the same machine
// as the app reads the app's db.
//
// --- WHY --- the db is opened READ-WRITE for every bin: the read-only
// bins need a write-capable connection for the backup api, and the
// write bins obviously need one. the file's own mode is what protects
// it, not the connection.
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::path::{Path, PathBuf};
use std::time::Duration;
// ========================================
// --- CONFIG -----------------------------
// how long a statement waits for the write lock before giving up.
//
// --- WHY --- it is SHORTER than the app's: a bin is a one-shot command
// and a person is waiting for it. a busy db means the app is writing,
// and the bin should say so instead of hanging.
const DB_ADMIN_BUSY_TIMEOUT: Duration = Duration::from_secs(5);
// ========================================
// --- PATH -------------------------------
// the db file's path, from the flag when it was given, and from the app's
// own resolution otherwise.
//
// --- WHY --- THE APP'S FUNCTION IS CALLED AND NOT COPIED: the env var
// and the default path are read in one place in the app, and a second
// copy of that rule would be a second thing to keep in step.
pub fn db_admin_db_path_fun(db_path: Option<PathBuf>) -> PathBuf {
    match db_path {
        Some(path) => path,
        None => crate::backend::logic::db::vault_db_path_fun::vault_db_path_fun(),
    }
}

// ========================================
// --- OPEN -------------------------------
// open the db at the path, without touching its schema.
//
// --- WHY --- `create_if_missing` is OFF: a bin pointed at a db that is
// not there must say "there is no db at that path" and stop. creating
// an empty db would be a bin that quietly made a new file where the
// real one should have been.
//
// --- WHY --- the parent folder is NOT created either: same reason.
//
// --- WHY --- WAL is set like the app sets it: a db written by the app
// is in WAL mode, and a bin that opened it in a different mode would
// change the file's format behind the app's back.
//
// --- WHY --- `foreign_keys` is ON: the deletes rely on the cascade, and
// a bin that deleted the parent rows and left the children behind would
// leave a broken db.
pub async fn db_admin_db_open_fun(path: &Path) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(false)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .foreign_keys(true)
        .busy_timeout(DB_ADMIN_BUSY_TIMEOUT);

    SqlitePoolOptions::new()
        .max_connections(1)
        .acquire_timeout(DB_ADMIN_BUSY_TIMEOUT)
        .connect_with(options)
        .await
}

// ========================================
// --- THE TABLES ------------------------
// the tables a bin works on, in the order they hold each other.
pub const DB_ADMIN_TABLES: [&str; 6] = [
    "api_keys",
    "tx_queue",
    "tx_queue_transactions",
    "tx_queue_accounts",
    "swap_queue",
    "swap_queue_accounts",
];

// ========================================
// --- CHECK ------------------------------
// is the db there and does it have our tables?
//
// --- WHY --- it is CHECKED and not assumed: a bin pointed at a folder or
// at a random sqlite file would otherwise fail on the first query with
// a confusing "no such table", and the message would not say what was
// actually wrong.
pub async fn db_admin_db_has_tables_fun(pool: &SqlitePool) -> Result<bool, sqlx::Error> {
    for table in DB_ADMIN_TABLES {
        let found: (i64,) = sqlx::query_as(
            "SELECT EXISTS(
               SELECT 1 FROM sqlite_master
                WHERE type = 'table' AND name = ?1)",
        )
        .bind(table)
        .fetch_one(pool)
        .await?;
        if found.0 == 0 {
            return Ok(false);
        }
    }
    Ok(true)
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
