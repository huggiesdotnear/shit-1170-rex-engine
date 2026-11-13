// ========================================
// queue_args_wallet_action.ts
// the wallet's ACTION shape for a queued call.
//
// --- WHY THIS FILE IS ITS OWN ---
// the shape of the arguments decides whether the bytes survive, and the
// reason is long enough that it needs a home of its own. this file makes
// the shape and explains what it has to be for each kind of call.
//
// --- THE TWO KINDS OF CALL ---
// a queued call carries its arguments one of two ways:
//
//   the JSON way — `args` is a json object with named keys.
//   the BYTE way — `argsBase64` is the raw bytes of the call, and the
//                  contract reads them itself.
//
// the byte way is what the FastFS protocol (`__fastdata_fastfs`) needs:
// it wants borsh bytes, and a json object cannot be borsh bytes.
//
// --- WHY THE BYTE WAY NEEDS A DIFFERENT ROUTE ---
// the indexer reads the action's args with `borsh::from_slice`, so the
// bytes have to reach the chain as bytes. a json object is not borsh,
// and a byte array that went through `JSON.stringify` is not borsh
// either — it becomes an object keyed by byte index.
//
// near-connect has exactly ONE way to hand a transaction to a wallet:
//
//   wallet.signAndSendTransactions({ transactions })
//
// and it runs every action through `nearActionsToConnectorActions`,
// which turns an action into the CONNECTOR shape:
//
//   { type: "FunctionCall", params: { methodName, args, gas, deposit } }
//
// that shape is fine for json arguments. for raw bytes it is not, and
// the reason is in the wallet's own Rust:
//
//   impl From<WalletSelectorAction> for NearAction {
//       ... args: serde_json::to_vec(&args).unwrap_or_default() ...
//   }
//
// the arguments are a `serde_json::Value` and they are SERIALIZED AS
// JSON. every json value serializes to text. a `Uint8Array` becomes the
// json text of a byte map and a base64 string becomes the json text of
// a string. neither is the file's bytes, so the indexer finds no borsh
// and DROPS THE ACTION SILENTLY: the receipt fails the way a FastFS
// receipt always fails, the file never appears at its url, and nothing
// anywhere says why.
//
// --- THE ROUTE THAT CARRIES BYTES ---
// the wallet has a second path, its NATIVE action, and it is tried
// FIRST because its action enum is untagged:
//
//   #[serde(untagged)]
//   enum SendTransactionsAction { Native(NearAction), WalletSelector(...) }
//
// the native action holds the arguments as bytes:
//
//   pub struct FunctionCallAction {
//       pub method_name: String,
//       #[serde_as(as = "Base64")]
//       pub args: Vec<u8>,     // base64 in json -> RAW BYTES on chain
//       ...
//   }
//
// so the native path decodes the base64 into the exact bytes, and those
// go on chain verbatim.
//
// the native path is UNREACHABLE through near-connect's own methods,
// because `nearActionsToConnectorActions` refuses an action it does not
// recognize:
//
//   if ("type" in action) return action;      // passthrough
//   if (action.functionCall) { ...convert... }
//   throw new Error("Unsupported action type");
//
// a native action has no `type` key, so it hits the throw. and adding a
// `type` key does not help: the native variant is an EXTERNALLY TAGGED
// enum, so a sibling `type` key makes serde look for a variant named
// `type` and fail with
//
//   unknown variant `type`, expected `FunctionCall`
//
// we compiled the wallet's exact serde types and ran them:
//
//   native, no type key            -> OK Native, raw bytes, first 0x00
//   native WITH type key           -> ERR unknown variant `type`
//   wallet-selector, args a string -> OK, but the args become JSON text
//   wallet-selector, args an array -> OK, but the args become JSON text
//
// --- THE WAY THROUGH ---
// the wallet's `executor` is a PUBLIC field, and the executor posts its
// params straight to the wallet iframe as JSON, with NO conversion:
//
//   async call(method, params) { ... iframe.postMessage({ method, params, id }) ... }
//
// so calling the executor directly skips the conversion, and the native
// action reaches the wallet untouched. `executor.call` with the method
// `wallet:signAndSendTransactions` takes the WHOLE BATCH, so this route
// keeps the one-popup-for-the-batch behaviour the app has.
//
// so: a call with bytes goes out as a native action through the
// executor, and a call with json goes out as the connector shape
// through the wallet's own method.
import type { SHIT_1170_TX_QUEUE_TRANSACTION } from "@huggies/shit-1170-rex-engine-middleman";
import {
  shit_1170_deposit_fun,
  shit_1170_gas_fun,
} from "../shared/gas_deposit";
import { shit_1170_queue_args_bytes_fun } from "../shared/queue_args";
// ========================================
// ========================================
// --- TYPES ------------------------------
// the wallet's transaction shape.
//
// --- WHY THE ACTIONS ARE TYPED LOOSE: the two kinds of call take
// different action shapes, and the shape of an action is the wallet's
// business. what this file guarantees is the one thing that matters: the
// arguments of a binary call are bytes, in the form the wallet reads.
export interface SHIT_1170_WALLET_TRANSACTION {
  receiverId: string;
  actions: Array<Record<string, unknown>>;
}

// the wallet's manifest, as far as this file looks at it.
//
// --- WHY ONLY THE ID: the id is what says which wallet this is, and the
// rest of the manifest is the wallet's business.
export interface SHIT_1170_WALLET_MANIFEST {
  id: string;
}

// the wallet that can carry raw bytes.
//
// --- WHY THIS ONE --- the Intear Wallet's native action holds its
// arguments as `Vec<u8>` with `#[serde_as(as = "Base64")]`, so the
// native path decodes the base64 into the exact bytes. it is the wallet
// we tested an upload with, end to end, on mainnet.
//
// --- WHY THE OTHERS ARE NOT ON THE LIST --- the browser-signer wallets
// — MyNearWallet, OKX, Nightly, HOT Wallet — borsh-serialize the whole
// transaction in the browser, so the bytes survive the connector shape
// too. but their raw-byte route has not been tested here, and a
// FastFS upload that is DROPPED SILENTLY is worse than one that is
// refused out loud. a wallet added to this list needs its own test.
const SHIT_1170_BYTE_WALLET_IDS = ["intear-wallet"];
// ========================================
// ========================================
// --- HELPERS ----------------------------
// the wallet's id, or an empty string when it is not known.
//
// --- WHY --- THE ID IS READ DEFENSIVELY: the manifest is the wallet's
// own object, and a wallet that does not carry one is not a wallet this
// app knows how to ask.
export function shit_1170_wallet_id_fun(manifest: unknown): string {
  if (typeof manifest !== "object" || manifest === null) return "";
  const id = (manifest as SHIT_1170_WALLET_MANIFEST).id;
  return typeof id === "string" ? id : "";
}

// does this call carry raw bytes?
//
// --- WHY --- THE BYTES ARE DECIDED BY THE PAYLOAD AND NOT BY THE
// CONTRACT: a contract can be called either way. what says whether the
// arguments are bytes is which field of the queue holds them.
//
// --- WHY --- A KEY ACTION IS NEVER A BYTE CALL: a key action has no
// arguments at all — it names a public key — and the bytes route turns
// every action into a native FunctionCall. a key action on that route
// would be turned into a call that does not exist.
export function shit_1170_transaction_has_bytes_fun(
  transaction: SHIT_1170_TX_QUEUE_TRANSACTION,
): boolean {
  if (transaction.keyAction !== null && transaction.keyAction !== undefined) {
    return false;
  }
  return (
    transaction.argsBase64 !== null && transaction.argsBase64 !== undefined
  );
}

// does the batch carry any call with raw bytes?
export function shit_1170_batch_has_bytes_fun(
  transactions: ReadonlyArray<SHIT_1170_TX_QUEUE_TRANSACTION>,
): boolean {
  return transactions.some(shit_1170_transaction_has_bytes_fun);
}

// true when the wallet can carry raw bytes to the chain.
//
// --- WHY --- THE APP ASKS: the reader is about to approve a call, and a
// FastFS upload through a wallet that cannot carry bytes is a call that
// goes out and does nothing. the app says so instead of letting the
// reader approve it in the dark.
export function shit_1170_wallet_can_carry_bytes_fun(
  manifest: unknown,
): boolean {
  return SHIT_1170_BYTE_WALLET_IDS.includes(shit_1170_wallet_id_fun(manifest));
}

// the executor the wallet exposes, when it has one.
//
// --- WHY --- THE EXECUTOR IS THE ONLY WAY BYTES CROSS: it posts the
// params to the wallet iframe with no conversion. the wallet's own
// `signAndSendTransaction` runs `nearActionsToConnectorActions` first,
// and that turns the bytes into json text.
//
// --- WHY --- IT IS CHECKED AND NOT ASSUMED: a wallet without an
// executor cannot take the native route, so there is nothing to send it.
export function shit_1170_wallet_executor_fun(
  wallet: unknown,
): { call: (method: string, params: unknown) => Promise<unknown> } | null {
  if (typeof wallet !== "object" || wallet === null) return null;
  const executor = (wallet as { executor?: unknown }).executor;
  if (typeof executor !== "object" || executor === null) return null;
  const call = (executor as { call?: unknown }).call;
  if (typeof call !== "function") return null;
  return executor as {
    call: (method: string, params: unknown) => Promise<unknown>;
  };
}
// ========================================
// ========================================
// --- TRANSACTION -----------------------
// one queued call, in the CONNECTOR shape.
//
// --- WHY --- THIS IS THE SHAPE FOR A CALL WITH JSON ARGUMENTS: this is
// what near-connect builds, and every wallet reads it.
//
// --- WHY --- THE RECEIVER IS THE CONTRACT: the queue holds the contract
// the call is made against, and for a FastFS upload that is the namespace
// the file is served under.
//
// --- WHY --- THE GAS AND THE DEPOSIT ARE THE RAW NUMBERS: the queue
// stores what the agent wrote — "30 Tgas", "1 NEAR" — and the wallet wants
// the raw number. the conversion is the toolbox's rule and it lives in
// shared/gas_deposit.ts.
export function shit_1170_wallet_transaction_fun(
  transaction: SHIT_1170_TX_QUEUE_TRANSACTION,
): SHIT_1170_WALLET_TRANSACTION {
  const gas = shit_1170_gas_fun(transaction.gas);
  const deposit = shit_1170_deposit_fun(transaction.deposit);

  return {
    receiverId: transaction.contractId,
    actions: [
      {
        type: "FunctionCall",
        params: {
          methodName: transaction.method,
          args: transaction.args ?? {},
          gas,
          deposit,
        },
      },
    ],
  };
}

// one queued call, in the NATIVE shape.
//
// --- WHY --- THE FIELD NAMES ARE SNAKE CASE: the native action is the
// wallet's own type, and its fields are `method_name`, `args`, `gas` and
// `deposit`. the connector shape's `methodName` is a different type.
//
// --- WHY --- THERE IS NO `type` KEY: the native variant is an
// externally tagged enum, so a sibling `type` key makes serde look for
// a variant named `type` and fail with
// `unknown variant 'type', expected 'FunctionCall'`.
//
// --- WHY --- THE ARGS ARE BASE64: the native action holds its arguments
// as `Vec<u8>` with `#[serde_as(as = "Base64")]`, so the base64 string
// is DECODED into the exact bytes, and those go on chain verbatim. a
// json string would come out quoted, and those quotes are bytes on chain.
//
// --- WHY --- THE GAS AND THE DEPOSIT ARE STRINGS: the wallet's `Gas` and
// `NearToken` use `dec_format` in serde, so they are decimal strings.
export function shit_1170_wallet_native_transaction_fun(
  transaction: SHIT_1170_TX_QUEUE_TRANSACTION,
): SHIT_1170_WALLET_TRANSACTION {
  const gas = shit_1170_gas_fun(transaction.gas);
  const deposit = shit_1170_deposit_fun(transaction.deposit);

  // --- WHY --- THE BYTES ARE READ FROM THE QUEUE: the queue holds the
  // base64, and the wallet's native action wants base64 too. decoding
  // and re-encoding would be a round trip that can only lose.
  const args_base64 = transaction.argsBase64 ?? "";

  return {
    receiverId: transaction.contractId,
    actions: [
      {
        FunctionCall: {
          method_name: transaction.method,
          args: args_base64,
          gas,
          deposit,
        },
      },
    ],
  };
}
// ========================================
// ========================================
// --- BYTES CROSS THE BOUNDARY ---------
// the bytes of a base64 string, as the wallet takes them.
//
// --- WHY --- THIS EXISTS: a call with json arguments gives the wallet a
// json object, and a call with bytes gives the wallet the base64 string
// it decodes itself. the two are never mixed.
export function shit_1170_wallet_bytes_fun(
  transaction: SHIT_1170_TX_QUEUE_TRANSACTION,
): Uint8Array {
  return shit_1170_queue_args_bytes_fun(transaction.argsBase64 ?? "");
}
// ========================================
// copyright 2026 by sleet.near
