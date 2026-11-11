// ========================================
// the keys page's start: it begins the page's load.
//
// the page's import file pulls this in, so the load starts before the
// components mount.
//
// --- WHY THE LOAD FOLLOWS THE AUTH: the keys belong to the signed-in
// account, and the wallet may not have answered yet when the page's
// script runs. so the store watches the auth — the load runs when there
// is an account, and it runs again whenever the account changes — a
// sign-in, a sign-out or a switch to another account.
//
// --- WHY THE STORE OWNS THE LOAD AND THIS FILE ONLY ASKS FOR IT: the
// fetching itself lives in lib/ts/keys/app_page_keys_load.ts, so the
// page's request list is still knowable by reading one file.
import { SHIT_1170_AUTH } from "../shared/auth_status";
import { SHIT_1170_APP_PAGE_KEYS_STORE } from "./app_page_keys_store";
import { shit_1170_loading_bar_start_fun } from "../shared/loading_bar_start";
// ========================================
// ========================================
// --- START -----------------------------
// the loading bar's patch goes in before the page's first request.
shit_1170_loading_bar_start_fun();
// the auth is initialised so the wallet's answer is on its way.
SHIT_1170_AUTH.init();
// the keys belong to the signed-in account, so the load follows the
// auth by itself.
SHIT_1170_APP_PAGE_KEYS_STORE.watch();
void SHIT_1170_APP_PAGE_KEYS_STORE.load();
// ========================================
// copyright 2026 by sleet.near
// ========================================
