// ========================================
// BIN
// the logic behind the db admin bins: the bins that do db work on a
// vault from the outside.
//
// --- WHY --- THIS IS IN THE LIB AND NOT IN THE BIN FILES: the bins are
// four entry points that all need the same things — the db path, the
// open, the prune, the export. putting that here means the bins are
// four short files that parse their flags and call into here, and the
// logic can be tested without running a binary.
//
// --- WHY --- THE BINS ARE THE DEFAULT WAY TO DO DB WORK: the app has no
// migration system and no admin routes for wiping the queue. anything
// that changes the db from outside the app goes through a bin, so the
// rules of the db live in one place and the app never has to know that
// the bins exist.
pub mod db_admin_args_fun;
pub mod db_admin_backup_fun;
pub mod db_admin_db_fun;
pub mod db_admin_export_fun;
pub mod db_admin_list_fun;
pub mod db_admin_now_fun;
pub mod db_admin_prune_fun;
#[cfg(test)]
pub mod db_admin_test;
// ========================================
// copyright 2026 by sleet.near
// ========================================
