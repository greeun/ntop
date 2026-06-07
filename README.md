# ntop

A fast, real-time TUI tool for monitoring and managing server processes across multiple runtimes — Node.js, Python, Java, Deno, Bun, Ruby, PHP, and .NET.

Built with Rust for instant startup and minimal resource usage.

![License](https://img.shields.io/badge/license-MIT-blue.svg)

## Features

- **Real-time process monitoring** with configurable refresh rate (default 3s)
- **Rule-based runtime & framework detection** — Node/Next/Nuxt/Nest, Python/FastAPI/Flask/Django, Java/Spring Boot, Ruby/Rails, PHP/Laravel, Deno, Bun, .NET (no filesystem reads, so globally-launched CLIs aren't misclassified)
- **Node-only toggle** — press `n` to show only Node servers
- **Process tree view** — parent-child relationships with expand/collapse, with distinct coloring for node leaves vs. tree-parent rows
- **Split-panel TUI** — process list + tabbed detail panel (Info / Log / Net / Env)
- **Incremental search** — `/` filter is applied on every keystroke
- **Scrollable help dialog** — long key reference scrolls within the popup
- **Log streaming** — real-time tail from detected log files
- **Network inspection** — listening ports and active TCP connections per process
- **Environment variables** — with automatic sensitive value masking
- **Accurate metrics** — real CPU readings and macOS `phys_footprint` memory that matches Activity Monitor
- **Full kill control** — SIGTERM, SIGKILL, SIGHUP, SIGINT, SIGUSR1, SIGUSR2
- **Graceful shutdown** — SIGTERM with configurable timeout, optional SIGKILL escalation
- **Tree kill** — terminate parent + all child processes
- **Multi-select** — batch kill multiple processes at once
- **CLI subcommands** — `list`, `kill`, `info`, `log` with JSON/CSV output
- **Configurable** — TOML config at `~/.config/ntop/config.toml`

## Installation

### From source (cargo)

```bash
cargo install --git https://github.com/greeun/ntop
```

### From release binary

Download the latest binary from [Releases](https://github.com/greeun/ntop/releases).

## Usage

### TUI Mode (default)

```bash
ntop
```

Launches the interactive dashboard:

```
┌─────────────────────────────────────────────────────────────────┐
│ ntop v0.1.3  |  CPU: 12.3%  MEM: 4.2GB  |  Nodes: 7  | [H]elp│
├──────────────────────────┬──────────────────────────────────────┤
│  PROCESS LIST            │  [Info] [Log] [Net] [Env]           │
│                          │                                      │
│  ▸ ● Next.js dev :3000  │  PID:       12345                    │
│    ├ next-server         │  Framework: Next.js                  │
│    └ next-router-worker  │  Port:      3000                     │
│  ▸ ● Express    :4000   │  CPU:       3.2%                     │
│    ● Node.js    :8080   │  Memory:    128.0 MB                 │
│                          │  Uptime:    2h 13m 5s                │
├──────────────────────────┴──────────────────────────────────────┤
│ [q] Quit | [Up/Down] Navigate | [Tab] Tab | [x] Kill | ...    │
└─────────────────────────────────────────────────────────────────┘
```

### Key Bindings

| Key | Action |
|-----|--------|
| `↑/↓` or `j/k` | Navigate process list |
| `Enter` | Expand/collapse process tree |
| `Tab` | Switch detail tab (Info → Log → Net → Env) |
| `Space` | Toggle multi-select |
| `/` | Search/filter processes |
| `n` | Toggle Node-only view |
| `s` | Cycle sort column |
| `x` | Kill selected process(es) |
| `K` | Kill process tree |
| `S` | Open signal picker |
| `q` / `Esc` | Quit |

### CLI Mode

```bash
# List all Node.js processes
ntop list
ntop list --json
ntop list --format csv

# Kill a process
ntop kill <PID>
ntop kill --tree <PID>
ntop kill --signal SIGKILL <PID>
ntop kill --all
ntop kill --no-confirm <PID>

# Show detailed info
ntop info <PID>

# Stream logs
ntop log <PID>

# Show config path
ntop config
```

### Example: `ntop list`

```
PID      NAME                 FRAMEWORK    PORT       CPU      MEM        UPTIME
--------------------------------------------------------------------------------
12345    node                 Next.js      3000       3.2%     128.0 MB   2h 13m
12390    node                 Express      4000       1.1%     64.0 MB    5h 2m
12401    node                 Node.js      8080       0.3%     32.0 MB    12m
```

## Configuration

Config file: `~/.config/ntop/config.toml`

```toml
[general]
refresh_interval = 3          # seconds
default_signal = "SIGTERM"
graceful_timeout = 10         # seconds
confirm_before_kill = true

[display]
show_tree = true
color_theme = "auto"          # auto | dark | light
mask_env_values = true        # mask PASSWORD, SECRET, TOKEN, etc.

[filter]
include_bun = false
include_tsx = false
include_ts_node = false
```

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

## Requirements

- macOS or Linux
- Rust 1.70+ (for building from source)

## Testing

```bash
cargo test
```

The suite covers framework detection, process tree building, signal
handling, network address parsing, log streaming, and CLI dispatch.

## License

MIT
