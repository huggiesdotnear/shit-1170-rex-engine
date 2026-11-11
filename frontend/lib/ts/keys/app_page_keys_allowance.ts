// ========================================
// the ALLOWANCE of a limited access key: how much the key may spend,
// and how that is written and read.
//
// --- WHAT AN ALLOWANCE IS: a function-call key may attach a deposit to
// the calls it makes. the allowance is the CEILING on that deposit —
// across every call the key ever makes, not per call. a key with no
// allowance can call the methods but can attach nothing.
//
// --- WHY THE READER TYPES NEAR AND NOT YOCTONEAR: a yoctoNEAR amount
// is a 25-digit number for one NEAR. nobody types that. so the box
// takes NEAR — "1.5" — and the app does the conversion.
//
// --- WHY THE APP PARSES IT AND NOT NEAR KIT: near-kit's `parseAmount`
// refuses a bare number on purpose — "1.5" with no unit is ambiguous,
// and a wrong guess about the unit is a wrong amount of money. that
// rule is right for a program and wrong for a box that SAYS NEAR on
// it. so the box's own unit is the unit, and the number is parsed
// against it.
//
// --- WHY THE RAW FIGURE IS KEPT ALONGSIDE: the human figure is
// rounded, and a reader who needs the exact number the chain recorded
// has to be able to get it. so the raw yoctoNEAR rides along and the
// page shows it in the row's title.
import { Amount, parseAmount } from "near-kit";
// ========================================
// ========================================
// --- CONFIG -----------------------------
// how many decimals a NEAR amount is shown with.
//
// --- WHY --- a yoctoNEAR is a 10^-24 NEAR. an allowance of one
// yoctoNEAR is a real thing and a real amount of money, but it is not
// a number a reader can hold in their head. so the figure is rounded
// to a readable number of decimals and the raw one is one hover away.
const SHIT_1170_ALLOWANCE_DECIMALS = 6;
// the number of decimals the raw figure is shown with, when the raw one
// is shown on its own.
const SHIT_1170_ALLOWANCE_RAW_DECIMALS = 24;
// one yoctoNEAR.
const SHIT_1170_YOCTO_PER_NEAR = 10n ** 24n;
// ========================================
// ========================================
// --- TYPES ------------------------------
// an allowance, in both its forms.
export interface SHIT_1170_APP_PAGE_KEYS_ALLOWANCE {
  // the raw yoctoNEAR figure, as the chain stores it. null when the
  // key has no allowance.
  yocto: string | null;
  // the human figure, ready to read. null when the key has no
  // allowance.
  human: string | null;
}
// ========================================
// ========================================
// --- HELPERS ----------------------------
// the reader's number, as a NEAR amount.
//
// --- WHY THE NUMBER IS CHECKED FIRST AND NOT JUST PARSED: "abc" and
// "1.2.3" are not numbers, and parseAmount would say so in its own
// words. the check here says it in the page's words.
//
// --- WHY A NUMBER WITH TOO MANY DECIMALS IS REFUSED: a NEAR amount
// with a fraction smaller than a yoctoNEAR cannot be sent — the chain
// would round it, and a key whose allowance is not what the reader
// typed is a key with an allowance the reader did not choose. so it is
// refused rather than quietly rounded.
function parse_near_number_fun(raw: string): string | null {
  const trimmed = raw.trim();
  if (trimmed.length === 0) return null;
  // a plain number, with a dot for the fraction.
  if (!/^\d+(\.\d*)?$/.test(trimmed) && !/^\.\d+$/.test(trimmed)) return null;
  const dot = trimmed.indexOf(".");
  if (dot === -1) return trimmed;
  const fraction = trimmed.slice(dot + 1);
  if (fraction.length > SHIT_1170_ALLOWANCE_DECIMALS) return null;
  return trimmed;
}
// ========================================
// the human figure of a yoctoNEAR amount.
//
// --- WHY IT IS BUILT HERE AND NOT TAKEN FROM NEAR KIT: near-kit's
// `formatAmount` shows two decimals, which would round a one
// yoctoNEAR allowance to "0.00 NEAR" — and a reader would then think
// the key can spend nothing, when it can spend exactly one unit of the
// smallest thing there is. the allowance gets more decimals because it
// is a LIMIT and not a balance.
//
// --- WHY A BIGINT AND NOT A NUMBER: a yoctoNEAR amount is a 24-decimal
// number, and a javascript number loses the last of those digits. the
// figure is a limit on money, so it is computed exactly.
function to_human_fun(yocto: bigint): string {
  const whole = yocto / SHIT_1170_YOCTO_PER_NEAR;
  const fraction = yocto % SHIT_1170_YOCTO_PER_NEAR;
  if (fraction === 0n) return `${whole.toString()} NEAR`;
  const digits = fraction
    .toString()
    .padStart(SHIT_1170_ALLOWANCE_RAW_DECIMALS, "0");
  // --- WHY THE CUT KEEPS THE FIRST SIGNIFICANT DIGIT: the decimals are
  // cut to a readable length, but the cut must never leave nothing
  // behind. an allowance of one yoctoNEAR has its only significant
  // digit at the very end, and a plain cut to six decimals would turn
  // it into "0.000000" — which says the key can spend nothing, and
  // that is a DIFFERENT statement from what the chain holds.
  //
  // so the cut moves forward until it finds a digit that is not zero.
  // the figure is cut and never rounded, so what is shown is never
  // MORE than the limit really is.
  let shown = digits.slice(0, SHIT_1170_ALLOWANCE_DECIMALS);
  if (
    shown === digits.slice(0, shown.length) &&
    shown.replace(/0/g, "") === ""
  ) {
    const first_significant = digits.search(/[1-9]/);
    if (first_significant >= 0) {
      shown = digits.slice(0, first_significant + 1);
    }
  }
  const trimmed = shown.replace(/0+$/, "");
  return trimmed.length === 0
    ? `${whole.toString()} NEAR`
    : `${whole.toString()}.${trimmed} NEAR`;
}
// ========================================
// ========================================
// --- PARSE -----------------------------
// the reader's NEAR amount, as the chain's yoctoNEAR.
//
// --- WHY IT RETURNS NULL AND NOT AN ERROR: a blank box means NO
// allowance — the key can call the methods and attach nothing — and
// that is a real choice, not a missing value. so blank is an answer.
function parse_allowance_fun(raw: string): string | null {
  const number = parse_near_number_fun(raw);
  if (number === null) return null;
  try {
    // --- WHY `Amount.NEAR` AND NOT A STRING: near-kit's types know a
    // NEAR amount by its SHAPE — a number followed by " NEAR" — and
    // `Amount.NEAR` is the constructor that makes one. a plain string
    // built here would be the right text but the wrong type, and the
    // type is what stops a bare number reaching the chain.
    return parseAmount(Amount.NEAR(number as `${number}`));
  } catch {
    return null;
  }
}
// ========================================
// --- FORMAT ----------------------------
// the chain's yoctoNEAR, as the reader reads it.
//
// --- WHY A NULL ALLOWANCE IS "NO ALLOWANCE" AND NOT "0 NEAR": they are
// different things. a key with NO allowance cannot attach a deposit at
// all. a key with an allowance of ZERO can attach a deposit of zero,
// and can still be called. the difference matters to a reader deciding
// whether a key is safe to hand to somebody.
function format_allowance_fun(
  yocto: string | null,
): SHIT_1170_APP_PAGE_KEYS_ALLOWANCE {
  if (yocto === null) return { yocto: null, human: null };
  try {
    return { yocto, human: to_human_fun(BigInt(yocto)) };
  } catch {
    return { yocto, human: null };
  }
}
// ========================================
// ========================================
// --- PUBLIC ---------------------------
export const SHIT_1170_APP_PAGE_KEYS_ALLOWANCE = {
  parse: parse_allowance_fun,
  format: format_allowance_fun,
};
// ========================================
// copyright 2026 by sleet.near
// ========================================
