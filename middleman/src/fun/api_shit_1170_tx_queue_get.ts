// ========================================
// API_SHIT_1170_TX_QUEUE_GET
// GET /api/shit-1170/tx-queue — the queue list, filterable.
//
// the list is OPEN: anyone with an api key can list the queues. a queue
// is an open offer, and hiding it would defeat the point.
//
// the answer is `{ "items": [ ...one batch each... ], "total": N,
// "nextCursor": "..." }`, where `total` is the count BEFORE the limit
// and the cursor took their cut. it is how many pages there are.
//
// --- WHY --- the query names are snake_case and are NOT renamed:
// `serde_urlencoded` matches the query key literally, so `accountId`
// would never see the `account_id` the route reads and the filter would
// be silently ignored.
//
// --- WHY --- `account_id` and `contract_id` go out as ONE COMMA
// SEPARATED STRING: the route reads one value per param, so a list
// would need the param to be repeated. one string carries many values,
// and a single value still works.
//
// --- WHY --- `hide_sent` needs an account and the check is made HERE
// as well as on the server: "has this account sent it" is a question
// about ONE account. a queue can be sent by alice and not by bob, so
// there is no single answer about all the accounts at once. asking for
// the filter without the account is a mistake worth naming before the
// round trip, and the backend answers a 400 for the same reason.
//
// --- WHY --- `hide_sent` goes out as the word `true` and not as `1`:
// the backend reads both, and the word is the readable one.
//
// --- WHY --- `cursor` is handed back EXACTLY as the backend gave it:
// the cursor is opaque, and a middleman that reshaped it would break
// the page the moment the backend changed the order.
//
// --- WHY --- EVERY TRANSACTION IN THE ANSWER IS CHECKED: a queued
// transaction is EITHER a contract call OR a key action, and the
// backend always sends the `keyAction` fields. a transaction that
// arrives without them, or with a `keyAction` that is some other word,
// is not one of ours, and a reader that treated it as a contract call
// would show the user something the user is not approving.
// ========================================

import { api_call } from "../stuff/api_call";
import { tx_queue_transaction_valid_fun } from "../stuff/tx_queue_transaction_valid_fun";
import type { REQUEST_OPTIONS } from "../stuff/request_options";
import type { SHIT_1170_TX_QUEUE_LIST } from "../types/shit_1170_tx_queue";
import type { SHIT_1170_TX_QUEUE_QUERY } from "../types/shit_1170_tx_queue_query";

export async function api_shit_1170_tx_queue_get(
  base_url: string,
  query: SHIT_1170_TX_QUEUE_QUERY = {},
  options: REQUEST_OPTIONS = {},
): Promise<SHIT_1170_TX_QUEUE_LIST> {
  const account_id = query.account_id?.trim();
  if (query.hide_sent && !account_id) {
    throw new Error(
      "api_shit_1170_tx_queue_get failed: invalid_request: hide_sent needs an account_id — it is a filter on one account's view",
    );
  }

  const json = await api_call<SHIT_1170_TX_QUEUE_LIST>({
    base_url,
    key: "api_shit_1170_tx_queue",
    query: {
      account_id,
      hide_sent: query.hide_sent,
      contract_id: query.contract_id?.trim(),
      method: query.method?.trim(),
      limit: query.limit,
      cursor: query.cursor?.trim(),
    },
    method: "GET",
    options,
    label: "SHIT_1170 TX QUEUE GET",
    fallback: "api_shit_1170_tx_queue_get failed",
  });

  if (
    typeof json !== "object" ||
    json === null ||
    Array.isArray(json) ||
    !Array.isArray(json.items) ||
    typeof json.total !== "number" ||
    (json.nextCursor !== null && typeof json.nextCursor !== "string")
  ) {
    throw new Error(
      "api_shit_1170_tx_queue_get failed: invalid_response: expected a json object with an items array, a total and a nextCursor",
    );
  }

  // --- WHY --- EVERY BATCH IN THE PAGE IS CHECKED: the items are the
  // batches, and a batch holds transactions. a transaction that is
  // missing its `keyAction` fields, or that carries a `keyAction` that
  // is neither `add_key` nor `delete_key`, is not a transaction the
  // backend wrote.
  for (const item of json.items) {
    if (
      typeof item !== "object" ||
      item === null ||
      Array.isArray(item) ||
      !Array.isArray(item.transactions) ||
      !item.transactions.every((one) => tx_queue_transaction_valid_fun(one))
    ) {
      throw new Error(
        "api_shit_1170_tx_queue_get failed: invalid_response: every transaction must carry its keyAction fields",
      );
    }
  }

  return json;
}
// ========================================
// copyright 2026 by sleet.near
