// ========================================
// the DETAILS of a queued key action: what the key can do, and how much
// it may spend.
//
// --- WHY THIS FILE EXISTS: the queue carries a key action with two
// halves that say different things. an ADD says what the NEW key will
// be able to do — the contract, the methods and the allowance — because
// the queue stored those when the agent asked for the key. a REMOVE says
// only WHICH key goes, and the queue deliberately drops the rest: the
// chain is the one that knows what a key on the account may call, and an
// agent's guess about it would be a claim about a different key.
//
// so a remove has to ASK THE CHAIN what the key could do, or the reader
// approves the removal of something they cannot see. this file is that
// ask, and the one place both the feed and the queue page get it.
//
// --- WHY IT IS SHARED AND NOT IN THE PAGE: the feed card and the queue
// page both draw a key action, and a reader who sees the same key
// described two ways on two screens learns to trust neither. the rule
// about what a key action says belongs with the rule about what a key
// action IS.
//
// --- WHY THE CHAIN IS ASKED AND NOT THE BACKEND: the access keys live
// on the chain, and the chain is the only thing that knows them. the
// backend keeps the app's api keys, which are a different thing
// entirely.
//
// --- WHY THE ANSWER IS CACHED: the feed draws one card per batch and
// the same batch can be drawn again on every re-render, and a reader
// scrolling the feed must not make the rpc answer the same question
// twice. the cache is per account and per public key, and it is dropped
// when the reader signs out — a key's reach can change, and a stale
// answer about it is worse than no answer.
import { near_kit_client } from "@near-kit-tool-box/web";
import { SHIT_1170_APP_PAGE_KEYS_ALLOWANCE } from "../keys/app_page_keys_allowance";
import type { SHIT_1170_TX_QUEUE_TRANSACTION } from "@huggies/shit-1170-rex-engine-middleman";
// ========================================
// --- TYPES ------------------------------
// what a key action's key can do.
//
// --- WHY THE ALLOWANCE IS A FIGURE AND NOT A NUMBER: a yoctoNEAR
// amount is a 24-decimal number and a javascript number loses the last
// of those digits. the figure is a LIMIT on money, so it is carried as
// the chain's own string and formatted as a string.
export interface SHIT_1170_QUEUE_KEY_DETAILS_ROW {
  // the contract the key may call. null when it is not known.
  receiver_id: string | null;
  // the methods the key may call. empty means EVERY method on that
  // contract is allowed.
  method_names: string[];
  // the ceiling on what the key may attach, in yoctoNEAR. null when the
  // key may attach nothing, or when it is not known.
  allowance_yocto: string | null;
  // the allowance, ready to read. null when there is none to read.
  allowance_human: string | null;
  // true when the details came from the chain, and true when they could
  // not be found — the reader needs to know if they are looking at the
  // key's real reach or at nothing at all.
  resolved: boolean;
}
// ========================================
// --- CACHE -----------------------------
// the chain's answers, per account and per public key.
//
// --- WHY THE ACCOUNT IS PART OF THE KEY: the same public key can sit
// on two accounts with two different reaches, and the key alone does
// not say which one is being asked about.
const details_cache = new Map<string, SHIT_1170_QUEUE_KEY_DETAILS_ROW>();
// the accounts whose keys have been read, so a second lookup of a key
// that is NOT on the account does not ask the rpc again.
const accounts_read = new Set<string>();
// ========================================
// --- HELPERS ---------------------------
// an empty details answer: a key we know nothing about.
function empty_details_fun(): SHIT_1170_QUEUE_KEY_DETAILS_ROW {
  return {
    receiver_id: null,
    method_names: [],
    allowance_yocto: null,
    allowance_human: null,
    resolved: false,
  };
}
// the cache key for one account and one public key.
function cache_key_fun(account_id: string, public_key: string): string {
  return `${account_id}::${public_key}`;
}
// ========================================
// --- CHAIN READ ------------------------
// read one account's keys and remember them.
//
// --- WHY A FAILED READ IS NOT AN ERROR THE READER SEES: the reader is
// looking at a queued batch, and a rpc that is down must not take the
// batch off the screen. the answer simply says the reach is unknown.
async function read_account_fun(
  account_id: string,
): Promise<Map<string, SHIT_1170_QUEUE_KEY_DETAILS_ROW>> {
  const found = new Map<string, SHIT_1170_QUEUE_KEY_DETAILS_ROW>();
  try {
    const response = await near_kit_client().getAccessKeys(account_id);
    const keys = (response as { keys?: unknown }).keys;
    if (!Array.isArray(keys)) return found;
    for (const entry of keys) {
      const key = entry as {
        public_key?: unknown;
        access_key?: { permission?: unknown };
      };
      if (typeof key.public_key !== "string") continue;
      const details = read_permission_fun(key.access_key?.permission);
      found.set(key.public_key, details);
    }
  } catch (err) {
    console.warn("[queue_key_details] the chain did not answer", err);
  }
  return found;
}
// the receiver, the methods and the allowance of one limited key.
//
// --- WHY THE GAS KEY SHAPES ARE READ HERE TOO: the chain knows three
// more permission shapes — the gas key variants — and every one of them
// is a limited key with a receiver and methods on it. the reader wants
// the same three facts for all of them.
function read_permission_fun(permission: unknown): SHIT_1170_QUEUE_KEY_DETAILS_ROW {
  const empty = empty_details_fun();
  if (typeof permission !== "object" || permission === null) return empty;
  const record = permission as Record<string, unknown>;
  const details =
    typeof record.FunctionCall === "object" && record.FunctionCall !== null
      ? (record.FunctionCall as Record<string, unknown>)
      : typeof record.GasKeyFunctionCall === "object" &&
          record.GasKeyFunctionCall !== null
        ? (record.GasKeyFunctionCall as Record<string, unknown>)
        : null;
  if (details === null) return empty;
  const receiver_id =
    typeof details.receiver_id === "string" ? details.receiver_id : null;
  const method_names = Array.isArray(details.method_names)
    ? details.method_names.filter(
        (name): name is string => typeof name === "string",
      )
    : [];
  const allowance_yocto =
    typeof details.allowance === "string" ? details.allowance : null;
  return {
    receiver_id,
    method_names,
    allowance_yocto,
    allowance_human:
      SHIT_1170_APP_PAGE_KEYS_ALLOWANCE.format(allowance_yocto).human,
    resolved: true,
  };
}
// ========================================
// --- PUBLIC ---------------------------
// the details of a key action, as far as they are known.
//
// --- WHY THE ACCOUNT COMES IN: a remove names the key, and the chain
// needs the ACCOUNT to say what that key may call on it.
export async function shit_1170_queue_key_details_fun(
  transaction: SHIT_1170_TX_QUEUE_TRANSACTION,
  account_id: string | null,
): Promise<SHIT_1170_QUEUE_KEY_DETAILS_ROW> {
  // --- WHY AN ADD IS ANSWERED FROM THE QUEUE AND NOT FROM THE CHAIN:
  // the key does not exist yet. the queue already holds what the agent
  // asked for, and that IS the key that will be made — so the queue is
  // the only place the answer can come from.
  if (transaction.keyAction === "add_key") {
    const allowance_yocto = transaction.keyAllowance ?? null;
    return {
      receiver_id: transaction.keyReceiverId ?? null,
      method_names: transaction.keyMethodNames ?? [],
      allowance_yocto,
      allowance_human:
        SHIT_1170_APP_PAGE_KEYS_ALLOWANCE.format(allowance_yocto).human,
      resolved: true,
    };
  }
  // --- WHY A REMOVE WITH NO ACCOUNT IS EMPTY AND NOT A CHAIN ASK: the
  // chain would need an account to look the key up on, and a batch the
  // reader is not on does not have one.
  if (account_id === null) return empty_details_fun();
  const public_key = transaction.keyPublicKey ?? "";
  if (public_key === "") return empty_details_fun();
  const cached = details_cache.get(cache_key_fun(account_id, public_key));
  if (cached !== undefined) return cached;
  // --- WHY THE ACCOUNT IS READ ONCE AND NOT PER KEY: the account's
  // keys come back in one answer, and asking the rpc per key would be
  // one request per row of the feed.
  if (!accounts_read.has(account_id)) {
    const found = await read_account_fun(account_id);
    for (const [key, details] of found) {
      details_cache.set(cache_key_fun(account_id, key), details);
    }
    accounts_read.add(account_id);
  }
  const found = details_cache.get(cache_key_fun(account_id, public_key));
  if (found !== undefined) return found;
  // --- WHY A KEY THAT IS NOT ON THE ACCOUNT IS CACHED TOO: a key that
  // was already removed answers nothing, and asking again for every
  // re-render of the same card would be asking the rpc for nothing.
  const empty = empty_details_fun();
  details_cache.set(cache_key_fun(account_id, public_key), empty);
  return empty;
}
// --- FORGET -----------------------------
// drop the cached answers.
//
// --- WHY THE CACHE IS DROPPED WHEN THE READER SIGNS OUT: a key's reach
// can change, and an answer about it that was true for the last reader
// is not true for this one.
export function shit_1170_queue_key_details_forget_fun(): void {
  details_cache.clear();
  accounts_read.clear();
}
// ========================================
// --- PUBLIC ---------------------------
export const SHIT_1170_QUEUE_KEY_DETAILS = {
  read: shit_1170_queue_key_details_fun,
  forget: shit_1170_queue_key_details_forget_fun,
};
// ========================================
// copyright 2026 by sleet.near
// ========================================
