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
// pipe it, and a script can read it without a file in the way.
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_args_fun::db_admin_flag_fun;
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_args_fun::db_admin_switch_fun;
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_export_fun::db_admin_export_fun;
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_prelude_fun::db_admin_prelude_fun;
// ========================================
// the usage text.
//
// --- WHY --- IT IS A CONSTANT and not a function: the prelude prints it
// and the prelude has no other use for it, and a constant says plainly
// that it is data.
const HELP_TEXT: &str = r#"db-admin-export - write a vault out as json

usage:
  db-admin-export --db <path> [--out <file>] [--with-api-keys]

  --db <path>          the db file to read
  --out <file>         where the json goes (default: stdout)
  --with-api-keys      include the api keys. they are credentials, and
                       they are left out unless this is given"#;
// ========================================
#[tokio::main]
async fn main() {
    // --- THE FLAGS AND THE DB -----------
    // --- WHY --- THE PRELUDE DOES THE FLAGS AND THE DB: every bin opens
    // the db the same way and refuses an unknown flag the same way, and
    // a bin that keeps its own copy of that can fall behind the flags it
    // takes.
    let prelude = db_admin_prelude_fun(
        "db-admin-export",
        &["db", "out", "with-api-keys"],
        HELP_TEXT,
    )
    .await;
    let args = &prelude.args;
    let pool = &prelude.pool;

    // --- THE EXPORT --------------------
    let with_api_keys = db_admin_switch_fun(args, "with-api-keys");
    let export = match db_admin_export_fun(pool, with_api_keys).await {
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
    match db_admin_flag_fun(args, "out") {
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
// ========================================
// copyright 2026 by sleet.near
// ========================================
