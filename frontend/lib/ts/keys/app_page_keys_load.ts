// ========================================
// the keys page's ONE loader: it asks the chain for the access keys of
// the signed-in account.
//
// --- WHY THE RPC AND NOT OUR BACKEND: the access keys live on the
// chain, and the chain is the only thing that knows them. our backend
// keeps the app's api keys, which are a different thing entirely —
// they are not NEAR access keys and they are not on this page.
//
// --- WHY THE NEAR KIT CLIENT FROM THE TOOL BOX: the page reads the
// chain through the same client every other page reads the chain
// through — near_kit_client() from @near-kit-tool-box/web — so the
// network, the retry behaviour and the rpc url are the same on every
// page of the app.
//
// --- WHY THE ACCOUNT IS THE SIGNED-IN ONE: this page is about the
// reader's own keys. an account's access keys are readable by anyone,
// but there is no reason to show a stranger's.
import { near_kit_client } from "@near-kit-tool-box/web";
import { SHIT_1170_APP_PAGE_KEYS_ROWS } from "./app_page_keys_types";
import type {
  SHIT_1170_APP_PAGE_KEYS_DATA,
  SHIT_1170_APP_PAGE_KEYS_ROW,
} from "./app_page_keys_types";
// ========================================
// ========================================
// --- LOAD ------------------------------
// the access keys of one account.
//
// --- WHY A SIGNED-OUT ACCOUNT IS NOT AN ERROR: there is nothing to
// show and nothing to fail at. the page says SIGN IN and leaves it at
// that, so the load answers with an empty list and no error.
async function app_page_keys_load_fun(
  account_id: string | null,
): Promise<SHIT_1170_APP_PAGE_KEYS_DATA> {
  if (account_id === null) {
    return { account_id: null, keys: [], error: null };
  }
  try {
    const response = await near_kit_client().getAccessKeys(account_id);
    const keys: SHIT_1170_APP_PAGE_KEYS_ROW[] =
      SHIT_1170_APP_PAGE_KEYS_ROWS.rows_from_access_keys(response);
    return { account_id, keys, error: null };
  } catch (err) {
    return {
      account_id,
      keys: [],
      error: err instanceof Error ? err.message : String(err),
    };
  }
}
// ========================================
// --- PUBLIC ---------------------------
export const SHIT_1170_APP_PAGE_KEYS = {
  load: app_page_keys_load_fun,
};
// ========================================
// copyright 2026 by sleet.near
// ========================================
