// ========================================
// the feed's ONE call line: the first transaction of the post, its args,
// and the token it moves when it moves one.
//
// --- WHY THE POST SHOWS ONLY THE FIRST CALL: the post is SMALL. it
// answers "is this worth my attention?" and the queue page answers
// "exactly what is this?". a post that carried every transaction would
// be a wall of text, and a feed of walls is not a feed.
//
// --- WHY THE CONTRACT AND THE METHOD ARE ALWAYS ON THEIR OWN LINE AND
// ALWAYS THE SAME WAY: the post reads the same for every kind of call.
// a line that changed shape depending on whether the call moved a token
// would be a line the reader had to learn twice.
//
// --- WHY THE ARGS ARE ALWAYS SHOWN AND FLATTENED: the reader is about
// to SIGN this. the contract and the method say what is called; the
// args say what it is called WITH. a post that showed the amount but
// not the args would leave the reader signing something they have not
// read.
//
// --- WHY THE ARGS ARE FLATTENED TO ONE LINE: the args are a json
// object, and json printed across several lines is a block of code the
// reader has to parse. flattened, it is one line of `key=value` pairs
// that reads like the call it describes.
//
// --- WHY THE TOKEN DETAILS ARE ON A LINE OF THEIR OWN: they are a
// DIFFERENT kind of thing from the args. the args are what the call
// asks for; the token line is what that means in money — the icon, the
// amount, the symbol and the worth. they are the only part of the post
// that needs the token registry.
//
// --- WHY THE RENDERING IS THE SHARED ONE: the token's icon, the amount
// divided by its decimals, the symbol and the usd worth all come from
// the shared transfer amount component. a queued transfer and a
// finished transfer must read identically.
//
// --- WHY IT DOES NOT SELF-MOUNT: the call line is drawn INSIDE the
// post, once per post. a mount element would be the same id on every
// post, so the component takes the post as a prop and the post's own
// css file carries the rules — see
// SHIT_1170_COMPONENT_APP_PAGE_FEED_TRANSACTIONS.css.
import {
  shit_1170_transfer_amount_fun,
  shit_1170_transfer_icon_fun,
} from "../shared/transfer_amount";
import { shit_1170_queue_first_transfer_fun } from "../../ts/shared/queue_transfers";
import { shit_1170_queue_args_display_fun } from "../../ts/shared/queue_args_display";
import { shit_1170_code_block_fun } from "../tx/code_block";
import {
  shit_1170_queue_is_key_action_fun,
  shit_1170_queue_key_action_line_fun,
  shit_1170_queue_key_detail_lines_fun,
} from "../../ts/shared/queue_key_action";
import { use_queue_key_details_fun } from "../../ts/shared/use_queue_key_details";
import { SHIT_1170_AUTH } from "../../ts/shared/auth_status";
import type { SHIT_1170_TRANSFER_TOKEN } from "../../ts/shared/transfer_tokens";
import type {
  SHIT_1170_TX_QUEUE,
  SHIT_1170_TX_QUEUE_TRANSACTION,
} from "@huggies/shit-1170-rex-engine-middleman";
// ========================================
// ========================================
// --- HELPERS ----------------------------
// the args of one call, as the json to put in the code block.
//
// --- WHY --- THE WHOLE OBJECT AND NOT A SUMMARY: the reader is about
// to SIGN this. the args are what the call asks with, and a summary
// that leaves out a key is a call the reader has not fully read.
//
// --- WHY --- THE KEYS ARE SORTED: the order the agent wrote the args in
// is not meaningful, and a post whose block changed shape between loads
// would be unreadable.
//
// --- WHY --- A CALL WITH BYTES SHOWS THE BYTES: a FastFS call carries
// its payload in `argsBase64` and its `args` is an EMPTY object, so
// reading only `args` showed NOTHING for it. the reader was asked to
// approve a call whose arguments were not on the page. the shared
// reader knows what the bytes are and says so.
function args_json_fun(args: Record<string, unknown>): string | null {
  const keys = Object.keys(args).sort();
  if (keys.length === 0) return null;
  const sorted: Record<string, unknown> = {};
  for (const key of keys) {
    sorted[key] = args[key];
  }
  return JSON.stringify(sorted, null, 2);
}
// ========================================
// --- EMPTY -----------------------------
// the transaction a batch with no calls is asked about.
//
// --- WHY IT EXISTS: the hook that reads a key's reach cannot be called
// conditionally, and a batch can have no transactions at all. so the
// hook is asked about THIS when there is nothing to ask about, and the
// hook sees a transaction with no key action — which is the one thing it
// already knows how to leave alone.
const SHIT_1170_FEED_EMPTY_TRANSACTION = {
  txId: "",
  position: 0,
  contractId: "",
  method: "",
  args: {},
  argsBase64: null,
  gas: null,
  deposit: null,
  keyAction: null,
  keyPublicKey: null,
  keyReceiverId: null,
  keyMethodNames: null,
  keyAllowance: null,
  accounts: [],
} as unknown as SHIT_1170_TX_QUEUE_TRANSACTION;
// ========================================
// --- COMPONENT --------------------------
export function SHIT_1170_COMPONENT_APP_PAGE_FEED_TRANSACTIONS_FUN(props: {
  queue: SHIT_1170_TX_QUEUE;
  // the token metadata for the transfers the batches name.
  tokens: Map<string, SHIT_1170_TRANSFER_TOKEN>;
}) {
  const first = props.queue.transactions[0];
  // --- WHY --- THE HOOK IS CALLED BEFORE THE EARLY RETURN: a hook cannot
  // be called conditionally, and a batch whose first transaction is
  // missing would then change the NUMBER of hooks between renders. so
  // the hook is asked about the first transaction — or about nothing,
  // when there is no first transaction.
  const account_id = SHIT_1170_AUTH.get_account_id();
  const details = use_queue_key_details_fun(
    first ?? SHIT_1170_FEED_EMPTY_TRANSACTION,
    account_id,
  );
  // a batch with no calls has nothing to show here.
  if (first === undefined) return null;
  const transfer = shit_1170_queue_first_transfer_fun(props.queue);
  const args_display = shit_1170_queue_args_display_fun(first);
  const args_json = args_display.binary
    ? null
    : args_json_fun(first.args ?? {});
  // --- WHY --- A KEY ACTION GETS ITS OWN LINE: a key action is not a
  // call — it changes the READER'S KEYS — and the contract line would
  // show an empty contract and an empty method. the reader has to see
  // WHAT will happen to their keys before they approve it.
  const is_key_action = shit_1170_queue_is_key_action_fun(first);
  // --- WHY --- THE KEY'S REACH IS SHOWN UNDER THE HEADLINE: the headline
  // says WHICH key changes. the reach says what the key CAN DO — the
  // contract, the methods and the ceiling on what it may spend — and
  // that is what the reader is really approving. a post that named a
  // key and said nothing about it would ask the reader to approve a
  // blank.
  const key_detail_lines = is_key_action
    ? shit_1170_queue_key_detail_lines_fun(first, details)
    : [];

  return (
    <div class="feed-call">
      {/* --- WHY --- THE CONTRACT AND THE METHOD ARE THE SAME FOR EVERY
          CALL: the post reads the same whatever it moves. */}
      <div class="feed-call-line">
        {is_key_action ? (
          <span class="feed-call-key">
            {shit_1170_queue_key_action_line_fun(first)}
          </span>
        ) : (
          <>
            <code>{first.contractId}</code>
            <span class="feed-call-method">{first.method}</span>
          </>
        )}
        {/* --- WHY --- THE COUNT IS SHOWN ONLY FOR A BATCH: one call needs
            no count. a batch of three says so, so the reader knows the
            post is showing one of three. */}
        {props.queue.transactions.length > 1 && (
          <span class="feed-call-more">
            +{props.queue.transactions.length - 1} more
          </span>
        )}
      </div>
      {/* --- WHY --- THE KEY'S REACH IS SHOWN UNDER THE HEADLINE: the
          headline says WHICH key changes; the reach says what the key
          CAN DO. the reader is about to sign this, and a key named
          without its reach is a blank approval. */}
      {key_detail_lines.length > 0 && (
        <div class="feed-call-key-detail">
          {key_detail_lines.map((line) => (
            <p class="feed-call-key-detail-line" key={line}>
              {line}
            </p>
          ))}
        </div>
      )}
      {/* --- WHY --- THE ARGS ARE A CODE BLOCK AND NOT PLAIN TEXT: it is
          the same box every other piece of json in the app sits in, so
          a queued call's args and a finished transaction's args look
          the same. */}
      {!is_key_action &&
        args_json !== null &&
        shit_1170_code_block_fun({ code: args_json })}
      {/* --- WHY --- A CALL WITH BYTES SHOWS THE FILE'S FACTS AND THE HEX:
          the reader is about to approve an upload, and "notes/small.txt,
          text/plain, 13 bytes" is what that approval means. the hex is
          there so the reader can see the payload itself. */}
      {!is_key_action &&
        args_display.binary &&
        args_display.byte_length > 0 && (
          <div class="feed-call-bytes">
            {args_display.fastfs_path !== "" && (
              <div class="feed-call-bytes-line">
                <span class="feed-call-bytes-path">
                  {args_display.fastfs_path}
                </span>
                {args_display.fastfs_mime !== "" && (
                  <span class="feed-call-bytes-mime">
                    {args_display.fastfs_mime}
                  </span>
                )}
                <span class="feed-call-bytes-size">
                  {args_display.byte_length} BYTES
                </span>
              </div>
            )}
            {args_display.fastfs_path === "" && (
              <div class="feed-call-bytes-line">
                <span class="feed-call-bytes-size">
                  {args_display.byte_length} BYTES
                </span>
              </div>
            )}
            <div class="feed-call-bytes-hex">
              {shit_1170_code_block_fun({ code: args_display.hex })}
            </div>
          </div>
        )}
      {/* --- WHY --- THE TOKEN LINE IS ONLY DRAWN FOR A CALL THAT MOVES
          ONE: there is nothing to say about the tokens of a call that
          moves none. */}
      {transfer !== null && (
        <div class="feed-call-token">
          {shit_1170_transfer_icon_fun(
            props.tokens.get(transfer.token_id) ?? null,
          )}
          {shit_1170_transfer_amount_fun({
            amount: transfer.amount,
            token: props.tokens.get(transfer.token_id) ?? null,
            contract_id: transfer.token_id,
          })}
          {transfer.receiver_id !== null && (
            <span class="feed-call-token-to">
              &rarr; {transfer.receiver_id}
            </span>
          )}
        </div>
      )}
    </div>
  );
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
