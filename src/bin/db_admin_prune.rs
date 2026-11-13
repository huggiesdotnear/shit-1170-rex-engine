// ========================================
// DB_ADMIN_PRUNE
// the bin that takes old queues and swaps off a vault.
//
// usage:
//   db-admin-prune --db <path> [filters] [--confirm]
//
// the filters:
//   --older-than <days>   only the rows queued more than <days> ago
//   --before <epoch-ms>   only the rows queued before that moment
//   --after <epoch-ms>    only the rows queued after that moment
//   --never-sent          only the rows no account ever sent
//   --queue-ids <a,b,c>   only these queue ids
//   --swap-ids <a,b,c>    only these swap ids
//   --account <account>   only the rows this account is on
//   --include-sent        also take the rows that were sent
//   --swaps               also take the queued swaps
//   --confirm             actually delete
//   --db <path>           the db file to work on
//
// --- WHY --- THE DELETE IS A DRY RUN UNTIL `--confirm` IS GIVEN: this
// bin deletes rows and there is no undo. the plan is printed first and
// the plan is the whole point, so the bin shows what it would take and
// then waits.
//
// --- WHY --- `--confirm` IS A SWITCH AND NOT A VALUE: a value would
// let `--confirm no` be read as "yes", and there is nothing to be
// careful with here.
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_args_fun::{
    DB_ADMIN_ARGS, db_admin_args_fun, db_admin_flag_fun, db_admin_id_list_fun, db_admin_switch_fun,
};
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_db_fun::{
    db_admin_db_has_tables_fun, db_admin_db_open_fun, db_admin_db_path_fun,
};
use shit_1170_rex_engine::backend::logic::db::bin::db_admin_prune_fun::{
    DB_ADMIN_PRUNE_FILTERS, db_admin_prune_count_children_fun,
    db_admin_prune_count_swap_accounts_fun, db_admin_prune_delete_fun,
    db_admin_prune_queue_plan_fun, db_admin_prune_swap_plan_fun,
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

    // --- THE FLAGS WE KNOW --------------
    let known: Vec<&str> = vec![
        "db",
        "older-than",
        "before",
        "after",
        "never-sent",
        "queue-ids",
        "swap-ids",
        "account",
        "include-sent",
        "swaps",
        "confirm",
    ];
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
        eprintln!("run `db-admin-prune --help` to see the flags");
        std::process::exit(2);
    }

    // --- THE FILTERS --------------------
    let filters = match db_admin_filters_fun(&args) {
        Ok(filters) => filters,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(2);
        }
    };

    if filters.older_than_days.is_none()
        && filters.before_ms.is_none()
        && filters.after_ms.is_none()
        && !filters.never_sent
        && filters.queue_ids.is_empty()
        && filters.swap_ids.is_empty()
        && filters.account_id.is_none()
    {
        eprintln!("no filter was given, so this would take the whole queue.");
        eprintln!(
            "give one of --older-than, --before, --after, --never-sent, --queue-ids, --swap-ids or --account."
        );
        std::process::exit(2);
    }

    // --- THE DB -------------------------
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

    // --- THE PLAN -----------------------
    let queue_ids = match db_admin_prune_queue_plan_fun(&pool, &filters).await {
        Ok(ids) => ids,
        Err(err) => {
            eprintln!("could not read the queues: {err}");
            std::process::exit(1);
        }
    };
    let swap_ids = match db_admin_prune_swap_plan_fun(&pool, &filters).await {
        Ok(ids) => ids,
        Err(err) => {
            eprintln!("could not read the swaps: {err}");
            std::process::exit(1);
        }
    };

    let (transaction_rows, account_rows) =
        match db_admin_prune_count_children_fun(&pool, &queue_ids).await {
            Ok(counts) => counts,
            Err(err) => {
                eprintln!("could not count the queue rows: {err}");
                std::process::exit(1);
            }
        };
    let swap_account_rows = match db_admin_prune_count_swap_accounts_fun(&pool, &swap_ids).await {
        Ok(count) => count,
        Err(err) => {
            eprintln!("could not count the swap rows: {err}");
            std::process::exit(1);
        }
    };

    println!("db: {}", db_path.display());
    println!(
        "queues: {}, transactions: {}, account rows: {}",
        queue_ids.len(),
        transaction_rows,
        account_rows
    );
    println!(
        "swaps: {}, swap account rows: {}",
        swap_ids.len(),
        swap_account_rows
    );
    for id in &queue_ids {
        println!("  queue {id}");
    }
    for id in &swap_ids {
        println!("  swap {id}");
    }

    if queue_ids.is_empty() && swap_ids.is_empty() {
        println!("nothing matched.");
        return;
    }

    // --- THE DRY RUN -------------------
    if !db_admin_switch_fun(&args, "confirm") {
        println!();
        println!("this was a dry run. nothing was deleted.");
        println!("add --confirm to actually delete these rows.");
        return;
    }

    // --- THE DELETE --------------------
    let plan =
        shit_1170_rex_engine::backend::logic::db::bin::db_admin_prune_fun::DB_ADMIN_PRUNE_PLAN {
            queue_ids,
            swap_ids,
            transaction_rows,
            account_rows,
            swap_account_rows,
        };
    match db_admin_prune_delete_fun(&pool, &plan).await {
        Ok((queues, swaps)) => {
            println!();
            println!("deleted {} queues and {} swaps.", queues, swaps);
        }
        Err(err) => {
            eprintln!("the delete failed and nothing was changed: {err}");
            std::process::exit(1);
        }
    }
}
// ========================================
// --- HELPERS ----------------------------
// read the filters out of the flags.
//
// --- WHY --- A FLAG THAT IS NOT A NUMBER IS AN ERROR and not a zero: a
// `--older-than abc` that meant "no age filter" would take everything
// the other filters match, which is the opposite of what was asked.
fn db_admin_filters_fun(args: &DB_ADMIN_ARGS) -> Result<DB_ADMIN_PRUNE_FILTERS, String> {
    let mut filters = DB_ADMIN_PRUNE_FILTERS {
        never_sent: db_admin_switch_fun(args, "never-sent"),
        include_sent: db_admin_switch_fun(args, "include-sent"),
        include_swaps: db_admin_switch_fun(args, "swaps"),
        queue_ids: db_admin_id_list_fun(args, "queue-ids"),
        swap_ids: db_admin_id_list_fun(args, "swap-ids"),
        account_id: db_admin_flag_fun(args, "account").map(|value| value.to_string()),
        ..DB_ADMIN_PRUNE_FILTERS::default()
    };

    if let Some(value) = db_admin_flag_fun(args, "older-than") {
        let days: u32 = value
            .parse()
            .map_err(|_| format!("--older-than wants a number of days, not '{value}'"))?;
        filters.older_than_days = Some(days);
    }
    if let Some(value) = db_admin_flag_fun(args, "before") {
        let ms: i64 = value
            .parse()
            .map_err(|_| format!("--before wants epoch milliseconds, not '{value}'"))?;
        filters.before_ms = Some(ms);
    }
    if let Some(value) = db_admin_flag_fun(args, "after") {
        let ms: i64 = value
            .parse()
            .map_err(|_| format!("--after wants epoch milliseconds, not '{value}'"))?;
        filters.after_ms = Some(ms);
    }

    Ok(filters)
}

// the usage text.
fn print_help_fun() {
    println!(
        r#"db-admin-prune - take old queues and swaps off a vault

usage:
  db-admin-prune --db <path> [filters] [--confirm]

the filters:
  --older-than <days>   only the rows queued more than <days> ago
  --before <epoch-ms>   only the rows queued before that moment
  --after <epoch-ms>    only the rows queued after that moment
  --never-sent          only the rows no account ever sent
  --queue-ids <a,b,c>   only these queue ids
  --swap-ids <a,b,c>    only these swap ids
  --account <account>   only the rows this account is on
  --include-sent        also take the rows that were sent
  --swaps               also take the queued swaps
  --confirm             actually delete
  --db <path>           the db file to work on

without --confirm this prints the plan and deletes nothing."#
    );
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
