// ========================================
// the queue page's transactions: every call of the batch, in position
// order, with its raw arguments.
//
// --- WHY THE RAW ARGS ARE SHOWN: a queued call is a function call, and
// the arguments are the whole story of what it does. the transfers
// widget reads them for the tokens; this component shows them as they
// are, so nothing is hidden.
//
// --- WHY THE POSITION IS SHOWN: the batch is a list, and the order is
// part of it — a swap's legs only make sense in order.
//
// --- WHY A CALL WITH RAW BYTES SHOWS A HEX PREVIEW AND NOT JSON: some
// contracts take their arguments as raw bytes — the FastFS protocol
// (`__fastdata_fastfs`) wants borsh bytes — and those bytes are not
// json. printing them as json would show a reader a broken blob that
// says nothing. the byte count and the hex say what the bytes are.
//
// --- WHY THE FASTFS CARD IS NOT HERE: the card is what the bytes MEAN,
// and it now lives in the queue page's FILES section, beside the
// transfers — one row per upload, the same way the transaction page
// shows it. a reader checking the bytes still sees them here, because
// the hex IS the bytes and it stays.
//
// mounts into <div id="SHIT_1170_COMPONENT_QUEUE_TRANSACTIONS">.
import { render } from "preact";
import { SHIT_1170_MOUNT_ID } from "../../ts/shared/mount_ids";
import { SHIT_1170_APP_PAGE_QUEUE_STORE } from "../../ts/queue/app_page_queue_store";
import { use_page_store_fun } from "../../ts/shared/use_page_store";
import { shit_1170_code_block_fun } from "../tx/code_block";
import { shit_1170_queue_args_fun } from "../../ts/shared/queue_args";
import {
  shit_1170_queue_is_key_action_fun,
  shit_1170_queue_key_action_line_fun,
  shit_1170_queue_key_detail_lines_fun,
} from "../../ts/shared/queue_key_action";
import { use_queue_key_details_fun } from "../../ts/shared/use_queue_key_details";
import { SHIT_1170_AUTH } from "../../ts/shared/auth_status";
import type { SHIT_1170_TX_QUEUE_TRANSACTION } from "@huggies/shit-1170-rex-engine-middleman";
// ========================================
// ========================================
// --- ROW -------------------------------
// one transaction: its position, its contract and method, its gas and
// deposit, and its arguments.
//
// --- WHY THE GAS AND THE DEPOSIT ARE SHOWN AS THEY WERE WRITTEN: the
// queue stores what the agent wrote — "30 Tgas", "1 NEAR" — and that is
// the thing the reader can check against the agent's own words.
function transaction_fun(
  transaction: SHIT_1170_TX_QUEUE_TRANSACTION,
): preact.JSX.Element {
  const args = shit_1170_queue_args_fun(transaction);
  // --- WHY --- THIS ROW IS FOR A CALL AND NOT FOR A KEY ACTION: a key
  // action is not a call — it changes the account's own keys — so the
  // contract and the method are both empty for it and the arguments say
  // nothing. a key action gets its own row below.
  return (
    <div class="queue-transaction">
      <div class="queue-transaction-head">
        <span class="queue-transaction-position">
          {String(transaction.position + 1)}
        </span>
        <code class="queue-transaction-contract">
          {transaction.contractId}
        </code>
        <span class="queue-transaction-method">{transaction.method}</span>
      </div>
      {(transaction.gas !== null || transaction.deposit !== null) && (
        <div class="queue-transaction-meta">
          {transaction.gas !== null && (
            <span class="queue-transaction-gas">GAS {transaction.gas}</span>
          )}
          {transaction.deposit !== null && (
            <span class="queue-transaction-deposit">
              DEPOSIT {transaction.deposit}
            </span>
          )}
        </div>
      )}
      {args.binary ? (
        <div class="queue-transaction-binary">
          <span class="queue-transaction-binary-label">
            RAW BYTES — {args.byte_length} BYTE
            {args.byte_length === 1 ? "" : "S"}
          </span>
          <pre class="queue-transaction-binary-hex">
            <code>{args.hex}</code>
          </pre>
        </div>
      ) : (
        shit_1170_code_block_fun({
          code: args.json,
        })
      )}
    </div>
  );
}
// ========================================
// --- KEY ACTION ROW -------------------
// one key action: its headline, and what the key can do.
//
// --- WHY THE KEY ACTION HAS ITS OWN ROW AND IS NOT THE PLAIN ROW: a
// key action is not a call. it changes the account's own keys, and the
// reader has to see the REACH of the key — the contract it calls, the
// methods it may use and the ceiling on what it may spend — before they
// approve it. that reach is not in the arguments, so it has nowhere else
// to go.
//
// --- WHY THE HOOK IS CALLED HERE AND NOT IN THE PLAIN ROW: the hook is
// a hook, and a hook cannot be called conditionally. the plain row does
// not need the answer, so the key action gets a row of its own and the
// hook lives inside it.
//
// --- WHY THE READER'S ACCOUNT IS THE ACCOUNT ASKED: a remove names a
// key, and the key's reach belongs to the account it sits on. the
// reader's own account is the one they are signing as, and the one the
// key action acts on.
function key_action_row_fun(
  transaction: SHIT_1170_TX_QUEUE_TRANSACTION,
): preact.JSX.Element {
  const account_id = SHIT_1170_AUTH.get_account_id();
  const details = use_queue_key_details_fun(transaction, account_id);
  const lines = shit_1170_queue_key_detail_lines_fun(transaction, details);
  return (
    <div class="queue-transaction is-key-action">
      <div class="queue-transaction-head">
        <span class="queue-transaction-position">
          {String(transaction.position + 1)}
        </span>
        <span class="queue-transaction-key">
          {shit_1170_queue_key_action_line_fun(transaction)}
        </span>
      </div>
      {lines.length > 0 && (
        <div class="queue-transaction-key-detail">
          {lines.map((line) => (
            <p class="queue-transaction-key-detail-line" key={line}>
              {line}
            </p>
          ))}
        </div>
      )}
    </div>
  );
}
// ========================================
// ========================================
// --- COMPONENT --------------------------
export function SHIT_1170_COMPONENT_QUEUE_TRANSACTIONS_FUN() {
  const state = use_page_store_fun(SHIT_1170_APP_PAGE_QUEUE_STORE);
  const queue = state.queue;
  if (queue === null) return null;
  const transactions = queue.transactions;
  if (transactions.length === 0) return null;
  return (
    <section class="queue-transactions">
      <h2>CALLS ({transactions.length})</h2>
      <div class="queue-transactions-list">
        {transactions.map((transaction) => (
          <div key={transaction.txId}>
            {shit_1170_queue_is_key_action_fun(transaction) ? (
              key_action_row_fun(transaction)
            ) : (
              transaction_fun(transaction)
            )}
          </div>
        ))}
      </div>
    </section>
  );
}
// ========================================
// --- MOUNT ------------------------------
// self-mount guarded by id check.
const SELF_MOUNT_ID = SHIT_1170_MOUNT_ID.SHIT_1170_COMPONENT_QUEUE_TRANSACTIONS;
if (typeof document !== "undefined") {
  const el = document.getElementById(SELF_MOUNT_ID);
  if (el !== null) render(<SHIT_1170_COMPONENT_QUEUE_TRANSACTIONS_FUN />, el);
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
