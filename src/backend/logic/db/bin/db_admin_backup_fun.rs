// ========================================
// DB_ADMIN_BACKUP_FUN
// a copy of the db file that is safe to take while the app is running.
//
// --- WHY --- `VACUUM INTO` AND NOT A FILE COPY: a sqlite file in WAL
// mode is three files — the db, the `-wal` and the `-shm` — and a plain
// copy of the db file alone is a db with the recent writes left out.
// copying it while a write is in flight can copy a half-written page.
// `VACUUM INTO` asks sqlite itself to write a fresh, whole, consistent
// copy, and it takes the lock it needs while it does it.
//
// --- WHY --- THE COPY GOES NEXT TO THE ORIGINAL and not to a temp dir:
// the copy is of the vault, and the vault's folder is where the person
// who made it will look for it.
//
// --- WHY --- THE NAME CARRIES THE TIME: two backups in the same second
// would collide, and the second one would overwrite the first.
use sqlx::AssertSqlSafe;
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
// ========================================
// --- BACKUP ---------------------------
// write a consistent copy of the db next to it.
//
// --- WHY --- THE DESTINATION IS BUILT HERE and not taken from the
// caller: the name has the time in it, and a caller that built the name
// would have to build the same name the same way.
//
// --- WHY --- THE COPY IS CHECKED FOR EXISTENCE FIRST: `VACUUM INTO`
// fails if the file is there, and it fails with a message that does not
// say which file. the check says it plainly.
//
// --- WHY --- THE ORIGINAL IS NOT TOUCHED: the backup is a read of the
// vault and a write of a new file. the vault is the thing that matters.
pub async fn db_admin_backup_fun(
    pool: &SqlitePool,
    db_path: &Path,
) -> Result<PathBuf, sqlx::Error> {
    let now = crate::backend::logic::db::bin::db_admin_now_fun::now_ms_fun();

    let stem = db_path
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .unwrap_or_else(|| String::from("vault"));

    let mut destination = db_path.to_path_buf();
    destination.set_file_name(format!("{stem}.backup-{now}.db"));

    if destination.exists() {
        return Err(sqlx::Error::Protocol(format!(
            "the backup file is already there: {}",
            destination.display()
        )));
    }

    // the destination is bound as a string because sqlite takes it as
    // an expression, and a path with a quote in it would break the
    // statement if it went in as text.
    let destination_string = destination.to_string_lossy().to_string();
    sqlx::query(AssertSqlSafe(format!(
        "VACUUM INTO '{}'",
        destination_string.replace('\'', "''")
    )))
    .execute(pool)
    .await?;

    Ok(destination)
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
