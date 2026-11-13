// ========================================
// the chess page's ONE load: every fetch the page makes happens here.
//
// WHAT THE PAGE REQUESTS
//   1. GET /stats  the global game counts for the header.
//   2. GET /games  the games on screen, with their api cursor.
//
// WHY THE COUNTS AND THE GAMES ARE TWO REQUESTS
// --- /stats is the api's own total over every game it indexes. the games
// list is cursor paged and carries no total with a cursor, so the count of
// the whole contract cannot be worked out from the rows on screen.
//
// WHY THE GAMES ARE ASKED FOR BY STATUS
// --- the api's default is the active games. the page asks for the active
// games, so the table is the list of games that can still be played.
//
// WHY THE WEB APP LINK IS BUILT HERE
// --- the api's game id is a tuple: [block height, white, black]. the web
// app takes the block height and the two account ids, so the link is built
// from the same tuple the table shows.
import {
  api_protocol_pawns_games,
  api_protocol_pawns_stats,
} from "@sleet-js/middleman-protocol-pawns-api";
import type {
  PROTOCOL_PAWNS_GAME_ID,
  PROTOCOL_PAWNS_GAME_OVERVIEW,
} from "@sleet-js/middleman-protocol-pawns-api";
// ========================================
// ========================================
// --- CONFIG -----------------------------
// how many games one page holds.
const SHIT_1170_CHESS_GAME_PAGE_SIZE = 50;
// the web app that opens a game.
const SHIT_1170_CHESS_WEB_APP_URL = "https://app.protocol-pawns.com/game";
// ========================================
// ========================================
// --- TYPES ------------------------------
// one game as the page shows it.
export interface SHIT_1170_CHESS_GAME {
  // the api's game id tuple: [block height, white, black].
  game_id: PROTOCOL_PAWNS_GAME_ID;
  // the game's own link in the protocol pawns web app.
  href: string;
  white: string;
  black: string;
  // true when one side is played by the ai.
  is_ai: boolean;
  status: string;
  outcome: string;
  created_at: string;
  finished_at: string | null;
}
// ========================================
// everything the chess page has loaded.
export interface SHIT_1170_CHESS_PAGE_DATA {
  games: ReadonlyArray<SHIT_1170_CHESS_GAME>;
  total_games: number;
  active_games: number;
  finished_games: number;
  cancelled_games: number;
  total_moves: number;
  // true when the api has more games behind this one.
  has_more: boolean;
  // the cursor of the next page. the api's own cursor, handed straight back.
  next_cursor: string | null;
  errors: ReadonlyArray<string>;
}
// ========================================
// ========================================
// --- HELPERS ----------------------------
// the readable name of one side of a game.
function player_name_fun(
  player: PROTOCOL_PAWNS_GAME_OVERVIEW["white"],
): string {
  return player.type === "Human" ? player.value : `AI (${player.value})`;
}
// ========================================
// the readable outcome of a game.
function outcome_fun(game: PROTOCOL_PAWNS_GAME_OVERVIEW): string {
  if (game.status === "cancelled") return "CANCELLED";
  const outcome = game.outcome;
  if (outcome === null || outcome === undefined) return "IN PROGRESS";
  if (outcome.result === "Stalemate") return "STALEMATE";
  return `${outcome.color} WINS`;
}
// ========================================
// the web app link for one game.
//
// --- WHY --- the api's black account is null for an ai game, and the web
// app's path wants an empty string in that place.
function game_href_fun(game_id: PROTOCOL_PAWNS_GAME_ID): string {
  const [height, white, black] = game_id;
  const black_path = black ?? "";
  return `${SHIT_1170_CHESS_WEB_APP_URL}/${height}/${white}/${black_path}`;
}
// ========================================
// map one api game into the display record.
function map_game_fun(
  game: PROTOCOL_PAWNS_GAME_OVERVIEW,
): SHIT_1170_CHESS_GAME {
  return {
    game_id: game.game_id,
    href: game_href_fun(game.game_id),
    white: player_name_fun(game.white),
    black: player_name_fun(game.black),
    is_ai: game.black.type === "Ai" || game.white.type === "Ai",
    status: game.status ?? "in_progress",
    outcome: outcome_fun(game),
    created_at: game.created_at ?? "",
    finished_at: game.finished_at ?? null,
  };
}
// ========================================
// ========================================
// --- LOAD -------------------------------
// fetch the global counts and one page of games.
//
// --- WHY --- the two requests run together: they are two different
// endpoints, so the page is as slow as its slowest one.
async function load_games_fun(
  cursor: string | null,
): Promise<SHIT_1170_CHESS_PAGE_DATA> {
  const [stats_result, games_result] = await Promise.all([
    api_protocol_pawns_stats({}),
    api_protocol_pawns_games({
      status: "active",
      limit: SHIT_1170_CHESS_GAME_PAGE_SIZE,
      ...(cursor !== null ? { cursor } : {}),
    }),
  ]);
  const games = games_result.items.map((game) => map_game_fun(game));
  return {
    games,
    total_games: stats_result.total_games,
    active_games: stats_result.active_games,
    finished_games: stats_result.finished_games,
    cancelled_games: stats_result.cancelled_games,
    total_moves: stats_result.total_moves,
    has_more: games_result.next_cursor !== null,
    next_cursor: games_result.next_cursor,
    errors: [],
  };
}
// ========================================
// load the NEXT page of games and append it to the rows on screen.
async function load_more_games_fun(
  cursor: string | null,
): Promise<SHIT_1170_CHESS_PAGE_DATA | null> {
  if (cursor === null) return null;
  const games_result = await api_protocol_pawns_games({
    status: "active",
    limit: SHIT_1170_CHESS_GAME_PAGE_SIZE,
    cursor,
  });
  return {
    games: games_result.items.map((game) => map_game_fun(game)),
    total_games: 0,
    active_games: 0,
    finished_games: 0,
    cancelled_games: 0,
    total_moves: 0,
    has_more: games_result.next_cursor !== null,
    next_cursor: games_result.next_cursor,
    errors: [],
  };
}
// ========================================
// ========================================
// --- PUBLIC ---------------------------
export const SHIT_1170_CONTRACT_PAGE_CHESS = {
  load: load_games_fun,
  load_more: load_more_games_fun,
};
// ========================================
// copyright 2026 by sleet.near
