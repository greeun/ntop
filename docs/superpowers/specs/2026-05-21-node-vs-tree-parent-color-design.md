# Design: Distinguish Real Node Processes from Tree-Context Parents

Date: 2026-05-21

## Problem

`ProcessScanner::scan()` returns two kinds of rows:

1. **Real Node processes** — picked up by the first pass via binary name
   match (`node`, `next-server`, …) or `command` parsing.
2. **Tree-context parents** — picked up by the second pass because they
   are the parent of a real Node process but are not themselves Node
   (e.g. `claude`, `launchd`). Shown so the tree view is rooted.

The UI shows both as plain rows. A user comparing ntop to Activity
Monitor cannot tell at a glance which rows are actual Node workloads
and which are just present for tree context.

## Goal

Make the distinction visible at a glance:

- Real Node rows → bright `LightCyan` foreground.
- Tree-context parents → dim `DarkGray` foreground.

Selection / health / focus styles continue to override as today.

## Data model change

Add one boolean to `ProcessInfo`:

```rust
pub struct ProcessInfo {
    // ...existing fields...
    pub is_node: bool,
}
```

Default is `false` in `ProcessInfo::new`. The scanner sets it.

## Scanner change

`src/process/scanner.rs` already has two distinct passes:

- First pass — `is_node = true` on every collected entry.
- Second pass (parent backfill) — `is_node = false`.

Pure data change; no new lookups or system calls.

## TUI rendering change

`src/tui/widgets/process_list.rs`: in the per-row closure, derive a base
foreground color from `proc_info.is_node` and apply it to the row's
`Style`:

```rust
let base_fg = if proc_info.is_node {
    Color::LightCyan
} else {
    Color::DarkGray
};

let row_style = if is_selected_row {
    Style::default().bg(Color::DarkGray).fg(base_fg).add_modifier(Modifier::BOLD)
} else {
    Style::default().fg(base_fg)
};
```

The health dot, checkbox, and tree prefix keep their own explicit
colors (they already use `Style::default().fg(...)` per span), so they
are not flattened to `base_fg`.

## CLI / JSON / info change

- `cmd_list` JSON output (`src/main.rs::print_json`) includes
  `"is_node": <bool>` in each object.
- `cmd_info` (`src/main.rs::cmd_info`) prints one extra line:
  `Type:      Node` for real Node, `Type:      Tree parent` otherwise.
- CSV and the table printer are unchanged. ANSI color is not injected
  into table output, since it would corrupt redirected/piped output.

## Tests

- `tests/scanner_test.rs::test_scanner_returns_vec` extended: assert
  there is at least one process with `is_node == true` when any Node
  process is running, and that every tree-context parent has
  `is_node == false`.
- Existing tests for `ProcessInfo::new` get one assertion: default
  `is_node == false`.

## Out of scope

- Theming / config toggles for the colors.
- Highlighting any other process category (e.g. tsx, bun).
- Changing the column layout or adding a new "Type" column.

## Verification

1. `cargo test --release` — all green.
2. Run `ntop` in the existing dev environment and confirm:
   - vite, tsx, npm, preflight, context7-mcp rows are LightCyan.
   - claude and launchd rows are DarkGray.
3. `ntop list --format=json` includes `is_node` per row.
4. `ntop info 78208` shows `Type: Tree parent`;
   `ntop info 78272` shows `Type: Node`.
