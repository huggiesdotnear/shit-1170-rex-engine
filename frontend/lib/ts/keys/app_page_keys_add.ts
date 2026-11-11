// ========================================
// add an access key to the signed-in account.
//
// --- WHY THE KEY IS GENERATED IN THE BROWSER: the private key of a new
// access key has to be shown to the reader ONCE, and nowhere else. if
// it were generated on a server, the server would hold it. so the
// browser makes it, adds it to the chain, and shows it to the reader.
//
// --- WHY ONLY LIMITED KEYS ARE MADE: a limited key is one that can
// only call named methods on one contract. it is safe to hand to
// somebody else — a bot, a script, a teammate — because it cannot move
// the account's funds and cannot touch anything else. a full-access key
// is the account itself, and this page never makes one.
import { generateKey } from "near-kit";
import { near_kit_client } from "@near-kit-tool-box/web";
import { SHIT_1170_AUTH } from "../shared/auth_status";
// ========================================
// ========================================
// --- TYPES ------------------------------
// what an add produced.
export interface SHIT_1170_APP_PAGE_KEYS_ADD_RESULT {
  // the new key's public key.
  public_key: string;
  // the new key's private key — the ONE time the reader sees it.
  secret_key: string;
  // the contract the key may call.
  receiver_id: string;
  // the methods the key may call.
  method_names: string[];
  // the message for the reader, when it failed.
  error: string | null;
}
// ========================================
// ========================================
// --- HELPERS ----------------------------
// the reader declined the wallet popup.
//
// --- WHY A DECLINE IS ITS OWN THING: the reader said no. that is not
// a failure, and a toast that said "it failed" would be telling the
// reader they did something wrong.
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
// --- ADD -------------------------------
// make a limited key and put it on the chain.
//
// --- WHY THE RECEIVER IS A CONTRACT: a function-call key calls
// methods on ONE contract. a contract that is not deployed cannot be
// called, so a typo in the receiver id would burn the key's usefulness.
// the reader's own account is always a valid receiver, so it is the
// fallback.
async function add_key_fun(
  receiver_id: string,
  method_names: string[],
): Promise<SHIT_1170_APP_PAGE_KEYS_ADD_RESULT> {
  const account_id = SHIT_1170_AUTH.get_account_id();
  if (account_id === null) {
    return {
      public_key: "",
      secret_key: "",
      receiver_id,
      method_names,
      error: "YOU ARE NOT SIGNED IN",
    };
  }

  const key_pair = generateKey();
  const public_key = key_pair.publicKey.toString();
  const secret_key = key_pair.secretKey;

  try {
    await near_kit_client()
      .transaction(account_id)
      .addKey(public_key, {
        type: "functionCall",
        receiverId: receiver_id,
        methodNames: method_names,
      })
      .send();
    return {
      public_key,
      secret_key,
      receiver_id,
      method_names,
      error: null,
    };
  } catch (err) {
    return {
      public_key: "",
      secret_key: "",
      receiver_id,
      method_names,
      error: is_declined_fun(err)
        ? "YOU DECLINED THE WALLET"
        : err instanceof Error
          ? err.message
          : String(err),
    };
  }
}
// ========================================
// --- PUBLIC ---------------------------
export const SHIT_1170_APP_PAGE_KEYS_ADD = {
  add: add_key_fun,
};
// ========================================
// copyright 2026 by sleet.near
// ========================================
