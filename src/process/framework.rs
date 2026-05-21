// Framework detection logic.
//
// The dictionary of frameworks lives in `framework_rules.rs`. This file
// only contains the matching pipeline; add a new framework by editing
// the dictionary, not this file.

use super::framework_rules::{Rule, RULES};
use super::FrameworkKind;

/// Detects which Node.js framework a process is running.
///
/// Detection is process-local only — based on the process name and the
/// command line. Filesystem inspection (e.g. reading package.json) is
/// deliberately avoided so that globally launched processes (npx-run
/// MCP servers, CLI tools, etc.) are not misclassified by an inherited
/// cwd that happens to belong to an unrelated framework project.
pub struct FrameworkDetector;

impl FrameworkDetector {
    /// Run detection. The `cwd` parameter is accepted for call-site
    /// compatibility but is not used; the second tuple element (version)
    /// is always `None`.
    pub fn detect(name: &str, command: &str, _cwd: &str) -> (FrameworkKind, Option<String>) {
        let normalized_name = normalize_name(name);
        let cmd_binary = command_binary(command);

        // Priority 1: exact name match.
        for rule in RULES {
            if matches_name(rule, normalized_name) {
                return (rule.framework.clone(), None);
            }
        }

        // Priority 2: command binary exact match.
        if let Some(bin) = cmd_binary {
            for rule in RULES {
                if matches_binary(rule, bin) {
                    return (rule.framework.clone(), None);
                }
            }
        }

        // Priority 3: command substring.
        for rule in RULES {
            if matches_substring(rule, command) {
                return (rule.framework.clone(), None);
            }
        }

        (FrameworkKind::Generic, None)
    }

    /// Helper: detect by process name only (priority 1).
    pub fn detect_by_name(name: &str) -> Option<FrameworkKind> {
        let normalized = normalize_name(name);
        for rule in RULES {
            if matches_name(rule, normalized) {
                return Some(rule.framework.clone());
            }
        }
        None
    }

    /// Helper: detect by command substring only (priority 3).
    pub fn detect_by_command(command: &str) -> Option<FrameworkKind> {
        for rule in RULES {
            if matches_substring(rule, command) {
                return Some(rule.framework.clone());
            }
        }
        None
    }
}

fn matches_name(rule: &Rule, name: &str) -> bool {
    rule.name_exact.iter().any(|n| *n == name)
}

fn matches_binary(rule: &Rule, binary: &str) -> bool {
    rule.command_binary.iter().any(|n| *n == binary)
}

fn matches_substring(rule: &Rule, command: &str) -> bool {
    rule.command_contains.iter().any(|s| command.contains(s))
}

/// macOS truncates the `comm` field to 16 chars, so a process that set
/// `process.title = "next-server (v16.2.4)"` arrives as
/// `"next-server (v16"`. Strip the trailing version blob by taking the
/// first whitespace-separated token before exact-matching.
fn normalize_name(name: &str) -> &str {
    name.split_whitespace().next().unwrap_or(name)
}

/// Basename of the command's first whitespace-separated token.
fn command_binary(command: &str) -> Option<&str> {
    let first = command.split_whitespace().next()?;
    let basename = first.rsplit('/').next().unwrap_or(first);
    if basename.is_empty() {
        None
    } else {
        Some(basename)
    }
}
