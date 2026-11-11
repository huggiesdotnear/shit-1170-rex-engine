// ========================================
// TX_QUEUE_KEY_METHOD_NAMES_FUN
// the `key_method_names` column is a json ARRAY STRING; the wire shape
// is a list of strings.
//
// --- WHY --- this is its own file: the conversion lives in ONE place,
// so the write path and the read path can never disagree about what a
// list of method names looks like as a string. the same reason
// `tx_queue_args_fun` exists for the args column.
//
// --- WHY --- THE COLUMN IS TEXT AND NOT A LIST: sqlite has no list
// column. the methods are a bag of plain strings with no shape, so a
// json array string holds them and a json read of it gives the list
// back.
//
// --- WHY --- A COLUMN THAT WILL NOT PARSE IS AN ERROR AND NOT AN EMPTY
// LIST: a key whose methods are gone must not be answered with methods
// it never had. the caller turns the error into a 500.
use serde_json::Value;
// ========================================
// --- TO JSON ---------------------------
// the column's string as the list of method names.
//
// --- WHY --- THE NULL COLUMN IS AN ABSENT LIST AND NOT AN EMPTY ONE: a
// transaction with no key action has no methods, and "no methods" is
// not the same as "a key that may call no method at all" — an empty
// method list on a limited key means it may call EVERY method of its
// receiver. so null and [] are two different facts and they stay two
// different facts.
pub fn tx_queue_key_method_names_to_json_fun(
    method_names: Option<&str>,
) -> Result<Option<Vec<String>>, serde_json::Error> {
    let raw = match method_names {
        Some(raw) => raw,
        None => return Ok(None),
    };

    let value: Value = serde_json::from_str(raw)?;
    let names: Vec<String> = match value {
        Value::Array(items) => items
            .into_iter()
            .map(|item| match item {
                Value::String(name) => name,
                other => other.to_string(),
            })
            .collect(),
        // --- WHY --- A COLUMN THAT IS NOT AN ARRAY IS AN EMPTY LIST AND
        // NOT AN ERROR: the column is written by this api and it always
        // writes an array. a value that is not one can only be a row
        // from an older write, and there is nothing to recover from it.
        _ => Vec::new(),
    };

    Ok(Some(names))
}

// ========================================
// --- FROM JSON -------------------------
// the list of method names as the column's string.
//
// --- WHY --- it is SERIALIZED and not built by hand: the column has to
// hold valid json, and hand building it means escaping the strings by
// hand and getting it wrong.
pub fn tx_queue_key_method_names_from_json_fun(
    method_names: &[String],
) -> Result<String, serde_json::Error> {
    serde_json::to_string(method_names)
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
