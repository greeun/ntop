# Multi-Runtime Server Monitoring + Node-Only Toggle Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend ntop from Node-only to a multi-runtime server monitor (Node, Python, Java, Deno, Bun, Ruby, PHP, .NET) using name/command rules, with an `n` key that toggles a Node-only view.

**Architecture:** Introduce a `Runtime` enum and a unified rule-table `classify()` that returns `(Runtime, FrameworkKind)`. Detection is two-tier: framework rules (3-pass priority) resolve first, then runtime-generic rules. The scanner uses `classify` as its filter and sets both `runtime` and `framework`, removing the duplicate `FrameworkDetector::detect` calls in `main.rs`. `ProcessInfo.is_node: bool` becomes `runtime: Option<Runtime>`.

**Tech Stack:** Rust, sysinfo, ratatui (TUI), serde_json, clap; tests via `cargo test`.

**Design spec:** `docs/superpowers/specs/2026-06-07-multi-runtime-server-monitoring-design.md`

---

## File Structure

- `src/process/mod.rs` — `Runtime` enum (new), `FrameworkKind` new variants, `ProcessInfo.runtime` field + `is_server()`/`is_node()` helpers.
- `src/process/framework_rules.rs` — `Rule.runtime` field; split into `FRAMEWORK_RULES` and `RUNTIME_RULES`.
- `src/process/framework.rs` — `classify()` (new) + `match_tier()` helper; `detect*` reimplemented over `FRAMEWORK_RULES`.
- `src/process/scanner.rs` — use `classify` as the filter; set `runtime` + `framework`; remove `is_node_*` helpers and name tables.
- `src/main.rs` — remove the three `FrameworkDetector::detect` loops; JSON `is_node` → `runtime`; info `Type` → `Runtime`.
- `src/tui/app.rs` — `node_only` field, `toggle_node_only()`, `rebuild_view()` predicate, `matches_filter` runtime term.
- `src/tui/ui.rs` — `n` global key.
- `src/tui/widgets/status_bar.rs` — `Servers:`/`Nodes:` label + `[Node-only]` indicator + server count.
- `src/tui/widgets/process_list.rs` — `is_server()` coloring; runtime badge.
- `src/tui/widgets/info_tab.rs` — `Runtime` row.
- `tests/types_test.rs`, `tests/framework_test.rs`, `tests/scanner_test.rs`, `tests/filter_test.rs` — updated/new tests.
- `README.md` — multi-runtime docs + `n` key.

---

## Task 1: Add `Runtime` enum and `FrameworkKind` variants

**Files:**
- Modify: `src/process/mod.rs`
- Test: `tests/types_test.rs`

- [ ] **Step 1: Write the failing tests**

Add to `tests/types_test.rs` (top `use` already imports `FrameworkKind`, `HealthStatus`, `ProcessInfo`; add `Runtime`):

Change the import line to:
```rust
use ntop::process::{FrameworkKind, HealthStatus, ProcessInfo, Runtime};
```

Append these tests:
```rust
#[test]
fn test_runtime_display() {
    assert_eq!(format!("{}", Runtime::Node), "Node");
    assert_eq!(format!("{}", Runtime::Python), "Python");
    assert_eq!(format!("{}", Runtime::Java), "Java");
    assert_eq!(format!("{}", Runtime::Deno), "Deno");
    assert_eq!(format!("{}", Runtime::Bun), "Bun");
    assert_eq!(format!("{}", Runtime::Ruby), "Ruby");
    assert_eq!(format!("{}", Runtime::Php), "PHP");
    assert_eq!(format!("{}", Runtime::DotNet), ".NET");
}

#[test]
fn test_runtime_serialization() {
    let variants = vec![
        Runtime::Node, Runtime::Python, Runtime::Java, Runtime::Deno,
        Runtime::Bun, Runtime::Ruby, Runtime::Php, Runtime::DotNet,
    ];
    for v in variants {
        let json = serde_json::to_string(&v).unwrap();
        let back: Runtime = serde_json::from_str(&json).unwrap();
        assert_eq!(v, back);
    }
}

#[test]
fn test_new_framework_kind_display() {
    assert_eq!(format!("{}", FrameworkKind::FastApi), "FastAPI");
    assert_eq!(format!("{}", FrameworkKind::Flask), "Flask");
    assert_eq!(format!("{}", FrameworkKind::Django), "Django");
    assert_eq!(format!("{}", FrameworkKind::SpringBoot), "Spring Boot");
    assert_eq!(format!("{}", FrameworkKind::Rails), "Rails");
    assert_eq!(format!("{}", FrameworkKind::Laravel), "Laravel");
    assert_eq!(format!("{}", FrameworkKind::AspNet), "ASP.NET");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --release --test types_test 2>&1 | tail -20`
Expected: compile error — `Runtime` and the new `FrameworkKind` variants do not exist.

- [ ] **Step 3: Add the `Runtime` enum and `FrameworkKind` variants**

In `src/process/mod.rs`, after the existing `use` lines and before `FrameworkKind`, add:
```rust
/// Server runtime / language family a process belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Runtime {
    Node,
    Python,
    Java,
    Deno,
    Bun,
    Ruby,
    Php,
    DotNet,
}

impl fmt::Display for Runtime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Runtime::Node => write!(f, "Node"),
            Runtime::Python => write!(f, "Python"),
            Runtime::Java => write!(f, "Java"),
            Runtime::Deno => write!(f, "Deno"),
            Runtime::Bun => write!(f, "Bun"),
            Runtime::Ruby => write!(f, "Ruby"),
            Runtime::Php => write!(f, "PHP"),
            Runtime::DotNet => write!(f, ".NET"),
        }
    }
}
```

Extend the `FrameworkKind` enum variants (add after `Hapi`, before `Generic`):
```rust
    FastApi,
    Flask,
    Django,
    SpringBoot,
    Rails,
    Laravel,
    AspNet,
```

Extend the `FrameworkKind` `Display` match arms (add before the `Generic` arm):
```rust
            FrameworkKind::FastApi => write!(f, "FastAPI"),
            FrameworkKind::Flask => write!(f, "Flask"),
            FrameworkKind::Django => write!(f, "Django"),
            FrameworkKind::SpringBoot => write!(f, "Spring Boot"),
            FrameworkKind::Rails => write!(f, "Rails"),
            FrameworkKind::Laravel => write!(f, "Laravel"),
            FrameworkKind::AspNet => write!(f, "ASP.NET"),
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --release --test types_test 2>&1 | tail -20`
Expected: all `types_test` tests PASS (the pre-existing `test_framework_kind_serialization` still passes — new variants are additive and not yet in its list; updated in Task 3).

- [ ] **Step 5: Commit**

```bash
git add src/process/mod.rs tests/types_test.rs
git commit -m "feat(process): add Runtime enum and multi-runtime FrameworkKind variants"
```

---

## Task 2: Unified `classify()` over two-tier rule tables

**Files:**
- Modify: `src/process/framework_rules.rs`
- Modify: `src/process/framework.rs`
- Test: `tests/framework_test.rs`

- [ ] **Step 1: Write the failing tests**

Append to `tests/framework_test.rs` (add imports at top: change line 1-2 to):
```rust
use ntop::config::Config;
use ntop::process::framework::FrameworkDetector;
use ntop::process::{FrameworkKind, Runtime};
```

Append:
```rust
// ─── classify: runtime + framework ───────────────────────────────────

fn cfg() -> Config { Config::default() }

#[test]
fn test_classify_node_generic() {
    assert_eq!(
        FrameworkDetector::classify("node", "node server.js", &cfg()),
        Some((Runtime::Node, FrameworkKind::Generic))
    );
}

#[test]
fn test_classify_nextjs() {
    assert_eq!(
        FrameworkDetector::classify("next-server", "next-server (v16)", &cfg()),
        Some((Runtime::Node, FrameworkKind::NextJs))
    );
}

#[test]
fn test_classify_python_generic_uvicorn() {
    assert_eq!(
        FrameworkDetector::classify("uvicorn", "/usr/bin/python -m uvicorn app:app", &cfg()),
        Some((Runtime::Python, FrameworkKind::Generic))
    );
}

#[test]
fn test_classify_fastapi_beats_python_name() {
    // command contains "fastapi" -> framework tier wins over the python name.
    assert_eq!(
        FrameworkDetector::classify("python", "python -m uvicorn main:app --factory fastapi", &cfg()),
        Some((Runtime::Python, FrameworkKind::FastApi))
    );
}

#[test]
fn test_classify_django() {
    assert_eq!(
        FrameworkDetector::classify("python", "python manage.py runserver", &cfg()),
        Some((Runtime::Python, FrameworkKind::Django))
    );
}

#[test]
fn test_classify_java_generic_and_spring() {
    assert_eq!(
        FrameworkDetector::classify("java", "java -jar app.jar", &cfg()),
        Some((Runtime::Java, FrameworkKind::Generic))
    );
    assert_eq!(
        FrameworkDetector::classify("java", "java org.springframework.boot.loader.JarLauncher", &cfg()),
        Some((Runtime::Java, FrameworkKind::SpringBoot))
    );
}

#[test]
fn test_classify_ruby_and_rails() {
    assert_eq!(
        FrameworkDetector::classify("ruby", "bin/rails server", &cfg()),
        Some((Runtime::Ruby, FrameworkKind::Rails))
    );
    assert_eq!(
        FrameworkDetector::classify("puma", "puma -C config/puma.rb", &cfg()),
        Some((Runtime::Ruby, FrameworkKind::Generic))
    );
}

#[test]
fn test_classify_php_and_laravel() {
    assert_eq!(
        FrameworkDetector::classify("php", "php artisan serve", &cfg()),
        Some((Runtime::Php, FrameworkKind::Laravel))
    );
    assert_eq!(
        FrameworkDetector::classify("php-fpm", "php-fpm", &cfg()),
        Some((Runtime::Php, FrameworkKind::Generic))
    );
}

#[test]
fn test_classify_dotnet_deno_bun() {
    assert_eq!(
        FrameworkDetector::classify("dotnet", "dotnet MyApp.dll", &cfg()),
        Some((Runtime::DotNet, FrameworkKind::Generic))
    );
    assert_eq!(
        FrameworkDetector::classify("deno", "deno run --allow-net server.ts", &cfg()),
        Some((Runtime::Deno, FrameworkKind::Generic))
    );
    assert_eq!(
        FrameworkDetector::classify("bun", "bun run start", &cfg()),
        Some((Runtime::Bun, FrameworkKind::Generic))
    );
}

#[test]
fn test_classify_non_server_is_none() {
    assert_eq!(FrameworkDetector::classify("bash", "bash deploy.sh", &cfg()), None);
    assert_eq!(FrameworkDetector::classify("ssh", "ssh user@host", &cfg()), None);
}

#[test]
fn test_classify_tsx_is_config_gated() {
    let mut c = Config::default();
    assert_eq!(FrameworkDetector::classify("tsx", "tsx watch src/index.ts", &c), None);
    c.filter.include_tsx = true;
    assert_eq!(
        FrameworkDetector::classify("tsx", "tsx watch src/index.ts", &c),
        Some((Runtime::Node, FrameworkKind::Generic))
    );
}

#[test]
fn test_classify_npx_mcp_in_nextjs_cwd_is_node_generic() {
    // Regression: process-local only; an npx MCP server is Node/Generic, not Next.
    assert_eq!(
        FrameworkDetector::classify(
            "node",
            "node /Users/u/.npm/_npx/abc/node_modules/.bin/context7-mcp",
            &cfg()
        ),
        Some((Runtime::Node, FrameworkKind::Generic))
    );
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --release --test framework_test 2>&1 | tail -20`
Expected: compile error — `FrameworkDetector::classify` and `Rule.runtime` do not exist.

- [ ] **Step 3: Rewrite the rule tables**

Replace the entire contents of `src/process/framework_rules.rs` with:
```rust
// Framework + runtime detection rules — the dictionary that drives tagging.
//
// To add a new framework: append a `Rule` to `FRAMEWORK_RULES`.
// To add a new runtime:   append a `Rule` to `RUNTIME_RULES`.
// Then add the variant to `FrameworkKind` / `Runtime` in `process/mod.rs`.
// No other code changes are required; `framework::classify` iterates these.

use super::{FrameworkKind, Runtime};

/// One detection rule = a (runtime, framework) tag plus the matchers that
/// fire it.
///
/// Three signal types, checked in reliability order within a tier:
/// 1. `name_exact`      — exact match against the normalized process name.
/// 2. `command_binary`  — exact match against the basename of the command's
///                        first whitespace-separated token.
/// 3. `command_contains`— substring search in the full command line. Use the
///                        most specific form possible to avoid false positives.
pub struct Rule {
    pub runtime: Runtime,
    pub framework: FrameworkKind,
    pub name_exact: &'static [&'static str],
    pub command_binary: &'static [&'static str],
    pub command_contains: &'static [&'static str],
}

/// Framework-specific rules. Resolved FIRST, as a tier, so a specific
/// framework substring (e.g. `fastapi`) beats a generic runtime name match
/// (e.g. `python`). Within this tier the three signal types apply in
/// priority order across all rules.
pub const FRAMEWORK_RULES: &[Rule] = &[
    Rule {
        runtime: Runtime::Node,
        framework: FrameworkKind::NextJs,
        name_exact: &["next-server", "next-router-worker", "next-router-page-worker"],
        command_binary: &["next-server", "next-router-worker", "next-router-page-worker"],
        command_contains: &["node_modules/.bin/next"],
    },
    Rule {
        runtime: Runtime::Node,
        framework: FrameworkKind::Nuxt,
        name_exact: &["nuxt", "nuxi"],
        command_binary: &["nuxt", "nuxi"],
        command_contains: &["node_modules/.bin/nuxt"],
    },
    Rule {
        runtime: Runtime::Node,
        framework: FrameworkKind::NestJs,
        name_exact: &[],
        command_binary: &[],
        command_contains: &["node_modules/.bin/nest"],
    },
    Rule {
        runtime: Runtime::Python,
        framework: FrameworkKind::Django,
        name_exact: &[],
        command_binary: &[],
        command_contains: &["manage.py", "django"],
    },
    Rule {
        runtime: Runtime::Python,
        framework: FrameworkKind::Flask,
        name_exact: &[],
        command_binary: &[],
        command_contains: &["flask"],
    },
    Rule {
        runtime: Runtime::Python,
        framework: FrameworkKind::FastApi,
        name_exact: &[],
        command_binary: &[],
        command_contains: &["fastapi"],
    },
    Rule {
        runtime: Runtime::Java,
        framework: FrameworkKind::SpringBoot,
        name_exact: &[],
        command_binary: &[],
        command_contains: &["org.springframework", "spring-boot"],
    },
    Rule {
        runtime: Runtime::Ruby,
        framework: FrameworkKind::Rails,
        name_exact: &["rails"],
        command_binary: &["rails"],
        command_contains: &["rails"],
    },
    Rule {
        runtime: Runtime::Php,
        framework: FrameworkKind::Laravel,
        name_exact: &[],
        command_binary: &[],
        command_contains: &["artisan"],
    },
    Rule {
        runtime: Runtime::DotNet,
        framework: FrameworkKind::AspNet,
        name_exact: &[],
        command_binary: &[],
        command_contains: &["Microsoft.AspNetCore", "aspnet"],
    },
];

/// Runtime-generic rules. Resolved SECOND, only when no framework rule fired.
/// These match the bare interpreter / VM by name.
pub const RUNTIME_RULES: &[Rule] = &[
    Rule {
        runtime: Runtime::Node,
        framework: FrameworkKind::Generic,
        name_exact: &["node"],
        command_binary: &["node"],
        command_contains: &[],
    },
    Rule {
        runtime: Runtime::Python,
        framework: FrameworkKind::Generic,
        name_exact: &["python", "python3", "uvicorn", "gunicorn", "hypercorn", "celery"],
        command_binary: &["python", "python3", "uvicorn", "gunicorn", "hypercorn", "celery"],
        command_contains: &[],
    },
    Rule {
        runtime: Runtime::Java,
        framework: FrameworkKind::Generic,
        name_exact: &["java"],
        command_binary: &["java"],
        command_contains: &[],
    },
    Rule {
        runtime: Runtime::Ruby,
        framework: FrameworkKind::Generic,
        name_exact: &["ruby", "puma", "unicorn", "rackup"],
        command_binary: &["ruby", "puma", "unicorn", "rackup"],
        command_contains: &[],
    },
    Rule {
        runtime: Runtime::Php,
        framework: FrameworkKind::Generic,
        name_exact: &["php", "php-fpm"],
        command_binary: &["php", "php-fpm"],
        command_contains: &[],
    },
    Rule {
        runtime: Runtime::DotNet,
        framework: FrameworkKind::Generic,
        name_exact: &["dotnet"],
        command_binary: &["dotnet"],
        command_contains: &[],
    },
    Rule {
        runtime: Runtime::Deno,
        framework: FrameworkKind::Generic,
        name_exact: &["deno"],
        command_binary: &["deno"],
        command_contains: &[],
    },
    Rule {
        runtime: Runtime::Bun,
        framework: FrameworkKind::Generic,
        name_exact: &["bun"],
        command_binary: &["bun"],
        command_contains: &[],
    },
];
```

- [ ] **Step 4: Rewrite the detector**

Replace the entire contents of `src/process/framework.rs` with:
```rust
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
    pub fn classify(name: &str, command: &str, config: &Config) -> Option<(Runtime, FrameworkKind)> {
        if let Some(rule) = match_tier(FRAMEWORK_RULES, name, command) {
            return Some((rule.runtime, rule.framework.clone()));
        }
        if let Some(rule) = match_tier(RUNTIME_RULES, name, command) {
            return Some((rule.runtime, rule.framework.clone()));
        }
        // Config-gated dev runners — opt-in JS/TS runners on the Node runtime.
        let nn = normalize_name(name);
        let bin = command_binary(command);
        if config.filter.include_tsx && (nn == "tsx" || bin == Some("tsx")) {
            return Some((Runtime::Node, FrameworkKind::Generic));
        }
        if config.filter.include_ts_node && (nn == "ts-node" || bin == Some("ts-node")) {
            return Some((Runtime::Node, FrameworkKind::Generic));
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
            .find(|r| r.name_exact.iter().any(|n| *n == normalized))
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
    for rule in rules {
        if rule.name_exact.iter().any(|n| *n == normalized) {
            return Some(rule);
        }
    }
    // Priority 2: command binary exact match.
    if let Some(bin) = command_binary(command) {
        for rule in rules {
            if rule.command_binary.iter().any(|n| *n == bin) {
                return Some(rule);
            }
        }
    }
    // Priority 3: command substring.
    for rule in rules {
        if rule.command_contains.iter().any(|s| command.contains(s)) {
            return Some(rule);
        }
    }
    None
}

/// macOS truncates the `comm` field to 16 chars, so a process that set
/// `process.title = "next-server (v16.2.4)"` arrives truncated. Strip the
/// trailing version blob by taking the first whitespace-separated token.
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
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --release --test framework_test 2>&1 | tail -25`
Expected: all framework tests PASS — both the new `classify` tests and the pre-existing `detect*` regression tests (truncated comm, npx-in-cwd, priority).

- [ ] **Step 6: Commit**

```bash
git add src/process/framework_rules.rs src/process/framework.rs tests/framework_test.rs
git commit -m "feat(process): two-tier classify() returning (Runtime, FrameworkKind)"
```

---

## Task 3: Replace `is_node` with `runtime` and wire the scanner

This task changes `ProcessInfo`'s shape, so every call site must update in one
commit to keep the tree compiling.

**Files:**
- Modify: `src/process/mod.rs`
- Modify: `src/process/scanner.rs`
- Modify: `src/main.rs`
- Modify: `src/tui/widgets/process_list.rs`
- Modify: `tests/types_test.rs`, `tests/scanner_test.rs`

- [ ] **Step 1: Update the failing tests first**

In `tests/types_test.rs`, replace the last line of `test_process_info_default`:
```rust
    assert!(!info.is_node);
```
with:
```rust
    assert!(info.runtime.is_none());
    assert!(!info.is_node());
    assert!(!info.is_server());
```

In `tests/types_test.rs`, extend the variant list in `test_framework_kind_serialization` to include the new variants:
```rust
    let variants = vec![
        FrameworkKind::NextJs,
        FrameworkKind::Express,
        FrameworkKind::Fastify,
        FrameworkKind::NestJs,
        FrameworkKind::Nuxt,
        FrameworkKind::Koa,
        FrameworkKind::Hapi,
        FrameworkKind::FastApi,
        FrameworkKind::Flask,
        FrameworkKind::Django,
        FrameworkKind::SpringBoot,
        FrameworkKind::Rails,
        FrameworkKind::Laravel,
        FrameworkKind::AspNet,
        FrameworkKind::Generic,
    ];
```

Replace the entire contents of `tests/scanner_test.rs` with:
```rust
use ntop::config::Config;
use ntop::process::framework::FrameworkDetector;
use ntop::process::scanner::ProcessScanner;
use ntop::process::Runtime;

#[test]
fn test_scanner_returns_vec() {
    let config = Config::default();
    let mut scanner = ProcessScanner::new(&config);
    let processes = scanner.scan();
    for p in &processes {
        assert!(p.pid > 0);
        assert!(!p.name.is_empty());
    }
}

#[test]
fn test_scanner_runtime_classification_is_consistent() {
    let config = Config::default();
    let mut scanner = ProcessScanner::new(&config);
    let processes = scanner.scan();

    for p in &processes {
        // is_server() and is_node() must agree with the runtime field.
        assert_eq!(p.is_server(), p.runtime.is_some());
        assert_eq!(p.is_node(), p.runtime == Some(Runtime::Node));
        // A classified server's runtime must match what classify() returns
        // for its own name/command (servers only — tree parents are None).
        if let Some(rt) = p.runtime {
            let classified = FrameworkDetector::classify(&p.name, &p.command, &config);
            assert_eq!(classified.map(|(r, _)| r), Some(rt));
        }
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --release --test types_test --test scanner_test 2>&1 | tail -20`
Expected: compile error — `is_node` field/`runtime` field/`is_server()` mismatch.

- [ ] **Step 3: Change the `ProcessInfo` data model**

In `src/process/mod.rs`, in the `ProcessInfo` struct, replace:
```rust
    /// True for processes the scanner classifies as real Node workloads
    /// (matched by binary name or `node` command). False for tree-context
    /// parents the second pass backfills (e.g. launchd, claude).
    pub is_node: bool,
```
with:
```rust
    /// `Some(runtime)` for processes the scanner classifies as servers.
    /// `None` for tree-context parents the second pass backfills (e.g.
    /// launchd, claude) so the tree view is rooted.
    pub runtime: Option<Runtime>,
```

In `ProcessInfo::new`, replace:
```rust
            is_node: false,
```
with:
```rust
            runtime: None,
```

Add these helpers to `impl ProcessInfo` (after `new`, before `uptime_display`):
```rust
    /// True if this process is a classified server (any runtime).
    pub fn is_server(&self) -> bool {
        self.runtime.is_some()
    }

    /// True if this process is a Node server specifically.
    pub fn is_node(&self) -> bool {
        self.runtime == Some(Runtime::Node)
    }
```

- [ ] **Step 4: Rewrite the scanner to classify via the rule table**

In `src/process/scanner.rs`, replace the top `use` block and constants. Change:
```rust
use crate::config::Config;
use crate::process::ProcessInfo;
use std::collections::HashSet;
use std::time::Duration;
use sysinfo::{ProcessRefreshKind, System, UpdateKind};
```
to:
```rust
use crate::config::Config;
use crate::process::framework::FrameworkDetector;
use crate::process::ProcessInfo;
use std::collections::HashSet;
use std::time::Duration;
use sysinfo::{ProcessRefreshKind, System, UpdateKind};
```

Delete the `NODE_PROCESS_NAMES` and `OPTIONAL_RUNTIMES` constants (lines 18-29).

Replace the first pass of `scan()` (the `for (pid, process) in self.sys.processes()` loop) with:
```rust
        // First pass: collect server processes (classified by the rule table).
        for (pid, process) in self.sys.processes() {
            let name = process.name().to_string_lossy().to_string();
            let cmd_parts: Vec<String> = process
                .cmd()
                .iter()
                .map(|s| s.to_string_lossy().to_string())
                .collect();
            let command = cmd_parts.join(" ");

            let Some((runtime, framework)) =
                FrameworkDetector::classify(&name, &command, self.config)
            else {
                continue;
            };

            let mut info = Self::collect_process_info(process, pid.as_u32());
            info.runtime = Some(runtime);
            info.framework = framework;
            node_pids.insert(pid.as_u32());
            results.push(info);
        }
```

(Keep the `let mut node_pids = HashSet::new();` line and the entire second
pass unchanged — backfilled parents keep the default `runtime: None`.)

Delete the now-unused helper methods `is_node_process_name`,
`is_node_process_name_with_config`, and `is_node_command` (lines 169-193).

- [ ] **Step 5: Update `main.rs` — remove duplicate detection, update output**

In `src/main.rs` `do_scan` (around line 154-170), replace the per-process loop body so it no longer calls `FrameworkDetector::detect` (the scanner set `framework` already):
```rust
    for proc in &mut processes {
        // Framework/runtime already set by the scanner's classify(); only
        // listening ports remain to be filled in here.
        if let Some(conns) = net_map.get(&proc.pid) {
            let ports: Vec<u16> = conns
                .iter()
                .filter(|c| c.state == "LISTEN")
                .map(|c| c.local_addr.port())
                .collect();
            if !ports.is_empty() {
                proc.ports = ports;
            }
        }
    }
```

Apply the identical edit to the loop in `cmd_list` (around line 206-221) —
remove the `FrameworkDetector::detect` lines, keep the ports block.

In `print_json` (around line 289), replace:
```rust
                "is_node": proc.is_node,
```
with:
```rust
                "runtime": proc.runtime.map(|r| r.to_string()),
```

In `cmd_info` (around line 466-483), remove the `let (framework, version) = FrameworkDetector::detect(...)` line and replace the `Type`/`Framework`/`Version` prints:
```rust
            println!("  Type:      {}", if process.is_node { "Node" } else { "Tree parent" });
            println!("  Framework: {}", framework);
            println!(
                "  Version:   {}",
                version.as_deref().unwrap_or("-")
            );
```
with:
```rust
            println!(
                "  Runtime:   {}",
                process.runtime.map(|r| r.to_string()).unwrap_or_else(|| "—".to_string())
            );
            println!("  Framework: {}", process.framework);
            println!(
                "  Version:   {}",
                process.framework_version.as_deref().unwrap_or("-")
            );
```

If `FrameworkDetector` is now unused in `main.rs`, remove its `use` import
(line 16) to avoid an unused-import warning. (Verify with the build step.)

- [ ] **Step 6: Update `process_list.rs` coloring**

In `src/tui/widgets/process_list.rs`, replace the `base_fg` block (lines 143-147):
```rust
            let base_fg = if proc_info.is_node {
                Color::LightCyan
            } else {
                Color::DarkGray
            };
```
with:
```rust
            let base_fg = if proc_info.is_server() {
                Color::LightCyan
            } else {
                Color::DarkGray
            };
```

And replace the `selected_fg` block (lines 152-156):
```rust
                let selected_fg = if proc_info.is_node {
                    Color::LightCyan
                } else {
                    Color::Gray
                };
```
with:
```rust
                let selected_fg = if proc_info.is_server() {
                    Color::LightCyan
                } else {
                    Color::Gray
                };
```

- [ ] **Step 7: Build and run the full suite**

Run: `cargo build --release 2>&1 | tail -20`
Expected: compiles with no errors and no unused-import/dead-code warnings.

Run: `cargo test --release 2>&1 | grep -E "test result:|error"`
Expected: every test binary reports `test result: ok` (0 failures).

- [ ] **Step 8: Commit**

```bash
git add src/process/mod.rs src/process/scanner.rs src/main.rs src/tui/widgets/process_list.rs tests/types_test.rs tests/scanner_test.rs
git commit -m "feat: scanner classifies all runtimes; ProcessInfo.is_node -> runtime"
```

---

## Task 4: Node-only toggle (`n` key)

**Files:**
- Modify: `src/tui/app.rs`
- Modify: `src/tui/ui.rs`
- Test: `tests/filter_test.rs`

- [ ] **Step 1: Write the failing tests**

In `tests/filter_test.rs`, change the import line:
```rust
use ntop::process::{FrameworkKind, ProcessInfo, Runtime};
use ntop::tui::app::App;
use ntop::config::Config;
```

Append:
```rust
fn server(pid: u32, name: &str, runtime: Option<Runtime>) -> ProcessInfo {
    let mut p = ProcessInfo::new(pid, name);
    p.runtime = runtime;
    p
}

#[test]
fn test_matches_filter_runtime() {
    let mut p = ProcessInfo::new(1, "uvicorn");
    p.runtime = Some(Runtime::Python);
    assert!(App::matches_filter(&p, "python"));
    assert!(App::matches_filter(&p, "PYTHON"));
    assert!(!App::matches_filter(&p, "java"));
}

#[test]
fn test_node_only_toggle_hides_other_runtimes() {
    let mut app = App::new(Config::default());
    app.update_processes(vec![
        server(1, "node", Some(Runtime::Node)),
        server(2, "uvicorn", Some(Runtime::Python)),
        server(3, "java", Some(Runtime::Java)),
        server(4, "launchd", None), // tree-context parent
    ]);

    // Default: all rows visible.
    assert_eq!(app.flat_list.len(), 4);

    // Node-only: Node servers + tree parents remain; Python/Java hidden.
    app.toggle_node_only();
    assert!(app.node_only);
    let pids: Vec<u32> = app.flat_list.iter().map(|(p, _)| p.pid).collect();
    assert!(pids.contains(&1)); // node
    assert!(pids.contains(&4)); // tree parent
    assert!(!pids.contains(&2)); // python hidden
    assert!(!pids.contains(&3)); // java hidden

    // Toggling off restores everything.
    app.toggle_node_only();
    assert!(!app.node_only);
    assert_eq!(app.flat_list.len(), 4);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --release --test filter_test 2>&1 | tail -20`
Expected: compile error — `App::new` may be private / `node_only` / `toggle_node_only` do not exist.

> Note: if `App::new` or `update_processes` are not usable from an integration
> test, verify they are `pub` (they are: `app.rs:188`, `app.rs:245`). `flat_list`
> and `node_only` are `pub` fields.

- [ ] **Step 3: Add `node_only` state and toggle**

In `src/tui/app.rs`, add the field to the `App` struct (after `focus: FocusPanel,`):
```rust
    /// When true, the list shows only Node servers (+ tree-context parents).
    pub node_only: bool,
```

In `App::new`, add to the struct literal (after `focus: FocusPanel::ProcessList,`):
```rust
            node_only: false,
```

Add the runtime term to `matches_filter` — replace the body's final expression:
```rust
        p.name.to_lowercase().contains(&f)
            || p.command.to_lowercase().contains(&f)
            || p.pid.to_string().contains(&f)
            || p.framework.to_string().to_lowercase().contains(&f)
            || p.ports.iter().any(|port| port.to_string().contains(&f))
```
with:
```rust
        p.name.to_lowercase().contains(&f)
            || p.command.to_lowercase().contains(&f)
            || p.pid.to_string().contains(&f)
            || p.framework.to_string().to_lowercase().contains(&f)
            || p
                .runtime
                .map(|r| r.to_string().to_lowercase().contains(&f))
                .unwrap_or(false)
            || p.ports.iter().any(|port| port.to_string().contains(&f))
```

Replace the filtering step at the top of `rebuild_view` (lines 262-270):
```rust
        let processes: Vec<ProcessInfo> = if self.filter_text.is_empty() {
            self.raw_processes.clone()
        } else {
            self.raw_processes
                .iter()
                .filter(|p| Self::matches_filter(p, &self.filter_text))
                .cloned()
                .collect()
        };
```
with:
```rust
        let node_only = self.node_only;
        let processes: Vec<ProcessInfo> = self
            .raw_processes
            .iter()
            // Node-only keeps Node servers and tree-context parents (None),
            // hiding only other-runtime servers.
            .filter(|p| !node_only || p.is_node() || p.runtime.is_none())
            .filter(|p| Self::matches_filter(p, &self.filter_text))
            .cloned()
            .collect();
```

Add the toggle method to `impl App` (after `toggle_sort`):
```rust
    /// Toggle the Node-only view filter and rebuild.
    pub fn toggle_node_only(&mut self) {
        self.node_only = !self.node_only;
        self.rebuild_view();
    }
```

Confirm `Runtime` is in scope in `app.rs`. The existing `use crate::process::ProcessInfo;` does not import it, but `is_node()` lives on `ProcessInfo` so `app.rs` itself does not name `Runtime`. No import change needed.

- [ ] **Step 4: Bind the `n` key**

In `src/tui/ui.rs`, in the global-keys `match key.code` block (after the `KeyCode::Char('e')` arm at line 285-288), add:
```rust
        KeyCode::Char('n') => {
            app.toggle_node_only();
            return;
        }
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --release --test filter_test 2>&1 | tail -20`
Expected: all `filter_test` tests PASS.

- [ ] **Step 6: Commit**

```bash
git add src/tui/app.rs src/tui/ui.rs tests/filter_test.rs
git commit -m "feat(tui): n key toggles Node-only view; runtime in text filter"
```

---

## Task 5: Display polish — status bar, badge, info row

These are UI rendering changes (no unit tests for ratatui rendering); verified
by build + manual run in Task 7.

**Files:**
- Modify: `src/tui/widgets/status_bar.rs`
- Modify: `src/tui/widgets/process_list.rs`
- Modify: `src/tui/widgets/info_tab.rs`

- [ ] **Step 1: Status bar — Servers/Nodes count + [Node-only] indicator**

In `src/tui/widgets/status_bar.rs` `render_top_bar`, replace:
```rust
    let node_count = app.flat_list.len();
```
with:
```rust
    let server_count = app.flat_list.iter().filter(|(p, _)| p.is_server()).count();
    let count_label = if app.node_only { "Nodes: " } else { "Servers: " };
```

Replace the count spans:
```rust
        Span::styled("Nodes: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{}", node_count),
            Style::default().fg(Color::White),
        ),
```
with:
```rust
        Span::styled(count_label, Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{}", server_count),
            Style::default().fg(Color::White),
        ),
        Span::styled(
            if app.node_only { " [Node-only]" } else { "" },
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ),
```

- [ ] **Step 2: Process list — runtime badge for non-Node servers**

In `src/tui/widgets/process_list.rs`, replace the `base_name` block (lines 121-125):
```rust
            let base_name = if proc_info.framework.to_string() != "Generic" {
                format!("{} ({})", proc_info.display_name(), proc_info.framework)
            } else {
                proc_info.display_name()
            };
```
with:
```rust
            let base_name = if proc_info.framework != crate::process::FrameworkKind::Generic {
                format!("{} ({})", proc_info.display_name(), proc_info.framework)
            } else if let Some(rt) = proc_info.runtime {
                if rt == crate::process::Runtime::Node {
                    // Node-generic stays clean, matching prior behavior.
                    proc_info.display_name()
                } else {
                    format!("{} [{}]", proc_info.display_name(), rt)
                }
            } else {
                proc_info.display_name()
            };
```

- [ ] **Step 3: Info tab — Runtime row**

In `src/tui/widgets/info_tab.rs`, in the `fields` vector, replace:
```rust
        ("Framework", process.framework.to_string()),
```
with:
```rust
        (
            "Runtime",
            process.runtime.map(|r| r.to_string()).unwrap_or_else(|| "-".to_string()),
        ),
        ("Framework", process.framework.to_string()),
```

- [ ] **Step 4: Build to verify**

Run: `cargo build --release 2>&1 | tail -20`
Expected: compiles with no warnings.

Run: `cargo test --release 2>&1 | grep -E "test result:|error"`
Expected: all `test result: ok`.

- [ ] **Step 5: Commit**

```bash
git add src/tui/widgets/status_bar.rs src/tui/widgets/process_list.rs src/tui/widgets/info_tab.rs
git commit -m "feat(tui): runtime-aware status bar, list badge, and info row"
```

---

## Task 6: README update

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Update title, description, and feature line**

In `README.md`, replace line 3:
```
A fast, real-time TUI tool for monitoring and managing Node.js / Next.js / Nuxt.js server processes.
```
with:
```
A fast, real-time TUI tool for monitoring and managing server processes across multiple runtimes — Node.js, Python, Java, Deno, Bun, Ruby, PHP, and .NET.
```

Replace the "Rule-based framework detection" feature bullet (line 12) with:
```
- **Rule-based runtime & framework detection** — Node/Next/Nuxt/Nest, Python/FastAPI/Flask/Django, Java/Spring Boot, Ruby/Rails, PHP/Laravel, Deno, Bun, .NET (no filesystem reads, so globally-launched CLIs aren't misclassified)
- **Node-only toggle** — press `n` to show only Node servers
```

- [ ] **Step 2: Document the `n` key**

In the Key Bindings table, add a row after the `/` row:
```
| `n` | Toggle Node-only view |
```

- [ ] **Step 3: Replace the "Supported Frameworks" section**

Replace the "Supported Frameworks" section (lines 139-154) with:
```markdown
## Supported Runtimes & Frameworks

Detection is **process-local only** — based on the process name and command
line. ntop deliberately does not read `package.json` so that globally launched
processes (e.g. `npx`-run MCP servers, CLI tools) are not misclassified.

Detection is two-tier: framework-specific rules resolve first, then
runtime-generic rules. A process matching no rule is not shown.

| Runtime | Frameworks detected | Generic match (name) |
|---------|---------------------|----------------------|
| Node    | Next.js, Nuxt.js, NestJS | `node` |
| Python  | FastAPI, Flask, Django | `python`, `python3`, `uvicorn`, `gunicorn`, `hypercorn`, `celery` |
| Java    | Spring Boot | `java` |
| Ruby    | Rails | `ruby`, `puma`, `unicorn`, `rackup` |
| PHP     | Laravel | `php`, `php-fpm` |
| .NET    | ASP.NET | `dotnet` |
| Deno    | — | `deno` |
| Bun     | — | `bun` |

Add a runtime or framework by appending one entry to
`src/process/framework_rules.rs` (`RUNTIME_RULES` or `FRAMEWORK_RULES`) — no
other code changes required.

> `tsx` / `ts-node` remain opt-in via `[filter]` config (they classify as the
> Node runtime). The `include_bun` config flag is **deprecated** — Bun is now a
> first-class runtime, always detected.
```

- [ ] **Step 4: Commit**

```bash
git add README.md
git commit -m "docs: README for multi-runtime monitoring and Node-only toggle"
```

---

## Task 7: Final verification

- [ ] **Step 1: Full test suite + build**

Run: `cargo test --release 2>&1 | grep -E "Running|test result:"`
Expected: every binary `test result: ok`, 0 failures. Total ≥ the original 119 plus the new tests.

Run: `cargo build --release 2>&1 | tail -5`
Expected: clean build, no warnings.

- [ ] **Step 2: Clippy (if available)**

Run: `cargo clippy --release 2>&1 | tail -20`
Expected: no errors (warnings acceptable if pre-existing).

- [ ] **Step 3: CLI smoke test**

Run: `cargo run --release -- list --json 2>&1 | head -30`
Expected: JSON objects include a `"runtime"` key (string or `null`); no `is_node` key.

Run: `cargo run --release -- list 2>&1 | head -15`
Expected: table prints; the FRAMEWORK column shows broader values when non-Node servers run.

- [ ] **Step 4: Manual TUI check (interactive — ask the user to run)**

```bash
cargo run --release
```
Confirm:
- Python (`uvicorn`/`gunicorn`) and Java (`java -jar`) servers appear alongside Node, if running.
- Non-Node servers show a `[Python]` / `[Java]` badge; frameworks show `(FastAPI)` etc.
- Status bar shows `Servers: N`.
- Pressing `n` filters to Node servers only and shows `Nodes: N [Node-only]`; pressing `n` again restores all runtimes.
- Info tab (`Tab`) shows a `Runtime` row.

---

## Self-Review

**Spec coverage** (against `2026-06-07-multi-runtime-server-monitoring-design.md`):
- Runtime enum + ProcessInfo.runtime + helpers → Task 1, Task 3. ✓
- FrameworkKind new variants → Task 1. ✓
- Rule.runtime + ordered/two-tier classify + scanner unify + main.rs dedup → Task 2, Task 3. ✓
- `n` Node-only toggle + rebuild_view predicate (`Some(Node) || None`) + AND with text filter → Task 4. ✓
- Status bar Servers/Nodes + [Node-only]; list badge; info Runtime row → Task 5. ✓
- CLI/JSON: `is_node` → `runtime`; info `Type` → `Runtime` → Task 3. ✓
- Config: Bun first-class, tsx/ts-node gated, include_bun deprecated → Task 2 (classify gating) + README note (Task 6). ✓
- README → Task 6. ✓
- Tests per runtime + Node-only + defaults → Tasks 1-4. ✓
- Out of scope (Go/Rust, ports, `list --runtime`, per-runtime colors, rename) → not implemented. ✓

**Placeholder scan:** No TBD/TODO; every code step shows complete code.

**Type consistency:** `classify(&str, &str, &Config) -> Option<(Runtime, FrameworkKind)>` used identically in scanner (Task 3 Step 4) and tests (Task 2/3). `is_server()`/`is_node()`/`runtime` names consistent across Tasks 1-5. `toggle_node_only()` defined in Task 4 Step 3, called in Task 4 Step 4. `FRAMEWORK_RULES`/`RUNTIME_RULES`/`match_tier` defined in Task 2 and referenced only there.
