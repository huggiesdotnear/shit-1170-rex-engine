// ========================================
// remove the access keys the reader picked.
//
// --- WHY EVERY PICKED KEY GOES IN ONE TRANSACTION: each removed key
// is one DeleteKey action, and a transaction can carry many of them. so
// one wallet popup removes the whole picked set, and the reader is not
// asked to sign the same thing five times.
//
// --- WHY THE FULL-ACCESS KEY CAN NEVER BE IN THE SET: the store
// refuses to pick it, so this does not have to check again — but the
// check is here anyway. a full-access key IS the account, and removing
// it locks the reader out of their own account. the check is cheap and
// the consequence of getting it wrong is severe.
import { near_kit_client } from "@near-kit-tool-box/web";
import { SHIT_1170_AUTH } from "../shared/auth_status";
// ========================================
// ========================================
// --- TYPES ------------------------------
// what a removal produced.
export interface SHIT_1170_APP_PAGE_KEYS_REMOVE_RESULT {
  // how many keys were actually removed.
  removed: number;
  // the message for the reader, when it failed.
  error: string | null;
  // true when the reader declined the wallet popup.
  declined: boolean;
}
// ========================================
// ========================================
// --- HELPERS ----------------------------
// the reader declined the wallet popup.
//
// --- WHY A DECLINE IS ITS OWN THING: the reader said no. that is not
// a failure.
function is_declined_fun(err: unknown): boolean {
  if (typeof err !== "object" || err === null) return false;
  const message = (err as { message?: unknown }).message;
  if (typeof message !== "string") return false;
  const lower = message.toLowerCase();
  return (
    lower.includes("declined") ||
    lower.includes("denied") ||
    lower.includes("rejected") ||
    lower.includes("cancelled") ||
    lower.includes("canceled")
  );
}
// ========================================
// ========================================
// --- REMOVE ----------------------------
// delete every picked key.
//
// --- WHY THE KEYS ARE DELETED FROM THE ACCOUNT AND NOT FROM A LIST:
// a DeleteKey action names the account whose key goes away, and the
// account here is the signed-in one — the same account the keys were
// read from.
async function remove_keys_fun(
  public_keys: ReadonlyArray<string>,
): Promise<SHIT_1170_APP_PAGE_KEYS_REMOVE_RESULT> {
  const account_id = SHIT_1170_AUTH.get_account_id();
  if (account_id === null) {
    return { removed: 0, error: "YOU ARE NOT SIGNED IN", declined: false };
  }
  if (public_keys.length === 0) {
    return { removed: 0, error: "NO KEYS PICKED", declined: false };
  }

  try {
    const builder = near_kit_client().transaction(account_id);
    for (const public_key of public_keys) {
      builder.deleteKey(account_id, public_key);
    }
    await builder.send();
    return { removed: public_keys.length, error: null, declined: false };
  } catch (err) {
    return {
      removed: 0,
      error: is_declined_fun(err)
        ? null
        : err instanceof Error
          ? err.message
          : String(err),
      declined: is_declined_fun(err),
    };
  }
}
// ========================================
// --- PUBLIC ---------------------------
export const SHIT_1170_APP_PAGE_KEYS_REMOVE = {
  remove: remove_keys_fun,
};
// ========================================
// copyright 2026 by sleet.near
// ========================================
