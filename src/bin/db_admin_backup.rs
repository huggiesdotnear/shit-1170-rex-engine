// ========================================
// DB_ADMIN_BACKUP
// the bin that takes a safe copy of a vault.
//
// usage:
//   db-admin-backup --db <path> [--out <path>]
//
// --- WHY --- IT IS `VACUUM INTO` AND NOT A FILE COPY: a sqlite file in
// WAL mode is three files — the db, the `-wal` and the `-shm` — and a
// copy of the db file alone is a db with the recent writes left out.
// `VACUUM INTO` asks sqlite to write a fresh, whole, consistent copy,
// and it is safe to run while the app is writing.
//
// --- WHY --- THE COPY GOES NEXT TO THE ORIGINAL by default: the copy is
// of the vault, and the vault's folder is where the person who made it
// will look for it.
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_args_fun::db_admin_flag_fun;
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_backup_fun::db_admin_backup_fun;
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_prelude_fun::db_admin_prelude_fun;
use std::path::PathBuf;
// ========================================
// the usage text.
//
// --- WHY --- IT IS A CONSTANT and not a function: the prelude prints it
// and the prelude has no other use for it, and a constant says plainly
// that it is data.
const HELP_TEXT: &str = r#"db-admin-backup - take a safe copy of a vault

usage:
  db-admin-backup --db <path> [--out <path>]

  --db <path>    the db file to copy
  --out <path>   where the copy goes (default: next to the db)

the copy is taken with sqlite's own backup, so it is safe to run while
the app is writing."#;
// ========================================
#[tokio::main]
async fn main() {
    // --- THE FLAGS AND THE DB -----------
    // --- WHY --- THE PRELUDE DOES THE FLAGS AND THE DB: every bin opens
    // the db the same way and refuses an unknown flag the same way, and
    // a bin that keeps its own copy of that can fall behind the flags it
    // takes.
    let prelude = db_admin_prelude_fun("db-admin-backup", &["db", "out"], HELP_TEXT).await;
    let args = &prelude.args;
    let db_path = &prelude.db_path;
    let pool = &prelude.pool;

    // --- THE COPY ----------------------
    let destination = match db_admin_flag_fun(args, "out") {
        Some(value) => PathBuf::from(value),
        None => match db_admin_backup_fun(pool, db_path).await {
            Ok(path) => path,
            Err(err) => {
                eprintln!("could not make the backup: {err}");
                std::process::exit(1);
            }
        },
    };

    // --- A NAMED COPY ------------------
    // --- WHY --- THE NAMED COPY IS MADE BY THE SAME `VACUUM INTO` and
    // not by a file copy: the same reason the default copy is made that
    // way. a copy of the db file alone is a db with the recent writes
    // left out.
    if db_admin_flag_fun(args, "out").is_some() {
        if destination.exists() {
            eprintln!("the file is already there: {}", destination.display());
            std::process::exit(1);
        }
        let destination_string = destination.to_string_lossy().to_string();
        if let Err(err) = sqlx::query(sqlx::AssertSqlSafe(format!(
            "VACUUM INTO '{}'",
            destination_string.replace('\'', "''")
        )))
        .execute(pool)
        .await
        {
            eprintln!("could not make the backup: {err}");
            std::process::exit(1);
        }
    }

    println!("backup written to {}", destination.display());
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
