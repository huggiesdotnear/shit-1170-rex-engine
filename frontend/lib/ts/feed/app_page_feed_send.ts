// ========================================
// send a queued batch: the wallet popup, the outcomes, the report to the
// backend and the toast.
//
// --- WHY THE WALLET IS CALLED DIRECTLY AND NOT THROUGH THE TOOLBOX'S
// HELPER: `generic_contract_call_wallet` returns only the hashes and
// throws the outcome away. the outcome is the thing the queue records —
// "succeeded" or "failed" — so the app asks the wallet for the
// outcomes itself and reads both.
//
// --- WHY ONE POPUP FOR THE WHOLE BATCH: the batch is one offer of
// work. a reader who signs it signs all of it, and the wallet is
// asked for the whole list at once.
//
// --- WHY THE ACCOUNT COMES FROM THE API KEY: the request carries the
// reader's key and the backend knows the account. the body does not
// repeat it, because a body that named the account could name somebody
// else's.
//
// --- WHY THE BYTES ARE DECODED HERE AND NOT AT THE BACKEND: the
// wallet takes the BYTES. a contract like the FastFS protocol
// (`__fastdata_fastfs`) wants borsh bytes and not json, and the wallet
// that can carry them decodes the base64 into the exact bytes itself.
// so the base64 the queue holds is handed to the wallet as it is, and
// the wallet's native action does the decoding.
//
// --- WHY A CALL WITH BYTES GOES OUT A DIFFERENT WAY: near-connect
// converts every action into the connector shape, and the wallet's Rust
// writes those arguments out as json text, which the FastFS indexer
// cannot read. a call with bytes goes out through the wallet's
// `executor` instead, which posts the params straight to the wallet
// with no conversion. the shape of that route lives in
// shared/queue_args_wallet_action.ts.
//
// --- WHY A KEY ACTION IS ITS OWN THING HERE: the queue carries two
// kinds of transaction, and a key action is an AddKey or a DeleteKey on
// the ACCOUNT ITSELF — not a call on a contract. it is built by hand
// from the same builders the keys page uses, because near-kit's own
// AddKey hands the wallet the BORSH permission and the wallet shows NO
// PREVIEW for it. the shape lives in shared/queue_key_action.ts.
import { near_connect_client } from "@near-kit-tool-box/web";
import {
  api_shit_1170_tx_queue_sent_post,
  type SHIT_1170_TX_QUEUE,
  type SHIT_1170_TX_QUEUE_TRANSACTION,
} from "@huggies/shit-1170-rex-engine-middleman";
import { SHIT_1170_BASE_URL } from "../shared/base_url";
import { api_key_options_fun } from "../shared/api_key_token";
import { shit_1170_outcome_hash_fun } from "../shared/outcome_hash";
import {
  shit_1170_batch_has_bytes_fun,
  shit_1170_wallet_can_carry_bytes_fun,
  shit_1170_wallet_executor_fun,
  shit_1170_wallet_native_transaction_fun,
  shit_1170_wallet_transaction_fun,
} from "../shared/queue_args_wallet_action";
import {
  shit_1170_queue_is_key_action_fun,
  shit_1170_queue_key_action_wallet_fun,
} from "../shared/queue_key_action";
import { SHIT_1170_TOAST_STORE } from "../shared/toast_store";
// ========================================
// ========================================
// --- TYPES ------------------------------
// what came of the send.
export interface SHIT_1170_QUEUE_SEND_RESULT {
  // true when the batch went out.
  sent: boolean;
  // how many of the transactions worked.
  succeeded: number;
  // how many of the transactions failed.
  failed: number;
  // the hash of the first transaction that was sent, for the link to
  // the transaction page. null when nothing went out.
  tx_hash: string | null;
  // the message for the reader, when something failed.
  error: string | null;
  // true when the reader DECLINED the wallet popup.
  //
  // --- WHY --- a decline is its own thing: the reader said no. that
  // is not a failure, and a toast that said "it failed" would be
  // telling the reader they did something wrong.
  declined: boolean;
}

// ========================================
// ========================================
// --- HELPERS ----------------------------
// the wallet's transaction shapes, and the reason a call with bytes
// goes out a different way, live in
// shared/queue_args_wallet_action.ts.
// ========================================
// ========================================
// --- DECLINED --------------------------
// did the reader decline the wallet popup.
//
// --- WHY --- the wallet's decline message is matched loosely: every
// wallet says it in its own words, and the app only needs to know that
// the reader said no rather than that the send broke.
function is_declined_fun(err: unknown): boolean {
  if (typeof err !== "object" || err === null) return false;
  const message = (err as { message?: unknown }).message;
  if (typeof message !== "string") return false;
  const lower = message.toLowerCase();
  return (
    lower.includes("declined") ||
    lower.includes("denied") ||
    lower.includes("rejected") ||
    lower.includes("cancelled") ||
    lower.includes("canceled")
  );
}
// ========================================
// ========================================
// --- SEND ------------------------------
// send the batch and report what came of it.
export async function shit_1170_queue_send_fun(
  queue: SHIT_1170_TX_QUEUE,
  account_id: string,
): Promise<SHIT_1170_QUEUE_SEND_RESULT> {
  const transactions = queue.transactions;
  if (transactions.length === 0) {
    return {
      sent: false,
      succeeded: 0,
      failed: 0,
      tx_hash: null,
      error: "THIS BATCH HAS NO CALLS TO SEND",
      declined: false,
    };
  }

  let outcomes: ReadonlyArray<unknown>;
  try {
    const connector = near_connect_client();
    const wallet = await connector.wallet();
    // --- WHY --- THE WALLET'S MANIFEST IS READ: the wallet reads the
    // arguments in its own way, and a call that carries raw bytes needs
    // the wallet that can carry them. the manifest says which wallet this
    // is, so the app can tell the reader before they approve a call that
    // would go out and do nothing. the reason is written down in
    // shared/queue_args_wallet_action.ts.
    const manifest = (wallet as { manifest?: unknown }).manifest;

    // --- WHY --- A CALL WITH BYTES IS REFUSED FOR A WALLET THAT CANNOT
    // CARRY THEM: a FastFS upload through such a wallet is DROPPED
    // SILENTLY by the indexer. the receipt fails the way a FastFS receipt
    // always fails, the file never appears at its url, and nothing says
    // why. refusing here tells the reader the truth instead of sending a
    // call that cannot work.
    // --- WHY --- THE TWO KINDS ARE ASKED FOR APART: a key action is an
    // AddKey or a DeleteKey — not a call — and the bytes route turns
    // every action into a NATIVE FunctionCall. a key action on that
    // route would go out as a call to nothing. so the app knows which
    // kinds are in the batch before it picks the route.
    const has_key_action = transactions.some(shit_1170_queue_is_key_action_fun);
    const has_bytes_call = shit_1170_batch_has_bytes_fun(transactions);
    // --- WHY --- A BATCH WITH BOTH IS REFUSED AND NOT SENT: the bytes
    // route turns every action into a native call, and the wallet's own
    // route writes the bytes out as json text — which the indexer drops
    // SILENTLY. so one route loses the file and the other loses the key
    // action. a batch that cannot go out whole is not sent at all.
    if (has_bytes_call && has_key_action) {
      const message =
        "THIS BATCH HAS A FILE AND A KEY ACTION IN IT, AND THE WALLET CANNOT " +
        "SEND BOTH AT ONCE. THE FILE WOULD BE DROPPED SILENTLY, SO NOTHING " +
        "WAS SENT. APPROVE THE FILE AND THE KEY SEPARATELY.";
      SHIT_1170_TOAST_STORE.show(
        "error",
        "THIS BATCH CANNOT BE SENT",
        message,
        null,
      );
      return {
        sent: false,
        succeeded: 0,
        failed: 0,
        tx_hash: null,
        error: message,
        declined: false,
      };
    }
    if (has_bytes_call && !shit_1170_wallet_can_carry_bytes_fun(manifest)) {
      const message =
        "THIS WALLET CANNOT SEND RAW BYTES, AND THIS CALL CARRIES A FILE. " +
        "THE UPLOAD WOULD BE DROPPED SILENTLY AND THE FILE WOULD NEVER APPEAR. " +
        "SIGN IN WITH THE INTEAR WALLET TO UPLOAD A FILE.";
      SHIT_1170_TOAST_STORE.show(
        "error",
        "THIS WALLET CANNOT DO THIS",
        message,
        null,
      );
      return {
        sent: false,
        succeeded: 0,
        failed: 0,
        tx_hash: null,
        error: message,
        declined: false,
      };
    }

    // --- WHY --- A CALL WITH BYTES GOES OUT A DIFFERENT WAY: near-connect
    // converts every action into the connector shape, and the wallet's
    // Rust writes those arguments out with `serde_json::to_vec`, which
    // turns the bytes into json text and the indexer drops the action.
    // the wallet's `executor` posts the params straight to the wallet
    // iframe with no conversion, so the native action reaches the wallet
    // untouched and the bytes land on chain. the executor takes the whole
    // batch, so this is still one popup for the batch.
    if (has_bytes_call) {
      const executor = shit_1170_wallet_executor_fun(wallet);
      if (executor === null) {
        const message =
          "THIS WALLET HAS NO WAY TO SEND RAW BYTES. " +
          "THE UPLOAD WOULD BE DROPPED SILENTLY AND THE FILE WOULD NEVER APPEAR.";
        SHIT_1170_TOAST_STORE.show(
          "error",
          "THIS WALLET CANNOT DO THIS",
          message,
          null,
        );
        return {
          sent: false,
          succeeded: 0,
          failed: 0,
          tx_hash: null,
          error: message,
          declined: false,
        };
      }
      outcomes = (await executor.call("wallet:signAndSendTransactions", {
        signerId: account_id,
        network: connector.network,
        transactions: transactions.map((transaction) => ({
          ...shit_1170_wallet_native_transaction_fun(transaction),
          signerId: account_id,
        })),
      })) as ReadonlyArray<unknown>;
    } else {
      // --- WHY --- EVERY TRANSACTION BECOMES ONE WALLET TRANSACTION: the
      // outcomes are matched back by position, so one queued transaction
      // must produce exactly one wallet transaction and exactly one
      // outcome. a batch that mixes calls and key actions keeps that
      // one-for-one rule, and the order of the list is the order of the
      // positions.
      //
      // --- WHY --- THE KEY ACTION IS HAND-BUILT: near-kit hands the
      // wallet the BORSH permission and the wallet shows NO PREVIEW for
      // it. the hand-built action is the connector's own shape, and it
      // says what the key is for.
      outcomes = await wallet.signAndSendTransactions({
        signerId: account_id,
        transactions: transactions.map((transaction) => ({
          ...(shit_1170_queue_is_key_action_fun(transaction)
            ? shit_1170_queue_key_action_wallet_fun(transaction, account_id)
            : shit_1170_wallet_transaction_fun(transaction)),
          signerId: account_id,
        })),
      });
    }
  } catch (err) {
    // --- WHY --- A DECLINE IS NOT AN ERROR: the reader said no, and
    // that is not a failure. it is reported as a decline and the reader
    // is not toasted.
    if (is_declined_fun(err)) {
      return {
        sent: false,
        succeeded: 0,
        failed: 0,
        tx_hash: null,
        error: null,
        declined: true,
      };
    }
    const message = err instanceof Error ? err.message : String(err);
    SHIT_1170_TOAST_STORE.show("error", "SEND FAILED", message, null);
    return {
      sent: false,
      succeeded: 0,
      failed: 0,
      tx_hash: null,
      error: message,
      declined: false,
    };
  }

  // --- WHY --- THE OUTCOMES ARE MATCHED BY POSITION AND NOT BY ORDER:
  // the wallet answers in the order it sent, and the queue's own
  // `position` is the truth about which call is which.
  const tx_hashes: Array<{
    txId: string;
    txHash: string;
    status?: "succeeded" | "failed";
  }> = [];
  let succeeded = 0;
  let failed = 0;
  let first_hash: string | null = null;
  for (const transaction of transactions) {
    const outcome = outcomes[transaction.position];
    if (outcome === undefined) continue;
    const hash = shit_1170_outcome_hash_fun(outcome);
    if (hash === null) continue;
    if (first_hash === null) first_hash = hash;
    // --- WHY --- THE STATUS IS READ AND NOT GUESSED: the outcome
    // carries its error when the call failed. a status the app cannot
    // back is not sent, and the backend records "sent" — it went out
    // and nobody knows how it went yet.
    const error = (outcome as { error?: unknown }).error;
    if (error !== undefined && error !== null) {
      failed += 1;
      tx_hashes.push({
        txId: transaction.txId,
        txHash: hash,
        status: "failed",
      });
      continue;
    }
    succeeded += 1;
    tx_hashes.push({
      txId: transaction.txId,
      txHash: hash,
      status: "succeeded",
    });
  }

  if (tx_hashes.length === 0) {
    const message = "THE WALLET SENT NOTHING";
    SHIT_1170_TOAST_STORE.show("error", "SEND FAILED", message, null);
    return {
      sent: false,
      succeeded: 0,
      failed: 0,
      tx_hash: null,
      error: message,
      declined: false,
    };
  }

  // --- WHY --- THE BACKEND IS TOLD IN ONE CALL: the wallet sent the
  // whole batch in one popup, so the app learned everything about the
  // whole batch at once. one call reports it all.
  try {
    const options = await api_key_options_fun(true, account_id);
    await api_shit_1170_tx_queue_sent_post(
      SHIT_1170_BASE_URL,
      queue.queueId,
      { txHashes: tx_hashes },
      options,
    );
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    // --- WHY --- THE SEND IS STILL A SEND: the transactions went out
    // whatever the backend says about them. the toast says so and the
    // hash is still linked, so the reader can look at the chain.
    SHIT_1170_TOAST_STORE.show(
      "info",
      "SENT — NOT RECORDED",
      message,
      first_hash,
    );
    return {
      sent: true,
      succeeded,
      failed,
      tx_hash: first_hash,
      error: message,
      declined: false,
    };
  }

  // --- WHY --- THE TOAST SAYS HOW MANY WORKED WHEN NOT ALL OF THEM
  // DID: "SENT" alone would say the whole batch worked, and a reader
  // who had two failures out of three would not know.
  if (failed > 0) {
    SHIT_1170_TOAST_STORE.show(
      "info",
      "SENT",
      `${succeeded} of ${transactions.length} transactions worked`,
      first_hash,
    );
  } else {
    SHIT_1170_TOAST_STORE.show("success", "SENT", null, first_hash);
  }

  return {
    sent: true,
    succeeded,
    failed,
    tx_hash: first_hash,
    error: null,
    declined: false,
  };
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
