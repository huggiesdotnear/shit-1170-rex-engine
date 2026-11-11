# frontend/app/keys.html

the page shows the access keys of the signed-in account — the full access keys first, then the limited ones with their contract, their methods and their allowance — and it lets the reader pick the limited keys and remove them, and add a new limited key.

the full access keys are shown at the top of the list and are never pickable: a full access key IS the account, so removing it locks the reader out of their own account.

```html
<body>
  <!--==========================================-->
  <div id="SHIT_1170_COMPONENT_LOADING_BAR"></div>
  <!--==========================================-->
  <!--==========================================-->
  <main>
    <h1>ACCESS KEYS</h1>
    <!--==========================================-->
    <div id="SHIT_1170_COMPONENT_APP_PAGE_KEYS_ADD"></div>
    <!--==========================================-->
    <div id="SHIT_1170_COMPONENT_APP_PAGE_KEYS_LIST"></div>
    <!--==========================================-->
  </main>
  <!--==========================================-->
  <nav id="SHIT_1170_COMPONENT_NAV"></nav>
  <!--==========================================-->
  <div id="SHIT_1170_COMPONENT_AUTH_BUTTON"></div>
  <!--==========================================-->
  <!--script-->
  <script src="../lib/ts/import-keys.ts"></script>
  <!--script-->
  <!--==========================================-->
</body>
```

====================
<br/>
copyright 2026 by sleet.near
