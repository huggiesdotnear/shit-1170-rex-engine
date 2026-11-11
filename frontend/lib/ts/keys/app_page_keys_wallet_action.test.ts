// ========================================
// the keys page's WALLET ACTION shape, tested.
//
// this is the file that decides whether the reader SEES what they are
// approving. the wallet draws its transaction preview from the action
// it is handed, and near-connect hands an action through untouched when
// it already carries a `type`:
//
//   if ("type" in action) return action;      // passthrough
//
// near-kit's own wallet adapter produces exactly such an action for an
// AddKey, and its permission is the BORSH one:
//
//   { functionCall: { receiverId, methodNames, allowance } }
//
// while the connector's AddKeyAction wants the FLAT one:
//
//   { receiverId, allowance?, methodNames? }
//
// so the wallet is handed a permission with no receiverId and no
// methodNames, cannot say what the key is for, and shows NO PREVIEW.
// the transaction still goes out — silently, unread.
//
// so the shape is built by hand here, and this test pins it.
//
// run it with:
//   bun test frontend/lib/ts/keys/app_page_keys_wallet_action.test.ts
// ========================================

import { describe, expect, test } from "bun:test";

import {
  shit_1170_wallet_add_key_fun,
  shit_1170_wallet_delete_keys_fun,
} from "./app_page_keys_wallet_action";
// ========================================
// --- THE FIXTURES ----------------------
const ACCOUNT = "reader.near";
const PUBLIC_KEY = "ed25519:ABCDEF";
// ========================================
// --- THE TESTS ------------------------
describe("the add key action", () => {
  test("the receiver is the ACCOUNT, because the key goes on the account", () => {
    const transaction = shit_1170_wallet_add_key_fun(
      ACCOUNT,
      PUBLIC_KEY,
      "contract.near",
      ["method_one"],
    );
    expect(transaction.receiverId).toBe(ACCOUNT);
  });

  test("the permission is FLAT — receiverId and methodNames sit on it", () => {
    const transaction = shit_1170_wallet_add_key_fun(
      ACCOUNT,
      PUBLIC_KEY,
      "contract.near",
      ["method_one"],
    );
    const action = transaction.actions[0] as {
      type: string;
      params: {
        publicKey: string;
        accessKey: { nonce: number; permission: Record<string, unknown> };
      };
    };
    expect(action.type).toBe("AddKey");
    expect(action.params.publicKey).toBe(PUBLIC_KEY);
    // --- THE SHAPE THE WALLET READS. a permission nested under
    // `functionCall` is the borsh shape and the wallet cannot read it.
    expect(action.params.accessKey.permission.receiverId).toBe("contract.near");
    expect(action.params.accessKey.permission.methodNames).toEqual([
      "method_one",
    ]);
    // --- AND IT MUST NOT BE THE BORSH SHAPE.
    expect(action.params.accessKey.permission.functionCall).toBe(undefined);
  });

  test("an empty method list goes out as an empty list, meaning ANY method", () => {
    const transaction = shit_1170_wallet_add_key_fun(
      ACCOUNT,
      PUBLIC_KEY,
      "contract.near",
      [],
    );
    const action = transaction.actions[0] as {
      params: { accessKey: { permission: { methodNames: string[] } } };
    };
    expect(action.params.accessKey.permission.methodNames).toEqual([]);
  });

  test("the nonce is zero — a new key has never been used", () => {
    const transaction = shit_1170_wallet_add_key_fun(
      ACCOUNT,
      PUBLIC_KEY,
      "contract.near",
      [],
    );
    const action = transaction.actions[0] as {
      params: { accessKey: { nonce: number } };
    };
    expect(action.params.accessKey.nonce).toBe(0);
  });
});
// ========================================
describe("the delete keys action", () => {
  test("the receiver is the ACCOUNT, because the keys come off it", () => {
    const transaction = shit_1170_wallet_delete_keys_fun(ACCOUNT, [PUBLIC_KEY]);
    expect(transaction.receiverId).toBe(ACCOUNT);
  });

  test("every key is ONE DeleteKey action, in ONE transaction", () => {
    const transaction = shit_1170_wallet_delete_keys_fun(ACCOUNT, [
      "ed25519:ONE",
      "ed25519:TWO",
      "ed25519:THREE",
    ]);
    // --- ONE transaction, so ONE wallet popup for the whole batch.
    expect(transaction.actions.length).toBe(3);
    for (const action of transaction.actions) {
      expect((action as { type: string }).type).toBe("DeleteKey");
    }
    expect(
      (transaction.actions[0] as { params: { publicKey: string } }).params
        .publicKey,
    ).toBe("ed25519:ONE");
    expect(
      (transaction.actions[2] as { params: { publicKey: string } }).params
        .publicKey,
    ).toBe("ed25519:THREE");
  });

  test("a DeleteKey action carries no access key — it names one key", () => {
    const transaction = shit_1170_wallet_delete_keys_fun(ACCOUNT, [PUBLIC_KEY]);
    const action = transaction.actions[0] as {
      params: Record<string, unknown>;
    };
    expect(action.params.accessKey).toBe(undefined);
  });
});
// ========================================
// copyright 2026 by sleet.near
// ========================================
