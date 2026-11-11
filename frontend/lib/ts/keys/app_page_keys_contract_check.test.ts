// ========================================
// the keys page's CONTRACT CHECK, tested.
//
// this is the file that stops the reader spending a transaction on a
// key that can never call anything. a function-call key names ONE
// contract, and a contract with no code deployed cannot be called — so
// the key would be made, it would cost a transaction, and it would do
// nothing at all.
//
// --- WHY THE DECISION IS TESTED WITHOUT THE NETWORK: the decision —
// what an answer means — is a PURE function, and a pure function is
// tested by calling it. the chain is the real mainnet, and it is
// sometimes slow enough to time a test out. a flaky test is worse than
// no test, so the half that can be pinned offline IS pinned offline,
// and the chain is left to the page.
//
// the id rules are near-kit's own, so the id half needs no test of its
// own either.
//
// run it with:
//   bun test frontend/lib/ts/keys/app_page_keys_contract_check.test.ts
// ========================================

import { describe, expect, test } from "bun:test";

// --- WHY THE BROWSER IS STUBBED: the near kit client reads the
// browser's own storage to know which network to talk to, so it needs
// a localStorage to read. the stub is the smallest one that works: the
// client only ever READS the network id.
(globalThis as { localStorage?: unknown }).localStorage = {
  getItem: () => null,
  setItem: () => {},
  removeItem: () => {},
};

import { SHIT_1170_APP_PAGE_KEYS_CONTRACT } from "./app_page_keys_contract_check";
// ========================================
// --- THE DECISION ---------------------
describe("the contract check's decision", () => {
  test("a MALFORMED id is refused, and the chain is never asked", () => {
    const result = SHIT_1170_APP_PAGE_KEYS_CONTRACT.decide(false, 0, false);
    expect(result.valid_id).toBe(false);
    expect(result.has_contract).toBe(false);
    expect(result.not_found).toBe(false);
    expect(result.error).not.toBe(null);
  });

  test("a MISSING account is its own answer", () => {
    const result = SHIT_1170_APP_PAGE_KEYS_CONTRACT.decide(true, 0, true);
    expect(result.valid_id).toBe(true);
    expect(result.not_found).toBe(true);
    expect(result.has_contract).toBe(false);
    expect(result.error).toBe("THERE IS NO ACCOUNT WITH THAT ID");
  });

  test("an account with NO contract on it says so", () => {
    const result = SHIT_1170_APP_PAGE_KEYS_CONTRACT.decide(true, 0, false);
    expect(result.valid_id).toBe(true);
    expect(result.not_found).toBe(false);
    expect(result.has_contract).toBe(false);
    expect(result.error).toBe("THERE IS NO CONTRACT DEPLOYED ON THAT ACCOUNT");
  });

  test("an account WITH code passes", () => {
    const result = SHIT_1170_APP_PAGE_KEYS_CONTRACT.decide(true, 185625, false);
    expect(result.valid_id).toBe(true);
    expect(result.has_contract).toBe(true);
    expect(result.not_found).toBe(false);
    expect(result.error).toBe(null);
  });

  test("a MISSING account is NEVER a good contract", () => {
    // the whole point of the check: whatever the chain says, an account
    // that is not there must not come back as a key that can call it.
    const result = SHIT_1170_APP_PAGE_KEYS_CONTRACT.decide(true, 0, true);
    expect(result.has_contract).toBe(false);
    expect(result.error).not.toBe(null);
  });

  test("an account with NO code is NEVER a good contract", () => {
    const result = SHIT_1170_APP_PAGE_KEYS_CONTRACT.decide(true, 0, false);
    expect(result.has_contract).toBe(false);
    expect(result.error).not.toBe(null);
  });
});
// ========================================
// --- THE ID ---------------------------
describe("the contract check's id", () => {
  test("an EMPTY id is refused", async () => {
    const result = await SHIT_1170_APP_PAGE_KEYS_CONTRACT.check("");
    expect(result.valid_id).toBe(false);
    expect(result.error).toBe("THE CONTRACT THE KEY CALLS IS EMPTY");
  });

  test("a BLANK id is refused too — the reader's spaces are trimmed", async () => {
    const result = await SHIT_1170_APP_PAGE_KEYS_CONTRACT.check("   ");
    expect(result.valid_id).toBe(false);
    expect(result.error).not.toBe(null);
  });

  test("a MALFORMED id is refused without touching the chain", async () => {
    // an id with an uppercase letter, a space and a bang — none of
    // which the chain would accept. these never reach the network, so
    // they are quick and they never flake.
    for (const bad of ["Wrap.near", "bad id!", "a", "hello.near."]) {
      const result = await SHIT_1170_APP_PAGE_KEYS_CONTRACT.check(bad);
      expect(result.valid_id).toBe(false);
      expect(result.error).not.toBe(null);
      expect(result.has_contract).toBe(false);
    }
  });
});
// ========================================
// copyright 2026 by sleet.near
// ========================================
