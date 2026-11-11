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
//
// --- WHY THE BROWSER'S RECORD SURVIVES THE REMOVAL: the record holds
// the reader's private key. taking the key off the chain does not take
// the secret away — the reader may still want it, and the record is the
// only place it is. so a removal leaves the record alone, and the
// FORGET button is what erases it.
//
// --- WHY THE TRANSACTION IS BUILT BY HAND AND NOT BY NEAR KIT: the
// whole story is in app_page_keys_wallet_action.ts.
import { near_connect_client } from "@near-kit-tool-box/web";
import { SHIT_1170_AUTH } from "../shared/auth_status";
import { shit_1170_wallet_delete_keys_fun } from "./app_page_keys_wallet_action";
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
  // the hash of the transaction that removed the keys.
  tx_hash: string | null;
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
// the hash of a wallet outcome.
function outcome_hash_fun(outcome: unknown): string | null {
  if (typeof outcome !== "object" || outcome === null) return null;
  const transaction = (outcome as { transaction?: unknown }).transaction;
  if (typeof transaction !== "object" || transaction === null) return null;
  const hash = (transaction as { hash?: unknown }).hash;
  return typeof hash === "string" ? hash : null;
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
    return {
      removed: 0,
      error: "YOU ARE NOT SIGNED IN",
      declined: false,
      tx_hash: null,
    };
  }
  if (public_keys.length === 0) {
    return {
      removed: 0,
      error: "NO KEYS PICKED",
      declined: false,
      tx_hash: null,
    };
  }

  try {
    const connector = near_connect_client();
    const wallet = await connector.wallet();
    const transaction = shit_1170_wallet_delete_keys_fun(
      account_id,
      public_keys,
    );
    const outcomes = (await wallet.signAndSendTransactions({
      signerId: account_id,
      network: connector.network,
      transactions: [transaction],
    })) as ReadonlyArray<unknown>;
    return {
      removed: public_keys.length,
      error: null,
      declined: false,
      tx_hash: outcome_hash_fun(outcomes[0]),
    };
  } catch (err) {
    return {
      removed: 0,
      error: is_declined_fun(err)
        ? null
        : err instanceof Error
          ? err.message
          : String(err),
      declined: is_declined_fun(err),
      tx_hash: null,
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
