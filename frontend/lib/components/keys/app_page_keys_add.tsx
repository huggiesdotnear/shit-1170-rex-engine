// ========================================
// the keys page's add form: it makes a NEW LIMITED access key.
//
// --- WHY THE FORM IS ON THE PAGE AND NOT A POPUP: adding a key is a
// thing the reader comes here to do, so it has its own place on the
// page, above the list — the list says what the account has, and the
// form says what the reader can add to it.
//
// --- WHY THE PRIVATE KEY IS SHOWN ONCE AND THEN FORGOTTEN: the reader
// has to take it away with them — it is the only copy. so it is on the
// screen when the key is made, with a copy button, and it is not kept
// anywhere in the app. a key that the app kept would be a key the app
// could lose.
//
// mounts into <div id="SHIT_1170_COMPONENT_APP_PAGE_KEYS_ADD">.
import { render } from "preact";
import { useState as use_state } from "preact/hooks";
import { SHIT_1170_MOUNT_ID } from "../../ts/shared/mount_ids";
import { SHIT_1170_AUTH } from "../../ts/shared/auth_status";
import { SHIT_1170_APP_PAGE_KEYS_STORE } from "../../ts/keys/app_page_keys_store";
import { SHIT_1170_APP_PAGE_KEYS_ADD } from "../../ts/keys/app_page_keys_add";
import { SHIT_1170_TOAST_STORE } from "../../ts/shared/toast_store";
// ========================================
// ========================================
// --- HELPERS ----------------------------
// the receiver and the methods, read out of the two inputs.
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
  const [made_key, set_made_key] = use_state<{
    public_key: string;
    secret_key: string;
    receiver_id: string;
    method_names: string[];
  } | null>(null);
  const [error, set_error] = use_state<string | null>(null);

  if (account_id === null) {
    return (
      <div class="keys-add">
        <p class="keys-note">SIGN IN TO ADD AN ACCESS KEY</p>
      </div>
    );
  }

  const add_fun = async () => {
    if (adding) return;
    const receiver = receiver_id.trim();
    if (receiver.length === 0) {
      set_error("THE CONTRACT THE KEY CALLS IS EMPTY");
      return;
    }
    set_adding(true);
    set_error(null);
    try {
      const result = await SHIT_1170_APP_PAGE_KEYS_STORE.run_action(
        async () => {
          const outcome = await SHIT_1170_APP_PAGE_KEYS_ADD.add(
            receiver,
            parse_methods_fun(methods),
          );
          if (outcome.error !== null) return outcome.error;
          set_made_key({
            public_key: outcome.public_key,
            secret_key: outcome.secret_key,
            receiver_id: outcome.receiver_id,
            method_names: outcome.method_names,
          });
          set_receiver_id("");
          set_methods("");
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
            set_receiver_id((event.currentTarget as HTMLInputElement).value);
          }}
        />
      </label>

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

      <button
        type="button"
        disabled={adding || busy}
        onClick={() => {
          void add_fun();
        }}
      >
        {adding ? "WORKING…" : "ADD THE KEY"}
      </button>

      {error !== null && <p class="keys-note is-error">{error}</p>}

      {made_key !== null && (
        <div class="keys-made">
          <p class="keys-note">
            THE PRIVATE KEY IS BELOW. COPY IT — IT IS ALSO SAVED IN THIS
            BROWSER, UNDER THE KEY LIST.
          </p>
          <p class="keys-detail">PUBLIC KEY</p>
          <code class="keys-public-key">{made_key.public_key}</code>
          <p class="keys-detail">PRIVATE KEY</p>
          <code class="keys-secret-key">{made_key.secret_key}</code>
          <p class="keys-detail">
            CONTRACT: {made_key.receiver_id} — METHODS:{" "}
            {made_key.method_names.length === 0
              ? "ANY"
              : made_key.method_names.join(", ")}
          </p>
          <div class="keys-made-actions">
            <button
              type="button"
              onClick={() => {
                void copy_fun(made_key.secret_key);
              }}
            >
              COPY THE PRIVATE KEY
            </button>
            <button
              type="button"
              class="keys-clear"
              onClick={() => {
                set_made_key(null);
              }}
            >
              CLOSE
            </button>
          </div>
        </div>
      )}
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
