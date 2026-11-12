# FORHACKATHON.md

💩 **shit-1170 rex engine** — a read-and-queue engine for NEAR, where an AI agent prepares the work and a human signs it.

This document is for the NEAR HACKATHON judges. It explains what the project is, what it does, why it exists, what was built during the hackathon window, and where every piece of it lives.

**You do not need to run anything to judge this project.** Everything below is readable as-is. If you want to try it, the live site is <https://shit-1170.com/>.

---

## the problem

An AI agent that can send a transaction is one bad prompt away from draining an account.

So agents get the read-only half of the chain and the human keeps the write half. The problem with that split is that it makes the agent nearly useless: it can tell you what you hold, but it cannot *do* anything about it. Every write becomes a copy-paste ritual — the agent produces JSON, the human pastes it into a wallet, the human signs it, the human pastes the hash back into the chat so the agent can continue. That loop is slow, error-prone, and breaks the moment the agent has to act on something the human is not watching.

**shit-1170 rex engine is the missing middle.** It is a queue between the agent and the human. The agent prepares a fully-verified transaction and hands it to the engine. The engine holds it. The human opens one page and signs it with their own wallet. The engine records the hash and the agent reads it back from the queue it queued. Nobody pastes anything, and the agent still never holds a key.

The rule the whole project is built on:

> **the agent prepares the transaction and queues it. the human signs it.**

---

## what it is

| piece | what it is | where it lives |
| --- | --- | --- |
| **the engine** | the Rust (axum) backend, the Preact frontend, and the typed client — the queue, the feed, the swap queue, the vault | [shit-1170-rex-engine](https://github.com/huggiesdotnear/shit-1170-rex-engine) |
| **the agent** | REX — the agent profile: its identity, its rules, its references | [shit-1170-rex-engine-agent](https://github.com/huggiesdotnear/shit-1170-rex-engine-agent) |
| **the skills** | 13 skills — the tool library REX draws on, and the only way it is allowed to touch the chain | [shit-1170-rex-engine-skills](https://github.com/huggiesdotnear/shit-1170-rex-engine-skills) |
| **the tool box** | the reusable NEAR library the skills and the engine are built on — one year of work | [near-kit-tool-box](https://github.com/sleet-js/near-kit-tool-box-ts) |

**Live site:** <https://shit-1170.com/>

### the engine

- **Backend:** Rust, axum, SQLite (sqlx). One database, three tables: `QUEUE` → `TRANSACTION` → `ACCOUNT`.
- **Frontend:** TypeScript, Preact, no framework. Plain pages, plain CSS, one folder per page.
- **Middleman:** `@huggies/shit-1170-rex-engine-middleman` — the published typed client for the engine's HTTP API.
- **The queue:** `POST /api/shit-1170/tx-queue` takes a batch of calls and returns a queue id plus the signing link. The human signs it. `POST /api/shit-1170/tx-queue/{queue_id}/sent` records the hash.
- **The feed:** a live websocket at `GET /api/shit-1170/feed/ws` tells a subscribed agent *when* something changed. The engine's own routes say *what* it is.
- **The swap queue:** the same shape for swaps, with a live quote behind it.
- **The vault:** a local SQLite vault of API keys and limited access keys, mounted as a docker volume, never published to a host port.

### the agent — REX

REX is an [awesome-agents](https://github.com/pablof7z/awesome-agents) profile, so it installs into OpenCode, Claude Code, Codex, Goose, Hermes, or pi. It does not pin a model — it inherits whatever harness it is running in.

REX reads chain state and the engine's feed, verifies contract calls against the chain, and prepares transactions and swaps as queued batches a human signs.

**REX never signs or sends anything.** Every state-changing call becomes a queued payload:

```json
{
  "contractId": "usdt.tether-token.near",
  "method": "ft_transfer",
  "args": { "receiver_id": "bob.near", "amount": "25000000" },
  "gas": "30000000000000",
  "deposit": "1"
}
```

The engine holds it until a human signs it in the web app. The hash comes back from the queue — the human never pastes a transaction hash into a chat.

REX is also **proactive**: name an account and it goes and looks up what that account holds, what it can trade, whether it belongs to a DAO, what the feed holds, and what the account has done on chain, before it answers.

### the skills — 13 of them

REX draws on the skills in [shit-1170-rex-engine-skills](https://github.com/huggiesdotnear/shit-1170-rex-engine-skills). They are installed separately, with the skills installer:

```sh
bunx skills add huggiesdotnear/shit-1170-rex-engine-skills
```

| skill | what it owns |
| --- | --- |
| `shit-1170-rex-engine` | the queue: propose, verify, queue, track, and the chain checks |
| `shit-1170-rex-tx-explorer` | an account's transaction history, and transactions by hash |
| `shit-1170-rex-ft` | NEP-141 tokens: balances, metadata, holders, transfers, wNEAR |
| `shit-1170-rex-keys` | limited access keys: generate, offer, remove, and read an account's keys |
| `shit-1170-rex-unit-conversion` | human amounts to chain units, against the token's real decimals |
| `shit-1170-rex-sputnik-dao` | DAO governance: policy, proposals, bounties, and the votes |
| `shit-1170-rex-rhea-ammv2` | the Rhea AMM: pools, quotes, swaps, LP positions, the deposited balance |
| `shit-1170-rex-rhea-dclv2` | the Rhea concentrated-liquidity DEX: pools, liquidity, limit orders |
| `shit-1170-rex-near-social` | near.social: feeds, profiles, posts, and the social writes |
| `shit-1170-rex-fastfs` | FastFS uploads: the borsh payload, the chunking, and the serving URL |
| `shit-1170-rex-hello-sleet-near` | the hello.sleet.near greeting contract |
| `shit-1170-rex-irc` | the shit-1170 IRC server: read a channel, post to it, hold the connection open |
| `shit-1170-rex-chess` | on-chain chess on app.chess-game.near, in propose mode and in autonomous mode |

### it uses the skills' tools, and only the skills' tools

Every fact REX reports comes out of a bin in the skills library: the balances from the ft skill, a transaction from the tx explorer, the access keys from the keys skill, the queue from the engine skill.

**REX does not hand-roll a chain read.** No raw RPC calls, no hand-written JSON-RPC, no curling a chain endpoint for data a skill already covers. The bins hold the fact-checking — the decimals, the storage checks, the gas, the pagination — and a hand-rolled call skips all of it.

**When a skill does not cover something, REX does not work around it.** It says the library does not support it yet, and contributes the feature — as an issue on the repo that owns the gap, then a pull request. The library is supposed to cover everything. If it does not, the gap is a bug in the library, not a reason to improvise.

### the one exception to "the agent never signs"

Autonomous chess is the only place REX signs. The chess skill can sign its own calls on `app.chess-game.near` with a **limited access key** — a key that names one contract and cannot move the account's NEAR, that a human approved itself, and whose method list is checked before it is used. `storage_deposit`, `add_key` and `delete_key`, and anything with a token deposit stay queued for a human in both modes.

That is the whole design in one feature: **the agent may act on its own only inside a box a human drew, and only where the box cannot take money.**

---

## why NEAR

NEAR is not a rendering layer on this project. It is the reason the design works.

- **Function call keys are a first-class primitive.** The restricted-key model — one contract, an allowance, a method list — is exactly the box an agent needs to act without a human, and exactly the box a human can audit before approving. This is not a wrapper around a chain that happens to support it; it is the security primitive the whole project is built on.
- **The account *is* the wallet.** There is no separate key store, no separate signing service, no separate user table. A limited key is an access key on an account, and the chain enforces its scope. The engine's `keys` page can therefore verify a key against the chain and rebuild its list from the chain after every action — the UI never has to be trusted about what a key can do.
- **Cheap, fast, and readable transactions.** The engine's feed, the queue, the verification loop and the chess daemon all assume transactions are cheap enough and fast enough to be a stream of small events rather than rare heavy ones. A read-heavy, verify-everything agent workflow does not work on a chain where every call costs a day's fees.
- **Delegated actions (NEP-366).** The Rust tool box implements meta-transaction signing with a relayer, so an agent can act for an account with no balance of its own — the relayer pays the gas and the storage. This is the shape an agent economy needs, and NEAR has it built in.
- **Named accounts and sub-accounts.** The DAO, the vault, the chess player accounts, the treasury — each is a real account with real keys and real scopes, not a row in someone's database.

**How NEAR fits, concretely:** the engine uses NEAR's function-call-key model to let an agent act inside a human-drawn box, NEAR's account model to make every key auditable on-chain, NEAR's transaction receipts to verify that a queued call did what it claimed, and NEAR's low transaction cost to make a continuously-verifying agent affordable.

---

## tracks this project fits

The hackathon has thirteen tracks. This project fits these:

| track id | track | why it fits |
| --- | --- | --- |
| `agents` | Agent Economy | The literal subject. REX is an agent that reads the chain, verifies its own work, and prepares transactions it cannot sign — with an autonomous mode where a restricted key lets it act. The middleman is the paid client. |
| `security` | AI Security & Defense | The restricted-key model, the vault, the "agent never signs" rule enforced by the architecture rather than a prompt, and the one documented exception where the box is narrow enough to audit. |
| `defi` | Unstoppable DeFi | The swap queue with live quotes, the Rhea AMM and concentrated-liquidity skills, the DEX contract pages, and the token pages. |
| `business` | Autonomous Business | Treasury-shaped work: a DAO's proposals and bounties, Trezu treasuries, an agent that prepares payments and gets them signed. |
| `swarms` | AI Swarms | The engine is the coordination layer between many agents and many humans — a shared queue with a websocket feed, sub-accounts per participant, and an IRC channel where agents meet. |
| `open` | Open Innovation | The queue-between-agent-and-human pattern, the engine and its skills split into separate repos, and the tool box as the shared foundation. |

**The strongest three are `agents`, `security` and `open`.** If the jury wants one track: `agents`, because the project is about an agent that has to be safe in order to be useful at all.

---

## what was built during the hackathon window

The hackathon window opened with the announcement on **2026-10-19** and closes **2026-11-14**. Everything below was built in that window, on the default branch of each repo.

### the engine — 460 commits in the window

The engine was scaffolded on 2026-08-14 and shipped its whole surface during the window. Tags `v0.0.2` through `v0.0.8` all landed inside it.

| date | what |
| --- | --- |
| 2026-10-19 | the axum server, the ping endpoint, the static serve — the engine starts |
| 2026-10-20 → 10-21 | NEP-413 sign-in, the network pin, the account page with balances and USD, the token cache |
| 2026-10-22 | the docker setup (multi-stage Dockerfile, compose) and the Cloudflare tunnel — the app stops publishing a host port |
| 2026-10-23 | mainnet-only, multi-account sign-in |
| 2026-10-24 → 10-28 | the page pattern: one file per page, one CSS file per page, the shared components, the Sputnik DAO page |
| 2026-10-29 → 11-04 | the DAO page (members, proposals, balances, status filter), the DEX contract pages, and **the queue and the feed** |
| 2026-11-01 → 11-04 | the swap page, the switch-direction bar, the live quote, the swap queue, the queue and feed pages with real filters and cursor pagination |
| 2026-11-05 | the live feed socket, the `llms.txt` agent map of the engine, the vault bins, the middleman's argument parsing and JSON printing as public API |
| 2026-11-05 → 11-06 | the middleman becomes a real published package (`v0.0.6`, `v0.0.7`), the queue carries **raw bytes** for FastFS, the wallet's own action shapes |
| 2026-11-07 | the FastFS upload section, the REX agent install card on the skill page, the `llm.md` move |
| 2026-11-11 | the **access keys page**: add and remove limited keys directly from the UI, the chain check on the key's contract, the allowance, and the list rebuilt from the chain after every action |
| 2026-11-11 | the chess contract contact page and the chess games table |

The middleman is published to npm as `@huggies/shit-1170-rex-engine-middleman`, currently `0.0.7`.

### the skills — 82 commits in the window, all 13 skills new

Every skill in the library was written during the window.

| date | what |
| --- | --- |
| 2026-11-03 | the unit-conversion skill, with its eval assertions |
| 2026-11-05 | the engine, ft and hello skills; the hash comes from the engine and the agent can watch the feed; the skills stop depending on each other |
| 2026-11-06 | the wrap.near skill with its conditional storage registration; the FastFS skill learns to tell a borsh action from a stringified one, and uploads through the engine |
| 2026-11-09 | the Sputnik DAO skill, the Rhea AMM v2 skill, the Rhea concentrated-liquidity skill, and the tx-explorer skill with its local SQLite store |
| 2026-11-11 | the **IRC skill, rewritten in Rust** as a long-lived connection, with a local store, a bot mode, and a `reply` command that posts and waits in one call |
| 2026-11-11 | the near.social skill — reads through the middleman, writes queued for a human |
| 2026-11-12 | the **keys skill** — generate a limited key, offer it, queue its removal, read an account's keys; and the **chess skill** — propose mode and an autonomous daemon |

### the agent — 5 commits

The REX profile itself, the skill catalogue, and the tools-only rule with the chess exception written down as the one carve-out.

### the near kit tool box — 88 commits in the TS repo, 6 in the Rust repo, 49 in the skills repo

**See the [dedicated section below](#the-near-kit-tool-box) — this is the part that predates the hackathon and the part everything else stands on.**

---

## the near kit tool box

**This is the foundation, and it is the reason the rest of the project is possible.**

### what it is

A reusable library for interacting with NEAR smart contracts, built on [near-kit](https://github.com/near-kit/near-kit-rs). Three parts, three repos, two languages:

| repo | what it is |
| --- | --- |
| [near-kit-tool-box-ts](https://github.com/sleet-js/near-kit-tool-box-ts) | the TypeScript monorepo — `@near-kit-tool-box/web`, `/env`, `/fun` — published to npm |
| [near-kit-tool-box-rs](https://github.com/sleet-rs/near-kit-tool-box-rs) | the Rust crate — `near-kit-tool-box`, the same helpers in Rust |
| [near-kit-tool-box-skills](https://github.com/sleetplayground/near-kit-tool-box-skills) | the skill library — 22 skills documenting every contract's methods, args, gas and deposit |

### why it was built

I started it because I knew where this was going.

Agents were going to start making transactions on NEAR, and an agent that touches a smart contract by hand — hand-rolled JSON-RPC, guessed arguments, a deposit guessed from a blog post, an allowance typed in yoctoNEAR when someone meant half a NEAR — is exactly how you lose a wallet. The failure modes are not exotic: a wrong decimal, a missing `storage_deposit`, a method that does not exist, a restricted key with no method list, an unbounded allowance.

So I built the boring layer first: **one proven, tested place where the arguments, the gas, the deposit and the decimals are already right.** Every contract gets a typed helper. Every helper gets a bin that is also the test. Every response gets a schema. And every fact gets a skill that says where it came from, so an agent reading the skill gets the fact rather than a guess.

**It is now the substrate for the rex engine.** The engine's frontend imports `@near-kit-tool-box/fun` and `@near-kit-tool-box/web`. The rex skills import `@near-kit-tool-box/fun` and `/env`. The chess skill, the FT skill, the DAO skill, the Rhea skills — all of them are thin layers over the tool box's helpers. When REX refuses to hand-roll a chain read, it is because the tool box already holds the correct one.

### a year of groundwork

The TypeScript tool box started on **2026-03-29** — before the hackathon, before the rex engine existed, before the agent. It shipped releases through April (`0.0.2`), August (`v0.1.7`) and October (`0.2.x`), and **72 of its 160 commits predate the hackathon window.** Those pre-window commits are the baseline: the contract helpers, the type system, the argument parsing, the client setup, the naming conventions.

The Rust crate and the skill library are the later siblings of that same body of work, and they carry the same conventions: one folder per contract, one file per method, one helper per helper, one bin per helper.

**What that means for a judge:** the hackathon window is not when the tool box appeared. The hackathon window is when it got the two big features it was missing, and when its output started driving a real product.

### the tool box during the hackathon window

**`near-kit-tool-box-ts` — 88 commits, 6 releases.**

The releases, all tagged `vX.Y.Z` and matching the npm versions:

| tag | date | what it carries |
| --- | --- | --- |
| `v0.2.4` | 2026-10-29 | |
| `v0.2.5` | 2026-10-29 | |
| `v0.2.6` | 2026-10-29 | |
| `v0.3.0` | 2026-10-29 | the big one — the generic contract call helpers, the borsh codec replaced with `@zorsh/zorsh`, and the CI release automation |
| `v0.3.1` | 2026-11-09 | |
| `v0.3.3` | 2026-11-12 | the current release — the chess support lands in it |

The commits that mattered, in the order they landed:

- **2026-10-20 — `ae94b8c` — the restructuring.** `refactor(fun): restructure to mirror rust crate, one folder/file per contract/method`. The whole `fun` package is rearranged to the Rust crate's shape: one folder per contract, one file per method, the args split out of the type files, the per-network contract-id resolvers extracted into `lib/const_id/`, the method names extracted into `lib/methods/`. This is the commit that makes the TS and Rust libraries mirror each other, which is what lets the skills be written once against both.
- **2026-10-24 — `62c4d9b` / `64447a4` — the env vars.** `NEAR_KIT_` prefix on every environment variable, so the tool box's config can never collide with near-kit's own.
- **2026-10-24 — `8a973b3` — one bin per helper.** Every helper gets a runnable bin with CLI flag arguments, and the bins are the test harness. A helper with no bin is a helper that was never actually run against the chain.
- **2026-10-28 — `278f162` — the Sputnik DAO feature.** `feat(fun): the Sputnik DAO factory and daos, a generic view helper, and the rhea shares batch`. The DAO factory and the DAOs it deploys land, together with a generic view helper and a batched Rhea pool-shares read.
- **2026-10-29 — `191d178` — every view validated.** `feat(fun): validate every JSON view response and document every type`. Around forty view helpers gain a zod response schema, and `DOCS/live_responses.md` records what the chain actually returned. A helper that returns a shape the chain does not actually send is now a type error, not a runtime surprise.
- **2026-10-29 — `d2973bb` → `2b184aa` → `e929444` — the generic contract call helpers.** This is the single most important addition in the window. `generic_contract_call_fun` takes a list of transactions and sends one NEAR transaction per input, with the actions in array order, sequential by default so a single private key is safe, optionally parallel. `generic_contract_call_wallet` batches the same list through a browser wallet's `signAndSendTransactions` — one popup for the whole batch. Gas and deposit accept raw units *or* near-kit's human forms (`"30 Tgas"`, `"1 NEAR"`, `"… yocto"`).
- **2026-10-29 — `b11ca39` — the borsh codec replaced.** `refactor(fun): replace the hand-rolled borsh codec with @zorsh/zorsh`. The custom codec is swapped for the real one, touching all fifteen INTEAR DEX helpers. `c9b0977` fixes the bug it was hiding: the hand-rolled integer readers were reading a Node Buffer's whole pool.
- **2026-10-29 — `e868118` — the release automation.** `feat(ci): automate the npm release with a trusted-publishing workflow`. A manual workflow bumps the patch version, syncs all three packages, tags `vX.Y.Z`, builds, publishes env → fun → web through **npm OIDC trusted publishing** (no long-lived token), verifies each published version against the registry, and cuts a GitHub release. `bd12f7d` fixes the verification to hit the registry directly instead of `npm view`.
- **2026-10-30 — `1a9bfa7` / `1b4ce61` / `664ca50` — the DAO V2 work.** `feat(sputnik-dao): support V2 calls and testnet` adds twenty DAO and factory helpers with the V2 call shapes and the testnet contract id; `feat(sputnik-dao): add FunctionCall proposal helper and bin` adds the FunctionCall proposal helper with base64-encoded args; `fix(sputnik-dao): remove non-callable get_user_weight helper` deletes a helper for a method that does not exist — the kind of correction that only a live chain surfaces.
- **2026-11-09 — `43a9bae` — the DAO source metadata.** `feat(sputnik-dao): add the contract_source_metadata views and the dao update call`.
- **2026-11-11 — `02d47a0` — the chess feature.** `feat: add app.chess-game.near support to the ts tool box`. Twenty-eight new helpers, the args module, the contract-id resolver, the method names, the types, and **25 bins** with their own README and shared helper. This is the flagship addition of the window.
- **2026-11-12 — `b3328ad` — the matchmaking fix.** `fix(chess): the matchmaking queue holds 500 entries, not 200`, correcting the limit after reading it off the live contract.

**`near-kit-tool-box-rs` — 6 commits, the whole crate landing in the window.**

- **2026-10-30 — `8732883` — the crate lands.** 214 files: 17 helper modules, 19 contract-id constant modules, 19 method-name modules, 18 type modules, 57 bins, and an mdBook with a page per contract. The Rust sibling of the TS package, with the same conventions.
- **2026-11-11 — `1a8f616` — near-kit 0.20.1**, and every build warning cleared.
- **2026-11-11 — `df275cb` — the chess feature.** `feat: add app.chess-game.near helpers to the rust tool box`. Seventeen chess helpers, three bins, five types, and the doc page — the Rust mirror of the TS addition, one day later.
- **2026-11-11 — `c3e05ff` — the deposit fix.** `fix(chess): drop the join_matchmaking deposit, it is not payable`. The method takes no deposit, and the helper was sending one.
- **2026-11-11 — `8886255` — the storage_withdraw fix.** `fix(chess): storage_withdraw is not implemented on the contract`. The contract does not implement it; the helper was still calling it and failing with `OperationNotSupported`.
- **2026-11-12 — `0e67a8f` — the matchmaking fix**, the same 500-not-200 correction as the TS repo.

**`near-kit-tool-box-skills` — 49 commits, the documentation layer.**

- **2026-10-21 → 10-22 — the first 12 skills** land together: wrap.near, the NEAR TLD, ref-finance v2, the pumpopoly world, the pool factory, the pool staking pool, the NEP-141 FT, and the house-of-stake set.
- **2026-10-22 — `83f6e6f`** adds twelve more in a single day: the DEX, NFT, meme-cooking, meta-pool, linear protocol, berryclub and the remaining house-of-stake contracts.
- **2026-10-28 — `2c663c0`** adds the Sputnik DAO skill and the Rhea shares batch args.
- **2026-10-29 — `d5cc30b`** splits the Rhea concentrated-liquidity skill into `views`, `calls`, `ft_transfer_call`, `multi` and `recipes`. `8f3277f` adds the missing YAML frontmatter and documents the **limited access key**.
- **2026-10-30 — `7e57e11` / `9103810` / `c54f173`** document the Sputnik DAO V2 calls, the testnet contract, and the FunctionCall proposals.
- **2026-11-09 — `ed8aa06` / `9228a56`** document how a Sputnik DAO policy gates proposals and votes, and the `contract_source_metadata` views.
- **2026-11-11 — `ba3eaff`** splits the chess skill into `views`, `calls` and `bets`, and verifies its arguments against the contract. `005c207` adds the SOURCE, LINKS and MIDDLEMAN PACKAGE sections to every skill, so each skill names the contract's own source repository, the site and the typed client.

### what the tool box gives the agent

The tool box is the answer to "how does an agent interact with a smart contract safely?" — and the answer is *not* "carefully".

- **Typed args, one file per method.** `near_add_key_fun_json`, `chess_game_play_move_fun_json`, `rhea_ammv2_swap_fun_json` — the argument shape is in the type, the method name is in a constant, the contract id is in a per-network resolver.
- **One contract id per network.** `rhea_ammv2_contractid(get_stored_network_id())` returns the mainnet or the testnet id. The agent never types a contract id from memory.
- **Gas and deposit as human strings.** `"30 Tgas"`, `"1 NEAR"`, `"1 yocto"` — and the deposit for a token is the token's own `storage_balance_bounds.min`, read from the chain, not guessed.
- **Validated responses.** Every view has a schema, so a chain that changes shape fails loudly instead of silently.
- **Bins as the test harness.** Every helper has a bin that ran against the live chain. A helper that was never run does not exist.
- **A generic path for everything else.** `generic_contract_view_fun_json` and `generic_contract_call_fun` cover any contract the library does not have yet — with the same validation, the same batching, and the same wallet path.

### the load balancer

The Rust crate ships a view-only RPC load balancer (`LOAD_BALANCING_NEAR`): round-robin or random across endpoints, per-endpoint retry configuration, and failover on any error. It proxies `view_account`, `view_function`, `view_state`, `block`, `block_effects`, `status`, `gas_price` and the rest.

**It is view-only by design.** Spreading writes across endpoints breaks nonce and replay protection — the whole point of the queue is that writes are ordered, deliberate and signed once.

---

## the resources this project ships

### the published npm packages

| package | what it is |
| --- | --- |
| [`@huggies/shit-1170-rex-engine-middleman`](https://www.npmjs.com/package/@huggies/shit-1170-rex-engine-middleman) | the engine's typed client — the queue, the swap queue, the API key, the argument parsing, the JSON printing |
| [`@near-kit-tool-box/fun`](https://www.npmjs.com/package/@near-kit-tool-box/fun) | the reusable contract helpers — 218 source files across 16 contract families |
| [`@near-kit-tool-box/env`](https://www.npmjs.com/package/@near-kit-tool-box/env) | the backend client, configured from `.env` |
| [`@near-kit-tool-box/web`](https://www.npmjs.com/package/@near-kit-tool-box/web) | the browser client, with Hot Wallet Connect and network switching |
| [`@sleet-js/middleman-intear-api`](https://www.npmjs.com/package/@sleet-js/middleman-intear-api) | the typed client for the INTEAR token registry API |
| [`@sleet-js/middleman-intear-dex-api`](https://www.npmjs.com/package/@sleet-js/middleman-intear-dex-api) | the typed client for the INTEAR DEX backend |
| [`@sleet-js/middleman-rhea-api`](https://www.npmjs.com/package/@sleet-js/middleman-rhea-api) | the typed client for the Rhea API |
| [`@sleet-js/middleman-near-social-api`](https://www.npmjs.com/package/@sleet-js/middleman-near-social-api) | the typed client for the near.social API |
| [`@sleet-js/middleman-protocol-pawns-api`](https://www.npmjs.com/package/@sleet-js/middleman-protocol-pawns-api) | the typed client for the Protocol Pawns API — the chess game, the elo, the leaderboard |
| [`@sleet-js/fastnear-explorer-api-client`](https://www.npmjs.com/package/@sleet-js/fastnear-explorer-api-client) | the typed client for the FastNear explorer API |
| [`@huggies/shit-bin`](https://www.npmjs.com/package/@huggies/shit-bin) | the shared CLI bin helpers |

### the skills — 13 in the rex library, 22 in the tool box library

**The rex skills** (installed with `bunx skills add huggiesdotnear/shit-1170-rex-engine-skills`) are listed in [the table above](#the-skills--13-of-them). Each one is a folder with a `SKILL.md`, a `README.md`, runnable `bin/`, the library in `src/`, and deeper `DOCS/` where the detail warrants it.

**The tool box skills** (installed with `bunx skills add sleetplayground/near-kit-tool-box-skills`) cover 22 contracts:

| skill | contract |
| --- | --- |
| `sleet-kit-hello-sleet-near` | `hello.sleet.near` |
| `sleet-kit-wrap-near` | `wrap.near` |
| `sleet-kit-near` | the NEAR account methods — keys, account creation, deletion |
| `sleet-kit-ft` | the NEP-141 standard |
| `sleet-kit-v2-ref-finance-near` | the Rhea AMM v2 |
| `sleet-kit-rhea-dclv2-ref-labs-near` | the Rhea concentrated-liquidity DEX — `views`, `calls`, `ft_transfer_call`, `multi`, `recipes` |
| `sleet-kit-sputnik-dao` | the Sputnik DAO factory and the DAOs — `factory`, `dao` |
| `sleet-kit-intear-dex` | the INTEAR DEX — `engine`, `xyk`, `otc`, `simple-amm`, `minimal` |
| `sleet-kit-app-chess-game-near` | `app.chess-game.near` — `views`, `calls`, `bets` |
| `sleet-kit-social-near` | near.social |
| `sleet-kit-world-pumpopoly-near` | the pumpopoly world |
| `sleet-kit-nft-pumpopoly-near` | the pumpopoly NFT |
| `sleet-kit-exchequer-pumpopoly-near` | the pumpopoly exchequer |
| `sleet-kit-pool-near-factory` | the NEAR pool factory |
| `sleet-kit-pool-near-staking` | the per-validator staking pool |
| `sleet-kit-hos-venear-dao` | the house-of-stake lockup DAO |
| `sleet-kit-hos-lockup` | the house-of-stake lockup sub-account |
| `sleet-kit-hos-vote-dao` | the house-of-stake vote DAO |
| `sleet-kit-meta-pool-near` | the meta pool |
| `sleet-kit-meme-cooking-near` | the meme.cooking launchpad |
| `sleet-kit-linear-protocol-near` | the linear protocol |
| `sleet-kit-berryclub-ek-near` | the berryclub contract |

Each tool box skill follows the same house convention: the contract id per network, each method's call type, the exact argument shape, the required gas and deposit, then a **SOURCE** section (the contract's own repository), a **LINKS** section (site, app, api, explorer) and a **MIDDLEMAN PACKAGE** section for the typed client.

### the agent profiles

- **[REX](https://github.com/huggiesdotnear/shit-1170-rex-engine-agent)** — the rex engine's agent. Install with `bunx awesome-agents add huggiesdotnear/shit-1170-rex-engine-agent --agent rex`.
- **[awesome-agents](https://github.com/pablof7z/awesome-agents)** — the CLI that installs it, into OpenCode, Claude Code, Codex, Goose, Hermes or pi.

### the docs inside the repos

| doc | where |
| --- | --- |
| the routes, the data model, the filters, the pages | `shit-1170-rex-engine/DOCS/ROUTES.md` |
| the architecture, the layout, the page rules | `shit-1170-rex-engine/DOCS/GOLD.md`, `DOCS/RULES.md`, `DOCS/FILES.md` |
| the vault | `shit-1170-rex-engine/DOCS/VAULT.md` |
| the queue payload shapes, the checks, the reporting rules | `shit-1170-rex-engine-agent/agents/rex/references/QUEUEING.md` |
| every skill, every bin, and what each one answers | `shit-1170-rex-engine-agent/agents/rex/references/SKILLS.md` |
| where the facts come from, and the middleman packages used | `shit-1170-rex-engine-skills/DOCS/TOOLS.md` |
| the agent's map of the engine, its skills and its tools | `shit-1170.com/llms.txt` |

---

## install instructions

**Nothing here is needed to judge the project.** This section is here for a judge who wants to run it.

### the agent

REX is an [awesome-agents](https://github.com/pablof7z/awesome-agents) profile.

```sh
# add
bunx awesome-agents add huggiesdotnear/shit-1170-rex-engine-agent --agent rex
# --list — see what is in the repo before installing
bunx awesome-agents add huggiesdotnear/shit-1170-rex-engine-agent --list

# one harness
bunx awesome-agents add huggiesdotnear/shit-1170-rex-engine-agent --agent rex --harness opencode
bunx awesome-agents add huggiesdotnear/shit-1170-rex-engine-agent --agent rex --harness claude-code
bunx awesome-agents add huggiesdotnear/shit-1170-rex-engine-agent --agent rex --harness codex

# every harness the CLI finds on your PATH
bunx awesome-agents add huggiesdotnear/shit-1170-rex-engine-agent --agent rex

# preview the writes without making them
bunx awesome-agents add huggiesdotnear/shit-1170-rex-engine-agent --agent rex --dry-run
```

Manage it afterwards:

```sh
bunx awesome-agents list
bunx awesome-agents update rex
bunx awesome-agents remove rex
```

From a checkout:

```sh
git clone https://github.com/huggiesdotnear/shit-1170-rex-engine-agent.git
cd shit-1170-rex-engine-agent
bunx awesome-agents add . --agent rex --harness opencode
```

Where it lands:

| harness | global install | project install |
| --- | --- | --- |
| Codex | `~/.codex/rex.config.toml` | not supported |
| Claude Code | `~/.claude/agents/rex.md` | `.claude/agents/rex.md` |
| OpenCode | `~/.config/opencode/agents/rex.md` | `.opencode/agents/rex.md` |
| Goose | `~/.agents/agents/rex.md` | `.agents/agents/rex.md` |
| Hermes | `~/.hermes/profiles/rex/SOUL.md` | not supported |
| pi | `~/.pi/rex/` + a `pi-rex` launcher | not supported |

Run it:

```sh
opencode --agent rex        # opencode
claude --agent rex          # claude code
codex --profile rex         # codex
pi-rex                      # pi
```

### the skills

```sh
# add
bunx skills add huggiesdotnear/shit-1170-rex-engine-skills
# --list
bunx skills add huggiesdotnear/shit-1170-rex-engine-skills --list
```

The skills are **not published to npm** — they install from the repo. The `package.json` in each skill folder names the package it *would* have if it were published, and those names are internal.

### the tool box

```sh
bun add @near-kit-tool-box/fun
bun add @near-kit-tool-box/env
bun add @near-kit-tool-box/web
```

```js
import { near_kit_client } from "@near-kit-tool-box/web";
import { greeting_get_greeting_fun_json } from "@near-kit-tool-box/fun";

const near = near_kit_client();
const greeting = await greeting_get_greeting_fun_json(near, "hello.sleet.near");
```

```js
import { near_kit_env } from "@near-kit-tool-box/env";
import { ft_balance_of_fun_json } from "@near-kit-tool-box/fun";

const balance = await ft_balance_of_fun_json(near_kit_env, "token.near", "account.near");
```

The Rust crate:

```sh
cargo add near-kit-tool-box
```

The tool box skills:

```sh
bunx skills add sleetplayground/near-kit-tool-box-skills
```

### the engine, from source

```sh
git clone https://github.com/huggiesdotnear/shit-1170-rex-engine.git
cd shit-1170-rex-engine

# frontend
bun i
bunx prettier . --write
bun tsc --noEmit
bun run build.ts

# backend
cargo fmt
cargo check
cargo run
# cargo build --release
./target/release/shit-1170-rex-engine
```

The app is deployed in docker behind a Cloudflare tunnel, with **no host ports published** — the only way in is the tunnel. The vault lives in a docker volume at `${HOME}/.shit-1170-rex-engine/shit-1170-rex-engine.db`.

---

## the contracts and accounts this project touches

| id | what | network |
| --- | --- | --- |
| `app.chess-game.near` | Protocol Pawns on-chain chess | mainnet |
| `hello.sleet.near` | the greeting contract | mainnet |
| `wrap.near` | wNEAR | mainnet |
| `usdt.tether-token.near` | USDT — 24 to 6 decimals, `storage_balance_bounds.min` 0.00125 NEAR | mainnet |
| `fastfs.near` | the FastFS receiver convention | mainnet |
| `social` | the near.social write target | mainnet |
| `sputnik-dao.near` | the Sputnik DAO factory | mainnet |
| `<name>.sputnik-dao.near` | a DAO — e.g. `huggies.sputnik-dao.near`, `sleet.sputnik-dao.near` | mainnet |
| `v2.ref-finance.near` | the Rhea AMM v2 | mainnet |
| `dclv2.ref-labs.near` | the Rhea concentrated-liquidity DEX | mainnet |
| `dex.intear.near` | the INTEAR DEX host contract | mainnet |
| `sputnikv2.testnet` | the Sputnik DAO factory | testnet |
| `ref-finance-101.testnet` | the Rhea AMM v2 | testnet |
| `dclv2.ref-dev.testnet` | the Rhea concentrated-liquidity DEX | testnet |

---

## the pre-hackathon baseline

Per the hackathon rules, pre-window work is baseline and the NEAR integration built in the window is what is judged. This is the honest accounting:

| what | when | status |
| --- | --- | --- |
| the engine's Rust scaffold | 2026-08-14 | baseline |
| the engine's full surface — the pages, the queue, the feed, the swap queue, the keys page | 2026-10-19 → 11-12 | **in the window** — 460 commits |
| the tool box (TS) | started 2026-03-29 | baseline for the 72 pre-window commits |
| the tool box's generic call helpers, the borsh codec swap, the CI release automation | 2026-10-28 → 10-29 | **in the window** |
| the tool box's Sputnik DAO V2 support | 2026-10-28 → 11-09 | **in the window** |
| the tool box's `app.chess-game.near` support | 2026-11-11 | **in the window** |
| the tool box (Rust) | 2026-10-30 → 11-12 | **in the window** — the whole crate |
| the tool box skills | 2026-10-21 → 11-12 | **in the window** — all 49 commits |
| the rex skills | 2026-11-03 → 11-12 | **in the window** — all 13 skills |
| the REX agent profile | 2026-11-12 | **in the window** |

The tool box's older releases — `0.0.2` in April, `v0.1.7` in August — are pre-window baseline, as are the contract helpers and the type system that came before the window. Everything the hackathon window added is listed in [the tool box section](#the-tool-box-during-the-hackathon-window).

---

## how a judge can check this project in five minutes

No install needed. Read, in this order:

1. **`README.md`** — what it is, in three lines.
2. **`shit-1170-rex-engine-agent/agents/rex/agent.yaml`** — the agent's rules. This is where "the agent never signs" is actually written down.
3. **`shit-1170-rex-engine-agent/agents/rex/references/QUEUEING.md`** — the exact queue payload shape and the checks that run before it is queued.
4. **`shit-1170-rex-engine-skills/shit-1170-rex-keys/SKILL.md`** — the restricted-key skill, the mechanism that makes autonomous chess safe.
5. **`shit-1170-rex-engine-skills/shit-1170-rex-chess/SKILL.md`** and **`DOCS/MODES.md`** — the one documented exception to the rule.
6. **`near-kit-tool-box-ts/README.md`** — the foundation, and the three packages the whole project is built on.

If you want to watch it work: <https://shit-1170.com/> — the feed, the queue, the swap page, the account page, the keys page, the DAO page and the chess page are all live.

---

## note: the "spund dao"

The DAO work in this project is **Sputnik DAO** — the factory at `sputnik-dao.near` and the DAOs it deploys, V2 on mainnet and testnet. There is no "spund dao" contract, and there was no such thing built during the window. If you saw that name somewhere, it was a mis-remembering of Sputnik.

---

=====================
<br/>
copyright 2026 by sleet.near
