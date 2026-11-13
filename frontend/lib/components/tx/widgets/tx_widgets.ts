// ========================================
// the transaction page's widgets: the sections that show ONE KIND OF
// THING the transaction did, and that show it only when the transaction
// actually did it.
//
// a widget is not a section of the page. the page's sections and their
// order are a fact about the page — the transfers sit above the files,
// the actions sit below them — and they do not change because a widget
// was added. the registry decides WHICH widgets apply to a transaction.
// the page decides WHERE they go.
//
// --- WHY --- THE REGISTRY DECIDES WHICH AND NOT WHERE: the page mounts
// each section into its own `<div>`, and the order of those divs is the
// page's layout. a registry that also carried the position would put a
// widget's PLACE in the registry, and then where a widget sits would be
// decided by where it happened to be registered — the order of the
// imports would become the order of the page. the order of the page is
// a decision, and it is the page's decision.
//
// --- WHY --- THERE IS NO DEFAULT AND NO RAW WIDGET: the fastnear
// explorer shows the same transfer twice — once in its TRANSFERS
// section, which comes from the log events, and once in its own
// ft-transfer widget, which comes from the action. the section reads
// what the chain's books say and the widget reads what the call asked
// for, and for one transaction they are the same fact twice. a reader
// sees one transfer here and one transfer there and cannot tell whether
// they are two transfers or one. so the transfers widget is ours and it
// is the ONLY one, and it comes from the log events.
//
// the RAW ACTION IS NOT A WIDGET EITHER: the actions section stays
// exactly what it is — the raw action and its arguments — because the
// raw action is the thing the chain signed and it is not a summary of
// anything.
//
// --- WHY --- `match` READS THE PARSED TRANSACTION: a widget decides
// whether it applies by looking at FACTS — the parsed actions and the
// transfers the client gave — and not at the DOM. a widget that asked
// the page what is on screen would be a widget about the page's markup,
// and it would break the moment the page moved a div.
//
// --- WHY --- THE WIDGETS ARE FROZEN ONES AND THE REGISTRY IS NOT: a
// widget's own fields cannot change after it is registered, so a
// consumer cannot swap a widget's `match` under the page's feet. the
// registry itself takes new widgets, so a widget can be added without
// touching this file.
import type { PARSE_RESULT } from "@sleet-js/fastnear-explorer-api-client";
// ========================================
// ========================================
// --- TYPES ------------------------------
// one widget: what it is called, and when it applies.
export interface TX_WIDGET {
  // the widget's own name. it is what a consumer asks for by name and
  // what a reader of the code sees.
  id: string;
  // does this widget apply to this transaction?
  //
  // it reads the PARSED transaction — the parsed actions and the
  // transfers the client gives — so a widget matches on facts and not
  // on the DOM.
  match: (parsed: PARSE_RESULT) => boolean;
  // what the widget is called in the page's own words.
  label: string;
}
// ========================================
// ========================================
// --- REGISTRY --------------------------
// the widgets that are registered.
//
// --- WHY --- A LIST AND NOT A MAP: the widgets are looked up by what
// they match, not by name — the page asks "which widgets apply?" and
// not "give me the widget called transfers". a list keeps the order the
// widgets were registered in, which is a fact about the registry and
// not about the page.
let registry: ReadonlyArray<TX_WIDGET> = [];
// ========================================
// ========================================
// --- MATCHERS --------------------------
// the transfers widget applies when the transaction moved something.
//
// --- WHY --- BOTH LISTS ARE ASKED FOR: a fungible transfer and an nft
// transfer are different things on the chain, and a transaction can
// carry one and not the other. a transaction that moved only an nft
// still moved something, and it still belongs under TRANSFERS.
function transfers_match_fun(parsed: PARSE_RESULT): boolean {
  return parsed.transfers.length > 0 || parsed.nft_transfers.length > 0;
}

// the files widget applies when the transaction uploaded a file.
//
// --- WHY --- THE PARSED ACTION IS THE PLACE TO LOOK: the parser hangs
// the decoded FastFS upload on the call it belongs to, so the uploads
// are found by looking for that — the same fact the files widget reads
// when it draws its rows.
function fastfs_match_fun(parsed: PARSE_RESULT): boolean {
  return parsed.actions.some((action) => action.fastfs !== undefined);
}
// ========================================
// ========================================
// --- REGISTER -------------------------
// add a widget to the registry.
//
// --- WHY --- A WIDGET WITH AN ID THAT IS ALREADY TAKEN IS REFUSED: the
// id is how a consumer names a widget, and two widgets under one name
// would make the name ambiguous. a duplicate is a mistake, and a
// mistake that shows up as a page with the wrong widget is worse than a
// mistake that shows up here.
//
// --- WHY --- THE REGISTRY IS REPLACED AND NOT PUSHED INTO: the exported
// registry is frozen, so a consumer cannot push into it. the new list
// is built and frozen again, and the exported one is replaced with it.
// what a consumer holds is never a list that is still being written to.
//
// --- WHY --- THE NAME IS THE REGISTRY'S OWN ACTION: a widget that wants
// to be on the page registers itself, and it asks for the one function
// that does that.
export function register_widget(widget: TX_WIDGET): void {
  for (const existing of registry) {
    if (existing.id === widget.id) {
      throw new Error(`[tx_widgets] the widget "${widget.id}" is registered`);
    }
  }
  registry = Object.freeze([...registry, Object.freeze(widget)]);
}
// ========================================
// ========================================
// --- PUBLIC -----------------------------
// the registry: the widgets that are registered.
//
// --- WHY --- IT IS A FUNCTION AND NOT A CONSTANT: a consumer that held
// the list itself would hold a list that a later registration cannot
// reach. reading the registry through a function means every consumer
// sees the same registry, whatever was registered when it asked.
function tx_widgets_fun(): ReadonlyArray<TX_WIDGET> {
  return registry;
}

// the widgets that apply to this transaction.
//
// --- WHY --- THE MATCH IS TRIED ON EVERY WIDGET AND NONE OF THEM CAN
// BREAK THE PAGE: a widget's `match` reads the parsed transaction, and
// a widget that throws would take the whole page down. the throw is
// caught and the widget is skipped — a widget that cannot decide is a
// widget that does not apply.
//
// --- WHY --- THE NAME IS THE REGISTRY'S OWN QUESTION: the registry's
// job is to say which widgets apply, and this is the function that says
// it.
export function get_matching_widgets(
  parsed: PARSE_RESULT,
): ReadonlyArray<TX_WIDGET> {
  const matching: TX_WIDGET[] = [];
  for (const widget of registry) {
    try {
      if (widget.match(parsed)) matching.push(widget);
    } catch (err) {
      console.error("[tx_widgets] a widget's match failed", widget.id, err);
    }
  }
  return matching;
}

// does the widget with this id apply to this transaction.
//
// --- WHY --- THE ASK IS "DOES THIS WIDGET APPLY" AND NOT "IS THIS
// WIDGET REGISTERED": a widget's own component asks whether it applies
// to the transaction on screen. whether the widget is REGISTERED is a
// different question — a widget can be registered and still not apply
// to this transaction, and then it shows nothing.
export function widget_applies(id: string, parsed: PARSE_RESULT): boolean {
  for (const widget of get_matching_widgets(parsed)) {
    if (widget.id === id) return true;
  }
  return false;
}
// ========================================
// ========================================
// --- THE WIDGETS -----------------------
// the two widgets the page has.
//
// --- WHY --- THEY ARE REGISTERED HERE AND NOT IN THE PAGE: the page
// imports the widgets' own components, and a component that registered
// itself would depend on the order the imports ran in. the registry is
// written down once, and the page reads it.
//
// --- WHY --- THE LABELS ARE THE PAGE'S OWN WORDS: a widget's label is
// what the page calls it, and the page's words are the ones the reader
// sees. the label is here so the name of the thing and the name the
// reader knows it by are one fact and not two.
register_widget({
  id: "transfers",
  match: transfers_match_fun,
  label: "TRANSFERS",
});
register_widget({
  id: "fastfs",
  match: fastfs_match_fun,
  label: "FILES (FASTFS)",
});
// ========================================
// ========================================
// --- EXPORT ---------------------------
export const SHIT_1170_TX_WIDGETS = {
  // the registered widgets.
  list: tx_widgets_fun,
  // the widgets that apply to this transaction.
  matching: get_matching_widgets,
  // does the widget with this id apply to this transaction?
  applies: widget_applies,
  // add a widget.
  register: register_widget,
} as const;
// ========================================
// copyright 2026 by sleet.near
// ========================================
