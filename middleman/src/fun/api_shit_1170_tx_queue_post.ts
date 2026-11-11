// ========================================
// API_SHIT_1170_TX_QUEUE_POST
// POST /api/shit-1170/tx-queue — queue a BATCH of transactions.
//
// a queue_id is a batch, not one transaction. the agent queues a swap
// that needs three calls, they go in together, and the user signs them
// as one batch in one wallet popup.
//
// the WHO is the `accounts` list — the accounts the agent is queuing
// this for. every one of them must be an account the agent holds an api
// key for, or the backend answers a 400. that is the NAMING RULE: an
// agent may not drop a tx into somebody else's feed.
//
// the api key travels in the header and names the account the batch is
// queued FOR.
//
// --- WHY --- the batch is checked HERE and not only on the server: an
// empty batch is a programming mistake, and catching it before the
// request means the caller gets a clear message instead of a 400 from
// the wire.
//
// --- WHY --- the cap is checked here too: fifty is the number the
// wallet popup can be read at, and a caller who goes over it deserves
// to hear about it before the round trip.
//
// --- WHY --- THE KEY ACTION IS CHECKED HERE AND NOT ONLY ON THE SERVER:
// a queued transaction is EITHER a contract call OR a key action, and
// the mistakes are the ones the backend answers with named codes —
// `key_action_and_contract_call`, `no_key_public_key`,
// `no_key_receiver_id`. they are refused here with the same code
// words, so the caller hears about them before the round trip.
// ========================================

import { api_call } from "../stuff/api_call";
import { tx_queue_key_action_check_fun } from "../stuff/tx_queue_key_action_check_fun";
import { tx_queue_valid_fun } from "../stuff/tx_queue_valid_fun";
import type { REQUEST_OPTIONS } from "../stuff/request_options";
import type {
  SHIT_1170_TX_QUEUE,
  SHIT_1170_TX_QUEUE_REQUEST,
} from "../types/shit_1170_tx_queue";

// how many transactions one batch may hold. it is the backend's cap,
// and it is the number the wallet popup can be read at.
const SHIT_1170_TX_QUEUE_MAX_TRANSACTIONS = 50;

export async function api_shit_1170_tx_queue_post(
  base_url: string,
  body: SHIT_1170_TX_QUEUE_REQUEST,
  options: REQUEST_OPTIONS = {},
): Promise<SHIT_1170_TX_QUEUE> {
  if (!Array.isArray(body.transactions) || body.transactions.length === 0) {
    throw new Error(
      "api_shit_1170_tx_queue_post failed: invalid_request: a queue must hold at least one transaction",
    );
  }
  if (body.transactions.length > SHIT_1170_TX_QUEUE_MAX_TRANSACTIONS) {
    throw new Error(
      `api_shit_1170_tx_queue_post failed: invalid_request: a queue holds at most ${SHIT_1170_TX_QUEUE_MAX_TRANSACTIONS} transactions`,
    );
  }

  // --- THE KEY ACTIONS -----------------
  // every transaction in the batch is checked, in the order they are
  // given, so the message names the position of the offending one.
  body.transactions.forEach((transaction, position) => {
    try {
      tx_queue_key_action_check_fun(transaction, position);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      throw new Error(`api_shit_1170_tx_queue_post failed: ${message}`);
    }
  });

  const json = await api_call<SHIT_1170_TX_QUEUE>({
    base_url,
    key: "api_shit_1170_tx_queue",
    method: "POST",
    body,
    options,
    label: "SHIT_1170 TX QUEUE POST",
    fallback: "api_shit_1170_tx_queue_post failed",
  });

  if (!tx_queue_valid_fun(json)) {
    throw new Error(
      "api_shit_1170_tx_queue_post failed: invalid_response: expected a json object with a queueId and a transactions array, and every transaction must carry its keyAction fields",
    );
  }

  return json;
}
// ========================================
// copyright 2026 by sleet.near
