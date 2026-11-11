// ========================================
// the browser's RECORD of the access keys the reader made on this
// device.
//
// --- WHY THE BROWSER KEEPS A RECORD AT ALL: a limited key's PRIVATE
// key is shown once, when the key is made. if the reader did not copy
// it then, it is gone — and a limited key with no private key is a
// key nobody can sign with, so it is dead weight on the account and a
// reason to remove it. so the browser keeps the secret, and the keys
// page can show it again later.
//
// --- WHY THE CHAIN IS STILL THE TRUTH ABOUT WHAT EXISTS: this record
// says what the reader once made on this device. it does NOT say what
// the account has now — a key can be removed from another browser, or
// the account itself can be replaced. so the list on the page is always
// read from the chain, and this record only ADDS the private keys and
// the history. a stored key the chain no longer has is shown as
// REMOVED.
//
// --- WHY IT IS INDEXEDDB AND NOT LOCALSTORAGE: this holds SECRETS.
// localStorage is a plain string map that any script on the origin can
// read in one line, and it is synchronously copied into every backup
// of the profile. indexeddb is still readable by any script on the
// origin — nothing on the web can hide a secret from the page that made
// it — but it keeps the bytes out of the string map, out of the
// clipboard-friendly places, and out of a careless `localStorage`
// dump.
//
// --- WHY THE SECRET IS STORED AND NOT RECOMPUTED: a private key
// cannot be derived from a public key. it is either kept or it is
// gone. so the one moment the reader has it — the moment the key is
// made — is the only moment it can be kept.
//
// --- WHY A ROW IS NEVER DELETED BY A CHAIN LOAD: the chain does not
// know about this record. a chain load that deleted rows would erase
// the reader's history every time a key was removed from the account,
// and the reader would lose the secret of a key they still want to
// keep. only the reader can forget a key, and that is what the forget
// button is for.
//
// the indexeddb mechanics live in lib/ts/shared/indexed_db_store.ts —
// this file only says WHICH store this is and what a row looks like.
import { SHIT_1170_INDEXED_DB_STORE } from "../shared/indexed_db_store";
// ========================================
// --- CONFIG -----------------------------
// the vault's own database, its own store. it is NOT the token
// cache's database: the token cache is a cache and can be dropped,
// and this one holds secrets that must not be.
const SHIT_1170_KEYS_DB_NAME = "shit_1170_keys";
const SHIT_1170_KEYS_DB_VERSION = 1;
const SHIT_1170_KEYS_STORE = "access_keys";
// the vault's identity, handed to the shared store module.
const SHIT_1170_KEYS_STORE_CONFIG = {
  db_name: SHIT_1170_KEYS_DB_NAME,
  db_version: SHIT_1170_KEYS_DB_VERSION,
  store_name: SHIT_1170_KEYS_STORE,
};
// ========================================
// ========================================
// --- SHAPE ------------------------------
// one row in the access keys store.
//
// `id` is the public key itself: a key is unique on the chain, and the
// chain is where the key belongs. the account is carried along because
// the same key id can sit on more than one account over time — a
// reader with two accounts can make the same key on both — so the
// record is read per ACCOUNT, and the id is the public key.
export interface SHIT_1170_KEY_VAULT_ROW {
  id: string;
  // the account the key was made on.
  account_id: string;
  // the public key, the way the chain spells it.
  public_key: string;
  // the private key, the ONLY copy. null when the reader chose not to
  // keep it.
  secret_key: string | null;
  // the contract the key may call.
  receiver_id: string;
  // the methods the key may call. empty means ANY method.
  method_names: string[];
  // the allowance, in yoctoNEAR. null when the key has none.
  allowance_yocto: string | null;
  // when the key was made, in milliseconds since the epoch.
  created_at: number;
}
// ========================================
// ========================================
// --- READ ------------------------------
// every row for one account.
//
// --- WHY THE ROWS ARE FILTERED BY ACCOUNT AND NOT READ WHOLE: a
// reader with two accounts sees two different key lists, and the one
// for the signed-in account is the one on the page. the ids are
// fetched in one batch and the account filter is applied here, so the
// db is touched once.
//
// --- WHY A MISSING DB IS AN EMPTY LIST AND NOT AN ERROR: the vault is
// a convenience. a browser that cannot open it — private mode, a
// blocked database — must still show the chain's keys.
async function list_vault_rows_fun(
  account_id: string,
): Promise<SHIT_1170_KEY_VAULT_ROW[]> {
  const ids = await list_all_ids_fun();
  if (ids.length === 0) return [];
  const found =
    await SHIT_1170_INDEXED_DB_STORE.get_many<SHIT_1170_KEY_VAULT_ROW>(
      SHIT_1170_KEYS_STORE_CONFIG,
      ids,
    );
  const rows: SHIT_1170_KEY_VAULT_ROW[] = [];
  for (const row of found.values()) {
    if (row.account_id !== account_id) continue;
    rows.push(row);
  }
  return rows;
}
// ========================================
// every id in the store.
//
// --- WHY THE IDS ARE READ IN THEIR OWN PASS: the shared store reads
// BY id, so the whole store cannot be listed without knowing the ids
// first. the ids are read through a cursor in one transaction.
async function list_all_ids_fun(): Promise<string[]> {
  const ids: string[] = [];
  try {
    const db = await open_vault_db_fun();
    await new Promise<void>((resolve) => {
      const tx = db.transaction(SHIT_1170_KEYS_STORE, "readwrite");
      const request = tx.objectStore(SHIT_1170_KEYS_STORE).getAllKeys();
      request.onsuccess = () => {
        for (const key of request.result) {
          if (typeof key === "string") ids.push(key);
        }
      };
      tx.oncomplete = () => resolve();
      tx.onerror = () => resolve();
      tx.onabort = () => resolve();
    });
  } catch (err) {
    console.warn("[key_vault] listing the ids failed", err);
  }
  return ids;
}
// ========================================
// open the vault database.
//
// --- WHY THE OPEN IS CACHED HERE AND NOT IN THE SHARED MODULE: the
// shared module's `get_many` and `put_many` need only an id list, and
// the id list needs a cursor, which the shared module does not have.
// so the vault opens its own connection for the cursor and the shared
// module opens its own for the rows. both are the SAME database, so
// there is one store, reached two ways.
let db_promise: Promise<IDBDatabase> | null = null;
function open_vault_db_fun(): Promise<IDBDatabase> {
  if (db_promise !== null) return db_promise;
  db_promise = new Promise<IDBDatabase>((resolve, reject) => {
    const request = indexedDB.open(
      SHIT_1170_KEYS_DB_NAME,
      SHIT_1170_KEYS_DB_VERSION,
    );
    request.onupgradeneeded = () => {
      const db = request.result;
      if (!db.objectStoreNames.contains(SHIT_1170_KEYS_STORE)) {
        db.createObjectStore(SHIT_1170_KEYS_STORE, { keyPath: "id" });
      }
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
  return db_promise;
}
// ========================================
// ========================================
// --- WRITE -----------------------------
// keep one key. an existing row for the same public key is replaced,
// so a key made twice does not leave two rows behind.
async function put_vault_row_fun(row: SHIT_1170_KEY_VAULT_ROW): Promise<void> {
  await SHIT_1170_INDEXED_DB_STORE.put_many(SHIT_1170_KEYS_STORE_CONFIG, [row]);
}
// ========================================
// ========================================
// --- FORGET ----------------------------
// drop one key from the record. this is the ONLY thing that deletes a
// row — a chain load never does.
async function delete_vault_row_fun(public_key: string): Promise<void> {
  try {
    const db = await open_vault_db_fun();
    await new Promise<void>((resolve) => {
      const tx = db.transaction(SHIT_1170_KEYS_STORE, "readwrite");
      tx.objectStore(SHIT_1170_KEYS_STORE).delete(public_key);
      tx.oncomplete = () => resolve();
      tx.onerror = () => resolve();
      tx.onabort = () => resolve();
    });
  } catch (err) {
    console.warn("[key_vault] forgetting a key failed", err);
  }
}
// ========================================
// ========================================
// --- PUBLIC ---------------------------
// the vault surface. every function is safe to call in a non-browser —
// it resolves empty or does nothing — so a caller never has to guard.
export const SHIT_1170_KEY_VAULT = {
  list: list_vault_rows_fun,
  put: put_vault_row_fun,
  forget: delete_vault_row_fun,
};
// ========================================
// copyright 2026 by sleet.near
// ========================================
