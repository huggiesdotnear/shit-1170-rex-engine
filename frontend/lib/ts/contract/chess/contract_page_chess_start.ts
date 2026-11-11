// ========================================
// the chess page's start: it begins the page's load.
//
// the page's import file pulls this in, so the load starts before the
// components mount, and without any component having to ask for it.
//
// --- WHY --- a script and not a component: a component that called the
// load would be doing two jobs — displaying and starting the page's
// work. the page's import file says when the page starts, so the page
// starts it.
import { SHIT_1170_CONTRACT_PAGE_CHESS_STORE } from "./contract_page_chess_store";
import { shit_1170_loading_bar_start_fun } from "../../shared/loading_bar_start";
// ========================================
// ========================================
// --- START -----------------------------
// the loading bar's patch goes in before the page's first request.
shit_1170_loading_bar_start_fun();
// begin the page's load.
void SHIT_1170_CONTRACT_PAGE_CHESS_STORE.load();
// ========================================
// copyright 2026 by sleet.near
