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
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_args_fun::{
    db_admin_args_fun, db_admin_flag_fun, db_admin_switch_fun,
};
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_db_fun::{
    db_admin_db_has_tables_fun, db_admin_db_open_fun, db_admin_db_path_fun,
};
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_list_fun::{
    db_admin_list_counts_fun, db_admin_list_queues_fun, db_admin_list_swaps_fun, db_admin_when_fun,
};
// ========================================
#[tokio::main]
async fn main() {
    let words: Vec<String> = std::env::args().skip(1).collect();
    let args = db_admin_args_fun(&words);

    if db_admin_switch_fun(&args, "help") {
        print_help_fun();
        return;
    }

    let known: Vec<&str> = vec!["db", "limit", "swaps", "queues"];
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
        eprintln!("run `db-admin-list --help` to see the flags");
        std::process::exit(2);
    }

    // --- THE LIMIT ---------------------
    let limit = match db_admin_flag_fun(&args, "limit") {
        Some(value) => match value.parse::<i64>() {
            Ok(limit) => Some(limit),
            Err(_) => {
                eprintln!("--limit wants a number, not '{value}'");
                std::process::exit(2);
            }
        },
        None => Some(50),
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

    // --- THE COUNTS --------------------
    println!("db: {}", db_path.display());
    let counts = match db_admin_list_counts_fun(&pool).await {
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
    let only_swaps = db_admin_switch_fun(&args, "swaps") && !db_admin_switch_fun(&args, "queues");
    let only_queues = db_admin_switch_fun(&args, "queues") && !db_admin_switch_fun(&args, "swaps");

    if !only_swaps {
        let queues = match db_admin_list_queues_fun(&pool, limit).await {
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
        let swaps = match db_admin_list_swaps_fun(&pool, limit).await {
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

// the usage text.
fn print_help_fun() {
    println!(
        r#"db-admin-list - show what is in a vault

usage:
  db-admin-list --db <path> [--limit <n>] [--queues] [--swaps]

  --db <path>    the db file to look at
  --limit <n>    how many of each to show (default 50)
  --queues       only the queues
  --swaps        only the swaps

this one only reads."#
    );
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
