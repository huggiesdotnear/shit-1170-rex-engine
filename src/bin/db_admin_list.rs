// ========================================
// DB_ADMIN_LIST
// the bin that shows what is in a vault.
//
// usage:
//   db-admin-list --db <path> [--limit <n>] [--swaps] [--queues]
//
// --- WHY --- IT PRINTS AND DOES NOT WRITE: it is the thing to run
// BEFORE a prune, and a bin that reads must not change what it reads.
//
// --- WHY --- THE LIMITS ARE PER TABLE and not one number for the whole
// list: a vault with a thousand queues and three swaps is interesting
// for different reasons, and one limit for both would hide one of them.
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_args_fun::db_admin_flag_fun;
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_args_fun::db_admin_switch_fun;
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_list_fun::{
    db_admin_list_counts_fun, db_admin_list_queues_fun, db_admin_list_swaps_fun, db_admin_when_fun,
};
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_prelude_fun::db_admin_prelude_fun;
// ========================================
// the usage text.
//
// --- WHY --- IT IS A CONSTANT and not a function: the prelude prints it
// and the prelude has no other use for it, and a constant says plainly
// that it is data.
const HELP_TEXT: &str = r#"db-admin-list - show what is in a vault

usage:
  db-admin-list --db <path> [--limit <n>] [--queues] [--swaps]

  --db <path>    the db file to look at
  --limit <n>    how many of each to show (default 50)
  --queues       only the queues
  --swaps        only the swaps

this one only reads."#;
// ========================================
#[tokio::main]
async fn main() {
    // --- THE FLAGS AND THE DB -----------
    // --- WHY --- THE PRELUDE DOES THE FLAGS AND THE DB: every bin opens
    // the db the same way and refuses an unknown flag the same way, and
    // a bin that keeps its own copy of that can fall behind the flags it
    // takes. a typo in a flag that DELETES ROWS runs a prune with no age
    // filter, so the check is not a style thing.
    let prelude = db_admin_prelude_fun(
        "db-admin-list",
        &["db", "limit", "swaps", "queues"],
        HELP_TEXT,
    )
    .await;
    let args = &prelude.args;
    let db_path = &prelude.db_path;
    let pool = &prelude.pool;

    // --- THE LIMIT ---------------------
    let limit = match db_admin_flag_fun(args, "limit") {
        Some(value) => match value.parse::<i64>() {
            Ok(limit) => Some(limit),
            Err(_) => {
                eprintln!("--limit wants a number, not '{value}'");
                std::process::exit(2);
            }
        },
        None => Some(50),
    };

    // --- THE COUNTS --------------------
    println!("db: {}", db_path.display());
    let counts = match db_admin_list_counts_fun(pool).await {
        Ok(counts) => counts,
        Err(err) => {
            eprintln!("could not count the rows: {err}");
            std::process::exit(1);
        }
    };
    for (table, count) in &counts {
        println!("  {table}: {count}");
    }

    // --- THE QUEUES --------------------
    let only_swaps = db_admin_switch_fun(args, "swaps") && !db_admin_switch_fun(args, "queues");
    let only_queues = db_admin_switch_fun(args, "queues") && !db_admin_switch_fun(args, "swaps");

    if !only_swaps {
        let queues = match db_admin_list_queues_fun(pool, limit).await {
            Ok(queues) => queues,
            Err(err) => {
                eprintln!("could not read the queues: {err}");
                std::process::exit(1);
            }
        };
        println!();
        println!("queues:");
        for queue in &queues {
            println!(
                "  {}  {}  txs: {}  accounts: {}  sent: {}  at {}",
                queue.queue_id,
                queue.what.clone().unwrap_or_else(|| String::from("-")),
                queue.transactions,
                queue.accounts,
                queue.sent,
                db_admin_when_fun(queue.created_at)
            );
        }
    }

    // --- THE SWAPS ---------------------
    if !only_queues {
        let swaps = match db_admin_list_swaps_fun(pool, limit).await {
            Ok(swaps) => swaps,
            Err(err) => {
                eprintln!("could not read the swaps: {err}");
                std::process::exit(1);
            }
        };
        println!();
        println!("swaps:");
        for swap in &swaps {
            println!(
                "  {}  {}  accounts: {}  sent: {}  at {}",
                swap.swap_id,
                swap.what.clone().unwrap_or_else(|| String::from("-")),
                swap.accounts,
                swap.sent,
                db_admin_when_fun(swap.created_at)
            );
        }
    }
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
