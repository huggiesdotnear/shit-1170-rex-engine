// ========================================
// queue a LIMITED access key to be removed from the signed-in account.
//
// --- WHY THE QUEUE AND NOT THE WALLET: the reader approves the removal
// in the feed, and the wallet shows a preview of the key that goes.
//
// --- WHY THE BACKEND CHECKS THE CHAIN FIRST: a full-access key IS the
// account, and removing it locks the reader out of their own account.
// the backend reads the chain and refuses the removal with
// `key_action_not_allowed`, and the reader is told — the message is
// shown, not swallowed.
//
// --- WHY EVERY PICKED KEY IS ITS OWN QUEUE: a queue is one thing the
// reader approves, and a key action names ONE key. a batch of removals
// would be one popup with several keys in it, and the reader could not
// tell which one was refused.
import {
  shit_1170_app_page_keys_queue_post_fun,
  type SHIT_1170_APP_PAGE_KEYS_QUEUE_RESULT,
} from "./app_page_keys_queue_post";
import { SHIT_1170_AUTH } from "../shared/auth_status";
// ========================================
// ========================================
// --- QUEUE -----------------------------
// put the removal of one key in the queue.
//
// --- WHY THE KEY IS NAMED AND NOTHING ELSE: a removal is about one
// public key. there is no receiver and no methods and no allowance —
// the key already exists, and what goes away is the key itself.
export async function shit_1170_app_page_keys_queue_remove_fun(
  public_key: string,
): Promise<SHIT_1170_APP_PAGE_KEYS_QUEUE_RESULT> {
  const account_id = SHIT_1170_AUTH.get_account_id();
  if (account_id === null) {
    return { queue: null, error: "YOU ARE NOT SIGNED IN" };
  }
  return shit_1170_app_page_keys_queue_post_fun(
    {
      contractId: "",
      method: "",
      keyAction: "delete_key",
      keyPublicKey: public_key,
    },
    `REMOVE A LIMITED KEY FROM ${account_id}`,
  );
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
