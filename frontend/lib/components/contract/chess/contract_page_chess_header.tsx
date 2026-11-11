// ========================================
// the chess page's header: the heading, the contract card and the game's
// own counts.
//
// this component does NOT fetch. the chess page has one loader
// (lib/ts/contract/chess/contract_page_chess.ts); this reads its store
// (lib/ts/contract/chess/contract_page_chess_store.ts).
// ========================================
import { render } from "preact";
import { SHIT_1170_MOUNT_ID } from "../../../ts/shared/mount_ids";
import { SHIT_1170_CONTRACT_PAGE_CHESS_STORE } from "../../../ts/contract/chess/contract_page_chess_store";
import { use_page_store_fun } from "../../../ts/shared/use_page_store";
// ========================================
// ========================================
// --- CONFIG -----------------------------
// --- WHY --- the id is a constant and not a fetched value: it is the
// contract the page is ABOUT, not something the api reports.
const SHIT_1170_CHESS_CONTRACT_ID = "app.chess-game.near";
// ========================================
// ========================================
// --- COMPONENT --------------------------
function SHIT_1170_COMPONENT_CONTRACT_CHESS_HEADER_FUN() {
  const page_data = use_page_store_fun(SHIT_1170_CONTRACT_PAGE_CHESS_STORE);
  const loading = SHIT_1170_CONTRACT_PAGE_CHESS_STORE.is_loading();

  return (
    <>
      <header>
        <h1>Chess</h1>
      </header>
      <section class="contracts">
        <article>
          <span class="contract-label">GAME</span>
          <strong class="contract-title">Protocol Pawns</strong>
          <code class="contract-id">{SHIT_1170_CHESS_CONTRACT_ID}</code>
        </article>
      </section>
      <section class="stats">
        <div>
          <strong>{(page_data?.total_games ?? 0).toLocaleString()}</strong>
          <span>GAMES</span>
        </div>
        <div>
          <strong>{(page_data?.active_games ?? 0).toLocaleString()}</strong>
          <span>ACTIVE</span>
        </div>
        <div>
          <strong>{(page_data?.finished_games ?? 0).toLocaleString()}</strong>
          <span>FINISHED</span>
        </div>
        <div>
          <strong>{(page_data?.cancelled_games ?? 0).toLocaleString()}</strong>
          <span>CANCELLED</span>
        </div>
        <div>
          <strong>{(page_data?.total_moves ?? 0).toLocaleString()}</strong>
          <span>MOVES</span>
        </div>
        {loading && (
          <div>
            <strong>...</strong>
            <span>LOADING</span>
          </div>
        )}
      </section>
    </>
  );
}
// ========================================
// ========================================
// --- MOUNT ------------------------------
// self-mount guarded by id check.
const SELF_MOUNT_ID =
  SHIT_1170_MOUNT_ID.SHIT_1170_COMPONENT_CONTRACT_CHESS_HEADER;
if (typeof document !== "undefined") {
  const el = document.getElementById(SELF_MOUNT_ID);
  if (el !== null) {
    render(<SHIT_1170_COMPONENT_CONTRACT_CHESS_HEADER_FUN />, el);
  }
}
// ========================================
// copyright 2026 by sleet.near
