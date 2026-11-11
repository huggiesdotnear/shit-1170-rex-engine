// ========================================
// SHIT_1170
// typed client for our own backend routes. every call is same-origin
// against the backend we serve, so the browser never talks to a third
// party api directly.
//
// the naming rule (see DOCS/ROUTES.md): a route is named after the api
// it fronts, not after us.
//   * `api_shit_1170_*` — our OWN routes, under /api/shit-1170/*.
//   * `api_trezu_*` — the trezu routes we proxy, under /api/trezu/*.
//
// the base url is a parameter on every call: the frontend passes the
// origin of the page it is served from, a bin whatever backend it was
// told to talk to.
// ========================================

export * from "./fun/api_shit_1170_api_key_me_get";
export * from "./fun/api_shit_1170_api_key_rotate_post";
export * from "./fun/api_shit_1170_api_keys_post";
export * from "./fun/api_shit_1170_feed_ws";
export * from "./fun/api_shit_1170_ping_get";
export * from "./fun/api_shit_1170_swap_queue_accounts_delete";
export * from "./fun/api_shit_1170_swap_queue_accounts_post";
export * from "./fun/api_shit_1170_swap_queue_get";
export * from "./fun/api_shit_1170_swap_queue_one_get";
export * from "./fun/api_shit_1170_swap_queue_post";
export * from "./fun/api_shit_1170_swap_queue_sent_post";
export * from "./fun/api_shit_1170_tx_queue_accounts_delete";
export * from "./fun/api_shit_1170_tx_queue_accounts_post";
export * from "./fun/api_shit_1170_tx_queue_facets_get";
export * from "./fun/api_shit_1170_tx_queue_get";
export * from "./fun/api_shit_1170_tx_queue_one_get";
export * from "./fun/api_shit_1170_tx_queue_post";
export * from "./fun/api_shit_1170_tx_queue_sent_post";
export * from "./fun/api_shit_1170_tx_queue_stats_contracts_get";
export * from "./fun/api_shit_1170_tx_queue_stats_methods_get";
export * from "./fun/api_trezu_treasuries_get";

export { api_call } from "./stuff/api_call";
export type { API_CALL_PARAMS } from "./stuff/api_call";
export { ENDPOINT_PATHS } from "./stuff/endpoint_paths";
export type { EndpointKey } from "./stuff/endpoint_paths";
export { feed_ws_url_fun } from "./stuff/feed_ws_url";
export { parse_error } from "./stuff/parse_error";
export type {
  SHIT_1170_API_ERROR,
  PARSE_ERROR_CONTEXT,
} from "./stuff/parse_error";
export { build_headers } from "./stuff/request_options";
export type { REQUEST_OPTIONS } from "./stuff/request_options";
export { parse_feed_ws_event } from "./stuff/parse_feed_ws_event";
export { tx_queue_key_action_check_fun } from "./stuff/tx_queue_key_action_check_fun";
export { tx_queue_transaction_valid_fun } from "./stuff/tx_queue_transaction_valid_fun";
export { tx_queue_valid_fun } from "./stuff/tx_queue_valid_fun";

// ---------- the tool primitives ---------
// the argument parsing and the JSON printing every bin uses. they live
// here because the middleman is what makes stdout noisy — it logs its
// request and response with console.log — so a tool that promises pure
// JSON on stdout needs the quieting and the printing from the same
// place as the logging.
export {
  parse_args_fun,
  args_string_fun,
  args_require_string_fun,
  args_bool_fun,
  args_number_fun,
} from "./stuff/args";
export type { ARGS, ARGS_OPTIONS } from "./stuff/args";
export {
  quiet_fun,
  print_json_fun,
  print_error_fun,
} from "./stuff/json_io";
export {
  queue_call_fun,
  DEFAULT_GAS,
  DEFAULT_DEPOSIT,
} from "./stuff/queue_payload";
export type { QUEUE_CALL, QUEUE_CALL_INPUT } from "./stuff/queue_payload";

export type { SHIT_1170_ERROR_RESPONSE } from "./types/common";
export type { SHIT_1170_FEED_WS_EVENT } from "./types/shit_1170_feed_ws";
export type { SHIT_1170_PING } from "./types/shit_1170_ping";
export type {
  SHIT_1170_API_KEY,
  SHIT_1170_API_KEYS_REQUEST,
  SHIT_1170_API_KEYS_RESPONSE,
  SHIT_1170_API_KEY_ME,
  SHIT_1170_SIGNED_ACCOUNT,
} from "./types/shit_1170_api_keys";
export type {
  SHIT_1170_SWAP_QUEUE,
  SHIT_1170_SWAP_QUEUE_ACCOUNT,
  SHIT_1170_SWAP_QUEUE_ACCOUNT_REQUEST,
  SHIT_1170_SWAP_QUEUE_LIST,
  SHIT_1170_SWAP_QUEUE_QUERY,
  SHIT_1170_SWAP_QUEUE_REQUEST,
  SHIT_1170_SWAP_QUEUE_SENT_REQUEST,
  SHIT_1170_SWAP_QUEUE_STATUS,
} from "./types/shit_1170_swap_queue";
export type {
  SHIT_1170_TX_QUEUE,
  SHIT_1170_TX_QUEUE_ACCOUNT,
  SHIT_1170_TX_QUEUE_ACCOUNT_REQUEST,
  SHIT_1170_TX_QUEUE_CONTRACT_STAT,
  SHIT_1170_TX_QUEUE_CONTRACT_STATS,
  SHIT_1170_TX_QUEUE_HASH_REQUEST,
  SHIT_1170_TX_QUEUE_KEY_ACTION,
  SHIT_1170_TX_QUEUE_LIST,
  SHIT_1170_TX_QUEUE_METHOD_STAT,
  SHIT_1170_TX_QUEUE_METHOD_STATS,
  SHIT_1170_TX_QUEUE_REQUEST,
  SHIT_1170_TX_QUEUE_SENT_REQUEST,
  SHIT_1170_TX_QUEUE_STATUS,
  SHIT_1170_TX_QUEUE_TRANSACTION,
  SHIT_1170_TX_QUEUE_TRANSACTION_REQUEST,
} from "./types/shit_1170_tx_queue";
export type {
  SHIT_1170_TX_QUEUE_FACET_ACCOUNT,
  SHIT_1170_TX_QUEUE_FACET_CONTRACT,
  SHIT_1170_TX_QUEUE_FACETS,
} from "./types/shit_1170_tx_queue_facets";
export type {
  SHIT_1170_TX_QUEUE_FACETS_QUERY,
  SHIT_1170_TX_QUEUE_QUERY,
  SHIT_1170_TX_QUEUE_STATS_QUERY,
} from "./types/shit_1170_tx_queue_query";
export type { TREZU_TREASURY } from "./types/trezu_treasury";
export type { TREZU_TREASURY_CONFIG } from "./types/trezu_treasury_config";
export type { TREZU_TREASURY_METADATA } from "./types/trezu_treasury_metadata";
export type { TREZU_TREASURIES_QUERY } from "./types/trezu_treasuries_query";
// ========================================
// copyright 2026 by sleet.near
