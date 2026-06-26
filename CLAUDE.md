# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`ntop` ("Node Top") is a Rust TUI + CLI for monitoring and managing server
processes across runtimes (Node, Python, Java, Deno, Bun, Ruby, PHP, .NET).
Single binary (`src/main.rs`), library crate (`src/lib.rs`). macOS + Linux +
Windows, with platform-specific code isolated in `src/process/platform.rs`.

## Commands

```bash
cargo build                       # debug build
cargo build --release             # release build
cargo run                         # launch the TUI (default, no subcommand)
cargo run -- list --json          # run a CLI subcommand
cargo test                        # full test suite (tests/ + inline #[cfg(test)])
cargo test --test framework_test  # one integration-test file
cargo test classify               # tests whose name matches "classify"
cargo clippy                      # lint — CI/commits expect clippy-clean
cargo fmt                         # format
```

Tests live in `tests/*.rs` (one file per module: `framework_test`, `scanner_test`,
`tree_test`, `killer_test`, `network_test`, `log_test`, `cli_test`, `config_test`,
`filter_test`, `types_test`). `cli_test` uses `assert_cmd` to drive the built binary.

## Runtime/framework detection — the core abstraction

All "is this a server, and what is it?" logic is **data-driven** by a rule table.
This is the main extension point.

- **`src/process/framework_rules.rs`** holds two `&[Rule]` tables: `FRAMEWORK_RULES`
  (specific frameworks, e.g. Next.js/FastAPI) and `RUNTIME_RULES` (bare
  interpreters, e.g. `node`/`python`). A `Rule` carries three matchers checked in
  reliability order: `name_exact` → `command_binary` → `command_contains`.
- **`src/process/framework.rs`** (`FrameworkDetector::classify`) runs the two tiers:
  framework rules first (so `fastapi` beats generic `python`), then runtime rules,
  then config-gated dev runners (`tsx`/`ts-node`). Returns `None` for non-servers —
  **a process matching no rule is not shown.**
- **To add a runtime or framework:** append one `Rule` to the appropriate table and
  add the variant to `Runtime` / `FrameworkKind` in `src/process/mod.rs`. No other
  code changes needed.

**Detection is process-local only** — name + command line, never filesystem reads.
This is deliberate: reading `package.json` would misclassify globally-launched
processes (npx MCP servers, CLI tools) by their inherited cwd. Don't add fs-based
detection without understanding this constraint.

`normalize_name` exists because macOS truncates the `comm` field to 16 chars; it
takes the first whitespace token to strip trailing version blobs.

## Scanning — gotchas that look like bugs

`src/process/scanner.rs` (`ProcessScanner`) and the `do_scan` flow in `main.rs`:

- **The `System` is long-lived on purpose.** sysinfo computes per-process CPU as the
  delta between two refreshes. Re-creating `System`/`ProcessScanner` each tick makes
  every process report `0.0%` CPU forever. `new()` primes with two refreshes +
  `MINIMUM_CPU_UPDATE_INTERVAL` sleep; `scan_blocking()` adds the third measurement
  refresh for one-shot CLI callers. Keep it persistent across the TUI loop.
- **Scan is two passes.** Pass 1 collects classified servers (`runtime = Some(_)`).
  Pass 2 backfills parent processes that aren't themselves servers (launchd, claude,
  shells) so the tree view has roots — these carry `runtime = None`. `is_server()` /
  `is_node()` / the Node-only filter all key off `runtime`. A `None` runtime means
  "tree-context parent, not a real server."
- **Ports are filled separately**, after classification, by `NetworkInspector::
  connections_by_pid()` (LISTEN state only). The scanner never fetches ports.

## TUI architecture

Event loop is `run_tui` in `src/main.rs`: `EventHandler` (async, tick + key +
resize) → on Tick rescan, on Key call `ui::handle_key`. Rendering and **all** key
handling live in `src/tui/ui.rs`.

- **`src/tui/app.rs` (`App`)** is the single state struct. Key fields: `raw_processes`
  (last scan, pre-filter) vs `process_trees`/`flat_list` (filtered, sorted, flattened
  for display). `rebuild_view()` re-applies the filter + node-only toggle + sort and
  re-flattens — call it on every filter keystroke so results update live without
  waiting for the next scan tick. Trees are built then sorted per-level; `flat_list`
  is `(ProcessInfo, depth)` honoring `expanded_pids`.
- **`ui::handle_key`** dispatches by mode: dialog open → `handle_dialog_key`, filter
  active → `handle_filter_key`, else `handle_normal_key` (then panel-specific
  `handle_list_key` / `handle_detail_key` based on `App::focus`).
- **Widgets** in `src/tui/widgets/` are pure render fns (`process_list`,
  `detail_panel` with Info/Log/Net/Env tabs, `kill_dialog`, `signal_picker`,
  `help_dialog`, `status_bar`, `empty_state`). Detail/help scroll bounds
  (`*_max_scroll`, `detail_content_lines/view_height`) are computed during render and
  read back next frame.
- Communication back to the loop is via `App` flags: `should_quit`, `needs_rescan`
  (e.g. after a kill), `refresh_changed` (rebuilds `EventHandler` with new interval).

## Other modules

- **`src/process/killer.rs`** — `KillSignal` (Unix has HUP/USR1/USR2; cross-platform
  has TERM/KILL/INT), `graceful_kill` (TERM, wait `graceful_timeout`, escalate to
  KILL), `kill_tree`, `is_alive`. Uses `nix` on Unix, `windows-sys` on Windows.
- **`src/process/platform.rs`** — FFI per OS. macOS `phys_footprint` (via
  `proc_pid_rusage`) is used over sysinfo RSS because it matches Activity Monitor
  (sysinfo RSS can underreport idle Node by 100×). Also `thread_count`, `open_fd_count`.
  Non-macOS/Windows targets get zero/`None` fallbacks.
- **`src/process/network.rs`** — `NetworkInspector` parses system command output
  (`lsof`/`netstat`) into `NetworkConnection`s mapped by PID.
- **`src/process/tree.rs`** — `TreeBuilder::build` partitions a flat list into
  roots/children by PID set, plus `flatten_with_depth`, `collect_pids`, `sort_recursive`.
- **`src/log/streamer.rs`** — `LogStreamer::detect_and_open(cwd)` globs `LOG_PATTERNS`,
  tails the most-recently-modified match, seeks to end, streams new lines.
- **`src/config.rs`** — TOML at `~/.config/ntop/config.toml`, all sections `#[serde(default)]`
  so partial/missing files fall back to defaults. `[filter].include_bun` is deprecated
  (Bun is now a first-class runtime); `include_tsx`/`include_ts_node` gate those dev runners.
- **`src/cli.rs`** — clap derive: `list`, `kill`, `info`, `log`, `config`; no subcommand
  launches the TUI.

## Conventions

- The library is exposed as crate `ntop` (`use ntop::...` in `main.rs` and tests).
- Adding a CLI subcommand: variant in `cli.rs::Commands` → match arm in `main.rs::main`
  → `cmd_*` handler. CLI handlers reuse `ProcessScanner` + `NetworkInspector` the same
  way `do_scan` does.
- Branches: work on `develop`, release to `main`. Version lives in `Cargo.toml`.
