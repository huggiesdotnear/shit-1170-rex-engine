// ========================================
// ARGS
// the argument parsing every bin and script uses.
//
// one parser, one behaviour. a bin declares which of its flags are
// BOOLEAN — the ones that stand alone with no value — and every other
// `--flag` takes the next token as its value.
//
// the flags are read through the accessors below, so a bin never
// touches the map directly and never has to guess whether a flag was
// given, given with a value, or given with the wrong value.
//
// --- WHY --- THE PARSER IS STRICT ABOUT A MISSING VALUE: a flag that
// needs a value and does not get one is a typo far more often than a
// deliberate empty string, and a value-taking flag must never swallow
// the next positional. so `--limit` with nothing after it is an error,
// not an empty string.
//
// --- WHY --- THE BOOLEAN FLAGS ARE DECLARED PER BIN and not one global
// set: a shared set would make one bin's flag swallow the next
// positional in another bin's argv.
//
// --- WHY --- THE ACCESSORS TAKE THE FLAG WITHOUT ITS DASHES: a bin
// writes `args.string("queue")`, and the map holds "--queue". the
// dashes are the caller's shorthand and they never reach the map.
// ========================================

export interface ARGS_OPTIONS {
  // the flags that stand alone with no value, written without dashes.
  boolean_flags?: Iterable<string>;
}

export interface ARGS {
  // the positional words, in the order they were given.
  positional: string[];
  // the flags, keyed by their name WITHOUT the leading dashes.
  flags: Map<string, string>;
  // the flags that stand alone, keyed by their name without dashes.
  booleans: Set<string>;
}

/**
 * split argv into positionals and flags.
 *
 * `argv` is the arguments WITHOUT the node/bun executable and the
 * script path — so `process.argv.slice(2)`.
 */
export function parse_args_fun(
  argv: string[],
  options: ARGS_OPTIONS = {},
): ARGS {
  const boolean_flags = new Set(options.boolean_flags ?? []);
  const positional: string[] = [];
  const flags = new Map<string, string>();
  const booleans = new Set<string>();

  for (let i = 0; i < argv.length; i++) {
    const token = argv[i];
    // --- WHY --- the guard: with `noUncheckedIndexedAccess` an index
    // into an array reads as `string | undefined`, and the loop bound
    // is not enough for the compiler to prove it. an empty argv skips
    // the loop entirely, so this branch is unreachable in practice.
    if (token === undefined) continue;
    if (!token.startsWith("--")) {
      positional.push(token);
      continue;
    }

    // --- WHY --- THE NAME IS THE TOKEN WITHOUT ITS DASHES: a bin asks
    // for "queue" and the map holds "queue", not "--queue". the
    // dashes are the caller's shorthand.
    const name = token.slice(2);

    // --- WHY --- `--flag=value` IS SUPPORTED: it is the shape a shell
    // writes when the value has a space in it, and it is how a
    // generator emits a flag.
    const equals = name.indexOf("=");
    if (equals >= 0) {
      const key = name.slice(0, equals);
      const value = name.slice(equals + 1);
      if (boolean_flags.has(key)) {
        booleans.add(key);
        continue;
      }
      flags.set(key, value);
      continue;
    }

    if (boolean_flags.has(name)) {
      booleans.add(name);
      continue;
    }

    const value = argv[i + 1];
    // --- WHY --- a flag with no value is an ERROR: a typo far more
    // often than a deliberate empty string, and a value-taking flag
    // must never swallow the next positional.
    if (value === undefined || value.startsWith("--")) {
      throw new Error(`--${name} was passed without a value`);
    }
    flags.set(name, value);
    i++;
  }

  return { positional, flags, booleans };
}

/**
 * the value of a flag, or the fallback when it was not given.
 *
 * a flag given with an empty value (`--note=`) is the empty string,
 * which is a value the caller chose. a flag not given at all is the
 * fallback.
 */
export function args_string_fun(
  args: ARGS,
  name: string,
  fallback?: string,
): string | undefined {
  const value = args.flags.get(name);
  if (value !== undefined) return value;
  return fallback;
}

/**
 * the value of a flag that must be there.
 *
 * a missing flag is an error, because a bin that needs a value and
 * does not have one cannot do its job.
 */
export function args_require_string_fun(args: ARGS, name: string): string {
  const value = args_string_fun(args, name);
  if (!value || value.trim().length === 0) {
    throw new Error(`--${name} is required`);
  }
  return value.trim();
}

/**
 * was a boolean flag given?
 *
 * the flag was declared as a boolean, so it stands alone and its
 * presence IS the answer.
 */
export function args_bool_fun(args: ARGS, name: string): boolean {
  return args.booleans.has(name);
}

/**
 * the value of a flag as a number, or the fallback.
 *
 * a value that is not a number is an error: a limit of "twenty" is a
 * typo, and a bin that silently treated it as no limit at all would
 * do the wrong thing quietly.
 */
export function args_number_fun(
  args: ARGS,
  name: string,
  fallback?: number,
): number | undefined {
  const value = args_string_fun(args, name);
  if (value === undefined) return fallback;
  const number = Number(value);
  if (!Number.isFinite(number)) {
    throw new Error(`--${name} must be a number`);
  }
  return number;
}
// ========================================
// copyright 2026 by sleet.near
