// ========================================
// the keys page's url parameters: `?account=` and `?methods=`.
//
// a link to the add form should say what the key is for:
// `keys.html?account=dex.intear.near&methods=swap`
//
// --- WHY THE PARAMETERS ARE NAMED AFTER WHAT THEY HOLD: `?account=`
// is the account the key may call, and `?methods=` is the list of
// methods it may call. a link that says what the key is FOR is worth
// keeping whole.
//
// --- WHY IT DOES NOT REUSE THE SHARED `?id=` READER: that reader is
// for a page that shows ONE thing — a token, an account, a dao — and
// this page's `?account=` is not the page's subject. it is a PREFILL
// for one box in the form, so it gets its own name. a link that said
// `?id=` on this page would be ambiguous: is that the account the key
// calls, or the account whose keys are on screen?
//
// --- WHY THE METHODS ARE A COMMA SEPARATED STRING: it is the same
// list the box holds and the same list the chain wants, so the url
// carries the chain's own shape and nothing has to be translated.
//
// --- WHY IT IMPORTS NOTHING: the rule is the whole of it.
const URL_ACCOUNT_PARAM = "account";
const URL_METHODS_PARAM = "methods";
// ========================================
// ========================================
// --- HELPERS ----------------------------
// trim a raw url value, treating blank as absent.
//
// --- WHY --- blank is treated as absent: a trailing `?account=` link
// points at nothing, and an empty box is not a value.
function normalize_param_fun(raw: string | null): string | null {
  if (raw === null) return null;
  const trimmed = raw.trim();
  return trimmed.length === 0 ? null : trimmed;
}
// ========================================
// ========================================
// --- READ ------------------------------
// the `?account=` off the current url, or null when there is none.
//
// --- WHY --- window checks: the module is imported by a component that
// runs in the browser, but the ts is also loaded in tests and on the
// server, where there is no window.
function read_url_account_fun(): string | null {
  if (typeof window === "undefined" || typeof window.location === "undefined") {
    return null;
  }
  const params = new URLSearchParams(window.location.search);
  return normalize_param_fun(params.get(URL_ACCOUNT_PARAM));
}
// ========================================
// the `?methods=` off the current url, or null when there is none.
//
// --- WHY THE RAW STRING AND NOT A LIST: the caller puts it in a text
// box, and the box holds a comma separated line. splitting it here and
// joining it again would be a round trip that can only lose.
function read_url_methods_fun(): string | null {
  if (typeof window === "undefined" || typeof window.location === "undefined") {
    return null;
  }
  const params = new URLSearchParams(window.location.search);
  return normalize_param_fun(params.get(URL_METHODS_PARAM));
}
// ========================================
// ========================================
// --- PUBLIC ---------------------------
export const SHIT_1170_APP_PAGE_KEYS_URL = {
  read_account: read_url_account_fun,
  read_methods: read_url_methods_fun,
  normalize: normalize_param_fun,
};
// ========================================
// copyright 2026 by sleet.near
// ========================================
