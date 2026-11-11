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
//
// --- WHY THE TRANSACTION IS BUILT BY HAND AND NOT BY NEAR KIT: the
// wallet cannot draw a preview of near-kit's AddKey, because near-kit
// hands the wallet the borsh permission where the wallet wants the
// flat one. the whole story is in app_page_keys_wallet_action.ts.
import { generateKey } from "near-kit";
import { near_connect_client } from "@near-kit-tool-box/web";
import { SHIT_1170_AUTH } from "../shared/auth_status";
import { SHIT_1170_KEY_VAULT } from "./app_page_keys_vault";
import { shit_1170_wallet_add_key_fun } from "./app_page_keys_wallet_action";
// ========================================
// ========================================
// --- TYPES ------------------------------
// what an add produced.
export interface SHIT_1170_APP_PAGE_KEYS_ADD_RESULT {
  // the new key's public key.
  public_key: string;
  // the new key's private key — the ONLY copy.
  secret_key: string;
  // the contract the key may call.
  receiver_id: string;
  // the methods the key may call.
  method_names: string[];
  // the message for the reader, when it failed.
  error: string | null;
  // true when the reader declined the wallet popup.
  declined: boolean;
  // the hash of the transaction that added the key, for the link to
  // the transaction page.
  tx_hash: string | null;
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
// the hash of a wallet outcome, so the reader can open the transaction.
//
// --- WHY THE HASH IS READ BY THE SHARED HELPER: a near-kit outcome
// carries its hash under `transaction.hash`, and the feed's send reads
// the same thing. one helper, one knowledge.
function outcome_hash_fun(outcome: unknown): string | null {
  if (typeof outcome !== "object" || outcome === null) return null;
  const transaction = (outcome as { transaction?: unknown }).transaction;
  if (typeof transaction !== "object" || transaction === null) return null;
  const hash = (transaction as { hash?: unknown }).hash;
  return typeof hash === "string" ? hash : null;
}
// ========================================
// ========================================
// --- ADD -------------------------------
// make a limited key and put it on the chain.
//
// --- WHY THE RECEIVER IS A CONTRACT: a function-call key calls
// methods on ONE contract. a contract that is not deployed cannot be
// called, so a typo in the receiver id would burn the key's usefulness.
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
      declined: false,
      tx_hash: null,
    };
  }

  const key_pair = generateKey();
  const public_key = key_pair.publicKey.toString();
  const secret_key = key_pair.secretKey;

  try {
    const connector = near_connect_client();
    const wallet = await connector.wallet();
    const transaction = shit_1170_wallet_add_key_fun(
      account_id,
      public_key,
      receiver_id,
      method_names,
    );
    const outcomes = (await wallet.signAndSendTransactions({
      signerId: account_id,
      network: connector.network,
      transactions: [transaction],
    })) as ReadonlyArray<unknown>;

    // --- WHY THE SECRET IS KEPT ONLY NOW: the key is on the chain, and
    // this is the only moment the secret exists anywhere but the
    // reader's screen. if the write fails the secret is lost for good,
    // so it is better to lose it than to write a secret that is out of
    // step with the chain.
    await SHIT_1170_KEY_VAULT.put({
      id: public_key,
      account_id,
      public_key,
      secret_key,
      receiver_id,
      method_names,
      created_at: Date.now(),
    });

    return {
      public_key,
      secret_key,
      receiver_id,
      method_names,
      error: null,
      declined: false,
      tx_hash: outcome_hash_fun(outcomes[0]),
    };
  } catch (err) {
    return {
      public_key: "",
      secret_key: "",
      receiver_id,
      method_names,
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
export const SHIT_1170_APP_PAGE_KEYS_ADD = {
  add: add_key_fun,
};
// ========================================
// copyright 2026 by sleet.near
// ========================================
