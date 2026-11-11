// ========================================
// the keys page's shared state: the loaded keys, the keys the reader
// picked, and the state of the two actions — adding a key and removing
// the picked ones.
//
// the components on this page do not fetch. they read this store, so
// there is exactly one loader (lib/ts/keys/app_page_keys_load.ts).
//
// --- WHY THE PICKED KEYS LIVE IN THE STORE AND NOT IN A COMPONENT:
// the list component picks the keys and the remove button removes
// them, and those are two different components. the picked set is
// shared state, so it belongs here with the rest of it.
import { SHIT_1170_APP_PAGE_KEYS } from "./app_page_keys_load";
import { SHIT_1170_AUTH } from "../shared/auth_status";
import { SHIT_1170_KEY_VAULT } from "./app_page_keys_vault";
import type {
  SHIT_1170_APP_PAGE_KEYS_DATA,
  SHIT_1170_APP_PAGE_KEYS_ROW,
} from "./app_page_keys_types";
// ========================================
// ========================================
// --- STATE ------------------------------
// the loaded data, or null while it is still loading.
let data: SHIT_1170_APP_PAGE_KEYS_DATA | null = null;
// the public keys the reader picked for removal. a Set, because a key
// is either picked or not and there is no order to it.
const picked: Set<string> = new Set();
// true while a load is running.
let loading = false;
// true while an action (add or remove) is running, so the buttons can
// say WORKING and stay off.
let busy = false;
// true once the account is being watched, so watch() is set up once.
let watching = false;
// the plain array so subscribe/unsubscribe ordering is stable.
const listeners: Array<() => void> = [];
// ========================================
// ========================================
// --- HELPERS ----------------------------
// broadcast a change to every subscriber; one throwing cannot break the notify.
function notify_fun(): void {
  for (const listener of [...listeners]) {
    try {
      listener();
    } catch (err) {
      console.error("[app_page_keys_store] listener failed", err);
    }
  }
}
// register a re-render callback; returns an unsubscribe to clean up on unmount.
function subscribe_fun(listener: () => void): () => void {
  listeners.push(listener);
  return () => {
    const index = listeners.indexOf(listener);
    if (index !== -1) listeners.splice(index, 1);
  };
}
// ========================================
// --- LOAD -------------------------------
// the load runs once per account. a second call while a load is running
// is ignored, so the page cannot fetch twice because two components
// asked for it.
async function app_page_keys_store_load_fun(): Promise<void> {
  const account_id = SHIT_1170_AUTH.get_account_id();
  if (loading) return;
  loading = true;
  notify_fun();
  try {
    data = await SHIT_1170_APP_PAGE_KEYS.load(account_id);
  } catch (err) {
    data = {
      account_id,
      keys: [],
      error: err instanceof Error ? err.message : String(err),
    };
  } finally {
    loading = false;
    notify_fun();
  }
}
// ========================================
// ========================================
// --- PICK ------------------------------
// pick or unpick one limited key.
//
// --- WHY ONLY LIMITED KEYS CAN BE PICKED: a full-access key IS the
// account. removing it locks the reader out of their own account, and
// it is the one key the wallet itself signs with. so the full-access
// key is never offered for removal, and the check lives here as well
// as in the component — the component hides the checkbox, and this
// refuses the key if it is asked for anyway.
function toggle_pick_fun(row: SHIT_1170_APP_PAGE_KEYS_ROW): void {
  if (row.full_access) return;
  if (picked.has(row.public_key)) {
    picked.delete(row.public_key);
  } else {
    picked.add(row.public_key);
  }
  notify_fun();
}
// unpick everything.
function clear_picked_fun(): void {
  if (picked.size === 0) return;
  picked.clear();
  notify_fun();
}
// the picked public keys.
function get_picked_fun(): ReadonlyArray<string> {
  return [...picked];
}
// ========================================
// ========================================
// --- ACTION ---------------------------
// run one of the page's actions — add a key, remove the picked ones —
// and reload the list when it worked.
//
// --- WHY THE RELOAD IS INSIDE THE ACTION: the chain is the only
// truth about the keys, and a local guess about what the chain now
// holds is a guess. so every action ends with a fresh load.
async function run_action_fun(
  action: () => Promise<string | null>,
): Promise<string | null> {
  if (busy) return null;
  busy = true;
  notify_fun();
  try {
    const error = await action();
    await app_page_keys_store_load_fun();
    return error;
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    await app_page_keys_store_load_fun();
    return message;
  } finally {
    busy = false;
    notify_fun();
  }
}
// ========================================
// ========================================
// --- WATCH -----------------------------
// follow the signed-in account.
//
// --- WHY THE SUBSCRIPTION IS SET UP ONCE: a second watch() would add a
// second listener, and the keys would load twice per change.
//
// --- WHY THE KEYS FOLLOW THE ACCOUNT: they belong to the account. a
// sign-in, a sign-out or a switch to another account means a different
// set of keys, so the page reloads by itself.
function app_page_keys_store_watch_fun(): void {
  if (watching) return;
  watching = true;
  const unsubscribe = SHIT_1170_AUTH.subscribe(() => {
    void SHIT_1170_APP_PAGE_KEYS_STORE.load();
  });
  // the subscription lives as long as the page does, so it is never
  // unsubscribed. the module is loaded once per document.
  void unsubscribe;
}
// ========================================
// ========================================
// --- FORGET ----------------------------
// drop one key from the browser's record.
//
// --- WHY FORGETTING IS ITS OWN ACTION AND NOT PART OF THE REMOVAL:
// the removal takes the key off the CHAIN, and it leaves the record
// alone — the record holds the reader's private key, and the reader may
// still want it. forgetting is the reader saying they want the secret
// gone from this browser, and that is a different decision from taking
// the key off the account.
//
// --- WHY FORGETTING DOES NOT TOUCH THE CHAIN: the key on the chain is
// not this browser's to remove. the reader removes it with the REMOVE
// button, and forgets it with this one.
async function forget_fun(public_key: string): Promise<void> {
  if (busy) return;
  busy = true;
  notify_fun();
  try {
    await SHIT_1170_KEY_VAULT.forget(public_key);
    await app_page_keys_store_load_fun();
  } finally {
    busy = false;
    notify_fun();
  }
}
// ========================================
// ========================================
// --- READ ------------------------------
function get_fun(): SHIT_1170_APP_PAGE_KEYS_DATA | null {
  return data;
}
// true while a load is running.
function is_loading_fun(): boolean {
  return loading;
}
// true while an action is running.
function is_busy_fun(): boolean {
  return busy;
}
// ========================================
// --- PUBLIC ---------------------------
export const SHIT_1170_APP_PAGE_KEYS_STORE = {
  load: app_page_keys_store_load_fun,
  subscribe: subscribe_fun,
  get: get_fun,
  is_loading: is_loading_fun,
  is_busy: is_busy_fun,
  toggle_pick: toggle_pick_fun,
  clear_picked: clear_picked_fun,
  get_picked: get_picked_fun,
  run_action: run_action_fun,
  forget: forget_fun,
  watch: app_page_keys_store_watch_fun,
};
// ========================================
// copyright 2026 by sleet.near
// ========================================
