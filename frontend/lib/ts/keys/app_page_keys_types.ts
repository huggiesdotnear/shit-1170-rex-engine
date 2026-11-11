// ========================================
// the keys page's shapes: what an access key row is, and what the page
// knows about one.
//
// --- WHY THE TYPES ARE IN THEIR OWN FILE: the loader, the store and
// both components all speak about access keys, and a type that lives in
// the loader cannot be imported by the loader's own consumers without
// dragging the loader in with it. the shapes are shared, so they live
// in the one file that holds nothing but shapes.
// ========================================
// --- WHY THE RPC'S SHAPE IS WRITTEN OUT HERE AND NOT IMPORTED: the
// near-kit root does not export the access-key list type, and the one
// place it lives is an internal path of the package. a page should not
// reach into a package's internals for a shape, so the few fields the
// page reads are written out here. the fields are the chain's own.
interface SHIT_1170_APP_PAGE_KEYS_RPC_KEY {
  public_key: string;
  access_key: {
    nonce: number;
    permission: unknown;
  };
}
interface SHIT_1170_APP_PAGE_KEYS_RPC_RESPONSE {
  keys: SHIT_1170_APP_PAGE_KEYS_RPC_KEY[];
}
// ========================================
// ========================================
// --- TYPES ------------------------------
// one access key, as the page shows it.
//
// --- WHY THE PAGE KEEPS ITS OWN SHAPE AND NOT THE RPC'S: the rpc
// answer carries block hashes and nonce bookkeeping that the reader
// never sees. the page's shape carries what the page shows, so the
// component does not have to know the rpc's field names.
export interface SHIT_1170_APP_PAGE_KEYS_ROW {
  // the public key, the way the chain spells it: "ed25519:...".
  public_key: string;
  // true for a full-access key, false for a limited (function-call) one.
  //
  // --- WHY FULL ACCESS IS A BOOLEAN AND NOT A KIND: there are exactly
  // two kinds the page draws — full access and limited — and a gas key
  // is a full-access key with a balance on it. the page draws both the
  // same way, so one flag carries the whole distinction.
  full_access: boolean;
  // the contract the key may call, for a limited key. null for a
  // full-access key.
  receiver_id: string | null;
  // the methods the key may call, for a limited key. empty means EVERY
  // method on that contract is allowed.
  method_names: string[];
  // the deposit the key may spend, in yoctoNEAR, for a limited key.
  // null when the key may spend nothing.
  allowance: string | null;
  // the nonce of the key, so the reader can see the key has been used.
  nonce: number;
}
// ========================================
// the page's data: the rows and what went wrong.
export interface SHIT_1170_APP_PAGE_KEYS_DATA {
  // the account the keys belong to, or null when nobody is signed in.
  account_id: string | null;
  // the rows, full-access keys FIRST and then the limited ones.
  keys: SHIT_1170_APP_PAGE_KEYS_ROW[];
  // the message for the reader, when the load failed.
  error: string | null;
}
// ========================================
// --- HELPERS ----------------------------
// the rows of one rpc answer, in the order the page shows them.
//
// --- WHY FULL ACCESS COMES FIRST: the full-access key is the account
// itself — the one that can do anything. it is the key the reader must
// NOT lose and must NOT remove by accident, so it sits at the top of
// the list where the eye lands first.
//
// --- WHY THE ORDER INSIDE EACH GROUP IS THE CHAIN'S: the rpc answers
// the keys in a stable order, and re-sorting the limited keys by
// anything of our own would only make the list jump around between
// loads.
function rows_from_access_keys_fun(
  response: SHIT_1170_APP_PAGE_KEYS_RPC_RESPONSE,
): SHIT_1170_APP_PAGE_KEYS_ROW[] {
  const full_access: SHIT_1170_APP_PAGE_KEYS_ROW[] = [];
  const limited: SHIT_1170_APP_PAGE_KEYS_ROW[] = [];

  for (const entry of response.keys) {
    const permission = entry.access_key.permission;
    const nonce = entry.access_key.nonce;
    const public_key = entry.public_key;

    if (permission === "FullAccess") {
      full_access.push({
        public_key,
        full_access: true,
        receiver_id: null,
        method_names: [],
        allowance: null,
        nonce,
      });
      continue;
    }

    // --- WHY THE OTHER PERMISSIONS ARE TREATED AS LIMITED: the chain
    // knows three more shapes — the gas key variants — and every one
    // of them is a limited key with a receiver and methods on it. the
    // page draws them as limited, which is what they are.
    const details = read_limited_permission_fun(permission);
    limited.push({
      public_key,
      full_access: false,
      receiver_id: details.receiver_id,
      method_names: details.method_names,
      allowance: details.allowance,
      nonce,
    });
  }

  return [...full_access, ...limited];
}
// ========================================
// the receiver, the methods and the allowance of a limited key.
//
// --- WHY THE GAS KEY SHAPES ARE READ HERE AND NOT ELSEWHERE: a gas key
// carries a balance and a nonce count on top of the usual function-call
// fields, but the page shows the SAME three things for it. so the extra
// fields are dropped here and the page never learns they exist.
function read_limited_permission_fun(permission: unknown): {
  receiver_id: string | null;
  method_names: string[];
  allowance: string | null;
} {
  if (typeof permission !== "object" || permission === null) {
    return { receiver_id: null, method_names: [], allowance: null };
  }
  const record = permission as Record<string, unknown>;
  const details =
    typeof record.FunctionCall === "object" && record.FunctionCall !== null
      ? (record.FunctionCall as Record<string, unknown>)
      : typeof record.GasKeyFunctionCall === "object" &&
          record.GasKeyFunctionCall !== null
        ? (record.GasKeyFunctionCall as Record<string, unknown>)
        : null;
  if (details === null) {
    return { receiver_id: null, method_names: [], allowance: null };
  }
  const receiver_id =
    typeof details.receiver_id === "string" ? details.receiver_id : null;
  const method_names = Array.isArray(details.method_names)
    ? details.method_names.filter(
        (name): name is string => typeof name === "string",
      )
    : [];
  const allowance =
    typeof details.allowance === "string" ? details.allowance : null;
  return { receiver_id, method_names, allowance };
}
// ========================================
// --- PUBLIC ---------------------------
export const SHIT_1170_APP_PAGE_KEYS_ROWS = {
  rows_from_access_keys: rows_from_access_keys_fun,
};
// ========================================
// copyright 2026 by sleet.near
// ========================================
