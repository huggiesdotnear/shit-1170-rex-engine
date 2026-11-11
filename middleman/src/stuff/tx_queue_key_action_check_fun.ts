// ========================================
// TX_QUEUE_KEY_ACTION_CHECK_FUN
// is a queued transaction's key action shaped the way the backend wants
// it?
//
// a queued transaction is EITHER a contract call OR a key action —
// `add_key` or `delete_key`. the backend refuses the mistakes with
// named codes, and this file refuses the same ones before the request
// goes out, so the caller gets a clear message instead of a 400 from
// the wire.
//
// --- WHY --- THE CHECKS ARE HERE AND NOT ONLY ON THE SERVER: the codes
// are already written down, and a caller who gets `no_key_public_key`
// from the backend has learned the same thing this file would have
// told them a moment earlier. the round trip is the slow part.
//
// --- WHY --- THE BACKEND'S CODES ARE THE ONES USED HERE: the error
// message of this file carries the backend's code word, so a message
// read here and a message read there say the same thing.
//
// --- WHY --- A MISSING OR EMPTY `keyAction` IS NOT A KEY ACTION: the
// field is optional, and a transaction that leaves it out is an
// ordinary contract call. an empty string says nothing, and nothing is
// a contract call.
//
// --- WHY --- THE KEY ACTION AND THE CONTRACT CALL ARE REFUSED TOGETHER:
// a key action has no contract to call, so a contract id on it is a
// mistake in the request, and the backend refuses it with
// `key_action_and_contract_call`. a transaction that carries both does
// not say what the user is approving.
//
// --- WHY --- AN `add_key` NEEDS A RECEIVER: a limited key with no
// receiver can call nothing, and a key with no receiver is not a
// limited key. the receiver is what makes it limited, so the backend
// refuses it with `no_key_receiver_id`.
//
// --- WHY --- A `delete_key` NEEDS NO RECEIVER AND NO METHODS: removing
// a key does not name what the key may call — the key already knows
// that, and the chain is the one that holds it.
//
// --- WHY --- THE ARGUMENTS OF A KEY ACTION ARE NOT CHECKED HERE: the
// backend drops them for a `delete_key`, and an `add_key` that carries
// them says something harmless and wrong. the backend is the one that
// decides, and it drops them.
import type {
  SHIT_1170_TX_QUEUE_KEY_ACTION,
  SHIT_1170_TX_QUEUE_TRANSACTION_REQUEST,
} from "../types/shit_1170_tx_queue";

// the two words a key action may be.
//
// --- WHY --- THE LIST IS WRITTEN OUT AND NOT READ OFF THE TYPE: a
// union type says the two words to the compiler, and it says nothing to
// the runtime, and this check runs at runtime.
const KEY_ACTIONS: readonly string[] = ["add_key", "delete_key"];

// is a transaction that claims to be a key action shaped like one?
//
// --- WHY --- THE POSITION IS IN EVERY MESSAGE: a batch holds up to
// fifty transactions, and a message that does not say WHICH one is
// wrong is a message the caller cannot act on.
export function tx_queue_key_action_check_fun(
  transaction: SHIT_1170_TX_QUEUE_TRANSACTION_REQUEST,
  position: number,
): void {
  const raw = transaction.keyAction?.trim();
  if (raw === undefined || raw.length === 0) return;

  if (!KEY_ACTIONS.includes(raw)) {
    throw new Error(
      `invalid_request: bad_key_action: the keyAction must be add_key or delete_key, and it is ${raw}`,
    );
  }
  const action = raw as SHIT_1170_TX_QUEUE_KEY_ACTION;

  // --- THE BOTH RULE -------------------
  if (
    transaction.contractId?.trim().length !== 0 ||
    transaction.method?.trim().length !== 0
  ) {
    throw new Error(
      `invalid_request: key_action_and_contract_call: the transaction at position ${position} is a key action and a contract call at once — it must be one or the other`,
    );
  }

  // --- THE PUBLIC KEY -----------------
  if (!transaction.keyPublicKey?.trim()) {
    throw new Error(
      `invalid_request: no_key_public_key: the key action at position ${position} must name the public key it acts on`,
    );
  }

  // --- THE RECEIVER -------------------
  // --- WHY --- ONLY AN `add_key` NEEDS IT: a `delete_key` removes a key
  // that is already on chain, and the chain is the one that knows what
  // it may call.
  if (action === "add_key" && !transaction.keyReceiverId?.trim()) {
    throw new Error(
      `invalid_request: no_key_receiver_id: the add_key at position ${position} must name the receiver id the key may call`,
    );
  }
}
// ========================================
// copyright 2026 by sleet.near
