// ========================================
// DB_ADMIN_ARGS_FUN
// the flags every db admin bin takes, and the parser that reads them.
//
// --- WHY --- there is NO clap and NO arg-parser crate: the bins take a
// handful of flags, and a dependency for that would be more code than
// the parser itself. the parser here is the whole thing.
//
// --- WHY --- the db path is a FLAG and not an argument: the db lives in
// a different place in docker than it does at home, and the flag says
// which one. the flag wins over the env var, and the env var wins over
// the default path — the same order the app uses.
//
// --- WHY --- the flags are COLLECTED and not validated here: each bin
// knows which flags it takes and what they mean, and a shared parser
// that knows every bin's flags would know too much.
//
// --- WHY --- an unknown flag is an ERROR and not a warning: a typo in a
// flag that says "--older-then 30" would otherwise run a prune with NO
// age filter and take everything. a bin that deletes rows has to fail
// loudly on a flag it does not know.
use std::collections::HashMap;
use std::path::PathBuf;
// ========================================
// --- TYPES ------------------------------
// the flags a bin was called with.
//
// --- WHY --- the values are STRINGS and not typed: each bin parses its
// own values, because "30" is thirty days in one bin and a unix second
// in another, and the shared parser cannot know which.
#[derive(Debug, Clone, Default)]
pub struct DB_ADMIN_ARGS {
    // the db file's path, when `--db` was given.
    pub db_path: Option<PathBuf>,
    // the flags that were given, by name, without the `--`.
    pub flags: HashMap<String, String>,
    // the flags that were given with no value, like `--confirm`.
    pub switches: Vec<String>,
    // the words that were not a flag.
    pub positional: Vec<String>,
}
// ========================================
// --- PARSE -----------------------------
// read the words of the command line into the flags.
//
// --- WHY --- `--flag value` and `--flag=value` BOTH work: the first is
// how people type things and the second is how scripts write them.
// accepting one and not the other would make every script break.
//
// --- WHY --- a `--flag` at the END of the line with nothing after it is a
// SWITCH and not a flag with an empty value: `--confirm` alone is the
// common case, and an empty value would have to be special-cased by
// every bin.
//
// --- WHY --- a `--` ENDS the flags: everything after it is positional.
// a path that starts with a dash is then possible.
//
// --- WHY --- an unknown flag is NOT an error HERE but is caught by the
// bin: the parser collects everything, and each bin checks the names it
// knows. this way the parser stays one thing for all the bins.
//
// --- WHY --- the value of `--flag value` is taken even when it starts
// with a dash: `--db --weird` is the caller's mistake, and a db path
// that starts with a dash is rarer than a typo. the bin's own checks
// catch the mistake.
pub fn db_admin_args_fun(words: &[String]) -> DB_ADMIN_ARGS {
    let mut args = DB_ADMIN_ARGS::default();
    let mut index = 0;

    while index < words.len() {
        let word = &words[index];

        // --- the end of the flags --------
        if word == "--" {
            for rest in &words[index + 1..] {
                args.positional.push(rest.clone());
            }
            break;
        }

        // --- a flag with its value in it --
        if let Some(rest) = word.strip_prefix("--") {
            if let Some((name, value)) = rest.split_once('=') {
                if name == "db" {
                    args.db_path = Some(PathBuf::from(value));
                } else {
                    args.flags.insert(name.to_string(), value.to_string());
                }
                index += 1;
                continue;
            }

            // --- a flag with its value next -
            let next = words.get(index + 1);
            match next {
                Some(value) if !value.starts_with("--") => {
                    if rest == "db" {
                        args.db_path = Some(PathBuf::from(value));
                    } else {
                        args.flags.insert(rest.to_string(), value.clone());
                    }
                    index += 2;
                    continue;
                }
                _ => {
                    // a flag with no value: a switch.
                    args.switches.push(rest.to_string());
                    index += 1;
                    continue;
                }
            }
        }

        // --- a word ----------------------
        args.positional.push(word.clone());
        index += 1;
    }

    args
}

// ========================================
// --- HELPERS ----------------------------
// the value of a flag, or None when it was not given.
pub fn db_admin_flag_fun<'a>(args: &'a DB_ADMIN_ARGS, name: &str) -> Option<&'a str> {
    args.flags.get(name).map(|value| value.as_str())
}

// was this switch given?
pub fn db_admin_switch_fun(args: &DB_ADMIN_ARGS, name: &str) -> bool {
    args.switches.iter().any(|switch| switch == name)
}

// the ids of a comma separated flag, with the blanks dropped.
//
// --- WHY --- a COMMA separated list and not a repeated flag: the shell
// line stays short and a script can build it in one string.
pub fn db_admin_id_list_fun(args: &DB_ADMIN_ARGS, name: &str) -> Vec<String> {
    match args.flags.get(name) {
        Some(value) => value
            .split(',')
            .map(|part| part.trim())
            .filter(|part| !part.is_empty())
            .map(|part| part.to_string())
            .collect(),
        None => Vec::new(),
    }
}
// ========================================
// copyright 2026 by sleet.near
// ========================================
