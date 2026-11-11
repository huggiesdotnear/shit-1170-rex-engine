# frontend/app/keys.html

the page shows the access keys of the signed-in account — the full access keys first, then the limited ones with their contract, their methods and their allowance — and it lets the reader pick the limited keys and remove them, and add a new limited key.

the full access keys are shown at the top of the list and are never pickable: a full access key IS the account, so removing it locks the reader out of their own account.

the list always comes fresh from the chain. the browser's own record of the keys it made is folded in on top: the private key of a limited key is kept in the browser's own indexeddb, so the reader can come back for it, and a stored key the chain no longer has stays on the page marked REMOVED rather than vanishing.

the add form sits on a white block. it takes the account the key may call, the methods it may call and the allowance — how much the key may spend — and the form can be prefilled from the url — `?account=`, `?methods=` and `?allowance=`. the account does not have to be a contract, so the app asks the chain nothing about it: the input lowercases as the reader types, and one local rule says whether the string is an account id the chain will accept. the key pair — the public key and the private key — is shown before the reader approves anything, so what they approve is what they were shown.

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
