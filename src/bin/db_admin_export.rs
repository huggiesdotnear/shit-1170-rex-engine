// ========================================
// DB_ADMIN_EXPORT
// the bin that writes a vault out as json.
//
// usage:
//   db-admin-export --db <path> [--out <file>] [--with-api-keys]
//
// --- WHY --- THE API KEYS ARE NOT IN THE FILE UNLESS THEY ARE ASKED FOR:
// the keys are credentials. a file of queues is a file you can hand
// around; a file of keys is a file of everyone's credentials.
//
// --- WHY --- IT PRINTS TO STDOUT WHEN THERE IS NO `--out`: a person can
// pipe it into a file or into `jq`, and the shell is the tool for
// moving data around.
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_args_fun::{
    db_admin_args_fun, db_admin_flag_fun, db_admin_switch_fun,
};
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_db_fun::{
    db_admin_db_has_tables_fun, db_admin_db_open_fun, db_admin_db_path_fun,
};
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_export_fun::db_admin_export_fun;
// ========================================
#[tokio::main]
async fn main() {
    let words: Vec<String> = std::env::args().skip(1).collect();
    let args = db_admin_args_fun(&words);

    if db_admin_switch_fun(&args, "help") {
        print_help_fun();
        return;
    }

    let known: Vec<&str> = vec!["db", "out", "with-api-keys"];
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
        eprintln!("run `db-admin-export --help` to see the flags");
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

    // --- THE EXPORT --------------------
    let with_api_keys = db_admin_switch_fun(&args, "with-api-keys");
    let export = match db_admin_export_fun(&pool, with_api_keys).await {
        Ok(export) => export,
        Err(err) => {
            eprintln!("could not read the vault: {err}");
            std::process::exit(1);
        }
    };

    let text = match serde_json::to_string_pretty(&export) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("could not write the export: {err}");
            std::process::exit(1);
        }
    };

    // --- WHERE IT GOES -----------------
    match db_admin_flag_fun(&args, "out") {
        Some(path) => match std::fs::write(path, &text) {
            Ok(()) => {
                println!("export written to {path}");
            }
            Err(err) => {
                eprintln!("could not write {path}: {err}");
                std::process::exit(1);
            }
        },
        None => {
            println!("{text}");
        }
    }
}

// the usage text.
fn print_help_fun() {
    println!(
        r#"db-admin-export - write a vault out as json

usage:
  db-admin-export --db <path> [--out <file>] [--with-api-keys]

  --db <path>          the db file to read
  --out <file>         where the json goes (default: stdout)
  --with-api-keys      include the api keys. they are credentials, and
                       they are left out unless this is given"#
    );
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
