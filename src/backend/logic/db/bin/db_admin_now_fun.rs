// ========================================
// DB_ADMIN_NOW_FUN
// the current time, in ms since the epoch.
//
// --- WHY --- IT IS HERE AND NOT TAKEN FROM THE APP'S STORE: the app's
// `now_ms_fun` lives inside an endpoint module, and a bin that reaches
// into an endpoint module for a clock is reaching into the wrong place.
// the bins are not endpoints.
//
// --- WHY --- IT IS THE SAME NUMBER THE APP WRITES: the `created_at` the
// app writes and the "now" the bins compare against have to be in the
// same clock, or an age filter would be off by however much the two
// clocks disagree.
use std::time::{SystemTime, UNIX_EPOCH};
// ========================================
// --- NOW -------------------------------
// the current time in ms since the epoch.
pub fn now_ms_fun() -> i64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(since) => since.as_millis() as i64,
        Err(_) => 0,
    }
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
