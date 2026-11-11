// ========================================
// the keys page's rows, tested.
//
// this is the file that decides what the list looks like. two rules
// matter and both of them are about the reader's safety:
//
// 1. the FULL ACCESS key comes first. it is the account itself, and
//    it is the one key that must never be picked for removal — so it
//    has to be where the eye lands.
// 2. only a LIMITED key is pickable, and the page's shape says which
//    one it is, so the component never has to guess.
//
// run it with:
//   bun test frontend/lib/ts/keys/app_page_keys_types.test.ts
// ========================================

import { describe, expect, test } from "bun:test";

import { SHIT_1170_APP_PAGE_KEYS_ROWS } from "./app_page_keys_types";
// ========================================
// --- THE FIXTURES ----------------------
// a limited key: one contract, one method, a small allowance.
const LIMITED_KEY = {
  public_key: "ed25519:limited1",
  access_key: {
    nonce: 5,
    permission: {
      FunctionCall: {
        receiver_id: "contract.near",
        method_names: ["method_one"],
        allowance: "1000",
      },
    },
  },
};
// a full access key: no contract, no methods, nothing to call.
const FULL_ACCESS_KEY = {
  public_key: "ed25519:full1",
  access_key: {
    nonce: 1,
    permission: "FullAccess",
  },
};
// a gas key: a function-call key with a balance and nonce slots on
// top. the page draws it as a limited key.
const GAS_KEY = {
  public_key: "ed25519:gas1",
  access_key: {
    nonce: 3,
    permission: {
      GasKeyFunctionCall: {
        balance: "100",
        num_nonces: 2,
        receiver_id: "other.near",
        method_names: ["method_two"],
        allowance: "5",
      },
    },
  },
};
// ========================================
// --- THE TESTS ------------------------
describe("the keys page's rows", () => {
  test("the full access key comes FIRST", () => {
    const rows = SHIT_1170_APP_PAGE_KEYS_ROWS.rows_from_access_keys({
      keys: [LIMITED_KEY, FULL_ACCESS_KEY, GAS_KEY],
    });
    expect(rows.length).toBe(3);
    expect(rows[0].public_key).toBe("ed25519:full1");
    expect(rows[0].full_access).toBe(true);
    expect(rows[0].receiver_id).toBe(null);
    expect(rows[0].method_names).toEqual([]);
  });

  test("the limited keys keep the chain's order after it", () => {
    const rows = SHIT_1170_APP_PAGE_KEYS_ROWS.rows_from_access_keys({
      keys: [LIMITED_KEY, FULL_ACCESS_KEY, GAS_KEY],
    });
    expect(rows[1].public_key).toBe("ed25519:limited1");
    expect(rows[2].public_key).toBe("ed25519:gas1");
  });

  test("a limited key carries its contract, its methods and its allowance", () => {
    const rows = SHIT_1170_APP_PAGE_KEYS_ROWS.rows_from_access_keys({
      keys: [LIMITED_KEY],
    });
    expect(rows[0].full_access).toBe(false);
    expect(rows[0].receiver_id).toBe("contract.near");
    expect(rows[0].method_names).toEqual(["method_one"]);
    expect(rows[0].allowance).toBe("1000");
    expect(rows[0].nonce).toBe(5);
  });

  test("a gas key is drawn as a limited key", () => {
    const rows = SHIT_1170_APP_PAGE_KEYS_ROWS.rows_from_access_keys({
      keys: [GAS_KEY],
    });
    expect(rows[0].full_access).toBe(false);
    expect(rows[0].receiver_id).toBe("other.near");
    expect(rows[0].method_names).toEqual(["method_two"]);
    expect(rows[0].allowance).toBe("5");
  });

  test("an empty method list means ANY method, and stays empty", () => {
    const rows = SHIT_1170_APP_PAGE_KEYS_ROWS.rows_from_access_keys({
      keys: [
        {
          public_key: "ed25519:any",
          access_key: {
            nonce: 0,
            permission: {
              FunctionCall: {
                receiver_id: "contract.near",
                method_names: [],
                allowance: null,
              },
            },
          },
        },
      ],
    });
    expect(rows[0].method_names).toEqual([]);
    expect(rows[0].allowance).toBe(null);
  });

  test("an account with no keys has no rows", () => {
    const rows = SHIT_1170_APP_PAGE_KEYS_ROWS.rows_from_access_keys({
      keys: [],
    });
    expect(rows.length).toBe(0);
  });
});
// ========================================
// --- THE MERGE ------------------------
describe("the keys page's merge of the chain and the record", () => {
  test("a stored key on the chain gets the chain's facts AND the secret", () => {
    const chain = SHIT_1170_APP_PAGE_KEYS_ROWS.rows_from_access_keys({
      keys: [LIMITED_KEY],
    });
    const merged = SHIT_1170_APP_PAGE_KEYS_ROWS.merge_vault(chain, [
      {
        public_key: "ed25519:limited1",
        secret_key: "ed25519:SECRET",
        receiver_id: "contract.near",
        method_names: ["method_one"],
      },
    ]);
    expect(merged.length).toBe(1);
    expect(merged[0].secret_key).toBe("ed25519:SECRET");
    expect(merged[0].removed).toBe(false);
    // the chain's own facts are kept.
    expect(merged[0].nonce).toBe(5);
    expect(merged[0].allowance).toBe("1000");
  });

  test("a stored key the chain does NOT have stays on the page, marked", () => {
    const chain = SHIT_1170_APP_PAGE_KEYS_ROWS.rows_from_access_keys({
      keys: [FULL_ACCESS_KEY],
    });
    const merged = SHIT_1170_APP_PAGE_KEYS_ROWS.merge_vault(chain, [
      {
        public_key: "ed25519:gone",
        secret_key: "ed25519:SECRET",
        receiver_id: "contract.near",
        method_names: ["method_one"],
      },
    ]);
    expect(merged.length).toBe(2);
    const stored = merged.find((row) => row.public_key === "ed25519:gone");
    expect(stored).toBeDefined();
    expect(stored?.removed).toBe(true);
    expect(stored?.secret_key).toBe("ed25519:SECRET");
    expect(stored?.receiver_id).toBe("contract.near");
    // the full access key is still first.
    expect(merged[0].full_access).toBe(true);
  });

  test("a chain key with no stored secret has NO secret", () => {
    const chain = SHIT_1170_APP_PAGE_KEYS_ROWS.rows_from_access_keys({
      keys: [LIMITED_KEY],
    });
    const merged = SHIT_1170_APP_PAGE_KEYS_ROWS.merge_vault(chain, []);
    expect(merged[0].secret_key).toBe(null);
    expect(merged[0].removed).toBe(false);
  });

  test("the record never says a key is still on the chain", () => {
    const chain = SHIT_1170_APP_PAGE_KEYS_ROWS.rows_from_access_keys({
      keys: [],
    });
    const merged = SHIT_1170_APP_PAGE_KEYS_ROWS.merge_vault(chain, [
      {
        public_key: "ed25519:gone",
        secret_key: null,
        receiver_id: "contract.near",
        method_names: [],
      },
    ]);
    expect(merged.length).toBe(1);
    expect(merged[0].removed).toBe(true);
  });
});
// ========================================
// copyright 2026 by sleet.near
// ========================================
