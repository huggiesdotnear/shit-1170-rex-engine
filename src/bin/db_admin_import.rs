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
//
// --- WHY --- THE VERSION OF THE FILE IS CHECKED BEFORE ANYTHING ELSE:
// the import writes the rows it is given, and a file from a newer
// version may have columns this version does not know about. importing
// it would drop them silently.
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_args_fun::db_admin_flag_fun;
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_args_fun::db_admin_switch_fun;
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_export_fun::db_admin_export_version_ok_fun;
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_export_fun::db_admin_import_fun;
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_prelude_fun::db_admin_prelude_fun;
// ========================================
// the usage text.
//
// --- WHY --- IT IS A CONSTANT and not a function: the prelude prints it
// and the prelude has no other use for it, and a constant says plainly
// that it is data.
const HELP_TEXT: &str = r#"db-admin-import - put an export back into a vault

usage:
  db-admin-import --db <path> --from <file> [--confirm]

  --db <path>    the db file to write into
  --from <file>  the export to bring in
  --confirm      actually write

the api keys in the file are not imported. sign in again for a new key."#;
// ========================================
#[tokio::main]
async fn main() {
    // --- THE FLAGS AND THE DB -----------
    // --- WHY --- THE PRELUDE DOES THE FLAGS AND THE DB: every bin opens
    // the db the same way and refuses an unknown flag the same way, and
    // a bin that keeps its own copy of that can fall behind the flags it
    // takes.
    let prelude =
        db_admin_prelude_fun("db-admin-import", &["db", "from", "confirm"], HELP_TEXT).await;
    let args = &prelude.args;
    let db_path = &prelude.db_path;
    let pool = &prelude.pool;

    // --- THE FILE ----------------------
    let from = match db_admin_flag_fun(args, "from") {
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

    // --- THE VERSION -------------------
    // --- WHY --- THE VERSION IS CHECKED BEFORE THE COUNTS ARE PRINTED:
    // the counts are a plan, and a plan for a file this bin cannot read
    // is a plan for the wrong thing.
    if let Err(message) = db_admin_export_version_ok_fun(&export) {
        eprintln!("{from}: {message}");
        std::process::exit(1);
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
    if !db_admin_switch_fun(args, "confirm") {
        println!();
        println!("this was a dry run. nothing was written.");
        println!("add --confirm to actually bring the rows in.");
        return;
    }

    // --- THE IMPORT --------------------
    match db_admin_import_fun(pool, &export).await {
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
// ========================================
// copyright 2026 by sleet.near
// ========================================
