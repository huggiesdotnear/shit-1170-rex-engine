// ========================================
// the keys page's key list: every access key the signed-in account has,
// with the full-access keys on top.
//
// --- WHY THE FULL-ACCESS KEYS ARE ON TOP AND NOT PICKABLE: the
// full-access key IS the account — it can do anything the account can
// do. removing it locks the reader out of their own account, and it is
// very often the key the wallet itself signs with. so it is shown
// first, where the eye lands, and it is never offered for removal.
//
// --- WHY THE LIMITED KEYS ARE PICKABLE: a limited key can only call
// named methods on one contract. it is safe to hand to a bot or a
// teammate, and it is safe to throw away — which is exactly what the
// reader needs when a bot is gone or a key leaked.
//
// mounts into <div id="SHIT_1170_COMPONENT_APP_PAGE_KEYS_LIST">.
import { render } from "preact";
import { useState as use_state } from "preact/hooks";
import { SHIT_1170_MOUNT_ID } from "../../ts/shared/mount_ids";
import { SHIT_1170_APP_PAGE_KEYS_STORE } from "../../ts/keys/app_page_keys_store";
import { use_page_store_fun } from "../../ts/shared/use_page_store";
import { SHIT_1170_AUTH } from "../../ts/shared/auth_status";
import { SHIT_1170_APP_PAGE_KEYS_REMOVE } from "../../ts/keys/app_page_keys_remove";
import { SHIT_1170_TOAST_STORE } from "../../ts/shared/toast_store";
// ========================================
// ========================================
// --- HELPERS ----------------------------
// the methods of a limited key, in one line.
//
// --- WHY AN EMPTY LIST SAYS "ANY METHOD": on the chain, a function-call
// key with no method names may call EVERY method on its contract. a
// reader looking at the list would otherwise read an empty list as
// "nothing", which is the opposite of what it means.
function methods_text_fun(method_names: ReadonlyArray<string>): string {
  if (method_names.length === 0) return "ANY METHOD";
  return method_names.join(", ");
}
// the allowance of a limited key, in one line.
//
// --- WHY THE ALLOWANCE IS IN YOCTONEAR AND NOT IN NEAR: the chain
// stores the allowance in yoctoNEAR and a rounding to NEAR would show
// "0" for every allowance that is not worth a whole NEAR — which is
// almost all of them. so the raw number is shown.
function allowance_text_fun(allowance: string | null): string {
  if (allowance === null) return "NO ALLOWANCE";
  if (allowance === "0") return "NO ALLOWANCE";
  return `${allowance} YOCTONEAR`;
}
// ========================================
// ========================================
// --- COMPONENT --------------------------
function SHIT_1170_COMPONENT_APP_PAGE_KEYS_LIST_FUN() {
  const state = use_page_store_fun(SHIT_1170_APP_PAGE_KEYS_STORE);
  const account_id = SHIT_1170_AUTH.get_account_id();
  const picked = SHIT_1170_APP_PAGE_KEYS_STORE.get_picked();
  const busy = SHIT_1170_APP_PAGE_KEYS_STORE.is_busy();
  const [removing, set_removing] = use_state(false);

  if (account_id === null) {
    return (
      <div class="keys-list">
        <p class="keys-note">SIGN IN TO SEE YOUR ACCESS KEYS</p>
      </div>
    );
  }

  const remove_fun = async () => {
    if (removing || picked.length === 0) return;
    set_removing(true);
    try {
      const result = await SHIT_1170_APP_PAGE_KEYS_STORE.run_action(
        async () => {
          const outcome = await SHIT_1170_APP_PAGE_KEYS_REMOVE.remove(picked);
          if (outcome.declined) return null;
          if (outcome.error !== null) return outcome.error;
          SHIT_1170_APP_PAGE_KEYS_STORE.clear_picked();
          SHIT_1170_TOAST_STORE.show(
            "success",
            "KEYS REMOVED",
            `${outcome.removed} KEY${outcome.removed === 1 ? "" : "S"} GONE`,
          );
          return null;
        },
      );
      if (result !== null) {
        SHIT_1170_TOAST_STORE.show(
          "error",
          "COULD NOT REMOVE THE KEYS",
          result,
        );
      }
    } finally {
      set_removing(false);
    }
  };

  if (state === null) {
    return (
      <div class="keys-list">
        <p class="keys-note">LOADING…</p>
      </div>
    );
  }

  const full_access = state.keys.filter((row) => row.full_access);
  const limited = state.keys.filter((row) => !row.full_access);

  return (
    <div class="keys-list">
      {state.error !== null && <p class="keys-note is-error">{state.error}</p>}

      {state.keys.length === 0 && state.error === null && (
        <p class="keys-note">NO ACCESS KEYS ON THIS ACCOUNT</p>
      )}

      {full_access.length > 0 && (
        <>
          <h3 class="keys-group">FULL ACCESS</h3>
          <p class="keys-note">
            THESE KEYS ARE THE ACCOUNT ITSELF. THEY CANNOT BE REMOVED FROM HERE.
          </p>
          {full_access.map((row) => (
            <div class="keys-row is-full-access" key={row.public_key}>
              <div class="keys-row-head">
                <span class="keys-kind">FULL ACCESS</span>
                <span class="keys-nonce">NONCE {row.nonce}</span>
              </div>
              <code class="keys-public-key">{row.public_key}</code>
            </div>
          ))}
        </>
      )}

      {limited.length > 0 && (
        <>
          <h3 class="keys-group">LIMITED</h3>
          <p class="keys-note">
            THESE KEYS CAN ONLY CALL THE CONTRACT AND THE METHODS BELOW. PICK
            THE ONES YOU WANT GONE.
          </p>
          {limited.map((row) => {
            const is_picked = picked.includes(row.public_key);
            return (
              <label
                class={`keys-row is-limited${is_picked ? " is-picked" : ""}`}
                key={row.public_key}
              >
                <input
                  type="checkbox"
                  checked={is_picked}
                  disabled={busy}
                  onChange={() => {
                    SHIT_1170_APP_PAGE_KEYS_STORE.toggle_pick(row);
                  }}
                />
                <div class="keys-row-body">
                  <div class="keys-row-head">
                    <span class="keys-kind">
                      {row.receiver_id ?? "NO CONTRACT"}
                    </span>
                    <span class="keys-nonce">NONCE {row.nonce}</span>
                  </div>
                  <code class="keys-public-key">{row.public_key}</code>
                  <p class="keys-detail">
                    METHODS: {methods_text_fun(row.method_names)}
                  </p>
                  <p class="keys-detail">
                    ALLOWANCE: {allowance_text_fun(row.allowance)}
                  </p>
                </div>
              </label>
            );
          })}
        </>
      )}

      {limited.length > 0 && (
        <div class="keys-actions">
          <button
            type="button"
            disabled={picked.length === 0 || busy || removing}
            onClick={() => {
              void remove_fun();
            }}
            title={
              picked.length === 0
                ? "pick the keys you want gone first"
                : "remove the picked keys"
            }
          >
            {removing
              ? "WORKING…"
              : picked.length === 0
                ? "REMOVE THE PICKED KEYS"
                : `REMOVE ${picked.length} KEY${picked.length === 1 ? "" : "S"}`}
          </button>
          {picked.length > 0 && (
            <button
              type="button"
              class="keys-clear"
              disabled={busy || removing}
              onClick={() => {
                SHIT_1170_APP_PAGE_KEYS_STORE.clear_picked();
              }}
            >
              UNPICK
            </button>
          )}
        </div>
      )}
    </div>
  );
}
// ========================================
// --- MOUNT ------------------------------
// self-mount guarded by id check.
const SELF_MOUNT_ID = SHIT_1170_MOUNT_ID.SHIT_1170_COMPONENT_APP_PAGE_KEYS_LIST;
if (typeof document !== "undefined") {
  const el = document.getElementById(SELF_MOUNT_ID);
  if (el !== null) render(<SHIT_1170_COMPONENT_APP_PAGE_KEYS_LIST_FUN />, el);
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
