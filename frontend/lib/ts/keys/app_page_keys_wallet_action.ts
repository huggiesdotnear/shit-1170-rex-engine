// ========================================
// the wallet's ACTION shape for an access key action.
//
// --- WHY THIS FILE EXISTS AND WHY IT DOES NOT USE NEAR KIT ---
// near-kit's wallet adapter converts its own actions to the connector
// shape in `convertActionToNearConnect`. for an AddKey it hands the
// permission straight through:
//
//   accessKey: { nonce, permission: ak.accessKey.permission }
//
// and that `permission` is the BORSH permission —
// `{ functionCall: { receiverId, methodNames, allowance } }` — while the
// connector's `AddKeyAction` wants the FLAT one:
//
//   permission: "FullAccess" | { receiverId, allowance?, methodNames? }
//
// near-connect does not fix it up, because its own converter passes any
// action that already carries a `type` straight through:
//
//   if ("type" in action) return action;      // passthrough
//
// so the wallet is handed a permission with no `receiverId` and no
// `methodNames` on it. the wallet cannot say what the key is for, so
// it shows NO TRANSACTION PREVIEW — the reader is asked to approve a
// transaction they cannot read.
//
// the same shape is built here by hand, in the connector's own shape,
// and handed to the wallet's own method. this is the same route the
// feed's sends take, and the same reason is written down in
// shared/queue_args_wallet_action.ts.
//
// --- WHY THE ACTIONS GO OUT AS A BATCH: the wallet's
// `signAndSendTransactions` (plural) takes the whole batch, so several
// DeleteKey actions are ONE popup instead of one popup each.
//
// --- WHY THE RECEIVER IS THE ACCOUNT: an AddKey and a DeleteKey act
// on the account that signs them, and the receiver of the transaction
// is that account.
import type { SHIT_1170_WALLET_TRANSACTION } from "../shared/queue_args_wallet_action";
// ========================================
// ========================================
// --- TYPES ------------------------------
// the wallet's permission for a limited key.
//
// --- WHY THE FIELDS ARE FLAT: this is the connector's own shape for a
// function-call permission, and it is what the wallet reads to draw the
// preview. the borsh shape — `{ functionCall: { ... } }` — is what
// goes on chain, and the wallet does that conversion itself.
export interface SHIT_1170_WALLET_KEY_PERMISSION {
  receiverId: string;
  methodNames: string[];
}
// ========================================
// ========================================
// --- ADD -------------------------------
// the transaction that adds one limited key.
//
// --- WHY THE METHOD NAMES GO OUT AS A LIST AND NOT NULL: an empty
// list means "every method on this account" on the chain, and the
// connector's shape says the same with an empty list. the wallet draws
// the same thing either way.
//
// --- WHY THE ALLOWANCE GOES OUT AS THE RAW STRING: it is a limit on
// money and the chain stores it as yoctoNEAR. the connector's shape
// carries it as a string, so the exact figure the reader chose is what
// reaches the wallet — not a rounded one.
//
// --- WHY A NULL ALLOWANCE IS LEFT OUT ENTIRELY: the connector's
// shape has the allowance as optional, and "no allowance" is the
// absence of the field. sending an empty string would be a different
// thing — a limit of zero — and a key with a limit of zero can still
// be called, while a key with no allowance can attach nothing.
export function shit_1170_wallet_add_key_fun(
  account_id: string,
  public_key: string,
  receiver_id: string,
  method_names: string[],
  allowance_yocto: string | null,
): SHIT_1170_WALLET_TRANSACTION {
  const permission: {
    receiverId: string;
    methodNames: string[];
    allowance?: string;
  } = {
    receiverId: receiver_id,
    methodNames: method_names,
  };
  if (allowance_yocto !== null) {
    permission.allowance = allowance_yocto;
  }
  return {
    receiverId: account_id,
    actions: [
      {
        type: "AddKey",
        params: {
          publicKey: public_key,
          accessKey: {
            nonce: 0,
            permission,
          },
        },
      },
    ],
  };
}
// ========================================
// ========================================
// --- DELETE ----------------------------
// the transaction that removes several keys.
//
// --- WHY EVERY KEY IS ONE ACTION IN THE SAME TRANSACTION: a
// DeleteKey action names one key, so several keys are several actions.
// they go in ONE transaction so the reader approves the whole removal
// once, in one popup.
export function shit_1170_wallet_delete_keys_fun(
  account_id: string,
  public_keys: ReadonlyArray<string>,
): SHIT_1170_WALLET_TRANSACTION {
  return {
    receiverId: account_id,
    actions: public_keys.map((public_key) => ({
      type: "DeleteKey",
      params: {
        publicKey: public_key,
      },
    })),
  };
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
