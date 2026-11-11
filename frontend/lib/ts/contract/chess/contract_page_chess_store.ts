// ========================================
// the chess page's shared state: the one load, held here, read by every
// component on the page.
//
// the store has the same shape as the other contract pages' stores: plain
// module state, a subscribe/unsubscribe pair, and a notify that a throwing
// subscriber cannot break.
// ========================================
import { SHIT_1170_CONTRACT_PAGE_CHESS } from "./contract_page_chess";
import type {
  SHIT_1170_CHESS_PAGE_DATA,
  SHIT_1170_CHESS_GAME,
} from "./contract_page_chess";
// ========================================
// ========================================
// --- STATE ------------------------------
let data: SHIT_1170_CHESS_PAGE_DATA | null = null;
let loading = false;
// the plain array so subscribe/unsubscribe ordering is stable.
const listeners: Array<() => void> = [];
// ========================================
// ========================================
// --- HELPERS ----------------------------
// broadcast a change to every subscriber; one throwing cannot break the
// flow that notified.
function notify_fun(): void {
  for (const listener of [...listeners]) {
    try {
      listener();
    } catch (err) {
      console.error("[contract_page_chess_store] listener failed", err);
    }
  }
}
// ========================================
// register a re-render callback; returns an unsubscribe so components
// can clean up on unmount instead of leaking.
function subscribe_fun(listener: () => void): () => void {
  listeners.push(listener);
  return () => {
    const index = listeners.indexOf(listener);
    if (index !== -1) listeners.splice(index, 1);
  };
}
// ========================================
// ========================================
// --- LOAD -------------------------------
// load the first page of games.
//
// the load runs once. a second call while a load is running is ignored, so
// the page cannot fetch twice because two components asked for it.
async function load_fun(): Promise<void> {
  if (loading) return;
  loading = true;
  notify_fun();
  try {
    data = await SHIT_1170_CONTRACT_PAGE_CHESS.load(null);
  } catch (err) {
    data = {
      games: [],
      total_games: 0,
      active_games: 0,
      finished_games: 0,
      cancelled_games: 0,
      total_moves: 0,
      has_more: false,
      next_cursor: null,
      errors: [err instanceof Error ? err.message : String(err)],
    };
  } finally {
    loading = false;
    notify_fun();
  }
}
// ========================================
// load the NEXT page of games and append it to the rows on screen.
async function load_more_fun(): Promise<void> {
  if (loading || data === null) return;
  if (!data.has_more) return;
  loading = true;
  notify_fun();
  try {
    const next_cursor = data.next_cursor;
    const result = await SHIT_1170_CONTRACT_PAGE_CHESS.load_more(next_cursor);
    if (result === null || data === null) return;
    data = {
      ...data,
      games: [...data.games, ...result.games],
      has_more: result.has_more,
      next_cursor: result.next_cursor,
    };
  } catch (err) {
    if (data === null) return;
    data = {
      ...data,
      errors: [
        ...data.errors,
        err instanceof Error ? err.message : String(err),
      ],
    };
  } finally {
    loading = false;
    notify_fun();
  }
}
// ========================================
// ========================================
// --- READ ------------------------------
// the loaded data, or null while it is still loading.
function get_fun(): SHIT_1170_CHESS_PAGE_DATA | null {
  return data;
}
// ========================================
function is_loading_fun(): boolean {
  return loading;
}
// ========================================
// the games on screen. read here so the component has one place to go.
function get_games_fun(): ReadonlyArray<SHIT_1170_CHESS_GAME> {
  return data?.games ?? [];
}
// ========================================
// ========================================
// --- PUBLIC ---------------------------
export const SHIT_1170_CONTRACT_PAGE_CHESS_STORE = {
  load: load_fun,
  load_more: load_more_fun,
  subscribe: subscribe_fun,
  get: get_fun,
  get_games: get_games_fun,
  is_loading: is_loading_fun,
};
// ========================================
// copyright 2026 by sleet.near
