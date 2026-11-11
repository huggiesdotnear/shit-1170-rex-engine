// ========================================
// the words after the middleman's own error prefix.
//
// --- WHY THE PREFIX IS CUT AT THE FIRST COLON AND NOT THE LAST: the
// detail itself carries colons — "invalid_request: bad_key_action: ..." —
// and cutting at the last one would throw the codes away.
export function shit_1170_app_page_keys_queue_strip_wrapper_fun(
  message: string,
): string {
  const marker = "api_shit_1170_tx_queue_post failed";
  if (!message.startsWith(marker)) return message;
  const after = message.slice(marker.length);
  // the wire failure is "<fallback> (<status> <statusText>): <detail>",
  // and the local check is "<fallback>: <detail>".
  if (after.startsWith(" (")) {
    const close = after.indexOf("): ");
    if (close !== -1) return after.slice(close + 3);
  }
  if (after.startsWith(": ")) return after.slice(2);
  return after;
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
