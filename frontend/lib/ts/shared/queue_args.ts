// ========================================
// the READING of a queued transaction's arguments.
//
// a queued transaction carries its arguments in one of two ways:
//
//   the JSON way   — `args` is a json object with named keys.
//   the BYTE way   — `argsBase64` is the raw bytes of the call, and
//                    the contract reads them itself.
//
// the byte way is what the FastFS protocol (`__fastdata_fastfs`) needs:
// it wants borsh bytes, and a json object cannot be borsh bytes.
//
// --- WHY --- THE READING IS HERE AND NOT IN THE COMPONENT: the queue
// page shows the arguments, and the feed's send path uses the bytes,
// and they must not drift on what the bytes are. one place reads them.
//
// --- WHY --- THE HEX PREVIEW AND NOT THE JSON: the bytes are not json,
// and printing them as json would show a reader a broken blob that
// says nothing. a hex preview says what the bytes are, and the length
// says how many there are.
//
// --- WHY --- THE HEX COMES FROM THE CLIENT: the format of a hex dump is
// one thing, and it lives in the fastnear client, next to the FastFS
// decoder that reads the same bytes. a second copy of the format here
// would be a second chance to drift.
import { fastfs_bytes_to_hex_fun } from "@sleet-js/fastnear-explorer-api-client";
import type { SHIT_1170_TX_QUEUE_TRANSACTION } from "@huggies/shit-1170-rex-engine-middleman";
// ========================================
// ========================================
// --- TYPES ------------------------------
// what a queued call's arguments look like on screen.
export interface SHIT_1170_QUEUE_ARGS {
  // true when the arguments are raw bytes and not a json object.
  binary: boolean;
  // the json, pretty printed, for a call with named arguments.
  json: string;
  // how many bytes the raw arguments are, for a call with bytes.
  byte_length: number;
  // the first bytes of the raw arguments, in hex.
  hex: string;
}
// ========================================
// ========================================
// --- HELPERS ----------------------------
// the bytes of a base64 string.
//
// --- WHY --- `atob` AND NOT `Buffer`: this runs in the browser, and
// `Buffer` is node's. `atob` is in every browser and in bun.
//
// --- WHY --- THE BYTES ARE BUILT BY HAND: `Uint8Array.from` on a
// string gives the CODE POINTS of the string, not the bytes of the
// base64. the base64 is decoded to a binary string first, and each
// character of that string is one byte.
//
// --- WHY --- IT IS EXPORTED: the feed's send path hands the wallet the
// same bytes, and the two must not decode them differently.
export function shit_1170_queue_args_bytes_fun(base64: string): Uint8Array {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index++) {
    bytes[index] = binary.charCodeAt(index);
  }
  return bytes;
}

// how many bytes of the preview are shown.
//
// --- WHY --- A LIMIT AND NOT ALL OF THEM: borsh arguments can be
// thousands of bytes, and a preview that shows all of them is a wall of
// hex. the length is shown beside it, so nothing is hidden — the
// reader knows how much there is.
const SHIT_1170_QUEUE_ARGS_HEX_LIMIT = 64;
// ========================================
// ========================================
// --- READ ------------------------------
// the arguments of one queued call, in the shape the page shows.
export function shit_1170_queue_args_fun(
  transaction: SHIT_1170_TX_QUEUE_TRANSACTION,
): SHIT_1170_QUEUE_ARGS {
  const args_base64 = transaction.argsBase64;
  if (args_base64 === null || args_base64 === undefined) {
    return {
      binary: false,
      json: JSON.stringify(transaction.args ?? {}, null, 2),
      byte_length: 0,
      hex: "",
    };
  }

  // --- WHY --- A BROKEN BASE64 SHOWS ITSELF AND DOES NOT THROW: the
  // page must not go blank because one call's bytes will not decode.
  // the base64 came from the backend, which checked it, so this is
  // belt and braces.
  let bytes: Uint8Array;
  try {
    bytes = shit_1170_queue_args_bytes_fun(args_base64);
  } catch {
    return {
      binary: true,
      json: "",
      byte_length: 0,
      hex: "",
    };
  }

  const shown = bytes.slice(0, SHIT_1170_QUEUE_ARGS_HEX_LIMIT);
  // --- WHY --- THE CUT IS MARKED: a preview that stops without saying
  // so looks like the whole thing. the reader is told there are more
  // bytes, and the byte count beside it says how many.
  //
  // --- WHY --- THE MARK IS ITS OWN LINE: the hex has a line break
  // every sixteen bytes, and a mark at the end of the last line would
  // sit on the same line as the bytes it talks about.
  const hex = fastfs_bytes_to_hex_fun(shown);
  return {
    binary: true,
    json: "",
    byte_length: bytes.length,
    hex: shown.length < bytes.length ? `${hex}\n…` : hex,
  };
}
// ========================================
// copyright 2026 by sleet.near
