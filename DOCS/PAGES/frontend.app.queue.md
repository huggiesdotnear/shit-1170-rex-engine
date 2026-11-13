# frontend/app/queue.html ?queue={queue_id}

the page shows one queued batch: what it is, every call it makes, every
account it is for, and where the reader stands on it.

the page's sections are STATUS, SUMMARY, TRANSFERS, FILES, CALLS,
ACCOUNTS and ACTIONS. TRANSFERS and FILES are the queue page's WIDGETS
— the sections that show one kind of thing the batch asks for, and only
when it asks for it — and they sit where the transaction page puts its
own: the files come after the transfers and before the raw calls, so
what the batch will DO is above and what it WILL CALL is below. the
registry in `frontend/lib/components/queue/widgets/queue_widgets.ts`
decides WHICH widgets apply and the page's divs decide WHERE they go.

a KEY ACTION in the CALLS section is not a call - it changes the
READER'S OWN KEYS - so it gets a row of its own and shows the action on
the headline, and the key's REACH under it: the CONTRACT the key may
call, the METHODS it may use, and the ALLOWANCE - the ceiling on what
the key may attach. nothing is said TWICE - an add's headline already
names the contract and the methods, so an add shows only the ALLOWANCE
under it, while a remove's headline is just the public key and the
reach says all of it. an ADD's reach comes from the QUEUE, and a
REMOVE's reach comes from the CHAIN, which is the one that knows what a
key on the account may call. the feed card shows the same reach the
same way, so a reader who saw the post and then opens the batch reads
one story.

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
    <h1>QUEUE</h1>
    <!--==========================================-->
    <div id="SHIT_1170_COMPONENT_QUEUE_STATUS"></div>
    <!--==========================================-->
    <div id="SHIT_1170_COMPONENT_QUEUE_SUMMARY"></div>
    <!--==========================================-->
    <div id="SHIT_1170_COMPONENT_QUEUE_TRANSFERS"></div>
    <!--==========================================-->
    <div id="SHIT_1170_COMPONENT_QUEUE_FASTFS"></div>
    <!--==========================================-->
    <div id="SHIT_1170_COMPONENT_QUEUE_TRANSACTIONS"></div>
    <!--==========================================-->
    <div id="SHIT_1170_COMPONENT_QUEUE_ACCOUNTS"></div>
    <!--==========================================-->
    <div id="SHIT_1170_COMPONENT_QUEUE_ACTIONS"></div>
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
  <script src="../lib/ts/import-queue.ts"></script>
  <!--script-->
  <!--==========================================-->
</body>
```

=====================
<br/>
copyright 2026 by sleet.near
