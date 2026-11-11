// ========================================
// the keys page's ONE entry: the page's html loads this file and
// nothing else.
//
// the order below IS the page's order of life, and it matters:
//
// 1. import-app-wide brings the network pin (hello.ts) and the
//    app-wide components.
// 2. the start begins the page's load, before any component mounts.
// 3. the components mount last and read what the load brought back.
import "./import-app-wide";
import "./keys/app_page_keys_start";
import "../components/keys/app_page_keys_list";
import "../components/keys/app_page_keys_add";
// ========================================
// copyright 2026 by sleet.near
// ========================================
