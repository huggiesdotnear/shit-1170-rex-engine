// ========================================
// TX_QUEUE_VALID_FUN
// does one batch of the answer look like a batch the backend wrote?
//
// the queue routes answer with the same shape every time: a queue, its
// transactions in `position` order, and the WHO. this file checks the
// whole of that shape, and it asks
// `tx_queue_transaction_valid_fun` about each transaction.
//
// --- WHY --- THE WHO IS CHECKED AS AN ARRAY AND NOT IN FULL: the
// account row is the WHO of one transaction, and its own shape is the
// backend's business. what matters HERE is that the list is a list, so
// a reader that walks it can walk it.
//
// --- WHY --- THE TRANSACTIONS ARE CHECKED ONE BY ONE: a batch of ten
// calls where one of them is malformed is a batch the page cannot show,
// and the malformed one must be found by name — which transaction it is
// — and not by a boolean that says "something is wrong".
import { tx_queue_transaction_valid_fun } from "./tx_queue_transaction_valid_fun";
import type { SHIT_1170_TX_QUEUE } from "../types/shit_1170_tx_queue";

// is one batch the shape the backend writes?
export function tx_queue_valid_fun(
  value: unknown,
): value is SHIT_1170_TX_QUEUE {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    return false;
  }
  const queue = value as Record<string, unknown>;

  if (
    typeof queue.queueId !== "string" ||
    !Array.isArray(queue.transactions) ||
    !Array.isArray(queue.accounts)
  ) {
    return false;
  }

  return queue.transactions.every((one) => tx_queue_transaction_valid_fun(one));
}
// ========================================
// copyright 2026 by sleet.near
