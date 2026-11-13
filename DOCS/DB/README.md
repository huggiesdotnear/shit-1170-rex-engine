# DB

the field lists of our sqlite db file, and nothing else.

this folder covers the DB — the one file the app reads and writes:

- [`tables.csv`](./tables.csv) — every table, every column, and what each
  column is for
- [`indexes.csv`](./indexes.csv) — every index, the columns it covers,
  and the query it serves

the prose side of the db is [`../VAULT.md`](../VAULT.md) — what is in it,
where the db file lives, where the status lives, and the bins that do db
work on it. the markdown holds the prose and the csvs hold the field
lists.

a route's field list is NOT here, because it is a list of the HTTP
surface and not of the db: that is
[`../ROUTES/routes.csv`](../ROUTES/routes.csv).

=====================
<br/>
copyright 2026 by sleet.near
