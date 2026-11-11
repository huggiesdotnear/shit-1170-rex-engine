// ========================================
// post a QUEUE of key actions from the keys page.
//
// --- WHY THE QUEUE AND NOT THE WALLET: the keys page used to sign the
// key action itself, and the reader approved it in a wallet popup right
// there. queueing puts it in the FEED instead, where the reader sees
// what is about to happen to their keys next to everything else that is
// waiting for them, and approves it there.
//
// --- WHY THE API KEY IS REQUIRED: the queue's POST route names the
// account the batch is for, and the key says who is asking. the keys
// page is the reader's own account, so the key is the reader's own.
//
// --- WHY THE ACCOUNT IS NAMED IN THE BODY: the WHO list is what puts
// the batch in somebody's feed, and the reader is the one it is for. a
// batch with an empty WHO list is a batch for the caller's own
// account, and naming the account makes the batch land on the reader's
// own feed with a row to approve.
import {
  api_shit_1170_tx_queue_post,
  type SHIT_1170_TX_QUEUE,
  type SHIT_1170_TX_QUEUE_TRANSACTION_REQUEST,
} from "@huggies/shit-1170-rex-engine-middleman";
import { SHIT_1170_BASE_URL } from "../shared/base_url";
import { api_key_options_fun } from "../shared/api_key_token";
import { SHIT_1170_AUTH } from "../shared/auth_status";
import { shit_1170_app_page_keys_queue_error_message_fun } from "./app_page_keys_queue_error";
// ========================================
// ========================================
// --- TYPES ------------------------------
// what a queued key action produced.
export interface SHIT_1170_APP_PAGE_KEYS_QUEUE_RESULT {
  // the queue that was written, or null when nothing was written.
  queue: SHIT_1170_TX_QUEUE | null;
  // the message for the reader, when it failed.
  error: string | null;
}
// ========================================
// ========================================
// --- POST ------------------------------
// put one key action in the queue.
//
// --- WHY THE TRANSACTION NAMES NO CONTRACT AND NO METHOD: a key action
// is not a call. it names a public key and what that key may do, and
// the backend refuses a transaction that claims to be both.
//
// --- WHY THE ERROR COMES BACK AS TEXT AND NOT AS A THROW: the reader
// has to be told WHY the key was refused — a full-access key cannot be
// removed by the queue, and a key that is not there cannot either — and
// the backend's message says which one it was.
export async function shit_1170_app_page_keys_queue_post_fun(
  transaction: SHIT_1170_TX_QUEUE_TRANSACTION_REQUEST,
  what: string,
): Promise<SHIT_1170_APP_PAGE_KEYS_QUEUE_RESULT> {
  const account_id = SHIT_1170_AUTH.get_account_id();
  if (account_id === null) {
    return { queue: null, error: "YOU ARE NOT SIGNED IN" };
  }
  try {
    const options = await api_key_options_fun(true, account_id);
    const queue = await api_shit_1170_tx_queue_post(
      SHIT_1170_BASE_URL,
      {
        what,
        transactions: [transaction],
        accounts: [account_id],
      },
      options,
    );
    return { queue, error: null };
  } catch (err) {
    return {
      queue: null,
      error: shit_1170_app_page_keys_queue_error_message_fun(err),
    };
  }
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
