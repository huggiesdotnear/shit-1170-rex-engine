// ========================================
// the message the reader sees when the queue refuses a key action.
//
// --- WHY THE BACKEND'S OWN WORDS ARE SHOWN AND NOT THE MIDDLEMAN'S
// WRAPPER: the middleman's error text carries its own prefix — "
// api_shit_1170_tx_queue_post failed (400 Bad Request): " — and the
// words after it are the ones that say WHY. the reader is not here to
// read a wrapper's prefix.
//
// --- WHY THE CODE IS KEPT: `key_action_not_allowed` says the key may
// not be removed — it is a full-access key, or it is not there — and
// that is a different fact from a bad request. the code is put in front
// of the words so the reader knows which thing happened.
import { shit_1170_app_page_keys_queue_strip_wrapper_fun } from "./app_page_keys_queue_strip_wrapper";
// ========================================
export function shit_1170_app_page_keys_queue_error_message_fun(
  err: unknown,
): string {
  if (typeof err !== "object" || err === null) return String(err);
  const record = err as { code?: unknown; message?: unknown };
  const message = typeof record.message === "string" ? record.message : "";
  const code = typeof record.code === "string" ? record.code : "";
  const detail = shit_1170_app_page_keys_queue_strip_wrapper_fun(message);
  if (code === "key_action_not_allowed") {
    return `THIS KEY MAY NOT BE REMOVED BY THE QUEUE — ${detail}`;
  }
  if (code.length > 0) return `${code.toUpperCase()} — ${detail}`;
  return detail;
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
