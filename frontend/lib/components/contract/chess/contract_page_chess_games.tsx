// ========================================
// the chess page's game table: one row per game, with the game id, the
// two players, the outcome and a link out to the game in the protocol
// pawns web app.
//
// this component does NOT fetch. the chess page has one loader
// (lib/ts/contract/chess/contract_page_chess.ts); this reads its store
// (lib/ts/contract/chess/contract_page_chess_store.ts).
//
// mounts into
// <div id="SHIT_1170_COMPONENT_CONTRACT_CHESS_GAMES">.
// ========================================
import { render } from "preact";
import { useEffect as use_effect } from "preact/hooks";
import { SHIT_1170_MOUNT_ID } from "../../../ts/shared/mount_ids";
import { SHIT_1170_CONTRACT_PAGE_CHESS_STORE } from "../../../ts/contract/chess/contract_page_chess_store";
import { use_page_store_fun } from "../../../ts/shared/use_page_store";
import type { SHIT_1170_CHESS_GAME } from "../../../ts/contract/chess/contract_page_chess";
// ========================================
// ========================================
// --- CONFIG -----------------------------
// how close to the bottom of the list the reader has to scroll before the
// next page is asked for.
const SHIT_1170_LOAD_MORE_THRESHOLD_PX = 600;
// ========================================
// ========================================
// --- HELPERS ----------------------------
// the game id as the table shows it: the block height and the two players.
function game_id_label_fun(game: SHIT_1170_CHESS_GAME): string {
  return String(game.game_id[0]);
}
// ========================================
// the date of a game, in the day and time the api gave it.
function date_fun(value: string): string {
  if (value.length === 0) return "";
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) return value;
  return parsed.toISOString().replace("T", " ").slice(0, 16) + " UTC";
}
// ========================================
// ========================================
// --- ROW -------------------------------
// --- WHY --- the row is a link out to the protocol pawns web app: the
// game itself is played there, and this app has no game board yet.
function game_row_fun(game: SHIT_1170_CHESS_GAME): preact.JSX.Element {
  return (
    <a class="game" href={game.href} target="_blank" rel="noreferrer">
      <span class="game-id">
        <strong>#{game_id_label_fun(game)}</strong>
        <span class="game-date">{date_fun(game.created_at)}</span>
      </span>
      <span class="game-players">
        <span class="game-player is-white">{game.white}</span>
        <span class="game-versus">VS</span>
        <span class="game-player is-black">{game.black}</span>
      </span>
      <span class="game-outcome">
        <span>{game.outcome}</span>
        {game.is_ai && <span class="game-tag">AI</span>}
      </span>
      <span class="game-link">OPEN</span>
    </a>
  );
}
// ========================================
// ========================================
// --- COMPONENT --------------------------
function SHIT_1170_COMPONENT_CONTRACT_CHESS_GAMES_FUN() {
  const page_data = use_page_store_fun(SHIT_1170_CONTRACT_PAGE_CHESS_STORE);
  const loading = SHIT_1170_CONTRACT_PAGE_CHESS_STORE.is_loading();
  const games = page_data?.games ?? [];
  const has_more = page_data?.has_more ?? false;
  const errors = page_data?.errors ?? [];

  // --- WHY --- the next page is asked for when the reader nears the bottom
  // of the list, not when they reach it exactly: the request takes a moment,
  // so asking early means the rows are usually there.
  use_effect(() => {
    if (!has_more || loading) return;
    const on_scroll_fun = (): void => {
      const distance =
        document.documentElement.scrollHeight -
        window.scrollY -
        window.innerHeight;
      if (distance > SHIT_1170_LOAD_MORE_THRESHOLD_PX) return;
      void SHIT_1170_CONTRACT_PAGE_CHESS_STORE.load_more();
    };
    window.addEventListener("scroll", on_scroll_fun);
    return () => {
      window.removeEventListener("scroll", on_scroll_fun);
    };
  }, [has_more, loading, games.length]);

  return (
    <section>
      {errors.map((message) => (
        <p class="error">{message}</p>
      ))}
      {!loading && games.length > 0 && (
        <p class="count">{games.length.toLocaleString()} GAMES</p>
      )}
      <div class="results-scroll">
        {games.length === 0 && !loading ? (
          <p>NO GAMES</p>
        ) : (
          <div class="results">{games.map((game) => game_row_fun(game))}</div>
        )}
        {loading && <p class="loading">LOADING GAMES...</p>}
        {!loading && has_more && <p class="loading">SCROLL FOR MORE...</p>}
      </div>
    </section>
  );
}
// ========================================
// ========================================
// --- MOUNT ------------------------------
// self-mount guarded by id check.
const SELF_MOUNT_ID =
  SHIT_1170_MOUNT_ID.SHIT_1170_COMPONENT_CONTRACT_CHESS_GAMES;
if (typeof document !== "undefined") {
  const el = document.getElementById(SELF_MOUNT_ID);
  if (el !== null) {
    render(<SHIT_1170_COMPONENT_CONTRACT_CHESS_GAMES_FUN />, el);
  }
}
// ========================================
// copyright 2026 by sleet.near
