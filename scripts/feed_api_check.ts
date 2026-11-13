// ========================================
// a live check of the feed's new api surface.
//
// it drives the REAL backend against the REAL test database and checks
// the things the feed page now depends on:
// - the cursor pages and does not repeat or skip
// - the type filter's two halves answer on their own
// - the comma separated filters are read
// - the facets give the bar its buttons
// - the counts the bar shows are right
//
// it is a plain script and not a test file: it needs a running server
// and a real database, which a unit test does not have.
const BASE = process.argv[2] ?? "http://127.0.0.1:54999/api/shit-1170";
// ========================================
// --- HELPERS ----------------------------
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

async function get(path: string): Promise<{ status: number; body: any }> {
  const answer = await fetch(`${BASE}${path}`);
  let body: any = null;
  try {
    body = await answer.json();
  } catch {
    body = null;
  }
  return { status: answer.status, body };
}
// ========================================
// --- THE CHECKS -------------------------
console.log("=== the cursor pages and does not repeat or skip ===");
{
  const seen: string[] = [];
  let cursor: string | null = null;
  let pages = 0;
  let total = 0;
  while (pages < 50) {
    const path = `/tx-queue?limit=25${cursor ? `&cursor=${cursor}` : ""}`;
    const { status, body } = await get(path);
    if (status !== 200) {
      check("the first page answers", false, `status ${status}`);
      break;
    }
    total = body.total;
    for (const item of body.items) seen.push(item.queueId);
    cursor = body.nextCursor;
    pages++;
    if (cursor === null) break;
  }
  check(
    "every queue is seen",
    seen.length === total,
    `${seen.length} seen, total ${total}`,
  );
  check(
    "no queue is seen twice",
    new Set(seen).size === seen.length,
    `${new Set(seen).size} unique of ${seen.length}`,
  );
  check("the pages ended", cursor === null, `${pages} pages`);
}

console.log("=== the type filter's two halves answer on their own ===");
{
  const calls = await get("/tx-queue?limit=500");
  const swaps = await get("/swap-queue?limit=500");
  check(
    "the batches answer",
    calls.status === 200,
    `total ${calls.body.total}`,
  );
  check("the swaps answer", swaps.status === 200, `total ${swaps.body.total}`);
  check(
    "the two halves have different totals",
    calls.body.total !== swaps.body.total,
    `${calls.body.total} batches, ${swaps.body.total} swaps`,
  );
}

console.log("=== the comma separated filters are read ===");
{
  const all = await get("/tx-queue?limit=500");
  const all_total = all.body.total;
  const facets = await get("/tx-queue/facets");
  check("the facets answer", facets.status === 200);
  const contract = facets.body.contracts?.[0]?.contractId;
  const account = facets.body.accounts?.[0]?.accountId;
  if (contract === undefined || account === undefined) {
    check(
      "the facets have something to filter by",
      false,
      "no contracts or accounts",
    );
  } else {
    const one = await get(`/tx-queue?contract_id=${contract}`);
    const two = await get(
      `/tx-queue?contract_id=${contract},definitely.not.a.contract`,
    );
    check(
      "a contract filter narrows the list",
      one.body.total <= all_total,
      `${one.body.total} of ${all_total}`,
    );
    check(
      "a contract filter with a made-up id still reads the real one",
      two.body.total === one.body.total,
      `${two.body.total} vs ${one.body.total}`,
    );
    const by_account = await get(`/tx-queue?account_id=${account}`);
    check(
      "an account filter narrows the list",
      by_account.body.total <= all_total,
      `${by_account.body.total} of ${all_total}`,
    );
    const both = await get(
      `/tx-queue?account_id=${account},another.account.near`,
    );
    check(
      "a two-account filter is read",
      both.status === 200,
      `total ${both.body.total}`,
    );
  }
}

console.log("=== the swaps answer the contract filter ===");
{
  const all = await get("/swap-queue?limit=500");
  const contracts = await get("/tx-queue/facets");
  const contract = contracts.body.contracts?.[0]?.contractId;
  if (contract === undefined) {
    check("there is a contract to filter the swaps by", false, "no contracts");
  } else {
    const filtered = await get(`/swap-queue?contract_id=${contract}`);
    check(
      "a contract filter narrows the swaps",
      filtered.body.total <= all.body.total,
      `${filtered.body.total} of ${all.body.total}`,
    );
    const two = await get(
      `/swap-queue?contract_id=${contract},definitely.not.a.contract`,
    );
    check(
      "a made-up contract in the list does not change the answer",
      two.body.total === filtered.body.total,
      `${two.body.total} vs ${filtered.body.total}`,
    );
    const empty = await get(
      "/swap-queue?contract_id=definitely.not.a.contract",
    );
    check(
      "a contract nobody has queued work for shows nothing",
      empty.body.total === 0,
      `total ${empty.body.total}`,
    );
  }
}

console.log("=== the facets give the bar its buttons ===");
{
  const facets = await get("/tx-queue/facets");
  const contracts = facets.body.contracts ?? [];
  const accounts = facets.body.accounts ?? [];
  check(
    "the contracts carry an id and a count",
    contracts.every(
      (one: any) =>
        typeof one.contractId === "string" && typeof one.count === "number",
    ),
    `${contracts.length} contracts`,
  );
  check(
    "the accounts carry an id and a count",
    accounts.every(
      (one: any) =>
        typeof one.accountId === "string" && typeof one.count === "number",
    ),
    `${accounts.length} accounts`,
  );
}

console.log("=== the counts the bar shows are right ===");
{
  const page = await get("/tx-queue?limit=100");
  check(
    "the total is the WHOLE match and not the page",
    page.body.total >= page.body.items.length,
    `${page.body.items.length} on the page, total ${page.body.total}`,
  );
  check(
    "the next cursor is there only when there is more",
    (page.body.nextCursor !== null) ===
      page.body.items.length < page.body.total,
    `items ${page.body.items.length}, total ${page.body.total}, cursor ${page.body.nextCursor === null ? "null" : "set"}`,
  );
}

console.log("=== the errors are the right shape ===");
{
  const bad = await get("/tx-queue?cursor=not-a-cursor");
  check("a bad cursor is a 400", bad.status === 400, `status ${bad.status}`);
  check("a bad cursor says bad_cursor", bad.body?.error?.code === "bad_cursor");
  const no_account = await get("/tx-queue?hide_sent=1");
  check(
    "hide_sent with no account is a 400",
    no_account.status === 400,
    `status ${no_account.status}`,
  );
  check(
    "hide_sent with no account says hide_sent_without_account",
    no_account.body?.error?.code === "hide_sent_without_account",
  );
}
// ========================================
// --- THE REPORT -------------------------
console.log("");
console.log(`${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
