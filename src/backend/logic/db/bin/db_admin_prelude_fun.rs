// ========================================
// DB_ADMIN_PRELUDE_FUN
// the three things every db admin bin does before it does its own work:
// the flags, the unknown-flag check, and the db.
//
// --- WHY --- THIS IS HERE AND NOT COPIED INTO EACH BIN: the five bins
// all opened the db the same way, all refused an unknown flag the same
// way, and all said the same thing when the db was not a vault. that is
// five copies of one thing, and a copy is a rule that can drift. the
// drift is not a style problem — it is that one bin's "unknown flag" list
// can fall behind the flags it takes, and then a typo in a flag that
// DELETES ROWS runs a prune with no age filter.
//
// --- WHY --- THE HELPER EXITS THE PROCESS and does not return an error:
// a bin that cannot open the db has nothing to do, and the exit code is
// the whole of the contract with the shell that called it. every bin
// already exited on these, so this does not change what any bin does —
// it only says it once.
//
// --- WHY --- THE NAME OF THE BIN IS PASSED IN and not derived: the name
// is what the error messages say ("run `db-admin-prune --help`"), and
// the bin knows its own name. a helper that worked it out from argv[0]
// would be right today and wrong the moment a binary is renamed or
// copied.
use sqlx::SqlitePool;
use std::path::PathBuf;
// ========================================
// --- THE ARGS AND THE DB ----------------
// the words of the command line, the db path and the pool.
//
// --- WHY --- THE ARGS ARE RE-EXPORTED HERE: the prelude hands back the
// parsed args, and a bin that has to import the args type from another
// module to read a flag is two imports for one thing.
pub use crate::backend::logic::db::bin::db_admin_args_fun::DB_ADMIN_ARGS;

use crate::backend::logic::db::bin::db_admin_args_fun::{db_admin_args_fun, db_admin_switch_fun};
use crate::backend::logic::db::bin::db_admin_db_fun::{
    db_admin_db_has_tables_fun, db_admin_db_open_fun, db_admin_db_path_fun,
};
// ========================================
// --- THE PRELUDE -----------------------
// the flags, the db path and the open db.
pub struct DB_ADMIN_PRELUDE {
    // the flags the bin was called with.
    pub args: DB_ADMIN_ARGS,
    // the db file's path.
    pub db_path: PathBuf,
    // the open db.
    pub pool: SqlitePool,
}
// ========================================
// --- THE PRELUDE -----------------------
// read the command line, refuse an unknown flag, and open the db.
//
// --- WHY --- THE UNKNOWN FLAG CHECK COMES BEFORE THE DB IS OPENED: a
// typo in a flag is a mistake in the COMMAND, and it should be said so
// before anything is read. opening the db first would mean a mistyped
// flag could only be found once the vault was in hand.
//
// --- WHY --- EVERY UNKNOWN FLAG IS LISTED and not just the first: a
// person who typed one wrong flag has usually typed two, and fixing them
// one run at a time is a waste of a round trip to production.
//
// --- WHY --- THE HELP SWITCH IS CHECKED FIRST and before the unknown
// flag check: `--help` is not a flag the bin knows, and a bin that
// refused an unknown flag would refuse its own help.
//
// --- WHY --- THE HELP TEXT IS PASSED IN and not written here: the usage
// text is about what the bin does, and each bin knows that best.
pub async fn db_admin_prelude_fun(
    bin_name: &str,
    known: &[&str],
    help_text: &str,
) -> DB_ADMIN_PRELUDE {
    let words: Vec<String> = std::env::args().skip(1).collect();
    let args = db_admin_args_fun(&words);

    if db_admin_switch_fun(&args, "help") {
        println!("{help_text}");
        std::process::exit(0);
    }

    let unknown: Vec<String> = args
        .flags
        .keys()
        .chain(args.switches.iter())
        .filter(|name| !known.contains(&name.as_str()))
        .cloned()
        .collect();

    if !unknown.is_empty() {
        for name in &unknown {
            eprintln!("unknown flag: --{name}");
        }
        eprintln!("run `{bin_name} --help` to see the flags");
        std::process::exit(2);
    }

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

    DB_ADMIN_PRELUDE {
        args,
        db_path,
        pool,
    }
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
