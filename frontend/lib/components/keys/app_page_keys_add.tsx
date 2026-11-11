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
// --- WHY THE PAIR IS MADE ON PAGE LOAD: the key is made in the
// browser and it costs nothing to make, so the page opens with one
// already on screen. the reader reads it, copies the private key if
// they want it, fills the boxes, and approves. a MAKE THE KEY button
// in front of that would be a click that buys nothing — and the
// GENERATE ANOTHER KEY button behind it would have thrown the key away
// and sent the reader back for a second click.
//
// --- WHY THE ACCOUNT IS NOT CHECKED AGAINST THE CHAIN: the account a
// key may call does not have to be a contract — it can be any account
// the reader means, and they know which one they mean. so the app does
// not ask the chain whether code is deployed there. it asks ONE thing,
// and it asks it locally: is this string an account id the chain will
// accept? an id is 2 to 64 characters of lowercase letters, digits,
// dots, dashes and underscores, and the input lowercases as the reader
// types so a capital letter never gets that far.
//
// --- WHY THE FORM IS PREFILLED FROM THE URL: a link to this page
// should say what the key is for — `keys.html?account=dex.intear.near
// &methods=swap` — so the reader arrives with the boxes already
// holding what they meant.
//
// mounts into <div id="SHIT_1170_COMPONENT_APP_PAGE_KEYS_ADD">.
import { render } from "preact";
import { useState as use_state } from "preact/hooks";
import { SHIT_1170_MOUNT_ID } from "../../ts/shared/mount_ids";
import { SHIT_1170_AUTH } from "../../ts/shared/auth_status";
import { SHIT_1170_APP_PAGE_KEYS_STORE } from "../../ts/keys/app_page_keys_store";
import { SHIT_1170_APP_PAGE_KEYS_ADD } from "../../ts/keys/app_page_keys_add";
import { SHIT_1170_APP_PAGE_KEYS_ACCOUNT_ID } from "../../ts/keys/app_page_keys_account_id";
import { SHIT_1170_APP_PAGE_KEYS_ALLOWANCE } from "../../ts/keys/app_page_keys_allowance";
import { SHIT_1170_APP_PAGE_KEYS_URL } from "../../ts/keys/app_page_keys_url";
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
  // --- WHY THE URL IS READ AS THE INITIAL VALUE: the boxes start with
  // what the link said. a link that carried no parameters starts empty,
  // and the reader types.
  const [receiver_id, set_receiver_id] = use_state(
    SHIT_1170_APP_PAGE_KEYS_URL.read_account() ?? "",
  );
  const [methods, set_methods] = use_state(
    SHIT_1170_APP_PAGE_KEYS_URL.read_methods() ?? "",
  );
  // the allowance, as the reader typed it — in NEAR, not yoctoNEAR.
  const [allowance, set_allowance] = use_state(
    SHIT_1170_APP_PAGE_KEYS_URL.read_allowance() ?? "",
  );
  const [adding, set_adding] = use_state(false);
  // the pair the reader is shown BEFORE they approve anything.
  //
  // --- WHY IT IS MADE ON PAGE LOAD AND NOT BY A BUTTON: the key is
  // made in the browser, and it costs nothing to make. so the page
  // opens with a pair already on screen — the reader reads it, copies
  // the private key if they want it, fills the boxes, and approves.
  // a MAKE THE KEY button in front of that is a click that buys
  // nothing.
  //
  // --- WHY IT IS A LAZY INITIAL STATE: the argument to use_state is
  // only used on the FIRST render, so the key is made exactly once and
  // a re-render does not throw the reader's key away and make another.
  const [pair, set_pair] = use_state(() =>
    SHIT_1170_APP_PAGE_KEYS_ADD.make_pair(),
  );
  const [error, set_error] = use_state<string | null>(null);

  if (account_id === null) {
    return (
      <div class="keys-add">
        <p class="keys-note">SIGN IN TO ADD AN ACCESS KEY</p>
      </div>
    );
  }

  // the account the key will call, as the chain will see it.
  const receiver = SHIT_1170_APP_PAGE_KEYS_ACCOUNT_ID.normalize(receiver_id);
  // --- WHY THE CHECK IS OFFLINE: it is a rule about a string, not a
  // question about the chain. no network call, no waiting.
  const receiver_ok =
    receiver.length > 0 &&
    SHIT_1170_APP_PAGE_KEYS_ACCOUNT_ID.is_valid(receiver);
  const method_names = parse_methods_fun(methods);
  // the allowance, in yoctoNEAR. null means the key may attach
  // nothing.
  const allowance_yocto = SHIT_1170_APP_PAGE_KEYS_ALLOWANCE.parse(allowance);
  // --- WHY A TYPED NUMBER THAT DOES NOT PARSE IS AN ERROR AND NOT A
  // SILENT NO ALLOWANCE: a reader who types "1,5" and gets a key with
  // no allowance has been given something they did not ask for. the
  // box says what is wrong.
  const allowance_typed = allowance.trim().length > 0;
  const allowance_bad = allowance_typed && allowance_yocto === null;
  const allowance_human =
    SHIT_1170_APP_PAGE_KEYS_ALLOWANCE.format(allowance_yocto).human;

  // make a fresh pair, right away — the reader sees the new one the
  // moment they click.
  const make_pair_fun = () => {
    set_pair(SHIT_1170_APP_PAGE_KEYS_ADD.make_pair());
    set_error(null);
  };

  const add_fun = async () => {
    if (adding) return;
    if (!receiver_ok) {
      set_error("THE ACCOUNT THE KEY CALLS IS NOT A NEAR ACCOUNT ID");
      return;
    }
    set_adding(true);
    set_error(null);
    try {
      const result = await SHIT_1170_APP_PAGE_KEYS_STORE.run_action(
        async () => {
          const outcome = await SHIT_1170_APP_PAGE_KEYS_ADD.add(
            pair,
            receiver,
            method_names,
            allowance_yocto,
          );
          if (outcome.declined) return null;
          if (outcome.error !== null) return outcome.error;
          // --- WHY A FRESH KEY AFTER A SUCCESS: the key that was just
          // added is ON THE CHAIN now, so showing it again would offer
          // the reader the same key twice. a fresh one is on screen and
          // ready for the next key.
          set_pair(SHIT_1170_APP_PAGE_KEYS_ADD.make_pair());
          set_receiver_id("");
          set_methods("");
          set_allowance("");
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
        A LIMITED KEY CAN ONLY CALL ONE ACCOUNT. THE KEY BELOW IS MADE NOW AND
        IS SAVED IN THIS BROWSER, SO YOU CAN COME BACK FOR IT.
      </p>

      <label class="keys-field">
        <span>ACCOUNT THE KEY CALLS</span>
        <input
          type="text"
          value={receiver_id}
          placeholder="contract.near"
          spellcheck={false}
          autocapitalize="none"
          autocorrect="off"
          disabled={adding || busy}
          onInput={(event) => {
            // --- WHY THE INPUT LOWERCASES AS THE READER TYPES: a NEAR
            // account id is lowercase, and the chain rejects an id with
            // a capital letter in it. so the capital never gets that
            // far — the box shows the id the chain will see.
            const next = SHIT_1170_APP_PAGE_KEYS_ACCOUNT_ID.normalize(
              (event.currentTarget as HTMLInputElement).value,
            );
            set_receiver_id(next);
          }}
        />
      </label>

      {receiver_id.length > 0 && !receiver_ok && (
        <p class="keys-note is-error">
          THAT IS NOT A NEAR ACCOUNT ID — ONLY LOWERCASE LETTERS, DIGITS, DOTS,
          DASHES AND UNDERSCORES
        </p>
      )}

      <label class="keys-field">
        <span>METHODS (COMMA SEPARATED, EMPTY MEANS ANY)</span>
        <input
          type="text"
          value={methods}
          placeholder="method_one,method_two"
          spellcheck={false}
          disabled={adding || busy}
          onInput={(event) => {
            set_methods((event.currentTarget as HTMLInputElement).value);
          }}
        />
      </label>

      <label class="keys-field">
        <span>ALLOWANCE IN NEAR (EMPTY MEANS THE KEY CAN SPEND NOTHING)</span>
        <input
          type="text"
          value={allowance}
          placeholder="1.5"
          spellcheck={false}
          inputMode="decimal"
          disabled={adding || busy}
          onInput={(event) => {
            set_allowance((event.currentTarget as HTMLInputElement).value);
          }}
        />
      </label>

      {allowance_bad && (
        <p class="keys-note is-error">
          THAT IS NOT A NEAR AMOUNT — A NUMBER LIKE 1.5, AND NO MORE THAN SIX
          DECIMALS
        </p>
      )}
      {allowance_typed && !allowance_bad && (
        <p class="keys-note is-good">ALLOWANCE: {allowance_human}</p>
      )}

      <div class="keys-preview">
        <p class="keys-detail">PUBLIC KEY</p>
        <code class="keys-public-key">{pair.public_key}</code>
        <p class="keys-detail">PRIVATE KEY</p>
        <code class="keys-secret-key">{pair.secret_key}</code>
        <p class="keys-detail">
          THIS KEY WILL CALL: {receiver.length === 0 ? "NO ACCOUNT" : receiver}
          {" — METHODS: "}
          {method_names.length === 0 ? "ANY" : method_names.join(", ")}
          {" — ALLOWANCE: "}
          {allowance_human ?? "NOTHING"}
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
              make_pair_fun();
            }}
          >
            GENERATE ANOTHER KEY
          </button>
        </div>
      </div>

      <button
        type="button"
        disabled={adding || busy || !receiver_ok}
        onClick={() => {
          void add_fun();
        }}
        title={
          receiver_ok
            ? "add this key to the account"
            : "the account the key calls is not an account id"
        }
      >
        {adding ? "WORKING…" : "ADD THE KEY"}
      </button>

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
