// ========================================
// the hook that shows what a queued key action DOES to the reader's keys.
//
// --- WHY A HOOK AND NOT A PLAIN FUNCTION: a remove's reach comes from
// the chain, and the chain answers over the network. the row has to be
// drawn BEFORE that answer arrives, and then drawn AGAIN when it does.
// so the answer is held in the component's own state, and the component
// asks for a fresh render when it changes.
//
// --- WHY THE ANSWER IS EMPTY AND NOT null WHILE IT LOADS: null says
// "this is not a key action". an empty answer says "this is a key
// action and the answer is not here yet", and the row can draw the
// headline while the details arrive.
import { useState as use_state, useEffect as use_effect } from "preact/hooks";
import { SHIT_1170_QUEUE_KEY_DETAILS } from "./queue_key_details";
import type { SHIT_1170_QUEUE_KEY_DETAILS_ROW } from "./queue_key_details";
import type { SHIT_1170_TX_QUEUE_TRANSACTION } from "@huggies/shit-1170-rex-engine-middleman";
// ========================================
// --- HOOK ------------------------------
// the details of one key action, as far as they are known.
//
// --- WHY THE EFFECT DEPENDS ON THE KEY AND NOT ON THE TRANSACTION: the
// same key action is re-rendered many times as the feed changes around
// it, and a new object every render would ask the chain again and
// again. the key and the account are what decide the answer.
export function use_queue_key_details_fun(
  transaction: SHIT_1170_TX_QUEUE_TRANSACTION,
  account_id: string | null,
): SHIT_1170_QUEUE_KEY_DETAILS_ROW | null {
  const [details, set_details] =
    use_state<SHIT_1170_QUEUE_KEY_DETAILS_ROW | null>(null);
  const public_key = transaction.keyPublicKey ?? "";
  const key_action = transaction.keyAction ?? "";

  use_effect(() => {
    // --- WHY A NON KEY ACTION IS LEFT ALONE: there is nothing to read
    // for it, and asking the chain about a contract call would be a
    // question with no answer.
    if (key_action === "") {
      set_details(null);
      return;
    }
    let alive = true;
    void SHIT_1170_QUEUE_KEY_DETAILS.read(transaction, account_id).then(
      (answer) => {
        // --- WHY THE LIVENESS CHECK: the component may be gone by the
        // time the chain answers, and setting state on a component that
        // is no longer there is a warning at best.
        if (!alive) return;
        set_details(answer);
      },
    );
    return () => {
      alive = false;
    };
  }, [key_action, public_key, account_id]);

  return details;
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
