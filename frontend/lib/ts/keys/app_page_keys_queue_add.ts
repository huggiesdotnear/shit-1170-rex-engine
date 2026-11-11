// ========================================
// queue a LIMITED access key to be added to the signed-in account.
//
// --- WHY THE QUEUE AND NOT THE WALLET: the reader approves the key in
// the feed, next to everything else that is waiting for them, and the
// wallet shows a preview of what the key may do.
//
// --- WHY THE PRIVATE KEY IS NOT SENT ANYWHERE: the queue carries the
// PUBLIC key only. the private key never leaves the browser — it stays
// in the browser's own record, and the chain only ever sees the public
// one.
//
// --- WHY THE ACCOUNT IS THE SIGNED-IN ONE: a key belongs to the
// account it is made on, and the reader is signed in to the account
// whose keys this page shows.
import {
  shit_1170_app_page_keys_queue_post_fun,
  type SHIT_1170_APP_PAGE_KEYS_QUEUE_RESULT,
} from "./app_page_keys_queue_post";
import { SHIT_1170_AUTH } from "../shared/auth_status";
// ========================================
// ========================================
// --- QUEUE -----------------------------
// put the new key in the queue.
//
// --- WHY THE RECEIVER AND THE METHODS GO AS THEY ARE: they are the
// key's reach, and the wallet's preview reads them off the permission.
// a key with no methods may call EVERY method on that contract, and an
// empty list says exactly that.
export async function shit_1170_app_page_keys_queue_add_fun(
  public_key: string,
  receiver_id: string,
  method_names: ReadonlyArray<string>,
  allowance_yocto: string | null,
): Promise<SHIT_1170_APP_PAGE_KEYS_QUEUE_RESULT> {
  const account_id = SHIT_1170_AUTH.get_account_id();
  if (account_id === null) {
    return { queue: null, error: "YOU ARE NOT SIGNED IN" };
  }
  return shit_1170_app_page_keys_queue_post_fun(
    {
      contractId: "",
      method: "",
      keyAction: "add_key",
      keyPublicKey: public_key,
      keyReceiverId: receiver_id,
      keyMethodNames: [...method_names],
      ...(allowance_yocto === null ? {} : { keyAllowance: allowance_yocto }),
    },
    `ADD A LIMITED KEY TO ${account_id} FOR ${receiver_id}`,
  );
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
