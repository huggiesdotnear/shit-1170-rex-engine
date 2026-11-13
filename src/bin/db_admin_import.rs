// ========================================
// DB_ADMIN_IMPORT
// the bin that puts an export back into a vault.
//
// usage:
//   db-admin-import --db <path> --from <file> [--confirm]
//
// --- WHY --- IT IS A DRY RUN UNTIL `--confirm` IS GIVEN: it writes rows
// into a vault and there is no undo.
//
// --- WHY --- THE API KEYS IN THE FILE ARE NOT IMPORTED: a key that has
// been in a file has been in the clear, so the accounts come back and
// the user signs in again for a new key.
//
// --- WHY --- IT IS AN UPSERT and not an insert: an import into a vault
// that already holds some of the rows brings the file's rows in and
// leaves the rest alone.
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_args_fun::{
    db_admin_args_fun, db_admin_flag_fun, db_admin_switch_fun,
};
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_db_fun::{
    db_admin_db_has_tables_fun, db_admin_db_open_fun, db_admin_db_path_fun,
};
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_export_fun::db_admin_import_fun;
// ========================================
#[tokio::main]
async fn main() {
    let words: Vec<String> = std::env::args().skip(1).collect();
    let args = db_admin_args_fun(&words);

    if db_admin_switch_fun(&args, "help") {
        print_help_fun();
        return;
    }

    let known: Vec<&str> = vec!["db", "from", "confirm"];
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
        eprintln!("run `db-admin-import --help` to see the flags");
        std::process::exit(2);
    }

    // --- THE FILE ----------------------
    let from = match db_admin_flag_fun(&args, "from") {
        Some(path) => path.to_string(),
        None => {
            eprintln!("--from <file> is needed: that is the export to bring in.");
            std::process::exit(2);
        }
    };
    let text = match std::fs::read_to_string(&from) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("could not read {from}: {err}");
            std::process::exit(1);
        }
    };
    let export: serde_json::Value = match serde_json::from_str(&text) {
        Ok(export) => export,
        Err(err) => {
            eprintln!("{from} is not a vault export: {err}");
            std::process::exit(1);
        }
    };

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

    // --- WHAT IS IN THE FILE -----------
    let count_of = |key: &str| -> usize {
        export
            .get(key)
            .and_then(serde_json::Value::as_array)
            .map(|items| items.len())
            .unwrap_or(0)
    };

    println!("db: {}", db_path.display());
    println!("from: {from}");
    println!(
        "queues: {}, transactions: {}, account rows: {}, swaps: {}, swap account rows: {}",
        count_of("tx_queue"),
        count_of("tx_queue_transactions"),
        count_of("tx_queue_accounts"),
        count_of("swap_queue"),
        count_of("swap_queue_accounts")
    );
    if export.get("api_keys").is_some() {
        println!("the api keys in the file are NOT imported. sign in again for a new key.");
    }

    // --- THE DRY RUN -------------------
    if !db_admin_switch_fun(&args, "confirm") {
        println!();
        println!("this was a dry run. nothing was written.");
        println!("add --confirm to actually bring the rows in.");
        return;
    }

    // --- THE IMPORT --------------------
    match db_admin_import_fun(&pool, &export).await {
        Ok((queues, transactions, accounts, swaps, swap_accounts)) => {
            println!();
            println!(
                "imported {} queues, {} transactions, {} account rows, {} swaps, {} swap account rows.",
                queues, transactions, accounts, swaps, swap_accounts
            );
        }
        Err(err) => {
            eprintln!("the import failed and nothing was changed: {err}");
            std::process::exit(1);
        }
    }
}

// the usage text.
fn print_help_fun() {
    println!(
        r#"db-admin-import - put an export back into a vault

usage:
  db-admin-import --db <path> --from <file> [--confirm]

  --db <path>    the db file to write into
  --from <file>  the export to bring in
  --confirm      actually write

the api keys in the file are not imported. sign in again for a new key."#
    );
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
