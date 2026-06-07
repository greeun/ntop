# Design: Multi-Runtime Server Monitoring + Node-Only Toggle

Date: 2026-06-07

## Problem

ntop only surfaces Node.js workloads. `ProcessScanner::scan()` hard-filters
every process through `is_node` (`src/process/scanner.rs:126`) and `continue`s
on anything that isn't `node` / `next-server` / … or a `node`-prefixed command.
Servers written in Python (FastAPI/uvicorn, Django), Java (Spring Boot), Ruby,
PHP, .NET, Deno, or Bun never appear, even though they are increasingly common
backends — Python especially for AI services.

A second wrinkle: detection is split across two mechanisms that must be kept in
sync.

1. **Scanner** decides "is this a server?" via hardcoded `NODE_PROCESS_NAMES`
   and `is_node_command` (`scanner.rs:18-29,169-193`).
2. **`main.rs`** separately runs `FrameworkDetector::detect` to tag the
   framework (`main.rs:155,207,469`).

## Goal

1. Detect and monitor server processes across multiple runtimes — Node,
   Python, Java, Deno, Bun, Ruby, PHP, .NET — using the existing **name /
   command rule** approach only (no filesystem reads, no port heuristics, so
   the "no false positives for globally-launched CLIs" philosophy is kept).
2. **Unify** the two detection paths into a single rule-table classifier, so
   adding a runtime or framework is still a one-entry edit.
3. Add an `n` key that toggles **Node-only** view, preserving the original
   ntop experience on demand.

Compiled Go/Rust binaries (no recognizable interpreter name) are explicitly
**out of scope** — they cannot be classified by name alone.

## Data model change (`src/process/mod.rs`)

New `Runtime` enum:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Runtime { Node, Python, Java, Deno, Bun, Ruby, Php, DotNet }
```

`Display`: `Node`, `Python`, `Java`, `Deno`, `Bun`, `Ruby`, `PHP`, `.NET`.

`ProcessInfo` replaces the boolean with a runtime classification:

```rust
pub struct ProcessInfo {
    // ...existing fields...
    pub runtime: Option<Runtime>,   // was: pub is_node: bool
}
```

- `Some(rt)` — a monitored server process (first scan pass).
- `None` — a tree-context parent backfilled by the second pass.

Backward-compatible helpers so existing call sites read cleanly:

```rust
impl ProcessInfo {
    pub fn is_server(&self) -> bool { self.runtime.is_some() }
    pub fn is_node(&self) -> bool { self.runtime == Some(Runtime::Node) }
}
```

`ProcessInfo::new` defaults `runtime: None`.

`FrameworkKind` gains web-framework variants (the `runtime` field carries the
language, so "Python, framework unknown" is `runtime=Python, framework=Generic`
— no per-runtime `Generic*` variants needed):

- Python: `FastApi`, `Flask`, `Django`
- Java: `SpringBoot`
- Ruby: `Rails`
- PHP: `Laravel`
- .NET: `AspNet`

`Generic`, and all existing Node variants (`NextJs`, `Express`, `Fastify`,
`NestJs`, `Nuxt`, `Koa`, `Hapi`), are retained.

## Detection change — unified classifier (`framework_rules.rs` + `framework.rs`)

`Rule` gains a `runtime` field:

```rust
pub struct Rule {
    pub runtime: Runtime,
    pub framework: FrameworkKind,
    pub name_exact: &'static [&'static str],
    pub command_binary: &'static [&'static str],
    pub command_contains: &'static [&'static str],
}
```

### Matching semantics

Switch from the current three-pass priority (all `name_exact`, then all
`command_binary`, then all `command_contains`) to **ordered single-pass**: the
`RULES` table is ordered most-specific (framework) → least-specific
(runtime-generic), and the first rule whose *any* matcher fires wins. Within a
rule, matchers are still checked in reliability order: `name_exact` →
`command_binary` → `command_contains`.

This is required so a framework rule beats a runtime-generic rule for the same
process. Example: `python -m uvicorn app:main` whose command contains
`fastapi` matches the `FastApi` rule (placed before `Python`/`Generic`) instead
of being flattened to `Python`/`Generic` by the `python` name match.

Substrings remain the most specific form available
(`node_modules/.bin/next`, `org.springframework`, `manage.py`) to avoid false
positives, as the existing rule doc already advises.

### Single classifier

```rust
// framework.rs
pub fn classify(name: &str, command: &str, config: &Config)
    -> Option<(Runtime, FrameworkKind)>
```

Returns the first matching rule's `(runtime, framework)`, or `None` if no rule
matches (process is not a recognized server). Honors config gating for opt-in
dev runners (`tsx`, `ts-node`).

Representative `RULES` entries (ordered specific → generic):

| runtime | framework | signals (any match wins) |
|---------|-----------|--------------------------|
| Node    | NextJs    | name `next-server`/`next-router-worker`/`next-router-page-worker`; `node_modules/.bin/next` |
| Node    | Nuxt      | name `nuxt`/`nuxi`; `node_modules/.bin/nuxt` |
| Node    | NestJs    | `node_modules/.bin/nest` |
| Python  | Django    | `manage.py`, `django` |
| Python  | Flask     | `flask` |
| Python  | FastApi   | `fastapi` |
| Python  | Generic   | name `uvicorn`/`gunicorn`/`hypercorn`/`celery`/`python`/`python3` |
| Java    | SpringBoot| `org.springframework`, `spring-boot` |
| Java    | Generic   | name `java` |
| Ruby    | Rails     | `rails`, `puma`, `unicorn` |
| Ruby    | Generic   | name `ruby` |
| Php     | Laravel   | `artisan` |
| Php     | Generic   | name `php`/`php-fpm` |
| DotNet  | AspNet    | `aspnet` |
| DotNet  | Generic   | name `dotnet` |
| Deno    | Generic   | name `deno` |
| Bun     | Generic   | name `bun` |

`FrameworkDetector::detect` / `detect_by_name` / `detect_by_command` are
reimplemented on top of the new ordered matcher (or thin-wrapped over
`classify`). Node-only rules keep producing identical results for the existing
tests.

### Scanner change (`src/process/scanner.rs`)

The scanner becomes the single classification site:

- First pass: call `classify(name, command, config)`. `None` → skip. `Some((rt,
  fw))` → set `info.runtime = Some(rt)`, `info.framework = fw`, collect.
- Second pass (parent backfill) unchanged in shape: parents not already
  collected get `runtime = None`.

Remove the now-redundant `NODE_PROCESS_NAMES`, `OPTIONAL_RUNTIMES`,
`is_node_process_name*`, and `is_node_command` helpers (their roles fold into
the rule table + classifier). Tests that referenced them move to classifier
tests.

### `main.rs` change

Delete the three standalone `FrameworkDetector::detect` calls
(`main.rs:155-156, 207-208, 469`) — the scanner already populated
`runtime` + `framework`. This is the duplication-removal win.

## Node-only toggle (`src/tui/app.rs` + `src/tui/ui.rs`)

- `App` gains `node_only: bool` (default `false`).
- `ui.rs` global key handler adds:
  ```rust
  KeyCode::Char('n') => { app.node_only = !app.node_only; app.rebuild_view(); return; }
  ```
  (`n` is currently unbound; taken global keys are `q / s r + - x K H S e`.)
- `rebuild_view()` adds a predicate to the existing filter step: when
  `node_only`, keep a process iff
  `p.runtime == Some(Runtime::Node) || p.runtime.is_none()` — Node servers and
  tree-context parents stay; **other-runtime servers are hidden**. Composes with
  the `/` text filter via logical AND.

## Display generalization

- `status_bar.rs:41` — `Nodes: N` becomes `Servers: N` (count of
  `is_server()`). When `node_only` is active, render `Nodes: N [Node-only]`.
- `process_list.rs:143,152` — the `is_node` color branch becomes `is_server()`
  (server rows bright vs tree-context parents dim — same visual distinction,
  generalized).
- `process_list.rs:121-122` — badge: show `(<Framework>)` when
  `framework != Generic`, otherwise `[<Runtime>]` (e.g. `uvicorn [Python]`,
  `next dev (Next.js)`).
- `info_tab.rs:36` — add a `Runtime` row above the existing `Framework` row.

Per-runtime color coding is **out of scope** (server-vs-parent stays binary).

## CLI / JSON change (`src/main.rs`)

- `print_json`: replace `"is_node": <bool>` with `"runtime": "<Runtime>" | null`.
- `cmd_info`: replace `Type: Node/Tree parent` with `Runtime: <Runtime>` for
  servers and `Runtime: —` for tree parents.
- CSV and the table printer keep their columns (the `FRAMEWORK` column now
  shows the broader framework set). No ANSI color in piped output.
- A `list --runtime <name>` filter flag is **out of scope** for this change
  (possible follow-up).

## Config change (`src/config.rs`)

- Deno and Bun are **first-class runtimes, always detected**.
- `tsx` / `ts-node` remain opt-in via the existing `[filter]` flags (dev-runner
  noise control). When enabled, they classify as `runtime = Node`,
  `framework = Generic` (they execute TS on the Node runtime); `classify`
  returns `None` for them when their flag is off.
- `include_bun` is **deprecated**: the field is retained so existing config
  files still parse, but it no longer gates detection. Documented in README.

## Tests

- `framework` classifier tests extended per runtime:
  `uvicorn` → `(Python, Generic)`; command with `fastapi` → `(Python, FastApi)`;
  `manage.py runserver` → `(Python, Django)`; `java -jar app.jar` → `(Java,
  Generic)`; `org.springframework...` → `(Java, SpringBoot)`; `deno`/`bun` →
  `(Deno/Bun, Generic)`; `artisan serve` → `(Php, Laravel)`; etc.
- Existing Next/Nuxt/Nest detection tests must still pass under ordered
  single-pass matching.
- `App` test: with `node_only = true`, non-Node servers are filtered out while
  Node servers and `runtime == None` parents remain; toggling off restores all.
- `ProcessInfo::new` defaults `runtime == None`; `is_server()`/`is_node()`
  return `false`.
- Scanner test: every first-pass row has `runtime.is_some()`; every backfilled
  parent has `runtime == None`.

## README

- Retitle from "Node.js / Next.js / Nuxt.js" to a multi-runtime description.
- Replace "Supported Frameworks" with a "Supported Runtimes & Frameworks"
  table covering all eight runtimes.
- Document the `n` Node-only toggle in Key Bindings.
- Note `include_bun` deprecation.
- The binary/tool name **`ntop` is kept** (node-top heritage); renaming is out
  of scope.

## Out of scope

- Go/Rust (compiled binary) detection and any port-based heuristic.
- `list --runtime` CLI flag.
- Per-runtime color coding in the list.
- Config allowlist of runtimes to show/hide.
- Renaming the `ntop` binary.

## Verification

1. `cargo test --release` — all green (existing + new classifier/toggle tests).
2. Run `ntop` with a Node server, a Python (`uvicorn`/`gunicorn`) server, and a
   Java (`java -jar`) server running; confirm all three appear with correct
   `Runtime`/`Framework`.
3. Press `n`: only Node rows (plus tree parents) remain; status bar shows
   `[Node-only]`. Press `n` again: all server runtimes return.
4. `ntop list --format=json` shows `"runtime"` per row; `ntop info <pid>` shows
   the `Runtime` line for a Python/Java server and `—` for a tree parent.
