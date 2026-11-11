// ========================================
// the keys page's ALLOWANCE, tested.
//
// the allowance is a LIMIT on how much a key may spend. two things
// about it are pinned here:
//
// 1. what the reader types. the box takes NEAR — "1.5" — and the app
//    converts. near-kit's own parser refuses a bare number on purpose,
//    and that rule is right for a program and wrong for a box that
//    SAYS NEAR on it.
//
// 2. what the reader sees. a yoctoNEAR is a 10^-24 NEAR, so the figure
//    is cut to a readable length — and the cut must NEVER turn a real
//    limit into "0", because "the key can spend nothing" is a
//    DIFFERENT statement from "the key can spend one yoctoNEAR".
//
// both are pure, so neither test touches the network.
//
// run it with:
//   bun test frontend/lib/ts/keys/app_page_keys_allowance.test.ts
// ========================================

import { describe, expect, test } from "bun:test";

import { SHIT_1170_APP_PAGE_KEYS_ALLOWANCE } from "./app_page_keys_allowance";
// ========================================
// --- THE PARSE ------------------------
describe("the allowance the reader types", () => {
  test("a WHOLE number of NEAR is converted", () => {
    expect(SHIT_1170_APP_PAGE_KEYS_ALLOWANCE.parse("1")).toBe(
      "1000000000000000000000000",
    );
    expect(SHIT_1170_APP_PAGE_KEYS_ALLOWANCE.parse("100")).toBe(
      "100000000000000000000000000",
    );
  });

  test("a FRACTION of NEAR is converted", () => {
    expect(SHIT_1170_APP_PAGE_KEYS_ALLOWANCE.parse("1.5")).toBe(
      "1500000000000000000000000",
    );
    expect(SHIT_1170_APP_PAGE_KEYS_ALLOWANCE.parse("0.1")).toBe(
      "100000000000000000000000",
    );
  });

  test("a BLANK allowance means NO allowance", () => {
    // --- WHY --- a blank box is an answer and not a missing value: the
    // key can call the methods and attach nothing, and that is a real
    // choice the reader made.
    expect(SHIT_1170_APP_PAGE_KEYS_ALLOWANCE.parse("")).toBe(null);
    expect(SHIT_1170_APP_PAGE_KEYS_ALLOWANCE.parse("   ")).toBe(null);
  });

  test("ZERO is an allowance, and not the absence of one", () => {
    // a key with a limit of zero can attach a deposit of zero and can
    // still be called. a key with NO allowance can attach nothing.
    expect(SHIT_1170_APP_PAGE_KEYS_ALLOWANCE.parse("0")).toBe("0");
  });

  test("text that is NOT a number is refused", () => {
    for (const bad of ["abc", "1,5", "1.2.3", "1e5", "-1", "1 NEAR", "NEAR"]) {
      expect(SHIT_1170_APP_PAGE_KEYS_ALLOWANCE.parse(bad)).toBe(null);
    }
  });

  test("TOO MANY decimals is refused, not quietly rounded", () => {
    // --- WHY --- a fraction smaller than a yoctoNEAR cannot be sent,
    // and a key whose allowance is not what the reader typed is a key
    // with an allowance the reader did not choose.
    expect(SHIT_1170_APP_PAGE_KEYS_ALLOWANCE.parse("0.0000001")).toBe(null);
    expect(SHIT_1170_APP_PAGE_KEYS_ALLOWANCE.parse("0.000001")).toBe(
      "1000000000000000000",
    );
  });
});
// ========================================
// --- THE FORMAT -----------------------
describe("the allowance the reader sees", () => {
  test("a WHOLE number of NEAR reads as NEAR", () => {
    expect(
      SHIT_1170_APP_PAGE_KEYS_ALLOWANCE.format("1000000000000000000000000")
        .human,
    ).toBe("1 NEAR");
    expect(
      SHIT_1170_APP_PAGE_KEYS_ALLOWANCE.format("1500000000000000000000000")
        .human,
    ).toBe("1.5 NEAR");
  });

  test("a FRACTION reads as NEAR", () => {
    expect(
      SHIT_1170_APP_PAGE_KEYS_ALLOWANCE.format("100000000000000000000000")
        .human,
    ).toBe("0.1 NEAR");
  });

  test("ONE YOCTONEAR is NEVER shown as zero", () => {
    // --- WHY THIS MATTERS MOST: a figure that rounds to "0" says the
    // key can spend nothing, and the key CAN spend — one yoctoNEAR is
    // the smallest thing there is. the cut must keep the first
    // significant digit however far down it is.
    expect(SHIT_1170_APP_PAGE_KEYS_ALLOWANCE.format("1").human).toBe(
      "0.000000000000000000000001 NEAR",
    );
    expect(SHIT_1170_APP_PAGE_KEYS_ALLOWANCE.format("1000").human).toBe(
      "0.000000000000000000001 NEAR",
    );
  });

  test("the figure is CUT and never rounded UP", () => {
    // 1.2345678 NEAR — the shown figure must not be more than the
    // limit really is.
    const shown = SHIT_1170_APP_PAGE_KEYS_ALLOWANCE.format(
      "1234567800000000000000000",
    ).human;
    expect(shown).toBe("1.234567 NEAR");
    expect(shown).not.toBe("1.234568 NEAR");
  });

  test("a NULL allowance has NO human figure", () => {
    // --- WHY --- "no allowance" and "0 NEAR" are different things. a
    // key with no allowance cannot attach a deposit at all; a key with
    // a limit of zero can attach a deposit of zero and can still be
    // called. the difference matters to a reader deciding whether a key
    // is safe to hand to somebody.
    const formatted = SHIT_1170_APP_PAGE_KEYS_ALLOWANCE.format(null);
    expect(formatted.human).toBe(null);
    expect(formatted.yocto).toBe(null);
  });

  test("a ZERO allowance reads as zero — it is not the same as none", () => {
    expect(SHIT_1170_APP_PAGE_KEYS_ALLOWANCE.format("0").human).toBe("0 NEAR");
  });

  test("the raw figure rides along", () => {
    const formatted = SHIT_1170_APP_PAGE_KEYS_ALLOWANCE.format(
      "1500000000000000000000000",
    );
    expect(formatted.yocto).toBe("1500000000000000000000000");
  });
});
// ========================================
// --- THE ROUND TRIP -------------------
describe("the allowance's round trip", () => {
  test("what the reader types is what the reader sees", () => {
    for (const typed of ["1", "1.5", "0.1", "0.000001", "100"]) {
      const yocto = SHIT_1170_APP_PAGE_KEYS_ALLOWANCE.parse(typed);
      const human = SHIT_1170_APP_PAGE_KEYS_ALLOWANCE.format(yocto).human;
      expect(human).toBe(`${typed} NEAR`);
    }
  });
});
// ========================================
// copyright 2026 by sleet.near
// ========================================
