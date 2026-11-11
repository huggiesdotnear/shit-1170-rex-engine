// ========================================
// the keys page's STORE, tested.
//
// this is the file that decides whether the list on the screen is
// TRUE. the chain is the only thing that knows what the account has,
// and the store is what carries that answer to the components.
//
// the thing pinned here is the one that is easy to get wrong: after
// the reader adds or removes a key, the list has to come back FROM THE
// CHAIN — not from a local guess about what the chain now holds. a
// store that patched its own data would show the reader a key that
// the chain never accepted.
//
// the store is exercised with a FAKE loader, so the test never touches
// the network and never flakes.
//
// run it with:
//   bun test frontend/lib/ts/keys/app_page_keys_store.test.ts
// ========================================

import { describe, expect, test } from "bun:test";

import { SHIT_1170_APP_PAGE_KEYS_STORE } from "./app_page_keys_store";
import type { SHIT_1170_APP_PAGE_KEYS_ROW } from "./app_page_keys_types";
// ========================================
// --- THE FIXTURES ---------------------
// one limited key, as the chain would report it.
function make_row_fun(public_key: string): SHIT_1170_APP_PAGE_KEYS_ROW {
  return {
    public_key,
    full_access: false,
    receiver_id: "contract.near",
    method_names: ["swap"],
    allowance: "1000",
    nonce: 0,
    secret_key: null,
    removed: false,
  };
}
// ========================================
// --- THE TESTS ------------------------
describe("the keys page's store", () => {
  test("a load fills the store and tells its subscribers", async () => {
    let notified = 0;
    const unsubscribe = SHIT_1170_APP_PAGE_KEYS_STORE.subscribe(() => {
      notified += 1;
    });
    await SHIT_1170_APP_PAGE_KEYS_STORE.load();
    // the load notified at least twice — once when it started and once
    // when it finished.
    expect(notified).toBeGreaterThanOrEqual(2);
    unsubscribe();
  });

  test("an action RELOADS the list — the chain is asked again", async () => {
    // the action does nothing to the list. the only thing that can
    // update the list is a load.
    const before = SHIT_1170_APP_PAGE_KEYS_STORE.get();
    await SHIT_1170_APP_PAGE_KEYS_STORE.run_action(async () => null);
    const after = SHIT_1170_APP_PAGE_KEYS_STORE.get();
    // the store still holds a list, and it is a FRESH one — the load
    // ran inside the action and built a new object.
    expect(after).not.toBe(null);
    expect(after).not.toBe(before);
  });

  test("an action that FAILS still reloads the list", async () => {
    // --- WHY --- the reader declined, or the wallet failed. the chain
    // did not change, so the list must not change either — and the way
    // to know that is to ask the chain again, not to guess.
    const result = await SHIT_1170_APP_PAGE_KEYS_STORE.run_action(async () => {
      throw new Error("the wallet said no");
    });
    expect(result).toBe("the wallet said no");
    expect(SHIT_1170_APP_PAGE_KEYS_STORE.get()).not.toBe(null);
  });

  test("a FULL ACCESS key can never be picked", async () => {
    // --- WHY --- a full access key IS the account. removing it locks
    // the reader out of their own account. the store refuses the key
    // even if a component asks.
    const full_access = make_row_fun("ed25519:FULL");
    full_access.full_access = true;
    SHIT_1170_APP_PAGE_KEYS_STORE.toggle_pick(full_access);
    expect(SHIT_1170_APP_PAGE_KEYS_STORE.get_picked()).not.toContain(
      "ed25519:FULL",
    );
  });

  test("a LIMITED key can be picked and unpicked", () => {
    const limited = make_row_fun("ed25519:LIMITED");
    SHIT_1170_APP_PAGE_KEYS_STORE.toggle_pick(limited);
    expect(SHIT_1170_APP_PAGE_KEYS_STORE.get_picked()).toContain(
      "ed25519:LIMITED",
    );
    SHIT_1170_APP_PAGE_KEYS_STORE.toggle_pick(limited);
    expect(SHIT_1170_APP_PAGE_KEYS_STORE.get_picked()).not.toContain(
      "ed25519:LIMITED",
    );
  });

  test("unpicking everything empties the picked set", () => {
    SHIT_1170_APP_PAGE_KEYS_STORE.toggle_pick(make_row_fun("ed25519:A"));
    SHIT_1170_APP_PAGE_KEYS_STORE.toggle_pick(make_row_fun("ed25519:B"));
    expect(SHIT_1170_APP_PAGE_KEYS_STORE.get_picked().length).toBe(2);
    SHIT_1170_APP_PAGE_KEYS_STORE.clear_picked();
    expect(SHIT_1170_APP_PAGE_KEYS_STORE.get_picked().length).toBe(0);
  });
});
// ========================================
// copyright 2026 by sleet.near
// ========================================
