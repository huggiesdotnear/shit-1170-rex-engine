// ========================================
// queue_args_display.ts
// what a queued call's arguments look like on screen.
//
// --- WHY THIS FILE IS ITS OWN ---
// a queued call can carry its arguments as JSON or as RAW BYTES, and the
// two look nothing alike on screen. the feed shows one line of json and
// the tx page shows a hex dump, and they must not disagree about what a
// call carries. one place reads the arguments.
//
// --- THE DISCONNECT THIS FIXES ---
// a FastFS call carries its bytes in `argsBase64`, and its `args` is an
// EMPTY object. the feed used to read only `args`, so a FastFS call showed
// NO arguments at all — no code block, nothing. the reader was asked to
// approve a call whose arguments were not on the page.
//
// a call with bytes now shows what the bytes are: the file's path, its
// mime type and its size, and the hex of the payload. that is the same
// information the queue page's FastFS widget shows, so the two agree.
import {
  decode_base64_bytes_raw,
  fastfs_bytes_to_hex_fun,
  fastfs_decode_fun,
  fastfs_is_call_fun,
} from "@sleet-js/fastnear-explorer-api-client";
import type { SHIT_1170_TX_QUEUE_TRANSACTION } from "@huggies/shit-1170-rex-engine-middleman";
// ========================================
// ========================================
// --- TYPES ------------------------------
// what the reader sees for one call's arguments.
export interface SHIT_1170_QUEUE_ARGS_DISPLAY {
  // true when the arguments are raw bytes and not a json object.
  binary: boolean;
  // the json, pretty printed, for a call with named arguments.
  json: string;
  // the hex of the bytes, for a call with raw bytes.
  hex: string;
  // how many bytes the arguments are.
  byte_length: number;
  // what the bytes are, for a FastFS upload: the path, the mime and the
  // size. empty for a call that is not a FastFS upload.
  fastfs_path: string;
  fastfs_mime: string;
  fastfs_size: number;
}
// ========================================
// ========================================
// --- HELPERS ----------------------------
// how many bytes of the hex are shown.
//
// --- WHY --- A LIMIT AND NOT ALL OF THEM: a FastFS payload can be a
// megabyte, and a preview of a megabyte of hex is a wall. the length is
// shown beside it, so nothing is hidden.
const SHIT_1170_QUEUE_ARGS_HEX_LIMIT = 64;
// ========================================
// ========================================
// --- DISPLAY ---------------------------
// the arguments of one queued call, in the shape the page shows.
//
// --- WHY --- THE BYTES ARE DECODED LOSSLESSLY: a hex dump has to show
// the bytes exactly, and a base64 decoded to utf-8 text and re-encoded
// is lossy for anything that is not valid utf-8 — a png is not utf-8,
// and the byte `89` comes back as `ef bf bd`.
export function shit_1170_queue_args_display_fun(
  transaction: SHIT_1170_TX_QUEUE_TRANSACTION,
): SHIT_1170_QUEUE_ARGS_DISPLAY {
  const empty: SHIT_1170_QUEUE_ARGS_DISPLAY = {
    binary: false,
    json: "",
    hex: "",
    byte_length: 0,
    fastfs_path: "",
    fastfs_mime: "",
    fastfs_size: 0,
  };

  const args_base64 = transaction.argsBase64;
  if (args_base64 === null || args_base64 === undefined) {
    // a call with json arguments. the keys are sorted, because the order
    // the agent wrote them in is not meaningful and a block that changed
    // shape between loads would be unreadable.
    const args = transaction.args ?? {};
    const keys = Object.keys(args).sort();
    if (keys.length === 0) return empty;
    const sorted: Record<string, unknown> = {};
    for (const key of keys) {
      sorted[key] = args[key];
    }
    return {
      ...empty,
      json: JSON.stringify(sorted, null, 2),
    };
  }

  const bytes = decode_base64_bytes_raw(args_base64);
  if (bytes === null) {
    return { ...empty, binary: true };
  }

  const shown = bytes.subarray(0, SHIT_1170_QUEUE_ARGS_HEX_LIMIT);

  // --- WHY --- THE FASTFS FACTS ARE FILLED IN: the reader is approving an
  // upload, and "a file called notes/small.txt, text/plain, 13 bytes" is
  // what that approval means. a hex dump alone says nothing.
  let fastfs_path = "";
  let fastfs_mime = "";
  let fastfs_size = 0;
  if (fastfs_is_call_fun(transaction.method.trim())) {
    const info = fastfs_decode_fun(bytes);
    if (info !== null) {
      fastfs_path = info.relative_path;
      fastfs_mime = info.mime_type;
      // --- WHY --- THE WHOLE FILE'S SIZE AND NOT THE CHUNK'S: a reader
      // approving an upload wants to know the size of the FILE. for a
      // simple upload the chunk is the file, so the two are the same.
      fastfs_size = info.kind === "partial" ? info.total_size : info.size;
    }
  }

  return {
    binary: true,
    json: "",
    hex:
      shown.length < bytes.length
        ? `${fastfs_bytes_to_hex_fun(shown)}\n…`
        : fastfs_bytes_to_hex_fun(shown),
    byte_length: bytes.length,
    fastfs_path,
    fastfs_mime,
    fastfs_size,
  };
}
// ========================================
// copyright 2026 by sleet.near
