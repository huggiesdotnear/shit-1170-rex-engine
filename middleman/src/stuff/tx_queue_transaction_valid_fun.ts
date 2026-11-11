// ========================================
// TX_QUEUE_TRANSACTION_VALID_FUN
// does one transaction of the answer look like a transaction the backend
// wrote?
//
// the queue routes answer with the same shape every time: a queue, its
// transactions in `position` order, and the WHO. this file checks the
// transaction half of that shape, so the three routes that answer with
// it — the list, the one-batch, and the post that creates one — cannot
// each grow their own idea of what a transaction looks like.
//
// --- WHY --- THE CHECKS ARE HERE AND NOT ONLY IN THE TYPESCRIPT TYPES:
// the wire is not trusted. a backend that answers with a half-written
// transaction, or a proxy that answers with something else, must not
// reach the page as a queue. a reader that finds a missing `keyAction`
// would have to guess whether the transaction is a contract call or a
// key action, and a guess here moves the wrong thing.
//
// --- WHY --- THE KEY FIELDS ARE ALWAYS PRESENT ON THE WIRE: the backend
// sends `keyAction`, `keyPublicKey`, `keyReceiverId`, `keyMethodNames`
// and `keyAllowance` on EVERY transaction, null when they do not apply.
// a transaction that arrives without them is not one of ours.
//
// --- WHY --- THE KEY ACTION IS CHECKED AGAINST THE TWO WORDS AND NOT
// JUST AGAINST "a string": a `keyAction` that is some other word says
// the backend and the client disagree about what a key action is, and
// the reader would treat it as a contract call and sign the wrong
// thing.
//
// --- WHY --- A NULLABLE STRING IS ACCEPTED AND A STRING IS TOO: the
// fields are null for a contract call and for a `delete_key`, and a
// string when they apply. anything else — a number, an object, a list —
// is not a shape this api produces.
//
// --- WHY --- THE METHOD NAMES ARE CHECKED AS A LIST OF STRINGS: they
// ride as a json array on the wire, and a list that holds something
// that is not a string is not a list of method names.
import type { SHIT_1170_TX_QUEUE_TRANSACTION } from "../types/shit_1170_tx_queue";

// the two words a key action may be.
//
// --- WHY --- THE LIST IS WRITTEN OUT AND NOT READ OFF THE TYPE: a
// union type says the two words to the compiler, and it says nothing to
// the runtime, and this check runs at runtime.
const KEY_ACTIONS: readonly string[] = ["add_key", "delete_key"];

// is this a nullable string? null is the shape of "this does not apply".
function is_nullable_string_fun(value: unknown): boolean {
  return value === null || typeof value === "string";
}

// is this a nullable list of strings?
function is_nullable_string_list_fun(value: unknown): boolean {
  if (value === null) return true;
  if (!Array.isArray(value)) return false;
  return value.every((one) => typeof one === "string");
}

// is this a key action, or nothing at all?
//
// --- WHY --- NULL IS ACCEPTED: a contract call has no key action, and
// the backend says so with a null.
function is_key_action_fun(value: unknown): boolean {
  if (value === null) return true;
  return typeof value === "string" && KEY_ACTIONS.includes(value);
}

// is one transaction the shape the backend writes?
export function tx_queue_transaction_valid_fun(
  value: unknown,
): value is SHIT_1170_TX_QUEUE_TRANSACTION {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    return false;
  }
  const transaction = value as Record<string, unknown>;

  return (
    typeof transaction.txId === "string" &&
    typeof transaction.position === "number" &&
    typeof transaction.contractId === "string" &&
    typeof transaction.method === "string" &&
    typeof transaction.args === "object" &&
    transaction.args !== null &&
    !Array.isArray(transaction.args) &&
    is_nullable_string_fun(transaction.argsBase64) &&
    is_nullable_string_fun(transaction.gas) &&
    is_nullable_string_fun(transaction.deposit) &&
    is_key_action_fun(transaction.keyAction) &&
    is_nullable_string_fun(transaction.keyPublicKey) &&
    is_nullable_string_fun(transaction.keyReceiverId) &&
    is_nullable_string_list_fun(transaction.keyMethodNames) &&
    is_nullable_string_fun(transaction.keyAllowance) &&
    Array.isArray(transaction.accounts)
  );
}
// ========================================
// copyright 2026 by sleet.near
