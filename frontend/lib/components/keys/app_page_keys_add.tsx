// ========================================
// the keys page's add form: it makes a NEW LIMITED access key.
//
// --- WHY THE FORM IS ON THE PAGE AND NOT A POPUP: adding a key is a
// thing the reader comes here to do, so it has its own place on the
// page, above the list — the list says what the account has, and the
// form says what the reader can add to it.
//
// --- WHY THE KEY PAIR IS SHOWN BEFORE THE READER APPROVES ANYTHING:
// the reader is asked to sign a transaction that adds a key. they must
// be able to read THAT key first — its public key and its private key —
// so they know what they are approving and what they are taking away.
// so the pair is made FIRST, shown, and only then offered to the chain.
// the pair on the screen IS the pair that goes on the chain.
//
// --- WHY THE CONTRACT IS CHECKED: a function-call key calls ONE
// contract, and a contract that is not deployed cannot be called. so the
// key would be made, it would cost a transaction, and it could never do
// anything. the reader is told before they sign, not after.
//
// mounts into <div id="SHIT_1170_COMPONENT_APP_PAGE_KEYS_ADD">.
import { render } from "preact";
import { useState as use_state } from "preact/hooks";
import { SHIT_1170_MOUNT_ID } from "../../ts/shared/mount_ids";
import { SHIT_1170_AUTH } from "../../ts/shared/auth_status";
import { SHIT_1170_APP_PAGE_KEYS_STORE } from "../../ts/keys/app_page_keys_store";
import { SHIT_1170_APP_PAGE_KEYS_ADD } from "../../ts/keys/app_page_keys_add";
import { SHIT_1170_APP_PAGE_KEYS_CONTRACT } from "../../ts/keys/app_page_keys_contract_check";
import { SHIT_1170_TOAST_STORE } from "../../ts/shared/toast_store";
// ========================================
// ========================================
// --- HELPERS ----------------------------
// the methods, read out of the input.
//
// --- WHY THE METHODS ARE SPLIT ON THE COMMA: a function-call key names
// its methods as a list, and a comma separated line is the plainest way
// to type a list. the names are trimmed and the blanks are dropped, so
// a trailing comma costs nothing.
function parse_methods_fun(raw: string): string[] {
  return raw
    .split(",")
    .map((name) => name.trim())
    .filter((name) => name.length > 0);
}
// ========================================
// ========================================
// --- COMPONENT --------------------------
function SHIT_1170_COMPONENT_APP_PAGE_KEYS_ADD_FUN() {
  const account_id = SHIT_1170_AUTH.get_account_id();
  const busy = SHIT_1170_APP_PAGE_KEYS_STORE.is_busy();
  const [receiver_id, set_receiver_id] = use_state("");
  const [methods, set_methods] = use_state("");
  const [adding, set_adding] = use_state(false);
  // the pair the reader is shown BEFORE they approve anything. null
  // until they ask for one.
  const [pair, set_pair] = use_state<{
    public_key: string;
    secret_key: string;
  } | null>(null);
  // the message about the contract, and whether it is good.
  const [contract_error, set_contract_error] = use_state<string | null>(null);
  const [contract_ok, set_contract_ok] = use_state(false);
  const [checking, set_checking] = use_state(false);
  const [error, set_error] = use_state<string | null>(null);

  if (account_id === null) {
    return (
      <div class="keys-add">
        <p class="keys-note">SIGN IN TO ADD AN ACCESS KEY</p>
      </div>
    );
  }

  const check_fun = async () => {
    set_checking(true);
    set_contract_ok(false);
    set_contract_error(null);
    try {
      const result = await SHIT_1170_APP_PAGE_KEYS_CONTRACT.check(receiver_id);
      set_contract_error(result.error);
      set_contract_ok(result.error === null);
    } finally {
      set_checking(false);
    }
  };

  const make_pair_fun = () => {
    set_pair(SHIT_1170_APP_PAGE_KEYS_ADD.make_pair());
    set_error(null);
  };

  const add_fun = async () => {
    if (adding) return;
    if (pair === null) {
      set_error("MAKE THE KEY FIRST");
      return;
    }
    set_adding(true);
    set_error(null);
    try {
      const result = await SHIT_1170_APP_PAGE_KEYS_STORE.run_action(
        async () => {
          const outcome = await SHIT_1170_APP_PAGE_KEYS_ADD.add(
            pair,
            receiver_id.trim(),
            parse_methods_fun(methods),
          );
          if (outcome.declined) return null;
          if (outcome.error !== null) return outcome.error;
          set_pair(null);
          set_receiver_id("");
          set_methods("");
          set_contract_ok(false);
          set_contract_error(null);
          SHIT_1170_TOAST_STORE.show(
            "success",
            "KEY ADDED",
            "THE PRIVATE KEY IS SAVED IN THIS BROWSER TOO",
            outcome.tx_hash,
          );
          return null;
        },
      );
      if (result !== null) set_error(result);
    } finally {
      set_adding(false);
    }
  };

  const copy_fun = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      SHIT_1170_TOAST_STORE.show("success", "COPIED", "");
    } catch {
      SHIT_1170_TOAST_STORE.show(
        "error",
        "COULD NOT COPY",
        "SELECT THE TEXT AND COPY IT BY HAND",
      );
    }
  };

  return (
    <div class="keys-add">
      <h3 class="keys-group">ADD A LIMITED KEY</h3>
      <p class="keys-note">
        A LIMITED KEY CAN ONLY CALL ONE CONTRACT. THE PRIVATE KEY IS SHOWN HERE
        AND SAVED IN THIS BROWSER, SO YOU CAN COME BACK FOR IT.
      </p>

      <label class="keys-field">
        <span>CONTRACT THE KEY CALLS</span>
        <input
          type="text"
          value={receiver_id}
          placeholder="contract.near"
          disabled={adding || busy}
          onInput={(event) => {
            const next = (event.currentTarget as HTMLInputElement).value;
            set_receiver_id(next);
            // --- WHY THE CHECK IS CLEARED ON EVERY CHANGE: a contract
            // that was checked is not a contract that is checked AFTER
            // the reader changed it.
            set_contract_ok(false);
            set_contract_error(null);
          }}
        />
      </label>

      {contract_error !== null && (
        <p class="keys-note is-error">{contract_error}</p>
      )}
      {contract_ok && <p class="keys-note is-good">CONTRACT OK</p>}

      <button
        type="button"
        class="keys-check"
        disabled={adding || busy || checking || receiver_id.trim().length === 0}
        onClick={() => {
          void check_fun();
        }}
      >
        {checking ? "CHECKING…" : "CHECK THE CONTRACT"}
      </button>

      <label class="keys-field">
        <span>METHODS (COMMA SEPARATED, EMPTY MEANS ANY)</span>
        <input
          type="text"
          value={methods}
          placeholder="method_one,method_two"
          disabled={adding || busy}
          onInput={(event) => {
            set_methods((event.currentTarget as HTMLInputElement).value);
          }}
        />
      </label>

      {pair === null ? (
        <button
          type="button"
          disabled={adding || busy}
          onClick={() => {
            make_pair_fun();
          }}
        >
          MAKE THE KEY
        </button>
      ) : (
        <>
          <div class="keys-preview">
            <p class="keys-detail">PUBLIC KEY</p>
            <code class="keys-public-key">{pair.public_key}</code>
            <p class="keys-detail">PRIVATE KEY</p>
            <code class="keys-secret-key">{pair.secret_key}</code>
            <p class="keys-detail">
              THIS KEY WILL CALL: {receiver_id.trim() || "NO CONTRACT"} —
              METHODS:{" "}
              {parse_methods_fun(methods).length === 0
                ? "ANY"
                : parse_methods_fun(methods).join(", ")}
            </p>
            <div class="keys-made-actions">
              <button
                type="button"
                onClick={() => {
                  void copy_fun(pair.secret_key);
                }}
              >
                COPY THE PRIVATE KEY
              </button>
              <button
                type="button"
                class="keys-clear"
                disabled={adding || busy}
                onClick={() => {
                  set_pair(null);
                }}
              >
                MAKE ANOTHER
              </button>
            </div>
          </div>

          <button
            type="button"
            disabled={adding || busy || !contract_ok}
            onClick={() => {
              void add_fun();
            }}
            title={
              contract_ok
                ? "add this key to the account"
                : "check the contract first"
            }
          >
            {adding ? "WORKING…" : "ADD THE KEY"}
          </button>
        </>
      )}

      {error !== null && <p class="keys-note is-error">{error}</p>}
    </div>
  );
}
// ========================================
// --- MOUNT ------------------------------
// self-mount guarded by id check.
const SELF_MOUNT_ID = SHIT_1170_MOUNT_ID.SHIT_1170_COMPONENT_APP_PAGE_KEYS_ADD;
if (typeof document !== "undefined") {
  const el = document.getElementById(SELF_MOUNT_ID);
  if (el !== null) render(<SHIT_1170_COMPONENT_APP_PAGE_KEYS_ADD_FUN />, el);
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
