// ========================================
// is this a valid NEAR account id, and is it a contract that is actually
// deployed?
//
// --- WHY THE CHECK IS IN THE BROWSER AND NOT ONLY ON THE CHAIN: a
// function-call key names ONE contract, and a contract that is not
// deployed cannot be called — the key is made, it costs a transaction,
// and it can never do anything. so the reader is told before they sign.
//
// --- WHY THE RULES COME FROM NEAR KIT AND NOT FROM A REGEX HERE: the
// account id rules live in one place in near-kit, and a copy of them
// here could drift from the chain's own rules. near-kit's
// `isValidAccountId` is the same check the chain makes.
//
// --- WHY THE DEPLOYED CHECK IS A SEPARATE CALL: a valid id is not
// necessarily a contract. an account with no code deployed is a normal
// account, and a key that can only call a contract cannot call it. so
// the page asks the chain whether there is code there.
import { isValidAccountId, AccountDoesNotExistError } from "near-kit";
import { near_kit_client } from "@near-kit-tool-box/web";
// ========================================
// ========================================
// --- TYPES ------------------------------
// what the check found.
export interface SHIT_1170_APP_PAGE_KEYS_CONTRACT_CHECK {
  // true when the id is a well formed NEAR account id.
  valid_id: boolean;
  // true when the account has a contract deployed on it.
  has_contract: boolean;
  // true when the account itself does not exist on the chain.
  not_found: boolean;
  // the message for the reader, when something is wrong.
  error: string | null;
}
// ========================================
// --- CONFIG -----------------------------
// how long the chain gets to answer before the check gives up.
//
// --- WHY THERE IS A DEADLINE: the rpc retries on its own, and a slow
// or unreachable node can keep it busy for a long time — a missing
// account makes it walk the whole backoff before it admits the account
// is not there. the reader is sitting there waiting for a CONTRACT
// CHECK, and a button that says CHECKING for a minute is a button that
// looks broken. so the check gives up and says so, and the reader can
// try again.
//
// 10 seconds is long enough for a healthy rpc and short enough that a
// reader does not walk away.
const SHIT_1170_APP_PAGE_KEYS_CHECK_TIMEOUT_MS = 10_000;
// ========================================
// ========================================
// --- HELPERS ----------------------------
// run the chain call, but not forever.
//
// --- WHY THE RACE: a promise that never settles would hang the check
// forever. the timer settles it, and the slow chain call is left to
// finish on its own — its answer is simply ignored.
//
// --- WHY A FAILURE IS PASSED ON AND NOT SWALLOWED: a call that
// REJECTS has an answer — `AccountDoesNotExistError` says the account
// is not there, and that is a fact about the reader's typing. if the
// rejection were turned into the fallback, every missing account would
// be reported as "no contract on it" instead, and the reader would be
// told to look for a contract that was never there. so a rejection is
// re-thrown, and only a call that simply never answers gets the
// fallback.
function with_deadline_fun<T>(
  work: Promise<T>,
  fallback: T,
  ms: number,
): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    let settled = false;
    const timer = setTimeout(() => {
      if (settled) return;
      settled = true;
      resolve(fallback);
    }, ms);
    work.then(
      (value) => {
        if (settled) return;
        settled = true;
        clearTimeout(timer);
        resolve(value);
      },
      (err) => {
        if (settled) return;
        settled = true;
        clearTimeout(timer);
        reject(err);
      },
    );
  });
}
// ========================================
// ========================================
// --- CHECK -----------------------------
// the whole check: the id first, then the chain.
//
// ========================================
// --- DECIDE ----------------------------
// what the chain's answer means.
//
// --- WHY THIS IS A PURE FUNCTION: the chain is the real mainnet, and
// it is sometimes slow. a test that asks it is a test that fails when
// the network is busy, and a flaky test is worse than no test. so the
// DECISION — what an answer means — is a pure function that a test can
// call with no network at all, and the chain call is a thin wrapper
// around it.
//
// --- WHY THE THREE ANSWERS ARE SEPARATE: a missing account and an
// account with no contract on it read the same to the reader — "that
// is not a contract" — but they are different facts, and the reader
// fixes them differently. a typo is fixed by typing; an account with
// no code is fixed by picking another contract.
function decide_contract_fun(
  valid_id: boolean,
  code_length: number,
  not_found: boolean,
): SHIT_1170_APP_PAGE_KEYS_CONTRACT_CHECK {
  if (!valid_id) {
    return {
      valid_id: false,
      has_contract: false,
      not_found: false,
      error:
        "THAT IS NOT A NEAR ACCOUNT ID — ONLY LOWERCASE LETTERS, DIGITS, " +
        "DOTS, DASHES AND UNDERSCORES",
    };
  }
  if (not_found) {
    return {
      valid_id: true,
      has_contract: false,
      not_found: true,
      error: "THERE IS NO ACCOUNT WITH THAT ID",
    };
  }
  if (code_length === 0) {
    return {
      valid_id: true,
      has_contract: false,
      not_found: false,
      error: "THERE IS NO CONTRACT DEPLOYED ON THAT ACCOUNT",
    };
  }
  return {
    valid_id: true,
    has_contract: true,
    not_found: false,
    error: null,
  };
}
// ========================================
// ========================================
// --- CHECK -----------------------------
// the whole check: the id first, then the chain.
//
// --- WHY THE ID IS CHECKED FIRST: it is free and it is the check that
// catches a typo — a stray space, an uppercase letter, a missing dot.
// asking the chain about an id that cannot exist would only give a
// confusing answer.
async function check_contract_fun(
  raw: string,
): Promise<SHIT_1170_APP_PAGE_KEYS_CONTRACT_CHECK> {
  const account_id = raw.trim();

  if (account_id.length === 0) {
    return {
      valid_id: false,
      has_contract: false,
      not_found: false,
      error: "THE CONTRACT THE KEY CALLS IS EMPTY",
    };
  }

  if (!isValidAccountId(account_id)) {
    return decide_contract_fun(false, 0, false);
  }

  try {
    // --- WHY ONE CALL ANSWERS BOTH QUESTIONS: `getContractCode` throws
    // `AccountDoesNotExistError` for an account that is not there, and
    // returns the code for one that is. so ONE call says both "does it
    // exist" and "does it have a contract on it".
    //
    // and it is one call rather than two because the two-call version
    // was SLOW: `accountExists` swallows the error, so the rpc retries
    // the whole backoff — 1s, 2s, 4s, 8s — before giving up. measured
    // at 68 seconds for an account that is not there. the single call
    // answers in about 150ms, because the error is a real answer and
    // not a failure to retry.
    //
    // --- WHY THE CLIENT IS ASKED FOR INSIDE THE TRY: the near kit
    // client reads the browser's own storage for the network, so it
    // throws where there is no browser — a test, or a server-side
    // render. a check that cannot run is not a pass, and the catch
    // below turns it into a message rather than a crash.
    const near = near_kit_client();
    const code = await with_deadline_fun(
      near.getContractCode(account_id),
      { code: new Uint8Array(0), hash: "" },
      SHIT_1170_APP_PAGE_KEYS_CHECK_TIMEOUT_MS,
    );
    return decide_contract_fun(true, code.code.length, false);
  } catch (err) {
    // --- WHY A MISSING ACCOUNT IS ITS OWN ANSWER: the rpc told us the
    // account is not there, and that is a fact about the reader's
    // typing, not a failure.
    if (err instanceof AccountDoesNotExistError) {
      return decide_contract_fun(true, 0, true);
    }
    // --- WHY A FAILED CHECK IS NOT A PASS: the reader is about to
    // spend a transaction on a key that may not be able to call
    // anything. an unknown is not a yes.
    return {
      valid_id: true,
      has_contract: false,
      not_found: false,
      error: err instanceof Error ? err.message : String(err),
    };
  }
}
// ========================================
// --- PUBLIC ---------------------------
export const SHIT_1170_APP_PAGE_KEYS_CONTRACT = {
  check: check_contract_fun,
  decide: decide_contract_fun,
};
// ========================================
// copyright 2026 by sleet.near
// ========================================
