# frontend/app/index.html ?hide_sent=1 ?sort={<date_added|most_popular>} ?type={<all|calls|swaps>} ?contracts={<ids>} ?accounts={<ids>} ?dir={<asc|desc>}

the feed. the app's front door, and a social feed of the work that is
queued and waiting to be signed: one post per queued batch of
transactions, one post per queued swap, a filter bar above them, and the
reader's own status on each one.

the filter bar has a TOP ROW - the ALL / MINE shortcut, the hide-sent
switch, the FILTERS button and the direction button - and a PANEL behind
the FILTERS button, and the panel has FOUR groups: the sort, the type,
the contracts and the accounts. the url holds all of them, so a filtered
feed is a link somebody can paste. `hide_sent` drops what the reader has
sent, `sort` is DATE ADDED or MOST POPULAR, `dir` is the way round the
sort, `type` is ALL, FUNCTION CALL or SWAPS, and `contracts` and
`accounts` are the picked contract and account filters. the contract and
account filters are LISTS - they go out to the api as comma separated
values, and the api reads them as an OR. the api pages the feed with an
opaque cursor, so the next page is asked for when the reader nears the
bottom.

`dir` is the way round the sort. `desc` is DESCENDING, so the NEWEST
post is at the TOP of the list and the oldest is at the bottom - the
same order the api's own list route answers in. `asc` is the flip: the
oldest post at the top and the newest at the bottom. `desc` is the
default, and the direction button's arrow shows the way the list runs -
down for `desc` and up for `asc`.

the ALL / MINE shortcut at the front of the bar is a SHORTCUT OVER THE
ACCOUNTS LIST, and the accounts list is the ONE source of truth. there
is no `?view=` url param and no `view` field in the saved filter.
**ALL** is lit when the accounts list is EMPTY, and clicking it clears
the accounts list. **MINE** is lit when the accounts list is EXACTLY ONE
entry and it is the signed-in reader's OWN account, and clicking it sets
the accounts list to just that account. any other accounts list -
somebody else's, or the reader's own plus others - leaves NEITHER button
lit, because the reader has gone past the shortcut with the ACCOUNTS
group. **MINE** is disabled when nobody is signed in. the ME button in
the ACCOUNTS group - the reader's own account, first in the row - sets
the accounts list to the reader's account ALONE, so ME and MINE always
agree: clicking one lights the other. the url carries the MINE view as
`?accounts=<your.id>`, so a shared link opens exactly the MINE view.
there is no `?view=` param.

the CONTRACTS group applies to the SWAPS as well as to the batches. a
queued swap names `tokenIn` - the token it sells - and `tokenOut` - the
token it buys - and both are contract ids, so a swap shows when EITHER
of its two tokens is a picked contract. picking usdt shows the swaps
that SELL usdt AND the swaps that BUY usdt.

a param at its default is LEFT OFF the url, so a link to the plain feed
opens the plain feed.

the ACCOUNTS group offers the reader's OWN account first, as a ME button
with the account id in its tooltip, so a reader can pick their own queues
without recognising their id in a long row of ids. it is there even when
the reader has no queued work of their own, because an account with no
queued work is not in the facets. ME sets the accounts list to the
reader's account ALONE, which is what makes ME and MINE agree.

the filters can also be SAVED, and they are restored from the browser's
own storage when the feed opens. the saved filter holds the switch, the
sort, the direction, the type and the two lists - a reader who saved
"just my account" saved that in the accounts list, and the accounts list
IS saved. the MINE shortcut is saved the same way, because it IS the
accounts list. the url wins over the saved filters, so a link somebody
pasted is the link's view and not the reader's saved one, and the saved
filters only fill in what the url does not say.

the feed's LIST requests need NO api key - the queue list and the swap
list are OPEN, and a reader who is not signed in reads them like
anybody else. the key is only needed for the things a reader ACTS on.

the feed is LIVE. one WEBSOCKET carries the new work - a newly queued
batch or a newly queued swap - and the new post appears on screen
without the reader polling or reloading. the socket is OPEN and carries
EVERY new post; the client decides what matches the reader's filter,
so a filter change costs nothing on the wire. the socket is opened when
the page's first load runs and is closed when the page goes away, and
the client reconnects it by itself when it drops. the sort and the
direction are the ORDER of the list, not a part of the filter, so a
post that arrives live lands where the sort says it lands - at the TOP
of a newest-first list and at the BOTTOM of an oldest-first one.

the TYPE group says which kind of post is FETCHED: FUNCTION CALL asks
for the queued batches only, SWAPS asks for the queued swaps only, and
ALL asks for both. one kind is half the requests.

a KEY ACTION is not a call - it changes the READER'S OWN KEYS - so a
post whose first transaction is one shows no contract and no method. it
shows the action on the headline, and the key's REACH under it: the
CONTRACT the key may call, the METHODS it may use, and the ALLOWANCE -
the ceiling on what the key may attach. the reach is there because the
reader is about to approve the key, and a key named without its reach is
a blank approval.

nothing is said TWICE. the headline and the reach are drawn one under
the other, so a line that repeats the one above it reads as a mistake.
an add's headline already names the contract and the methods, so an add
shows only the ALLOWANCE under it. a remove's headline is just the
public key, so a remove shows all three.

an ADD's reach comes from the QUEUE - it is what the agent asked for,
and it is known before anything is signed. a REMOVE's reach comes from
the CHAIN - the queue deliberately drops it, because the chain is the
one that knows what a key on the account may call - so the page asks the
chain and shows what it finds. a remove whose reach could not be read
says only that, and says nothing else about the key.

```html
<body>
  <!--==========================================-->
  <div id="SHIT_1170_COMPONENT_LOADING_BAR"></div>
  <!--==========================================-->
  <!--==========================================-->
  <!--==========================================-->
  <!--==========================================-->
  <!--==========================================-->
  <main>
    <h1>SHIT-1170</h1>
    <!--==========================================-->
    <div id="SHIT_1170_COMPONENT_APP_PAGE_FEED_FILTERS"></div>
    <!--==========================================-->
    <div id="SHIT_1170_COMPONENT_APP_PAGE_FEED_POST"></div>
    <!--==========================================-->
  </main>
  <!--==========================================-->
  <!--==========================================-->
  <!--==========================================-->
  <nav id="SHIT_1170_COMPONENT_NAV"></nav>
  <!--==========================================-->
  <div id="SHIT_1170_COMPONENT_AUTH_BUTTON"></div>
  <!--==========================================-->
  <div id="SHIT_1170_COMPONENT_TOAST_LAYER"></div>
  <!--==========================================-->
  <!--<footer>COPYRIGHT 2026</footer>-->
  <!--==========================================-->
  <!--==========================================-->
  <!--script-->
  <script src="../lib/ts/import-feed.ts"></script>
  <!--script-->
  <!--==========================================-->
</body>
```

=====================
<br/>
copyright 2026 by sleet.near
