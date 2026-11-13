// ========================================
// a queued KEY ACTION: is this transaction one, and what wallet
// transaction does it become.
//
// --- WHY THIS FILE EXISTS: the queue carries TWO kinds of transaction —
// a contract call and a key action — and they are different things the
// reader signs. a call is a FunctionCall on a contract. a key action is
// an AddKey or a DeleteKey on the ACCOUNT ITSELF, and it has no
// arguments at all. so the send cannot map every transaction with one
// function-call mapper, and the two shapes have to be told apart before
// either is built.
//
// --- WHY IT IS SHARED: the feed sends a key action and draws one, and
// the queue page draws one too. the rule about what a key action IS
// must not be written twice.
//
// --- WHY THE WALLET SHAPE IS BUILT BY HAND: near-kit hands the wallet
// the BORSH permission (`{ functionCall: {...} }`) while the connector
// wants the FLAT one (`{ receiverId, methodNames }`), and near-connect
// passes through any action that already carries a `type`. so the
// wallet shows NO TRANSACTION PREVIEW for a near-kit AddKey. the whole
// story is in keys/app_page_keys_wallet_action.ts, and the hand-built
// builders live there. this file only says WHICH builder a queued key
// action uses and what it needs.
//
// --- WHY THE ACCOUNT THAT SIGNS IS THE TARGET: an AddKey and a
// DeleteKey act on the account that signs them. the reader's own
// account is the one the feed signs as, and it is the account the queue
// is for — the transaction's own `accounts` rows carry the WHO.
import type { SHIT_1170_TX_QUEUE_TRANSACTION } from "@huggies/shit-1170-rex-engine-middleman";
import {
  shit_1170_wallet_add_key_fun,
  shit_1170_wallet_delete_keys_fun,
} from "../keys/app_page_keys_wallet_action";
import type { SHIT_1170_WALLET_TRANSACTION } from "./queue_args_wallet_action";
import type { SHIT_1170_QUEUE_KEY_DETAILS_ROW } from "./queue_key_details";
// ========================================
// ========================================
// --- DETECT ----------------------------
// is this transaction a key action.
//
// --- WHY THE FIELD IS READ AND NOT GUESSED FROM THE METHOD: a key
// action has no contract and no method, and the queue says which kind
// it is in one field. the method of a key action is empty, so a check
// on the method would be a guess about the wire.
export function shit_1170_queue_is_key_action_fun(
  transaction: SHIT_1170_TX_QUEUE_TRANSACTION,
): boolean {
  return transaction.keyAction !== null && transaction.keyAction !== undefined;
}
// ========================================
// ========================================
// --- WALLET ----------------------------
// the wallet transaction a queued key action becomes.
//
// --- WHY THE ACCOUNT COMES IN AND IS NOT TAKEN FROM THE TRANSACTION:
// the queue's transaction does not carry the account — the account is
// in the transaction's own `accounts` rows — and the wallet signs as
// ONE account. the reader's own account is the one the feed signs as,
// and a key action acts on the account that signs it. so the account
// that signs IS the account whose keys change.
//
// --- WHY A MISSING PUBLIC KEY IS AN EMPTY ACTION AND NOT A THROW: the
// backend refuses a key action without a public key before it is
// written, so a key action on the wire always has one. a defensive
// empty action keeps a broken answer from taking the whole batch down.
export function shit_1170_queue_key_action_wallet_fun(
  transaction: SHIT_1170_TX_QUEUE_TRANSACTION,
  account_id: string,
): SHIT_1170_WALLET_TRANSACTION {
  const public_key = transaction.keyPublicKey ?? "";
  if (transaction.keyAction === "delete_key") {
    return shit_1170_wallet_delete_keys_fun(account_id, [public_key]);
  }
  return shit_1170_wallet_add_key_fun(
    account_id,
    public_key,
    transaction.keyReceiverId ?? "",
    transaction.keyMethodNames ?? [],
    transaction.keyAllowance ?? null,
  );
}
// ========================================
// --- LINE ------------------------------
// the one line the app draws for a key action.
//
// --- WHY THE LINE IS BUILT HERE AND NOT IN A COMPONENT: the feed card
// draws it, the queue page draws it, and the tests pin it. a rule about
// what a key action says belongs with the rule about what a key action
// IS.
//
// --- WHY A REMOVE NAMES THE KEY AND AN ADD NAMES THE CONTRACT: a
// remove is about one key, and the reader has to know WHICH key goes.
// an add is about a new key's reach, and the reader has to know WHAT
// it may call. the two say different things for a reason.
//
// --- WHY THE LINE DOES NOT CARRY THE ALLOWANCE: the line is a
// HEADLINE — it names the action and nothing else, so it fits on one
// line and stays comparable between a card and a row. the reach of the
// key is a detail, and the details have their own block beside it.
export function shit_1170_queue_key_action_line_fun(
  transaction: SHIT_1170_TX_QUEUE_TRANSACTION,
): string {
  const public_key = transaction.keyPublicKey ?? "";
  if (transaction.keyAction === "delete_key") {
    return `REMOVE LIMITED KEY ${public_key}`;
  }
  const receiver_id = transaction.keyReceiverId ?? "";
  const method_names = transaction.keyMethodNames ?? [];
  const methods =
    method_names.length === 0 ? "ANY METHOD" : method_names.join(", ");
  return `ADD LIMITED KEY -> ${receiver_id} (${methods})`;
}
// ========================================
// --- DETAIL LINES ---------------------
// the lines that describe what a key action DOES to the reader's keys.
//
// --- WHY THESE ARE LINES AND NOT A COMPONENT: the feed card and the
// queue page both draw them, and a rule about what they say belongs
// with the rule about what a key action IS.
//
// --- WHY AN ADD SHOWS ITS ALLOWANCE AND A REMOVE SHOWS THE KEY'S REAL
// REACH: the two actions have different sources of truth. an add's
// reach is what the agent asked for and it is IN the queue, so it is
// known before anything is signed. a remove's reach belongs to a key
// that is already on the account, and the queue deliberately does not
// carry it — the chain is the one that knows.
//
// --- WHY NOTHING IS SAID TWICE: the headline and the detail block are
// drawn one under the other, so a line that says the same thing twice
// reads as a mistake. an add's headline already names the contract and
// the methods, so the detail block says only what the headline did not
// — the allowance. a remove's headline is just the public key, so
// there is nothing to repeat and the detail block says all of it.
//
// the rule is: a detail line is drawn only when the headline did not
// already say it.
export function shit_1170_queue_key_detail_lines_fun(
  transaction: SHIT_1170_TX_QUEUE_TRANSACTION,
  details: SHIT_1170_QUEUE_KEY_DETAILS_ROW | null,
): string[] {
  const lines: string[] = [];
  if (details === null) return lines;
  // --- WHY THE HEADLINE IS READ FIRST: it is what the reader has
  // already read, and a detail that repeats it is noise.
  const headline = shit_1170_queue_key_action_line_fun(transaction);
  // --- WHY AN UNREAD KEY SAYS ONLY THAT IT COULD NOT BE READ: a key
  // whose reach the chain did not answer for is a key about which
  // NOTHING is known. saying "ANY METHOD" would be a guess that reads
  // like a fact, and a reader who approved it on that would be
  // approving something nobody read.
  if (!details.resolved) {
    lines.push("THE KEY'S REACH COULD NOT BE READ");
    return lines;
  }
  // --- WHY THE RECEIVER IS SHOWN WHEN IT IS KNOWN AND NOT OTHERWISE:
  // a key with no contract has nothing to call, and saying so is better
  // than showing an empty line.
  if (
    details.receiver_id !== null &&
    details.receiver_id !== "" &&
    !headline.includes(details.receiver_id)
  ) {
    lines.push(`CONTRACT: ${details.receiver_id}`);
  }
  const methods =
    details.method_names.length === 0
      ? "ANY METHOD"
      : details.method_names.join(", ");
  // --- WHY THE METHODS ARE SHOWN ONLY WHEN THE HEADLINE DID NOT SAY
  // THEM: an add's headline already carries the methods in brackets.
  // the check is on the METHODS themselves and not on the whole line,
  // because a headline that names one method of ten still has nine the
  // reader has not seen.
  if (!headline.includes(methods)) {
    lines.push(`METHODS: ${methods}`);
  }
  // --- WHY A MISSING ALLOWANCE SAYS "NO ALLOWANCE" AND NOT "0 NEAR":
  // they are different things. a key with NO allowance cannot attach a
  // deposit at all. a key with an allowance of ZERO can attach a
  // deposit of zero, and can still be called.
  lines.push(
    details.allowance_human === null
      ? "ALLOWANCE: NO ALLOWANCE"
      : `ALLOWANCE: ${details.allowance_human}`,
  );
  return lines;
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
