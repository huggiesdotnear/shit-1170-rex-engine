# PAGES

one spec file per html page.

the file name is the page path, dots instead of slashes:
`frontend/app/account.html` -> [`frontend.app.account.md`](./frontend.app.account.md)

a page outside `app/` has no `app.` in its name:
`frontend/index.html` -> [`frontend.index.md`](./frontend.index.md)

## what is where

a page is described by three files:

- `frontend.app.account.md` | a one-line description and the html codeblock
- [`components.csv`](./components.csv) | the components each page mounts, and the source each reads from
- [`requests.csv`](./requests.csv) | the requests each page makes |

the split is by granularity. the md file is the page. the csvs are every
page at once, so a question like "which components read request 2?" has
one answer, in one place.

a page's md file holds a one-line description and the html codeblock,
nothing else. the description says what the page is for; the codeblock
is the layout.

## the shape of a page spec

every page spec has the SAME shape, so a reader knows where to look:

````
# frontend/app/<page>.html ?<param>={<value>}

the one-line description of what the page is for.

```html
<body>
  ...
</body>
````

=====================
<br/>
copyright 2026 by sleet.near

```

the `#` heading says which page the spec is for and which url
parameters it reads. the description is ONE paragraph — what the page
is for, not how it works. then the codeblock. then the footer.

there is NOTHING after the codeblock but the footer. a page that needs
more than that does not get extra sections in its spec:

- what the page SHOWS belongs in the description and in
  [`components.csv`](./components.csv) — one row per component
- what the page ASKS FOR belongs in [`requests.csv`](./requests.csv) — one
  row per request
- how the page WORKS belongs in the ts, in the file that does the work

a spec is the page. a spec that grows into a manual is a manual in the
wrong place, and it drifts from the code it describes.

the codeblock holds the page's WHOLE `<body>` — the loading bar, the
`main` column, the nav, the auth button and the page's script tag — and
it mirrors the page's html exactly. a spec that lists only the mount
ids is not a spec of the page: the loading bar, the nav and the script
tag are on the page too, and a reader of the spec should see the whole
body without opening the html.

## the components

the components a page mounts are listed in
[`components.csv`](./components.csv), not in the page spec. one row per
component, with the source it reads from and a note for the detail that
does not fit the other columns.

the csv is the single place to look for "what is on this page" — the
page spec holds the layout, and the csv holds the components and where
their data comes from.

## the requests

the requests a page makes are listed in [`requests.csv`](./requests.csv),
not in the page spec. one row per request, with the call it makes.

a page's requests come from one loader — one file that fetches
everything for that page. no component fetches anything.

## Pages

- [`TODO.md`](./TODO.md) - the pages that still need their spec written
- [`PATTERN.md`](./PATTERN.md) - how every page with data is built
- [`components.csv`](./components.csv) - the components each page mounts,
  and the source each one reads from
- [`requests.csv`](./requests.csv) - the requests each page makes
- [`frontend.index.md`](./frontend.index.md) - `frontend/index.html`
- [`frontend.app.index.md`](./frontend.app.index.md) - `frontend/app/index.html` - ?hide_sent= - ?sort= - ?type= - ?contracts= - ?accounts= - ?dir= - FEED
- [`frontend.app.queue.md`](./frontend.app.queue.md) - `frontend/app/queue.html` - ?queue= - QUEUE
- [`frontend.app.auth.md`](./frontend.app.auth.md) - `frontend/app/auth.html` - AUTH
- [`frontend.app.discover.md`](./frontend.app.discover.md) - `frontend/app/discover.html`
- [`frontend.app.burn.md`](./frontend.app.burn.md) - `frontend/app/burn.html` - BURN
- [`frontend.app.skill.md`](./frontend.app.skill.md) - `frontend/app/skill.html`
- [`frontend.app.unit-conversion.md`](./frontend.app.unit-conversion.md) - `frontend/app/unit-conversion.html` - ?ca= ?human= ?unit= - UNIT CONVERSION
- [`frontend.app.swap.md`](./frontend.app.swap.md) - `frontend/app/swap.html` - ?in= - ?out= - ?amount_in= - ?amount_out= - ?swap= - SWAP
- [`frontend.app.game.md`](./frontend.app.game.md) - `frontend/app/game.html`
- [`frontend.app.keys.md`](./frontend.app.keys.md) - `frontend/app/keys.html` - ?account= - ?methods= - ?allowance= - ACCESS KEYS
- [`frontend.app.account.md`](./frontend.app.account.md) - `frontend/app/account.html` - ?id= - ?tab=
- [`frontend.app.tx.md`](./frontend.app.tx.md) - `frontend/app/tx.html` - ?tx= - TRANSACTION
- [`frontend.app.token.index.md`](./frontend.app.token.index.md) - `frontend/app/token/index.html` - ?id= - TOKEN
- [`frontend.app.token.shit-1170.md`](./frontend.app.token.shit-1170.md) - `frontend/app/token/shit-1170.html`
- [`frontend.app.contract.rhea.md`](./frontend.app.contract.rhea.md) - `frontend/app/contract/rhea.html` - RHEA
- [`frontend.app.contract.chess.md`](./frontend.app.contract.chess.md) - `frontend/app/contract/chess.html` - CHESS
- [`frontend.app.contract.intear-dex.md`](./frontend.app.contract.intear-dex.md) - `frontend/app/contract/intear-dex.html`
- [`frontend.app.contract.sputnik-dao.md`](./frontend.app.contract.sputnik-dao.md) - `frontend/app/contract/sputnik-dao.html` - ?dao_id= - SPUTNIK DAO
- [`frontend.app.contract.pumpopoly.md`](./frontend.app.contract.pumpopoly.md) - `frontend/app/contract/pumpopoly.html` - ?id=

## The nav

- index.html - FEED
- account.html - PROFILE
- swap.html - SWAP
- discover.html - DISCOVER

=====================
<br/>
copyright 2026 by sleet.near
```
