// ========================================
// TX_QUEUE_KEY_ACTION_FUN
// the SHAPE of a queued key action: what it names, and whether it is
// allowed at all.
//
// a queued transaction is EITHER a contract call OR a key action. a key
// action is `add_key` or `delete_key` — "give this account a limited
// key" or "take this account's limited key away" — and the user
// approves it from the feed page like any other queued transaction.
//
// --- WHY --- this is its own file: the shape of a key action is a RULE,
// and the rule has to be written down once. the route asks this file
// whether the transaction is a key action and what it says, and the
// answer is a struct the route can carry into the write.
//
// --- WHY --- THE TARGET ACCOUNT IS NOT IN THE KEY ACTION: a key action
// names a PUBLIC KEY, and a public key is not enough to say whose key
// it is. the account the key belongs to is the account the QUEUE is FOR
// — the WHO list of the request, or the caller's own account when the
// WHO list is empty. the WHO list is the queue's own list of who is
// waiting for this, and it is the only thing that says whose account
// the key action is about.
//
// --- WHY --- `delete_key` NEEDS NO RECEIVER AND NO METHODS: removing a
// key does not name what the key may call — the key already knows that,
// and the chain is the one that holds it. an agent that wanted to
// remove a key and say "it may call THIS" would be describing a
// different key than the one on chain.
//
// --- WHY --- `add_key` NEEDS A RECEIVER: a limited key with no
// receiver can call nothing, and a key with no receiver is not a
// limited key — it is a full access key with a method list nobody
// reads. the receiver is what makes it limited.
//
// --- WHY --- THE ALLOWANCE IS A STRING AND NOT A NUMBER: it is
// yoctoNEAR, and a yoctoNEAR amount does not fit a json number safely
// when it is big. every other amount in this api is a string for the
// same reason.
use crate::backend::types::tx_queue_request::TX_QUEUE_TRANSACTION_REQUEST;
// ========================================
// --- TYPES ------------------------------
// the two key actions there are.
//
// --- WHY --- it is an enum and not a string: a string would let any
// word through, and the route would have to compare it by hand in two
// places. an enum makes the compiler hold the two allowed words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TX_QUEUE_KEY_ACTION {
    // give the account a new limited key.
    AddKey,
    // take a limited key off the account.
    DeleteKey,
}

// the key action, read out of a request transaction.
//
// --- WHY --- it is SEPARATE from the prepared transaction: the shape
// check happens before the chain check, and the chain check needs the
// public key as a parsed key while the write path needs it as a string.
// this struct carries the string, and the verifier parses it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TX_QUEUE_KEY_ACTION_SHAPE {
    // which of the two actions this is.
    pub action: TX_QUEUE_KEY_ACTION,
    // the public key the action names.
    pub public_key: String,
    // the contract the new key may call. it is only there for `add_key`.
    pub receiver_id: Option<String>,
    // the methods the new key may call. it is only there for `add_key`.
    pub method_names: Option<Vec<String>>,
    // the yoctoNEAR ceiling on the new key. it is only there for
    // `add_key`.
    pub allowance: Option<String>,
}
// ========================================
// --- READ ------------------------------
// the key action of a request transaction, if it is one.
//
// --- WHY --- it returns None and not an error for a transaction that is
// NOT a key action: a transaction without a `keyAction` is an ordinary
// contract call, and the caller checks that separately. the error case
// is a transaction that SAYS it is a key action and gets the shape
// wrong.
//
// --- WHY --- THE ACTION IS READ AS A STRING AND NOT MATCHED ON THE
// ENUM DIRECTLY: the wire carries a word, and the word is not one of
// the two allowed ones. that is a refusal, and the refusal says which
// word it was.
pub fn tx_queue_key_action_read_fun(
    transaction: &TX_QUEUE_TRANSACTION_REQUEST,
) -> Result<Option<TX_QUEUE_KEY_ACTION_SHAPE>, (String, String)> {
    // --- WHY --- A MISSING OR EMPTY `keyAction` IS NOT A KEY ACTION: the
    // field is optional, and a transaction that leaves it out is a
    // contract call. an empty string is the same thing — it says
    // nothing, and nothing is a contract call.
    let raw = match transaction.key_action.as_deref() {
        Some(raw) => raw.trim(),
        None => return Ok(None),
    };
    if raw.is_empty() {
        return Ok(None);
    }

    let action = match raw {
        "add_key" => TX_QUEUE_KEY_ACTION::AddKey,
        "delete_key" => TX_QUEUE_KEY_ACTION::DeleteKey,
        other => {
            return Err((
                "bad_key_action".to_string(),
                format!(
                    "the keyAction must be add_key or delete_key, and it is {other}"
                ),
            ));
        }
    };

    // --- THE PUBLIC KEY ------------------
    // a key action without a key names nothing.
    let public_key = transaction
        .key_public_key
        .as_deref()
        .map(|value| value.trim().to_string())
        .unwrap_or_default();
    if public_key.is_empty() {
        return Err((
            "no_key_public_key".to_string(),
            "a key action must name the public key it acts on".to_string(),
        ));
    }

    // --- THE ADD-KEY SHAPE ---------------
    // the receiver, the methods and the allowance are what make a key
    // limited. a `delete_key` carries none of them.
    let receiver_id = transaction
        .key_receiver_id
        .as_deref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let method_names = transaction.key_method_names.clone().map(|names| {
        names
            .into_iter()
            .map(|name| name.trim().to_string())
            .filter(|name| !name.is_empty())
            .collect::<Vec<String>>()
    });

    let allowance = transaction
        .key_allowance
        .as_deref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    if action == TX_QUEUE_KEY_ACTION::AddKey {
        // --- WHY --- AN EMPTY RECEIVER IS REFUSED AND NOT DEFAULTED: a
        // key with no receiver is not a limited key. the agent has to
        // say which contract the key may call, because that is the whole
        // point of a limited key.
        if receiver_id.is_none() {
            return Err((
                "no_key_receiver_id".to_string(),
                "an add_key must name the receiver id the key may call".to_string(),
            ));
        }
    } else {
        // --- WHY --- A DELETE DROPS THE RECEIVER, THE METHODS AND THE
        // ALLOWANCE: they describe the key that is BEING REMOVED, and
        // the chain is the one that knows what it may call. an agent
        // that filled them in was describing a different key than the
        // one on chain, so they are dropped rather than stored as a
        // claim about it.
        return Ok(Some(TX_QUEUE_KEY_ACTION_SHAPE {
            action,
            public_key,
            receiver_id: None,
            method_names: None,
            allowance: None,
        }));
    }

    Ok(Some(TX_QUEUE_KEY_ACTION_SHAPE {
        action,
        public_key,
        receiver_id,
        method_names,
        allowance,
    }))
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
