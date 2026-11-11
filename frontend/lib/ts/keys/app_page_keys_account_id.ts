// ========================================
// the rule for the ACCOUNT the key may call.
//
// --- WHY THERE IS NO CONTRACT CHECK: a function-call key names the
// account it may call, and that account is not always a contract — it
// can be an account with a contract on it, and the reader knows which
// one they mean. so the app does NOT ask the chain whether code is
// deployed there. it asks something simpler and cheaper.
//
// --- WHY THE INPUT IS LOWERCASE ONLY: a NEAR account id is lowercase.
// the chain rejects an id with an uppercase letter in it. so the input
// is lowercased as the reader types, and a capital letter never
// reaches the chain.
//
// --- WHY THE FORMAT IS CHECKED AT ALL: an account id is 2 to 64
// characters of lowercase letters, digits, dots, dashes and underscores.
// a string that cannot be an account id is a typo, and a typo that
// costs a transaction is a typo worth catching.
//
// the rules come from near-kit, which is where the chain's own rules
// live, so they cannot drift from what the chain will accept.
import { isValidAccountId } from "near-kit";
// ========================================
// ========================================
// --- HELPERS ----------------------------
// the id, as the chain will see it.
//
// --- WHY THE LOWERCASE HAPPENS HERE AND NOT IN THE INPUT: the input
// lowercases as the reader types, so the box shows the lowercase form.
// this lowercases again anyway, because the id can also arrive from the
// url — and a url is typed by hand.
function normalize_account_id_fun(raw: string): string {
  return raw.trim().toLowerCase();
}
// ========================================
// ========================================
// --- VALIDATE --------------------------
// is this an account id the chain will accept?
//
// --- WHY THE CHECK IS OFFLINE: this is a rule about a STRING, not a
// question about the chain. no network call, no waiting, no deadline —
// and a test can pin every branch of it without a network.
//
// --- WHY IT LOWERCASES FIRST: the id that goes on the chain is the
// LOWERCASED one — the input lowercases as the reader types and the url
// is lowercased on the way in. so the id that is checked is the id that
// is sent, and a capital letter in the box is not a capital letter in
// the transaction.
function is_valid_account_id_fun(raw: string): boolean {
  return isValidAccountId(normalize_account_id_fun(raw));
}
// ========================================
// --- VALIDATE AS WRITTEN --------------
// would the chain accept this string AS IT IS?
//
// --- WHY BOTH RULES EXIST: the normalized one is what the form uses,
// because the form lowercases as the reader types. this one is the
// chain's own rule on the untouched string, and it is what says a
// capital letter is not a NEAR account id at all.
function is_valid_as_written_fun(raw: string): boolean {
  return isValidAccountId(raw.trim());
}
// ========================================
// --- PUBLIC ---------------------------
export const SHIT_1170_APP_PAGE_KEYS_ACCOUNT_ID = {
  normalize: normalize_account_id_fun,
  is_valid: is_valid_account_id_fun,
  is_valid_as_written: is_valid_as_written_fun,
};
// ========================================
// copyright 2026 by sleet.near
// ========================================
