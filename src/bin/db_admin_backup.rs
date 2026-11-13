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
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_args_fun::{
    db_admin_args_fun, db_admin_flag_fun, db_admin_switch_fun,
};
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_backup_fun::db_admin_backup_fun;
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_db_fun::{
    db_admin_db_has_tables_fun, db_admin_db_open_fun, db_admin_db_path_fun,
};
use std::path::PathBuf;
// ========================================
#[tokio::main]
async fn main() {
    let words: Vec<String> = std::env::args().skip(1).collect();
    let args = db_admin_args_fun(&words);

    if db_admin_switch_fun(&args, "help") {
        print_help_fun();
        return;
    }

    let known: Vec<&str> = vec!["db", "out"];
    let unknown: Vec<String> = args
        .flags
        .keys()
        .filter(|name| !known.contains(&name.as_str()))
        .cloned()
        .collect();
    let unknown_switches: Vec<String> = args
        .switches
        .iter()
        .filter(|name| !known.contains(&name.as_str()))
        .cloned()
        .collect();
    if !unknown.is_empty() || !unknown_switches.is_empty() {
        eprintln!(
            "unknown flag: --{}",
            unknown
                .first()
                .cloned()
                .unwrap_or_else(|| unknown_switches.first().cloned().unwrap_or_default())
        );
        eprintln!("run `db-admin-backup --help` to see the flags");
        std::process::exit(2);
    }

    // --- THE DB ------------------------
    let db_path = db_admin_db_path_fun(args.db_path.clone());
    let pool = match db_admin_db_open_fun(&db_path).await {
        Ok(pool) => pool,
        Err(err) => {
            eprintln!("could not open the db at {}: {err}", db_path.display());
            std::process::exit(1);
        }
    };
    match db_admin_db_has_tables_fun(&pool).await {
        Ok(true) => {}
        Ok(false) => {
            eprintln!("the db at {} is not a shit-1170 vault.", db_path.display());
            std::process::exit(1);
        }
        Err(err) => {
            eprintln!("could not read the db at {}: {err}", db_path.display());
            std::process::exit(1);
        }
    }

    // --- THE COPY ----------------------
    let destination = match db_admin_flag_fun(&args, "out") {
        Some(value) => PathBuf::from(value),
        None => match db_admin_backup_fun(&pool, &db_path).await {
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
    if db_admin_flag_fun(&args, "out").is_some() {
        if destination.exists() {
            eprintln!("the file is already there: {}", destination.display());
            std::process::exit(1);
        }
        let destination_string = destination.to_string_lossy().to_string();
        if let Err(err) = sqlx::query(sqlx::AssertSqlSafe(format!(
            "VACUUM INTO '{}'",
            destination_string.replace('\'', "''")
        )))
        .execute(&pool)
        .await
        {
            eprintln!("could not make the backup: {err}");
            std::process::exit(1);
        }
    }

    println!("backup written to {}", destination.display());
}

// the usage text.
fn print_help_fun() {
    println!(
        r#"db-admin-backup - take a safe copy of a vault

usage:
  db-admin-backup --db <path> [--out <path>]

  --db <path>    the db file to copy
  --out <path>   where the copy goes (default: next to the db)

the copy is taken with sqlite's own backup, so it is safe to run while
the app is writing."#
    );
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
