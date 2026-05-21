// Framework detection rules — the dictionary that drives tagging.
//
// To add a new framework:
//   1. Append a variant to `FrameworkKind` in `process/mod.rs`.
//   2. Append a `Rule` entry below.
// No other code changes are required; `framework::FrameworkDetector`
// iterates this table.

use super::FrameworkKind;

/// One detection rule = one framework tag plus the matchers that fire it.
///
/// Three signal types, evaluated in priority order across all rules:
/// 1. `name_exact`    — exact match against the (normalized) process name.
///                       Most reliable on Linux where the kernel preserves
///                       the long binary name.
/// 2. `command_binary` — exact match against the basename of the command's
///                       first whitespace-separated token. Catches
///                       `process.title`-style names that survive macOS'
///                       16-char `comm` truncation (e.g. `next-server (v16.2.4)`).
/// 3. `command_contains` — substring search in the full command line.
///                       Use the most specific form possible
///                       (e.g. `"node_modules/.bin/next"`) to avoid false
///                       positives like `nuxt` matching `linuxtimer`.
///
/// Within a priority level, rules are tried in declaration order; the
/// first rule that fires wins.
pub struct Rule {
    pub framework: FrameworkKind,
    pub name_exact: &'static [&'static str],
    pub command_binary: &'static [&'static str],
    pub command_contains: &'static [&'static str],
}

/// Framework detection dictionary.
pub const RULES: &[Rule] = &[
    Rule {
        framework: FrameworkKind::NextJs,
        name_exact: &[
            "next-server",
            "next-router-worker",
            "next-router-page-worker",
        ],
        command_binary: &[
            "next-server",
            "next-router-worker",
            "next-router-page-worker",
        ],
        command_contains: &["node_modules/.bin/next"],
    },
    Rule {
        framework: FrameworkKind::Nuxt,
        name_exact: &["nuxt", "nuxi"],
        command_binary: &["nuxt", "nuxi"],
        command_contains: &["node_modules/.bin/nuxt"],
    },
    Rule {
        framework: FrameworkKind::NestJs,
        name_exact: &[],
        command_binary: &[],
        command_contains: &["node_modules/.bin/nest"],
    },
];
