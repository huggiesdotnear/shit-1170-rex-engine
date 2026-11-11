// ========================================
// the keys page's ACCOUNT ID rule and its URL parameters, tested.
//
// two things are pinned here:
//
// 1. the account id rule. it is the only check the form makes — the
//    account a key may call does NOT have to be a contract, so the app
//    does not ask the chain whether code is deployed there. it asks one
//    local thing: is this string an account id the chain will accept?
//
// 2. the url parameters that prefill the form.
//
// both are PURE — a rule about a string, and a read of the url — so
// neither test touches the network. a flaky test is worse than no test.
//
// run it with:
//   bun test frontend/lib/ts/keys/app_page_keys_account_id.test.ts
// ========================================

import { describe, expect, test } from "bun:test";

import { SHIT_1170_APP_PAGE_KEYS_ACCOUNT_ID } from "./app_page_keys_account_id";
import { SHIT_1170_APP_PAGE_KEYS_URL } from "./app_page_keys_url";
// ========================================
// --- THE ACCOUNT ID -------------------
describe("the keys page's account id", () => {
  test("a NORMAL id is accepted", () => {
    expect(SHIT_1170_APP_PAGE_KEYS_ACCOUNT_ID.is_valid("wrap.near")).toBe(true);
    expect(SHIT_1170_APP_PAGE_KEYS_ACCOUNT_ID.is_valid("a.b")).toBe(true);
    expect(
      SHIT_1170_APP_PAGE_KEYS_ACCOUNT_ID.is_valid("some-account_v2.near"),
    ).toBe(true);
  });

  test("an id that is NOT a contract is accepted — the rule is about the STRING", () => {
    // the account a key may call does not have to have code on it, and
    // the app does not ask the chain about that. it only asks whether
    // this could BE an account id.
    expect(
      SHIT_1170_APP_PAGE_KEYS_ACCOUNT_ID.is_valid("plain-account.near"),
    ).toBe(true);
    expect(SHIT_1170_APP_PAGE_KEYS_ACCOUNT_ID.is_valid("a.near")).toBe(true);
  });

  test("an UPPERCASE id is NOT a NEAR account id as it is written", () => {
    // --- WHY BOTH: the form lowercases as the reader types, so the id
    // that reaches the chain is lowercase. but the chain's own rule is
    // about the string as written, and a capital letter is not a NEAR
    // account id at all.
    expect(
      SHIT_1170_APP_PAGE_KEYS_ACCOUNT_ID.is_valid_as_written("Wrap.near"),
    ).toBe(false);
    expect(
      SHIT_1170_APP_PAGE_KEYS_ACCOUNT_ID.is_valid_as_written("wrap.near"),
    ).toBe(true);
  });

  test("a SPACE or a BANG is refused as it is written", () => {
    expect(
      SHIT_1170_APP_PAGE_KEYS_ACCOUNT_ID.is_valid_as_written("bad id"),
    ).toBe(false);
    expect(
      SHIT_1170_APP_PAGE_KEYS_ACCOUNT_ID.is_valid_as_written("bad!id"),
    ).toBe(false);
  });

  test("a TOO SHORT id is refused", () => {
    expect(SHIT_1170_APP_PAGE_KEYS_ACCOUNT_ID.is_valid("a")).toBe(false);
    expect(SHIT_1170_APP_PAGE_KEYS_ACCOUNT_ID.is_valid("")).toBe(false);
  });

  test("an id with a TRAILING DOT is refused", () => {
    expect(SHIT_1170_APP_PAGE_KEYS_ACCOUNT_ID.is_valid("hello.near.")).toBe(
      false,
    );
  });

  test("the id is LOWERCASED and TRIMMED on the way in", () => {
    expect(SHIT_1170_APP_PAGE_KEYS_ACCOUNT_ID.normalize("  Wrap.NEAR ")).toBe(
      "wrap.near",
    );
    expect(SHIT_1170_APP_PAGE_KEYS_ACCOUNT_ID.normalize("WRAP.NEAR")).toBe(
      "wrap.near",
    );
  });

  test("an UPPERCASE id that gets lowercased is then VALID", () => {
    // --- WHY THIS MATTERS: the input lowercases as the reader types, so
    // a capital letter never reaches the chain. the rule has to agree.
    const normalized =
      SHIT_1170_APP_PAGE_KEYS_ACCOUNT_ID.normalize("Wrap.NEAR");
    expect(SHIT_1170_APP_PAGE_KEYS_ACCOUNT_ID.is_valid(normalized)).toBe(true);
  });
});
// ========================================
// --- THE URL --------------------------
describe("the keys page's url parameters", () => {
  test("a blank parameter is ABSENT, not an empty string", () => {
    // --- WHY --- blank is treated as absent: a trailing `?account=` link
    // points at nothing, and an empty box is not a value.
    expect(SHIT_1170_APP_PAGE_KEYS_URL.normalize(null)).toBe(null);
    expect(SHIT_1170_APP_PAGE_KEYS_URL.normalize("")).toBe(null);
    expect(SHIT_1170_APP_PAGE_KEYS_URL.normalize("   ")).toBe(null);
  });

  test("a parameter is trimmed", () => {
    expect(SHIT_1170_APP_PAGE_KEYS_URL.normalize("  wrap.near  ")).toBe(
      "wrap.near",
    );
  });

  test("the reader is SAFE with no window — the ts also runs in tests", () => {
    // there is no window in bun's test runner, so this must not throw.
    expect(SHIT_1170_APP_PAGE_KEYS_URL.read_account()).toBe(null);
    expect(SHIT_1170_APP_PAGE_KEYS_URL.read_methods()).toBe(null);
  });
});
// ========================================
// copyright 2026 by sleet.near
// ========================================
