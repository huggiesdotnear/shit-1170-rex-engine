// ========================================
// a check of the feed page's SAVED filter and its url.
//
// it runs the feed's own url and saved modules against a FAKE window
// and a FAKE localStorage, so the behaviour a reader sees is checked
// without a browser.
//
// it is a plain script and not a test file: it fakes the browser's
// globals, and that belongs in its own file rather than in the app's.
import { SHIT_1170_APP_PAGE_FEED_URL } from "../frontend/lib/ts/feed/app_page_feed_url";
import { SHIT_1170_APP_PAGE_FEED_SAVED } from "../frontend/lib/ts/feed/app_page_feed_saved";
// ========================================
// --- THE FAKE BROWSER ------------------
let fake_search = "";
const fake_storage = new Map<string, string>();

(globalThis as any).window = {
  get location() {
    return {
      get href() {
        return `http://localhost/app/index.html${fake_search}`;
      },
      get search() {
        return fake_search;
      },
    };
  },
  history: {
    replaceState: (_a: unknown, _b: string, url: string) => {
      const parsed = new URL(url);
      fake_search = parsed.search;
    },
  },
};

(globalThis as any).localStorage = {
  getItem: (key: string) => fake_storage.get(key) ?? null,
  setItem: (key: string, value: string) => {
    fake_storage.set(key, value);
  },
  removeItem: (key: string) => {
    fake_storage.delete(key);
  },
};
// ========================================
// --- THE CHECKS ------------------------
let failed = 0;
let passed = 0;

function check(label: string, ok: boolean, detail: string): void {
  if (ok) {
    passed++;
    console.log(`  ok   ${label}${detail ? ` — ${detail}` : ""}`);
  } else {
    failed++;
    console.log(`  FAIL ${label}${detail ? ` — ${detail}` : ""}`);
  }
}

function set_url(search: string): void {
  fake_search = search;
}
// ========================================
console.log("=== the list params are comma separated and not encoded ===");
{
  set_url("");
  SHIT_1170_APP_PAGE_FEED_URL.write_filter(
    false,
    null,
    "calls",
    ["ref.finance.near", "nft.near"],
    ["sleet.near", "alice.near"],
    null,
  );
  check(
    "the url carries the lists with plain commas",
    fake_search.includes("accounts=sleet.near,alice.near"),
    fake_search,
  );
  check(
    "the contracts carry plain commas too",
    fake_search.includes("contracts=ref.finance.near,nft.near"),
    fake_search,
  );
  check(
    "nothing is percent encoded",
    !fake_search.includes("%2C"),
    fake_search,
  );
  check(
    "the type is on the url",
    fake_search.includes("type=calls"),
    fake_search,
  );
  check(
    "the defaults are left off",
    !fake_search.includes("sort=") && !fake_search.includes("dir="),
    fake_search,
  );
}

console.log("=== the lists are read back off the url ===");
{
  set_url("?contracts=a.near,b.near&accounts=c.near,d.near");
  const contracts = SHIT_1170_APP_PAGE_FEED_URL.read_contracts();
  const accounts = SHIT_1170_APP_PAGE_FEED_URL.read_accounts();
  check(
    "the contracts come back as a list",
    contracts !== null && contracts.length === 2 && contracts[0] === "a.near",
    JSON.stringify(contracts),
  );
  check(
    "the accounts come back as a list",
    accounts !== null && accounts.length === 2 && accounts[0] === "c.near",
    JSON.stringify(accounts),
  );
}

console.log("=== an absent list and an empty list are different ===");
{
  set_url("");
  check(
    "an absent list is null",
    SHIT_1170_APP_PAGE_FEED_URL.read_contracts() === null,
  );
  set_url("?contracts=");
  const empty = SHIT_1170_APP_PAGE_FEED_URL.read_contracts();
  check(
    "an empty list is an empty list and not null",
    empty !== null && empty.length === 0,
    JSON.stringify(empty),
  );
}

console.log("=== the saved filter survives a round trip ===");
{
  fake_storage.clear();
  SHIT_1170_APP_PAGE_FEED_SAVED.write({
    hide_sent: true,
    sort: "most_popular",
    direction: "asc",
    type: "swaps",
    contracts: ["ref.finance.near"],
    accounts: ["sleet.near"],
  });
  const read = SHIT_1170_APP_PAGE_FEED_SAVED.read();
  check("the saved filter comes back", read !== null);
  check("the switch came back", read?.hide_sent === true);
  check("the sort came back", read?.sort === "most_popular");
  check("the direction came back", read?.direction === "asc");
  check("the type came back", read?.type === "swaps");
  check(
    "the contracts came back",
    read?.contracts.length === 1 && read.contracts[0] === "ref.finance.near",
  );
  check(
    "the accounts came back",
    read?.accounts.length === 1 && read.accounts[0] === "sleet.near",
  );
}

console.log("=== a hand edited saved filter cannot break the app ===");
{
  fake_storage.clear();
  fake_storage.set("SHIT_1170_APP_PAGE_FEED_FILTER", "{not json at all");
  check(
    "a corrupt entry reads as nothing",
    SHIT_1170_APP_PAGE_FEED_SAVED.read() === null,
  );

  fake_storage.set(
    "SHIT_1170_APP_PAGE_FEED_FILTER",
    JSON.stringify({ sort: "not_a_sort" }),
  );
  check(
    "an unknown sort reads as nothing",
    SHIT_1170_APP_PAGE_FEED_SAVED.read() === null,
  );

  fake_storage.set(
    "SHIT_1170_APP_PAGE_FEED_FILTER",
    JSON.stringify({
      hide_sent: true,
      sort: "date_added",
      direction: "desc",
      type: "all",
      contracts: ["ok.near", 42],
      accounts: [],
    }),
  );
  check(
    "a number in a list reads as nothing",
    SHIT_1170_APP_PAGE_FEED_SAVED.read() === null,
  );

  fake_storage.set(
    "SHIT_1170_APP_PAGE_FEED_FILTER",
    JSON.stringify({
      hide_sent: "yes",
      sort: "date_added",
      direction: "desc",
      type: "all",
      contracts: [],
      accounts: [],
    }),
  );
  check(
    "a string where a boolean belongs reads as nothing",
    SHIT_1170_APP_PAGE_FEED_SAVED.read() === null,
  );
}
// ========================================
// --- THE REPORT -------------------------
console.log("");
console.log(`${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
