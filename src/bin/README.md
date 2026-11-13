# BIN

these are the db admin bins for the vault. they are the default way to do db
work on a vault from outside the app.

- `db-admin-list` — shows what is in a vault, including the queues, swaps, and row counts. read-only.
- `db-admin-prune` — removes queued batches and swaps by age, by never-sent, or by id.
- `db-admin-backup` — takes a safe copy of a vault using sqlite's backup.
- `db-admin-export` — writes a vault out as json.
- `db-admin-import` — brings an export back into a vault.

the implementation lives in `src/backend/logic/db/bin/`. the files in this folder
are the entry points.

## the safety

**`db-admin-prune` and `db-admin-import` are a dry run until `--confirm` is given.**
they print the plan and then stop. there is no undo, so the plan is the whole point.

**always take a `db-admin-backup` before a `db-admin-prune` on the production vault.**
the backup is safe to take while the app is running.

**never test against the production vault.** use `--db <path>` to point at a local
copy instead.

## the full usage

see [`DOCS/VAULT.md`](../../DOCS/VAULT.md) for the db path, prune filters, and
run commands.

=====================
<br/>
copyright 2026 by sleet.near
