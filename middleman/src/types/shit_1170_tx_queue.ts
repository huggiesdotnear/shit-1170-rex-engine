// ========================================
// SHIT_1170_TX_QUEUE
// the shapes of our own tx queue routes, under /api/shit-1170/tx-queue.
//
// a QUEUE is a BATCH of transactions, not one transaction. the agent
// queues a swap that needs three calls, they go in together under one
// queue_id, and the user signs them as one batch in one wallet popup.
//
// the model is three levels deep:
//
//   a QUEUE  (queue_id)          what / when / why
//     └── a TRANSACTION          contractId / method / args / gas / deposit
//           └── an ACCOUNT ROW    accountId / status / txHash
//
// --- WHY --- the fields are camelCase: they mirror the browser's own
// session object (`accountId`), and the near-kit toolbox's transaction
// object, so a queued transaction can be handed to
// `generic_contract_call_wallet` without being reshaped.
//
// --- WHY --- `args` is a json OBJECT and not a string: the toolbox
// takes a json object of arguments. a string would make the frontend
// parse it and then re-serialize it.
//
// --- WHY --- there is a SECOND WAY to carry the arguments,
// `argsBase64`: some contracts take their arguments as RAW BYTES and
// not as json — the FastFS protocol (`__fastdata_fastfs`) wants borsh
// bytes, and a json object cannot be borsh bytes. so the bytes ride as
// standard base64 beside the json, and the browser decodes them back
// into bytes for the wallet.
//
// --- WHY --- the two are ALTERNATIVES and not a pair: a transaction
// sets exactly one of them. a transaction that set both would not say
// which one the contract gets.
//
// --- WHY --- there is no status on the queue and none on the
// transaction: the truth is on the account row. a queue is never done —
// any account may still send it — so there is nothing for a queue
// status to say, and two accounts sending the same queued transaction
// produce two different on-chain transactions with two different
// outcomes.
//
// --- WHY --- a queued transaction is EITHER a contract call OR a key
// action: a call names a contract and a method, and a key action names
// a public key. they are different things the user approves, so they
// are different fields, and a transaction that carries both is refused
// rather than guessed at.
// ========================================

// the two key actions there are.
//
// --- WHY --- it is a named type and not a bare string union in two
// places: the wire says the same two words on the request and on the
// response, and one name keeps the two from drifting apart.
export type SHIT_1170_TX_QUEUE_KEY_ACTION = "add_key" | "delete_key";

// where one account stands with one transaction.
//
// --- WHY --- four states and not two: `sent` means "it went out and
// nobody knows how it went yet", which is the real state of a
// transaction the moment the wallet signs it. `succeeded` and `failed`
// come later, when the outcome is read off the chain.
export type SHIT_1170_TX_QUEUE_STATUS =
  "pending" | "sent" | "succeeded" | "failed";

// one account on one transaction — the WHO object.
export interface SHIT_1170_TX_QUEUE_ACCOUNT {
  // the account.
  accountId: string;
  // where the account is with this transaction.
  status: SHIT_1170_TX_QUEUE_STATUS;
  // the hash of the transaction the account sent. it is null until
  // somebody sends.
  txHash: string | null;
}

// one queued transaction — a contract call OR a key action.
export interface SHIT_1170_TX_QUEUE_TRANSACTION {
  // the transaction's id. it is what the sent route quotes back.
  txId: string;
  // the order of this transaction inside the batch, from zero.
  position: number;
  // the contract being called.
  contractId: string;
  // the method name.
  method: string;
  // the arguments, as a json object.
  args: Record<string, unknown>;
  // the arguments, as raw bytes, base64 encoded. it is null when the
  // arguments are the json object above.
  //
  // --- WHY --- IT IS NULLABLE AND NOT OPTIONAL: it comes from the
  // backend, and the backend always sends it — null for a json-args
  // transaction. a reader can tell the two kinds of call apart without
  // guessing.
  argsBase64: string | null;
  // the gas, as a string. it may be nothing.
  gas: string | null;
  // the deposit, as a string. it may be nothing.
  deposit: string | null;
  // the KEY ACTION — `add_key` or `delete_key`. it is null for an
  // ordinary contract call.
  //
  // --- WHY --- IT IS ALWAYS PRESENT AND NOT OPTIONAL: it comes from the
  // backend, and the backend always sends it. a reader can tell a key
  // action from a contract call without guessing.
  keyAction: SHIT_1170_TX_QUEUE_KEY_ACTION | null;
  // the public key the key action names. it is null for a contract call.
  keyPublicKey: string | null;
  // the contract the new key may call. it is null for a contract call
  // and for a `delete_key`.
  keyReceiverId: string | null;
  // the methods the new key may call. it is null for a contract call
  // and for a `delete_key`.
  keyMethodNames: string[] | null;
  // the yoctoNEAR ceiling on the new key. it is null for a contract
  // call and for a `delete_key`.
  keyAllowance: string | null;
  // the accounts on THIS transaction.
  accounts: SHIT_1170_TX_QUEUE_ACCOUNT[];
}

// the whole batch.
export interface SHIT_1170_TX_QUEUE {
  // the batch's id.
  queueId: string;
  // what the transactions do, in words. it may be nothing.
  what: string | null;
  // when they should happen, in words. it may be nothing.
  when: string | null;
  // why the user wants them. it may be nothing.
  why: string | null;
  // the machine-readable `when`, epoch ms. it may be nothing.
  whenAt: number | null;
  // the calls of the batch, in `position` order.
  transactions: SHIT_1170_TX_QUEUE_TRANSACTION[];
  // the WHO across the whole batch — the union of the transactions'
  // account rows.
  accounts: SHIT_1170_TX_QUEUE_ACCOUNT[];
  // when the batch was queued, in ms since the epoch.
  createdAt: number;
  // when the batch was last touched, in ms since the epoch.
  updatedAt: number;
}

// the answer of the list route.
export interface SHIT_1170_TX_QUEUE_LIST {
  // the batches on this page.
  items: SHIT_1170_TX_QUEUE[];
  // how many batches matched, BEFORE the limit and the cursor took
  // their cut. it is how many pages there are.
  total: number;
  // the cursor of the NEXT page, or null when this page is the last
  // one. it is handed back as `cursor` to get the page after this.
  //
  // --- WHY --- it is a STRING and not null only: the backend encodes
  // the page point and only the backend can read it back, so the
  // middleman never shapes it.
  nextCursor: string | null;
}

// --- REQUEST ----------------------------
// one queued transaction, as the agent writes it. it is a contract call
// or a key action, and the `keyAction` field says which.
//
// --- WHY --- the ids are absent here: the backend mints them. the
// agent names a contract and a method, and the backend answers with the
// tx ids it wrote.
export interface SHIT_1170_TX_QUEUE_TRANSACTION_REQUEST {
  // the contract being called.
  //
  // --- WHY --- IT IS EMPTY FOR A KEY ACTION: a key action has no
  // contract to call, and a transaction that names one is refused with
  // `key_action_and_contract_call`.
  contractId: string;
  // the method name. it is empty for a key action, for the same reason.
  method: string;
  // the arguments, as a json object.
  args?: Record<string, unknown>;
  // the arguments, as raw bytes, base64 encoded.
  //
  // --- WHY --- IT IS OPTIONAL HERE AND NOT NULLABLE: the agent writes
  // the request, and most calls carry json arguments. the binary form is
  // for the contracts that want raw bytes.
  argsBase64?: string;
  // the gas, as a string.
  gas?: string;
  // the deposit, as a string.
  deposit?: string;
  // the KEY ACTION — `add_key` or `delete_key`. it may be left out, and
  // then the transaction is an ordinary contract call.
  //
  // --- WHY --- IT IS OPTIONAL HERE AND NOT NULLABLE: the agent writes
  // the request, and most calls are contract calls. the field says what
  // kind of transaction this is, and the answer to "what kind" is the
  // field being absent.
  keyAction?: SHIT_1170_TX_QUEUE_KEY_ACTION;
  // the public key the key action names.
  keyPublicKey?: string;
  // the contract the new key may call. it is only there for `add_key`.
  keyReceiverId?: string;
  // the methods the new key may call. it is only there for `add_key`.
  keyMethodNames?: string[];
  // the yoctoNEAR ceiling on the new key. it is only there for
  // `add_key`.
  //
  // --- WHY --- IT IS A STRING AND NOT A NUMBER: it is yoctoNEAR, and a
  // yoctoNEAR amount does not fit a json number safely when it is big.
  // every other amount in this api is a string for the same reason.
  keyAllowance?: string;
}

// the queue body: a batch of transactions and the WHO it is for.
//
// --- WHY --- everything but the transactions is optional: an agent that
// knows the WHAT and not the WHY must still be able to queue. a
// made-up WHY is worse than no WHY.
export interface SHIT_1170_TX_QUEUE_REQUEST {
  // what the transactions do, in words.
  what?: string;
  // when they should happen, in words.
  when?: string;
  // why the user wants them.
  why?: string;
  // the machine-readable `when`, epoch ms.
  whenAt?: number;
  // the batch. it must hold between one and fifty transactions.
  transactions: SHIT_1170_TX_QUEUE_TRANSACTION_REQUEST[];
  // the WHO — the accounts the agent is queuing this for. every account
  // in it must be one the agent holds an api key for.
  accounts?: string[];
}

// the body of the add-an-account route.
export interface SHIT_1170_TX_QUEUE_ACCOUNT_REQUEST {
  // the account being added. it must be the account the api key
  // belongs to.
  accountId: string;
}

// one hash of one transaction that went out.
export interface SHIT_1170_TX_QUEUE_HASH_REQUEST {
  // which transaction of the batch the hash belongs to. it is
  // required — a hash without a transaction says nothing.
  txId: string;
  // the hash. it is required.
  txHash: string;
  // the outcome. it may be left out, and then the status is `sent` — it
  // went out and nobody knows how it went yet.
  status?: SHIT_1170_TX_QUEUE_STATUS;
}

// the body of the sent route.
export interface SHIT_1170_TX_QUEUE_SENT_REQUEST {
  // the hashes of the whole batch. one call, one wallet popup.
  txHashes: SHIT_1170_TX_QUEUE_HASH_REQUEST[];
}

// --- STATS ------------------------------
// one contract in the contract stats.
export interface SHIT_1170_TX_QUEUE_CONTRACT_STAT {
  // the contract.
  contractId: string;
  // how many queued transactions hit it.
  count: number;
}

// the answer of the contract stats route.
export interface SHIT_1170_TX_QUEUE_CONTRACT_STATS {
  items: SHIT_1170_TX_QUEUE_CONTRACT_STAT[];
}

// one method in the method stats.
export interface SHIT_1170_TX_QUEUE_METHOD_STAT {
  // the method.
  method: string;
  // the contract it was called on. it rides along because `ft_swap` on
  // two contracts is two different transactions, and a method count
  // without the contract would blend them into a misleading number.
  contractId: string;
  // how many queued transactions called it.
  count: number;
}

// the answer of the method stats route.
export interface SHIT_1170_TX_QUEUE_METHOD_STATS {
  items: SHIT_1170_TX_QUEUE_METHOD_STAT[];
}
// ========================================
// copyright 2026 by sleet.near
