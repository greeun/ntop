// Framework + runtime detection logic.
//
// The dictionaries live in `framework_rules.rs`. This file only contains the
// matching pipeline; add a framework/runtime by editing the dictionaries.

use super::framework_rules::{Rule, FRAMEWORK_RULES, RUNTIME_RULES};
use super::{FrameworkKind, Runtime};
use crate::config::Config;

/// Detects the runtime and framework of a process.
///
/// Detection is process-local only — based on the process name and command
/// line. Filesystem inspection (e.g. reading package.json) is deliberately
/// avoided so globally launched processes (npx MCP servers, CLI tools) are
/// not misclassified by an inherited cwd.
pub struct FrameworkDetector;

impl FrameworkDetector {
    /// Classify a process into `(Runtime, FrameworkKind)`, or `None` if it is
    /// not a recognized server.
    ///
    /// Two tiers: framework rules resolve first (so a specific framework
    /// substring beats a generic runtime name), then runtime-generic rules,
    /// then config-gated dev runners (tsx / ts-node).
    pub fn classify(
        name: &str,
        command: &str,
        config: &Config,
    ) -> Option<(Runtime, FrameworkKind)> {
        if let Some(rule) = match_tier(FRAMEWORK_RULES, name, command) {
            return Some((rule.runtime, rule.framework.clone()));
        }
        if let Some(rule) = match_tier(RUNTIME_RULES, name, command) {
            // MCP overlay: an otherwise-generic runtime process whose command
            // line carries an MCP signature is tagged MCP while keeping its
            // real runtime (Node/Python/Bun/…). Specific frameworks already
            // returned above, so this never overrides Next.js/FastAPI/etc.
            if rule.framework == FrameworkKind::Generic && is_mcp_command(command) {
                return Some((rule.runtime, FrameworkKind::Mcp));
            }
            return Some((rule.runtime, rule.framework.clone()));
        }
        // Config-gated dev runners — opt-in JS/TS runners on the Node runtime.
        let nn = normalize_name(name);
        let bin = command_binary(command);
        let is_dev_runner = (config.filter.include_tsx && (nn == "tsx" || bin == Some("tsx")))
            || (config.filter.include_ts_node && (nn == "ts-node" || bin == Some("ts-node")));
        if is_dev_runner {
            let framework = if is_mcp_command(command) {
                FrameworkKind::Mcp
            } else {
                FrameworkKind::Generic
            };
            return Some((Runtime::Node, framework));
        }
        None
    }

    /// Framework-only detection (priority across name → binary → substring),
    /// returning `Generic` when no framework rule matches. Runtime-generic
    /// rules are not consulted.
    pub fn detect(name: &str, command: &str, _cwd: &str) -> (FrameworkKind, Option<String>) {
        match match_tier(FRAMEWORK_RULES, name, command) {
            Some(rule) => (rule.framework.clone(), None),
            None => (FrameworkKind::Generic, None),
        }
    }

    /// Helper: detect framework by process name only.
    pub fn detect_by_name(name: &str) -> Option<FrameworkKind> {
        let normalized = normalize_name(name);
        FRAMEWORK_RULES
            .iter()
            .find(|r| r.name_exact.contains(&normalized))
            .map(|r| r.framework.clone())
    }

    /// Helper: detect framework by command substring only.
    pub fn detect_by_command(command: &str) -> Option<FrameworkKind> {
        FRAMEWORK_RULES
            .iter()
            .find(|r| r.command_contains.iter().any(|s| command.contains(s)))
            .map(|r| r.framework.clone())
    }
}

/// Match a single tier of rules using name → binary → substring priority.
/// Returns the first rule that fires by any signal, respecting priority order.
fn match_tier(rules: &'static [Rule], name: &str, command: &str) -> Option<&'static Rule> {
    let normalized = normalize_name(name);
    // Priority 1: exact name match.
    if let Some(rule) = rules.iter().find(|r| r.name_exact.contains(&normalized)) {
        return Some(rule);
    }
    // Priority 2: command binary exact match.
    if let Some(bin) = command_binary(command) {
        if let Some(rule) = rules.iter().find(|r| r.command_binary.contains(&bin)) {
            return Some(rule);
        }
    }
    // Priority 3: command substring.
    rules
        .iter()
        .find(|r| r.command_contains.iter().any(|s| command.contains(s)))
}

/// macOS truncates the `comm` field to 16 chars, so a process that set
/// `process.title = "next-server (v16.2.4)"` arrives truncated. Strip the
/// trailing version blob by taking the first whitespace-separated token.
fn normalize_name(name: &str) -> &str {
    name.split_whitespace().next().unwrap_or(name)
}

/// True if the command line looks like an MCP (Model Context Protocol) server.
///
/// MCP servers span runtimes, so this is an overlay on the generic-runtime
/// classification rather than a `Rule` (a `Rule` fixes one runtime). Matching
/// stays process-local:
/// - the official `@modelcontextprotocol/*` packages (substring), and
/// - the community `mcp-server-*` / `*-mcp` / `mcp_server_*` naming conventions,
///   checked per path segment so a deep `node_modules/...` path still matches
///   while unrelated names like `mcpherson` do not.
fn is_mcp_command(command: &str) -> bool {
    if command.contains("modelcontextprotocol") {
        return true;
    }
    command
        .split_whitespace()
        .flat_map(|token| token.split('/'))
        .any(|seg| {
            seg == "mcp"
                || seg.starts_with("mcp-")
                || seg.starts_with("mcp_")
                || seg.ends_with("-mcp")
                || seg.ends_with("_mcp")
                || seg.contains("mcp-server")
                || seg.contains("mcp_server")
        })
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
