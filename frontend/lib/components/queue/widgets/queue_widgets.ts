// ========================================
// the queue page's widgets: the sections that show ONE KIND OF THING a
// queued batch asks for, and that show it only when the batch asks for
// it.
//
// a widget is not a section of the page. the page's sections and their
// order are a fact about the page — the transfers sit above the files,
// the calls sit below them — and they do not change because a widget
// was added. the registry decides WHICH widgets apply to a batch. the
// page decides WHERE they go.
//
// --- WHY --- THE REGISTRY DECIDES WHICH AND NOT WHERE: the page mounts
// each section into its own `<div>`, and the order of those divs is the
// page's layout. a registry that also carried the position would put a
// widget's PLACE in the registry, and then where a widget sits would be
// decided by where it happened to be registered — the order of the
// imports would become the order of the page. the order of the page is
// a decision, and it is the page's decision.
//
// --- WHY --- THERE IS NO DEFAULT AND NO RAW WIDGET: the calls list
// stays exactly what it is — the raw call, its gas, its deposit and its
// arguments — because the raw call is the thing the chain will sign and
// it is not a summary of anything. and there is ONE transfer widget,
// not two: the fastnear explorer shows the same transfer twice — once in
// its TRANSFERS section, which comes from the log events, and once in
// its own ft-transfer widget, which comes from the action. a reader
// sees one transfer here and one transfer there and cannot tell whether
// they are two transfers or one. so the transfers widget is ours and it
// is the ONLY one.
//
// --- WHY --- THE MATCH READS THE QUEUE AND NOT A PARSED TRANSACTION: a
// queued batch has no parsed transaction and no chain records — nothing
// has happened yet. what it has is the CALLS it asks for, and a widget
// decides whether it applies by looking at FACTS. a widget that asked
// the page what is on screen would be a widget about the page's markup,
// and it would break the moment the page moved a div.
//
// --- WHY --- THE QUEUE'S TRANSFERS ARE INFERRED AND THE TX PAGE'S ARE
// READ: a queued transaction has no chain records yet. the tx page reads
// what HAPPENED — the events in the receipt logs — and the queue page
// reads what the agent ASKED FOR — the arguments, by the standard's
// fixed arg names. those are two different facts at two different
// stages, and they are NOT merged into one function. they may share the
// LOOK — the shared transfer amount already does that — but a shared
// reader would have to lie about one of the two.
//
// --- WHY --- THE WIDGETS ARE FROZEN ONES AND THE REGISTRY IS NOT: a
// widget's own fields cannot change after it is registered, so a
// consumer cannot swap a widget's `match` under the page's feet. the
// registry itself takes new widgets, so a widget can be added without
// touching this file.
import { fastfs_is_call_fun } from "@sleet-js/fastnear-explorer-api-client";
import { shit_1170_queue_transfers_fun } from "../../../ts/shared/queue_transfers";
import type { SHIT_1170_TX_QUEUE } from "@huggies/shit-1170-rex-engine-middleman";
// ========================================
// ========================================
// --- TYPES ------------------------------
// one widget: what it is called, and when it applies.
export interface QUEUE_WIDGET {
  // the widget's own name. it is what a consumer asks for by name and
  // what a reader of the code sees.
  id: string;
  // does this widget apply to this batch?
  //
  // it reads the QUEUE — the batch and the calls it asks for — so a
  // widget matches on facts and not on the DOM.
  match: (queue: SHIT_1170_TX_QUEUE) => boolean;
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
let registry: ReadonlyArray<QUEUE_WIDGET> = [];
// ========================================
// ========================================
// --- MATCHERS --------------------------
// the transfers widget applies when the batch asks for a token to move.
//
// --- WHY --- THE READING IS THE QUEUE'S OWN AND NOT A COPY HERE: a
// queued call is a function call, and the ft methods name their
// arguments in a fixed way — NEP-141 fixes the arg names — so what the
// agent asked for can be read off the arguments. a swap is the second
// tier: it names the token in and the amount in, and NOT the amount
// out, because a queued swap does not yet know what it will get.
//
// --- WHY --- THE MATCHER READS THE SHARED READER: the reading lives in
// one place — the shared queue transfers reader — and the matcher asks
// it. a matcher that listed the methods itself would be a second list
// of the same methods, and the two would drift: a new standard method
// would show up in the rows and not in the matcher, and the widget
// would sit there showing nothing.
function transfers_match_fun(queue: SHIT_1170_TX_QUEUE): boolean {
  return shit_1170_queue_transfers_fun(queue).length > 0;
}

// the files widget applies when the batch uploads a file.
//
// --- WHY --- THE CALL IS NAMED AND NOT DECODED HERE: a FastFS upload
// is a call to `__fastdata_fastfs`, and the method name is what says
// this IS a FastFS call. the bytes of any call could decode as FastFS
// borsh by accident, and a card about a file would be a lie.
//
// --- WHY --- THE NAME IS THE CLIENT'S AND NOT A COPY: the client owns
// the method name, and the matcher asks it the same way the widget's
// own reader does.
function fastfs_match_fun(queue: SHIT_1170_TX_QUEUE): boolean {
  for (const transaction of queue.transactions) {
    if (fastfs_is_call_fun(transaction.method.trim())) return true;
  }
  return false;
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
export function register_widget(widget: QUEUE_WIDGET): void {
  for (const existing of registry) {
    if (existing.id === widget.id) {
      throw new Error(
        `[queue_widgets] the widget "${widget.id}" is registered`,
      );
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
function queue_widgets_fun(): ReadonlyArray<QUEUE_WIDGET> {
  return registry;
}

// the widgets that apply to this batch.
//
// --- WHY --- THE MATCH IS TRIED ON EVERY WIDGET AND NONE OF THEM CAN
// BREAK THE PAGE: a widget's `match` reads the queue, and a widget that
// throws would take the whole page down. the throw is caught and the
// widget is skipped — a widget that cannot decide is a widget that does
// not apply.
//
// --- WHY --- THE NAME IS THE REGISTRY'S OWN QUESTION: the registry's
// job is to say which widgets apply, and this is the function that says
// it.
export function get_matching_widgets(
  queue: SHIT_1170_TX_QUEUE,
): ReadonlyArray<QUEUE_WIDGET> {
  const matching: QUEUE_WIDGET[] = [];
  for (const widget of registry) {
    try {
      if (widget.match(queue)) matching.push(widget);
    } catch (err) {
      console.error("[queue_widgets] a widget's match failed", widget.id, err);
    }
  }
  return matching;
}

// does the widget with this id apply to this batch.
//
// --- WHY --- THE ASK IS "DOES THIS WIDGET APPLY" AND NOT "IS THIS
// WIDGET REGISTERED": a widget's own component asks whether it applies
// to the batch on screen. whether the widget is REGISTERED is a
// different question — a widget can be registered and still not apply
// to this batch, and then it shows nothing.
export function widget_applies(id: string, queue: SHIT_1170_TX_QUEUE): boolean {
  for (const widget of get_matching_widgets(queue)) {
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
export const SHIT_1170_QUEUE_WIDGETS = {
  // the registered widgets.
  list: queue_widgets_fun,
  // the widgets that apply to this batch.
  matching: get_matching_widgets,
  // does the widget with this id apply to this batch?
  applies: widget_applies,
  // add a widget.
  register: register_widget,
} as const;
// ========================================
// copyright 2026 by sleet.near
// ========================================
