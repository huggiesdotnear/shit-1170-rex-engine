// ========================================
// DB_ADMIN_EXPORT_FUN
// the rows of a vault as json, and the json back into a vault.
//
// --- WHY --- THE EXPORT IS JSON and not sql: json is the shape the
// middleman and the frontend already speak, a person can read it, and
// it can be edited by hand before it goes back in. sql would only be
// readable by sqlite.
//
// --- WHY --- THE TABLES COME OUT IN DEPENDENCY ORDER and go back in the
// same order: the api keys, the queues, the transactions, the account
// rows, the swaps, the swap account rows. a row that points at another
// one has to land after it, or the foreign key refuses it.
//
// --- WHY --- THE API KEYS ARE OPTIONAL on the way out: the keys are
// credentials and a person exporting a queue to look at does not want
// their keys in a file. `--with-api-keys` says "take those too".
//
// --- WHY --- THE API KEYS ARE NEVER IMPORTED BACK: a key in a file is
// a key that has been around in the clear, and the import makes a NEW
// key for the account. the imported file can bring the accounts back,
// but it cannot bring the credentials back — the user signs in again.
use serde_json::{Value, json};
use sqlx::{Row, SqlitePool};
// ========================================
// --- THE VERSION -----------------------
// the version of the export format.
//
// --- WHY --- IT IS A NUMBER AND NOT A STRING: a number is compared, and
// a number that goes up means the format changed. a string would have to
// be compared as text, and "10" sorts before "9".
//
// --- WHY --- IT IS HERE AND NOT IN THE BIN: the export writes it and the
// import reads it, and they are the same number. a version that each
// side wrote for itself would be two versions that agree by accident.
pub const DB_ADMIN_EXPORT_VERSION: i64 = 1;
// ========================================
// --- EXPORT ----------------------------
// every row of every table, as a json object.
//
// --- WHY --- THE ROWS ARE READ AS TYPED TUPLES and not as json blobs:
// the export has to write the nulls as nulls and the numbers as
// numbers, and a row read into a tuple keeps its types.
pub async fn db_admin_export_fun(
    pool: &SqlitePool,
    with_api_keys: bool,
) -> Result<Value, sqlx::Error> {
    let mut export = json!({
        "version": DB_ADMIN_EXPORT_VERSION,
        "exported_at": crate::backend::logic::db::bin::db_admin_now_fun::now_ms_fun(),
    });

    // --- the api keys -------------------
    if with_api_keys {
        let rows = sqlx::query(
            "SELECT account_id, api_key, api_key_hash, created_at, updated_at
               FROM api_keys
              ORDER BY created_at ASC, account_id ASC",
        )
        .fetch_all(pool)
        .await?;

        let items: Vec<Value> = rows
            .iter()
            .map(|row| {
                json!({
                    "account_id": row.get::<String, _>("account_id"),
                    "api_key": row.get::<String, _>("api_key"),
                    "api_key_hash": row.get::<String, _>("api_key_hash"),
                    "created_at": row.get::<i64, _>("created_at"),
                    "updated_at": row.get::<i64, _>("updated_at"),
                })
            })
            .collect();
        export["api_keys"] = Value::Array(items);
    }

    // --- the queues ---------------------
    let rows = sqlx::query(
        "SELECT queue_id, what, \"when\", why, when_at, created_at, updated_at
           FROM tx_queue
          ORDER BY created_at ASC, queue_id ASC",
    )
    .fetch_all(pool)
    .await?;
    let items: Vec<Value> = rows
        .iter()
        .map(|row| {
            json!({
                "queue_id": row.get::<String, _>("queue_id"),
                "what": row.get::<Option<String>, _>("what"),
                "when": row.get::<Option<String>, _>("when"),
                "why": row.get::<Option<String>, _>("why"),
                "when_at": row.get::<Option<i64>, _>("when_at"),
                "created_at": row.get::<i64, _>("created_at"),
                "updated_at": row.get::<i64, _>("updated_at"),
            })
        })
        .collect();
    export["tx_queue"] = Value::Array(items);

    // --- the transactions ---------------
    let rows = sqlx::query(
        "SELECT tx_id, queue_id, position, contract_id, method, args, gas, deposit,
                created_at, updated_at
           FROM tx_queue_transactions
          ORDER BY created_at ASC, tx_id ASC",
    )
    .fetch_all(pool)
    .await?;
    let items: Vec<Value> = rows
        .iter()
        .map(|row| {
            json!({
                "tx_id": row.get::<String, _>("tx_id"),
                "queue_id": row.get::<String, _>("queue_id"),
                "position": row.get::<i64, _>("position"),
                "contract_id": row.get::<String, _>("contract_id"),
                "method": row.get::<String, _>("method"),
                "args": row.get::<String, _>("args"),
                "gas": row.get::<Option<String>, _>("gas"),
                "deposit": row.get::<Option<String>, _>("deposit"),
                "created_at": row.get::<i64, _>("created_at"),
                "updated_at": row.get::<i64, _>("updated_at"),
            })
        })
        .collect();
    export["tx_queue_transactions"] = Value::Array(items);

    // --- the account rows ---------------
    let rows = sqlx::query(
        "SELECT queue_id, tx_id, account_id, status, tx_hash, created_at, updated_at
           FROM tx_queue_accounts
          ORDER BY created_at ASC, tx_id ASC, account_id ASC",
    )
    .fetch_all(pool)
    .await?;
    let items: Vec<Value> = rows
        .iter()
        .map(|row| {
            json!({
                "queue_id": row.get::<String, _>("queue_id"),
                "tx_id": row.get::<String, _>("tx_id"),
                "account_id": row.get::<String, _>("account_id"),
                "status": row.get::<String, _>("status"),
                "tx_hash": row.get::<Option<String>, _>("tx_hash"),
                "created_at": row.get::<i64, _>("created_at"),
                "updated_at": row.get::<i64, _>("updated_at"),
            })
        })
        .collect();
    export["tx_queue_accounts"] = Value::Array(items);

    // --- the swaps ----------------------
    let rows = sqlx::query(
        "SELECT swap_id, what, \"when\", why, when_at, token_in, amount_in, token_out,
                amount_out, dex_id, created_at, updated_at
           FROM swap_queue
          ORDER BY created_at ASC, swap_id ASC",
    )
    .fetch_all(pool)
    .await?;
    let items: Vec<Value> = rows
        .iter()
        .map(|row| {
            json!({
                "swap_id": row.get::<String, _>("swap_id"),
                "what": row.get::<Option<String>, _>("what"),
                "when": row.get::<Option<String>, _>("when"),
                "why": row.get::<Option<String>, _>("why"),
                "when_at": row.get::<Option<i64>, _>("when_at"),
                "token_in": row.get::<String, _>("token_in"),
                "amount_in": row.get::<String, _>("amount_in"),
                "token_out": row.get::<String, _>("token_out"),
                "amount_out": row.get::<Option<String>, _>("amount_out"),
                "dex_id": row.get::<Option<String>, _>("dex_id"),
                "created_at": row.get::<i64, _>("created_at"),
                "updated_at": row.get::<i64, _>("updated_at"),
            })
        })
        .collect();
    export["swap_queue"] = Value::Array(items);

    // --- the swap account rows ----------
    let rows = sqlx::query(
        "SELECT swap_id, account_id, status, tx_hash, amount_out, created_at, updated_at
           FROM swap_queue_accounts
          ORDER BY created_at ASC, swap_id ASC, account_id ASC",
    )
    .fetch_all(pool)
    .await?;
    let items: Vec<Value> = rows
        .iter()
        .map(|row| {
            json!({
                "swap_id": row.get::<String, _>("swap_id"),
                "account_id": row.get::<String, _>("account_id"),
                "status": row.get::<String, _>("status"),
                "tx_hash": row.get::<Option<String>, _>("tx_hash"),
                "amount_out": row.get::<Option<String>, _>("amount_out"),
                "created_at": row.get::<i64, _>("created_at"),
                "updated_at": row.get::<i64, _>("updated_at"),
            })
        })
        .collect();
    export["swap_queue_accounts"] = Value::Array(items);

    Ok(export)
}

// ========================================
// --- IMPORT ----------------------------
// put the rows of an export into a vault.
//
// --- WHY --- IT IS AN UPSERT AND NOT AN INSERT: an import into a vault
// that already has some of the rows should bring the file's rows in and
// leave the rest alone. an insert would fail on the first row that is
// already there.
//
// --- WHY --- THE ORDER IS THE DEPENDENCY ORDER: the queues before the
// transactions, the transactions before the account rows. a row that
// points at another one has to land after it.
//
// --- WHY --- THE API KEYS ARE SKIPPED: a key in a file is a key that has
// been in the clear. the account comes back and the user signs in again
// for a new key.
//
// --- WHY --- THE COUNT IS RETURNED: the person who ran the bin wants to
// know how much landed, and "it worked" is not a number.
pub async fn db_admin_import_fun(
    pool: &SqlitePool,
    export: &Value,
) -> Result<(u64, u64, u64, u64, u64), sqlx::Error> {
    let mut transaction = pool.begin().await?;

    let mut queues = 0u64;
    let mut transactions = 0u64;
    let mut accounts = 0u64;
    let mut swaps = 0u64;
    let mut swap_accounts = 0u64;

    // --- the queues ---------------------
    if let Some(items) = export.get("tx_queue").and_then(Value::as_array) {
        for item in items {
            let result = sqlx::query(
                "INSERT INTO tx_queue
                 (queue_id, what, \"when\", why, when_at, created_at, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                ON CONFLICT (queue_id) DO UPDATE SET
                  what = excluded.what,
                  \"when\" = excluded.\"when\",
                  why = excluded.why,
                  when_at = excluded.when_at,
                  updated_at = excluded.updated_at",
            )
            .bind(db_admin_json_str_fun(item, "queue_id"))
            .bind(db_admin_json_opt_str_fun(item, "what"))
            .bind(db_admin_json_opt_str_fun(item, "when"))
            .bind(db_admin_json_opt_str_fun(item, "why"))
            .bind(db_admin_json_opt_int_fun(item, "when_at"))
            .bind(db_admin_json_int_fun(item, "created_at"))
            .bind(db_admin_json_int_fun(item, "updated_at"))
            .execute(&mut *transaction)
            .await?;
            queues += result.rows_affected();
        }
    }

    // --- the transactions ---------------
    if let Some(items) = export
        .get("tx_queue_transactions")
        .and_then(Value::as_array)
    {
        for item in items {
            let result = sqlx::query(
                "INSERT INTO tx_queue_transactions
                 (tx_id, queue_id, position, contract_id, method, args, gas, deposit,
                  created_at, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                ON CONFLICT (tx_id) DO UPDATE SET
                  position = excluded.position,
                  contract_id = excluded.contract_id,
                  method = excluded.method,
                  args = excluded.args,
                  gas = excluded.gas,
                  deposit = excluded.deposit,
                  updated_at = excluded.updated_at",
            )
            .bind(db_admin_json_str_fun(item, "tx_id"))
            .bind(db_admin_json_str_fun(item, "queue_id"))
            .bind(db_admin_json_int_fun(item, "position"))
            .bind(db_admin_json_str_fun(item, "contract_id"))
            .bind(db_admin_json_str_fun(item, "method"))
            .bind(db_admin_json_str_fun(item, "args"))
            .bind(db_admin_json_opt_str_fun(item, "gas"))
            .bind(db_admin_json_opt_str_fun(item, "deposit"))
            .bind(db_admin_json_int_fun(item, "created_at"))
            .bind(db_admin_json_int_fun(item, "updated_at"))
            .execute(&mut *transaction)
            .await?;
            transactions += result.rows_affected();
        }
    }

    // --- the account rows ---------------
    if let Some(items) = export.get("tx_queue_accounts").and_then(Value::as_array) {
        for item in items {
            let result = sqlx::query(
                "INSERT INTO tx_queue_accounts
                 (queue_id, tx_id, account_id, status, tx_hash, created_at, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                ON CONFLICT (tx_id, account_id) DO UPDATE SET
                  queue_id = excluded.queue_id,
                  status = excluded.status,
                  tx_hash = excluded.tx_hash,
                  updated_at = excluded.updated_at",
            )
            .bind(db_admin_json_str_fun(item, "queue_id"))
            .bind(db_admin_json_str_fun(item, "tx_id"))
            .bind(db_admin_json_str_fun(item, "account_id"))
            .bind(db_admin_json_str_fun(item, "status"))
            .bind(db_admin_json_opt_str_fun(item, "tx_hash"))
            .bind(db_admin_json_int_fun(item, "created_at"))
            .bind(db_admin_json_int_fun(item, "updated_at"))
            .execute(&mut *transaction)
            .await?;
            accounts += result.rows_affected();
        }
    }

    // --- the swaps ----------------------
    if let Some(items) = export.get("swap_queue").and_then(Value::as_array) {
        for item in items {
            let result = sqlx::query(
                "INSERT INTO swap_queue
                 (swap_id, what, \"when\", why, when_at, token_in, amount_in, token_out,
                  amount_out, dex_id, created_at, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                ON CONFLICT (swap_id) DO UPDATE SET
                  what = excluded.what,
                  \"when\" = excluded.\"when\",
                  why = excluded.why,
                  when_at = excluded.when_at,
                  token_in = excluded.token_in,
                  amount_in = excluded.amount_in,
                  token_out = excluded.token_out,
                  amount_out = excluded.amount_out,
                  dex_id = excluded.dex_id,
                  updated_at = excluded.updated_at",
            )
            .bind(db_admin_json_str_fun(item, "swap_id"))
            .bind(db_admin_json_opt_str_fun(item, "what"))
            .bind(db_admin_json_opt_str_fun(item, "when"))
            .bind(db_admin_json_opt_str_fun(item, "why"))
            .bind(db_admin_json_opt_int_fun(item, "when_at"))
            .bind(db_admin_json_str_fun(item, "token_in"))
            .bind(db_admin_json_str_fun(item, "amount_in"))
            .bind(db_admin_json_str_fun(item, "token_out"))
            .bind(db_admin_json_opt_str_fun(item, "amount_out"))
            .bind(db_admin_json_opt_str_fun(item, "dex_id"))
            .bind(db_admin_json_int_fun(item, "created_at"))
            .bind(db_admin_json_int_fun(item, "updated_at"))
            .execute(&mut *transaction)
            .await?;
            swaps += result.rows_affected();
        }
    }

    // --- the swap account rows ----------
    if let Some(items) = export.get("swap_queue_accounts").and_then(Value::as_array) {
        for item in items {
            let result = sqlx::query(
                "INSERT INTO swap_queue_accounts
                 (swap_id, account_id, status, tx_hash, amount_out, created_at, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                ON CONFLICT (swap_id, account_id) DO UPDATE SET
                  status = excluded.status,
                  tx_hash = excluded.tx_hash,
                  amount_out = excluded.amount_out,
                  updated_at = excluded.updated_at",
            )
            .bind(db_admin_json_str_fun(item, "swap_id"))
            .bind(db_admin_json_str_fun(item, "account_id"))
            .bind(db_admin_json_str_fun(item, "status"))
            .bind(db_admin_json_opt_str_fun(item, "tx_hash"))
            .bind(db_admin_json_opt_str_fun(item, "amount_out"))
            .bind(db_admin_json_int_fun(item, "created_at"))
            .bind(db_admin_json_int_fun(item, "updated_at"))
            .execute(&mut *transaction)
            .await?;
            swap_accounts += result.rows_affected();
        }
    }

    transaction.commit().await?;

    Ok((queues, transactions, accounts, swaps, swap_accounts))
}

// ========================================
// --- CHECK THE VERSION -----------------
// is this export a version this bin can read?
//
// --- WHY --- THE VERSION IS CHECKED AND NOT ASSUMED: the import is an
// UPSERT that writes the rows it is given, and a file from a newer
// version may have columns this version does not know about. importing
// it would drop them silently, and the person doing it would not know
// that the rows came back wrong.
//
// --- WHY --- A MISSING VERSION IS AN ERROR and not a default: a file
// without a version was not made by this format, and a file that was
// hand-edited may have had its version taken out. treating it as the
// current version would guess.
//
// --- WHY --- A NEWER VERSION IS AN ERROR and an OLDER ONE IS NOT: the
// format only ever goes up, so a newer file is from a future this bin
// cannot understand. an older file is a file this bin grew out of, and
// the rows it names are the rows this bin understands.
//
// --- WHY --- THE ERROR NAMES BOTH VERSIONS: a person who has a file
// from a newer build needs to know which build, and "wrong version" is
// not enough to go on.
pub fn db_admin_export_version_ok_fun(export: &Value) -> Result<(), String> {
    let version = match export.get("version") {
        Some(value) => value,
        None => {
            return Err(String::from(
                "the file has no version in it, so it is not a vault export.",
            ));
        }
    };

    let version = match version.as_i64() {
        Some(version) => version,
        None => {
            return Err(format!(
                "the version in the file is '{version}', and it should be a number."
            ));
        }
    };

    if version > DB_ADMIN_EXPORT_VERSION {
        return Err(format!(
            "the file is version {version} and this bin reads up to version {DB_ADMIN_EXPORT_VERSION}. \
             it was made by a newer build."
        ));
    }

    Ok(())
}
// ========================================
// --- JSON HELPERS ----------------------
// a string field of a json object.
//
// --- WHY --- THE HELPERS TAKE A DEFAULT and not a failure: an export
// that was hand-edited and lost a field should still import, with the
// missing field filled in rather than the whole import refused.
fn db_admin_json_str_fun(item: &Value, key: &str) -> String {
    item.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

// a string field that may be null.
fn db_admin_json_opt_str_fun(item: &Value, key: &str) -> Option<String> {
    item.get(key)
        .and_then(Value::as_str)
        .map(|value| value.to_string())
}

// an integer field.
fn db_admin_json_int_fun(item: &Value, key: &str) -> i64 {
    item.get(key).and_then(Value::as_i64).unwrap_or(0)
}

// an integer field that may be null.
fn db_admin_json_opt_int_fun(item: &Value, key: &str) -> Option<i64> {
    item.get(key).and_then(Value::as_i64)
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
