// ========================================
// QUEUE_PAYLOAD
// the shape the engine's tx-queue accepts.
//
// the agent NEVER sends a transaction. it builds a payload and the
// engine queues it for a human to sign in the web app.
//
// this is NOT a near-kit transaction and NOT a `near.call`. it is
// data.
//
// --- WHY --- IT IS HERE AND NOT IN A SKILL: a queue payload is the
// engine's own wire shape, and the engine's wire shapes live with the
// engine's typed client. a skill that builds a payload should build it
// from the same place the engine documents it, so the two cannot drift.
//
// --- WHY --- THE VALIDATION IS HERE: a payload the engine cannot
// trust is a payload that moves the wrong thing. an empty contract id,
// an empty method, an args that is not an object, an args that is not
// base64, and a gas or deposit that is not a raw digit string are all
// refused at build time, where the mistake is still visible.
// ========================================

/**
 * one queued function call.
 *
 * `gas` and `deposit` are raw digit strings: gas units and yoctoNEAR.
 * the web app's `shared/gas_deposit.ts` also accepts near-kit's human
 * forms ("30 Tgas", "0.00125 NEAR", "1 yocto") and converts them, so
 * a hand-written payload may use either — but raw digits are the
 * unambiguous form and are what this helper emits.
 */
export interface QUEUE_CALL {
  contractId: string;
  method: string;
  args: Record<string, unknown>;
  /**
   * the arguments, as raw bytes, base64 encoded.
   *
   * --- WHY --- it is OPTIONAL and not always present: a call with json
   * arguments does not carry it at all. it is here for the contracts
   * that take their arguments as raw bytes — the FastFS protocol
   * (`__fastdata_fastfs`) wants borsh bytes, and a json object cannot
   * be borsh bytes.
   */
  argsBase64?: string;
  gas: string;
  deposit: string;
}

/**
 * what goes into a queue payload.
 *
 * the gas and the deposit may be left out, and the defaults are used.
 */
export interface QUEUE_CALL_INPUT {
  contractId: string;
  method: string;
  /**
   * the arguments, as a json object.
   *
   * --- WHY --- it is OPTIONAL and not required: a call with raw-byte
   * arguments has no named arguments at all, and an empty object would
   * be a lie about that.
   */
  args?: Record<string, unknown>;
  /**
   * the arguments, as raw bytes, base64 encoded.
   *
   * --- WHY --- STANDARD BASE64 AND NOT THE URL-SAFE KIND: this is the
   * encoding the near wallet uses for action arguments, and the bytes
   * that come out of the wire are the bytes the chain expects. the
   * url-safe kind is for a url, and this does not go in one.
   */
  args_base64?: string;
  gas?: string;
  deposit?: string;
}

// 30 Tgas, the usual ceiling for a call that does real work.
export const DEFAULT_GAS = "30000000000000";
// no native NEAR attached.
export const DEFAULT_DEPOSIT = "0";

// raw digits only. "30 Tgas" and "0.00125 NEAR" are human forms the
// web app converts, but the queue payload itself carries raw units.
function raw_amount_fun(value: string, field: string): string {
  const trimmed = value.trim();
  if (!/^\d+$/.test(trimmed)) {
    throw new Error(
      `${field} must be a raw digit string (gas units or yoctoNEAR), got "${value}"`,
    );
  }
  return trimmed;
}

// the base64 alphabet, and what each character is worth.
const BASE64_ALPHABET =
  "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
// standard base64: the alphabet is A-Z a-z 0-9 + / and the padding is
// `=`.
//
// --- WHY --- THE CHECK IS WRITTEN OUT AND NOT DONE WITH `atob`: the
// builder runs in node as well as in the browser, and `atob` is not
// everywhere in node. the alphabet and the padding are the rule, so
// the rule is written down once here.
//
// --- WHY --- THE ALPHABET IS THE STANDARD ONE AND NOT THE URL-SAFE
// ONE: the url-safe alphabet swaps `+` and `/` for `-` and `_`, and it
// is for a url. this is the encoding the near wallet uses for action
// arguments, and the engine decodes it with the standard decoder. a
// url-safe string that came in by mistake is refused rather than
// quietly read as different bytes.
const BASE64_PATTERN = /^[A-Za-z0-9+/]*={0,2}$/;

// is this a plain base64 string?
//
// --- WHY --- IT IS CHECKED AND NOT DECODED: `atob` is not in node, and
// `Buffer` is not in the browser. the shape of the string is the same
// everywhere, and the engine decodes it again anyway.
//
// --- WHY --- THE CHECK MATCHES THE ENGINE'S DECODER EXACTLY: the engine
// refuses a string its decoder will not read, with a 400. a builder
// that accepts more than the engine does would let an agent build a
// payload that is refused on the way in — the mistake would only show
// up at the wire, and the check here exists so it does not get there.
//
// the standard decoder wants:
//   - the DATA characters (the ones before the padding) to be a multiple
//     of four with no padding, three short of it with one `=`, or two
//     short of it with two `==`
//   - the padding to be at the very end, and never more than two
//   - the alphabet to be the standard one
//   - the unused bits of the LAST data character to be zero — that is
//     what makes a base64 string canonical
function is_base64_fun(value: string): boolean {
  if (!BASE64_PATTERN.test(value)) return false;

  const padding = value.length - value.replace(/=+$/, "").length;
  const data = value.length - padding;
  const data_value = value.slice(0, data);

  // --- WHY --- THE DATA LENGTH AND THE PADDING MUST MATCH: base64 turns
  // three bytes into four characters, so the last group is short by one
  // character for one `=` and by two for two.
  const wanted_padding =
    data % 4 === 0 ? 0 : data % 4 === 3 ? 1 : data % 4 === 2 ? 2 : -1;
  if (wanted_padding < 0) return false;
  if (padding !== wanted_padding) return false;

  // --- WHY --- THE UNUSED BITS OF THE LAST CHARACTER MUST BE ZERO: a
  // base64 string that says more bits than it carries is not canonical,
  // and the standard decoder refuses it. the last data character holds
  // fewer bits than its six when there is padding, and the bits past
  // the bytes must be zero.
  if (padding === 0) return true;
  // --- WHY --- THE LAST CHARACTER IS READ INTO ITS OWN NAME: with
  // `noUncheckedIndexedAccess` an index into a string reads as
  // `string | undefined`, and the guard below is what the compiler
  // needs to see.
  const last_character = data_value[data_value.length - 1];
  if (last_character === undefined) return false;
  const last = BASE64_ALPHABET.indexOf(last_character);
  if (last < 0) return false;
  const unused_bits = padding * 2;
  return (last & ((1 << unused_bits) - 1)) === 0;
}

/**
 * build one queue payload.
 *
 * this does NOT send anything. it returns the data the engine queues
 * for a human to sign.
 */
export function queue_call_fun(input: QUEUE_CALL_INPUT): QUEUE_CALL {
  const contractId = input.contractId?.trim() ?? "";
  if (contractId.length === 0) {
    throw new Error("contractId is required");
  }

  const method = input.method?.trim() ?? "";
  if (method.length === 0) {
    throw new Error("method is required");
  }

  // --- THE TWO KINDS OF ARGS -------------
  // a call carries its arguments EITHER as a json object with named
  // keys OR as an opaque run of raw bytes.
  //
  // --- WHY --- BOTH AT ONCE IS REFUSED: the two carry different things,
  // and a payload that carries both does not say which one the contract
  // gets. picking one for the caller would be a guess, and a guess here
  // moves the wrong bytes.
  //
  // --- WHY --- AN EMPTY `args` DOES NOT COUNT AS "BOTH": the rule is
  // about what the call MEANS, and the empty object is the shape a
  // binary-args call takes on its way through the db. the engine reads
  // it the same way, so the two agree.
  const args_base64 = input.args_base64;
  const has_args_base64 = args_base64 !== undefined;
  const args_is_empty_object =
    typeof input.args === "object" &&
    input.args !== null &&
    !Array.isArray(input.args) &&
    Object.keys(input.args).length === 0;

  if (has_args_base64 && input.args !== undefined && !args_is_empty_object) {
    throw new Error("a call carries either args or args_base64, not both");
  }

  const args = input.args ?? {};
  if (typeof args !== "object" || args === null || Array.isArray(args)) {
    throw new Error("args must be a plain object");
  }

  const gas = raw_amount_fun(input.gas ?? DEFAULT_GAS, "gas");
  const deposit = raw_amount_fun(input.deposit ?? DEFAULT_DEPOSIT, "deposit");

  // --- THE RAW BYTES --------------------
  // --- WHY --- THE BASE64 IS CHECKED HERE: the engine refuses a string
  // that is not base64, and a payload that is refused at the engine is
  // a payload the agent finds out about too late. the check is here
  // instead.
  if (args_base64 !== undefined) {
    const trimmed = args_base64.trim();
    if (trimmed.length === 0) {
      throw new Error("args_base64 must not be empty");
    }
    if (!is_base64_fun(trimmed)) {
      throw new Error(
        `args_base64 must be a plain base64 string, got "${args_base64}"`,
      );
    }
    return { contractId, method, args: {}, argsBase64: trimmed, gas, deposit };
  }

  return { contractId, method, args, gas, deposit };
}
// ========================================
// copyright 2026 by sleet.near
