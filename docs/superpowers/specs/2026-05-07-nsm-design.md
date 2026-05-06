# NSM (Node Server Manager) Design Spec

## 1. One-line Pitch

Local Node.js/Next.js server process manager with real-time TUI dashboard, detailed inspection, log streaming, and full kill control.

## 2. Target User & Core Job-to-be-done

- **Target**: Developers and operators who run multiple Node.js/Next.js servers locally or on staging/production machines.
- **Core job**: Quickly see every running Node server, inspect its state in depth, stream its logs, and kill it cleanly — all from a single terminal tool.

## 3. Primary User Flows

### Flow 1: Launch TUI and browse servers
1. User runs `nsm` in terminal.
2. TUI opens with split-panel layout: left = real-time process list, right = detail panel.
3. Process list refreshes every 1s (configurable), showing PID, framework label, port, CPU, memory, uptime.
4. User navigates with `↑/↓` or `j/k` to highlight a process.
5. Right panel automatically updates with Info tab for the highlighted process.

### Flow 2: Inspect process details
1. User highlights a process in the list.
2. Right panel shows Info tab by default (PID, port, CPU, memory, uptime, cwd, full command, framework version).
3. User presses `Tab` to switch between Info, Log, Net, Env tabs.
4. Log tab streams real-time stdout/stderr from the process.
5. Net tab shows listening ports and active TCP connections.
6. Env tab shows environment variables (values masked by default).

### Flow 3: Kill a process
1. User highlights a process, presses `k`.
2. Confirmation dialog shows PID, name, port.
3. On confirm, SIGTERM is sent.
4. TUI shows "waiting for graceful shutdown..." with countdown.
5. If process exits within timeout → success indicator.
6. If timeout → prompt "Force kill with SIGKILL?"

### Flow 4: Kill a process tree
1. User highlights a parent process (e.g., `node next dev`), presses `K`.
2. Dialog lists all child processes that will be affected.
3. On confirm, SIGTERM sent to entire tree (children first, then parent).
4. Same graceful shutdown flow as single kill.

### Flow 5: Multi-select kill
1. User presses `Space` on multiple processes to select them.
2. Presses `k` to kill all selected.
3. Confirmation shows full list of selected processes.
4. Proceeds with graceful shutdown for each.

### Flow 6: Choose signal type
1. User presses `S` to open signal picker dialog.
2. Dialog lists: SIGTERM, SIGKILL, SIGHUP, SIGINT, SIGUSR1, SIGUSR2.
3. User selects signal, confirms target process(es).
4. Signal sent immediately (no graceful timeout for non-SIGTERM).

### Flow 7: Non-TUI CLI usage
1. `nsm list` prints a table of all Node processes.
2. `nsm list --json` outputs JSON for script consumption.
3. `nsm kill <PID>` kills a specific process from the command line.
4. `nsm info <PID>` prints detailed info for a process.
5. `nsm log <PID>` streams log output to the terminal.

### Flow 8: Process tree navigation
1. In the process list, parent processes show a `▸` collapse indicator.
2. User presses `Enter` to expand/collapse child processes.
3. Child processes are indented with tree connectors (`├─`, `└─`).

### Flow 9: Log tab fallback
1. User selects Log tab for a process.
2. Tool scans the process's cwd for common log files (*.log, .next/ logs, etc.).
3. If found → tail and stream in real-time.
4. If not found on Linux → attempt `/proc/<pid>/fd/1,2` direct access.
5. If no log source is available → display message "No log source detected. Use `nsm config` to set a custom log path for this process."

### Flow 10: No Node processes running
1. User launches `nsm` with no Node processes active.
2. TUI shows empty state: "No Node.js processes found. Waiting..." with a spinner.
3. List auto-populates when a Node process starts.

## 4. Feature List

| Feature | Description | User Value | AI-Assisted |
|---------|-------------|------------|-------------|
| Real-time process list | Live-updating list of all Node.js processes with CPU/memory/port | Instant visibility into what's running | N |
| Framework auto-detection | Identify Next.js, Express, Fastify, NestJS, Nuxt, generic Node | Know what each process is without reading command lines | N |
| Process tree view | Group parent-child processes with expandable tree UI | Understand process relationships (e.g., Next.js parent + workers) | N |
| Split-panel TUI | Left: process list, Right: tabbed detail panel | See list and details simultaneously | N |
| Info tab | PID, port, CPU, memory, uptime, cwd, full command, framework version | Deep inspection without leaving the tool | N |
| Log streaming | Real-time stdout/stderr via log file tail | Debug issues live without switching terminals | N |
| Network tab | Listening ports, active TCP connections, local/remote addresses | Understand network state per process | N |
| Environment tab | Environment variables with value masking | Inspect config without exposing secrets | N |
| Single process kill | Send configurable signal to one process | Quick cleanup | N |
| Tree kill | Kill parent + all children | Clean shutdown of complex process trees | N |
| Multi-select kill | Select multiple processes and kill in batch | Mass cleanup of stale processes | N |
| Signal picker | Choose from SIGTERM/SIGKILL/SIGHUP/SIGINT/SIGUSR1/SIGUSR2 | Fine-grained process control | N |
| Graceful shutdown | SIGTERM → wait → optional SIGKILL escalation | Safe shutdown without data loss | N |
| CLI subcommands | `list`, `kill`, `info`, `log` with JSON/CSV output | Script integration and automation | N |
| Status indicators | Color-coded health: green (normal), yellow (high load), red (unresponsive) | At-a-glance health assessment | N |
| Config file | TOML config for refresh rate, default signal, theme, filters | Persistent user preferences | N |
| Search/filter | `/` key to filter process list by name, port, framework | Find specific processes quickly | N |
| Column sorting | Sort by PID, name, CPU, memory, port | Organize the view as needed | N |

## 5. Data Model

### ProcessInfo
- `pid`: u32 — process ID
- `ppid`: u32 — parent process ID
- `name`: String — process name (from OS)
- `command`: String — full command line
- `cwd`: String — working directory
- `framework`: FrameworkKind — detected framework enum
- `framework_version`: Option<String> — from package.json
- `port`: Vec<u16> — listening ports
- `cpu_percent`: f32 — CPU usage
- `memory_rss`: u64 — resident set size in bytes
- `memory_vms`: u64 — virtual memory size in bytes
- `threads`: u32 — thread count
- `uptime`: Duration — time since process start
- `user`: String — process owner
- `status`: ProcessStatus — Running, Sleeping, Zombie, etc.
- `open_fds`: u32 — open file descriptor count
- `children`: Vec<ProcessInfo> — child processes (tree)

### FrameworkKind (enum)
- NextJs
- Express
- Fastify
- NestJs
- Nuxt
- Koa
- Hapi
- Generic (plain Node.js)

### NetworkConnection
- `local_addr`: SocketAddr
- `remote_addr`: Option<SocketAddr>
- `state`: TcpState (Listen, Established, TimeWait, etc.)
- `pid`: u32

### HealthStatus (enum)
- Healthy (green) — CPU < 80%, responsive
- Warning (yellow) — CPU > 80% or memory > 80% of system
- Critical (red) — unresponsive or zombie state

## 6. Screens / Surfaces

### Main TUI Screen
- **Top bar**: App name + version, system CPU/memory summary, total Node process count, help hint
- **Left panel (60% default width)**: Process list with tree view, real-time refresh, multi-select checkboxes, status indicators, column headers with sort
- **Right panel (40% default width)**: Tabbed detail view (Info / Log / Net / Env)
- Panel width ratio adjusts automatically based on terminal width; minimum 40 columns per panel
- **Bottom bar**: Key binding hints for current context

### Kill Confirmation Dialog (modal overlay)
- Target process info (PID, name, port)
- For tree kill: list of all affected children
- Signal type being sent
- Confirm / Cancel buttons

### Signal Picker Dialog (modal overlay)
- List of available signals with descriptions
- Highlight to select, Enter to confirm

### CLI Output Surfaces
- `nsm list`: formatted table (with optional `--json`, `--format csv`)
- `nsm info <PID>`: detailed key-value output
- `nsm log <PID>`: streaming text output

## 7. Non-goals

- Remote server monitoring (SSH-based or agent-based) — local only for v1
- Docker/container process management — out of scope
- Process restart/auto-restart capability — this is a monitor + killer, not a process manager like pm2
- Windows support in v1 — macOS and Linux only
- Web-based dashboard — terminal only
- Historical metrics or persistent data storage — real-time only
- Non-Node.js process management (Python, Ruby, etc.)

## 8. Definition of Done

- [ ] `nsm` launches a TUI with split-panel layout (process list + detail panel)
- [ ] Process list refreshes in real-time at configurable interval
- [ ] All running Node.js processes are detected (node, next-server, next-router-worker)
- [ ] Framework is auto-detected and labeled (Next.js, Express, Fastify, NestJS, Nuxt, generic)
- [ ] Process tree relationships are correctly displayed and expandable
- [ ] Info tab shows: PID, port, CPU%, memory, uptime, cwd, full command, framework version
- [ ] Log tab streams real-time output from detected log files
- [ ] Net tab shows listening ports and active TCP connections
- [ ] Env tab shows environment variables with value masking
- [ ] Single kill sends configurable signal with graceful shutdown flow
- [ ] Tree kill terminates parent + all children
- [ ] Multi-select allows batch killing
- [ ] Signal picker dialog offers SIGTERM/SIGKILL/SIGHUP/SIGINT/SIGUSR1/SIGUSR2
- [ ] Graceful shutdown: SIGTERM → timeout → optional SIGKILL prompt
- [ ] Kill confirmation dialog shows affected process details
- [ ] `nsm list` outputs process table (text, JSON, CSV formats)
- [ ] `nsm kill <PID>` works from CLI
- [ ] `nsm info <PID>` prints detailed info
- [ ] `nsm log <PID>` streams logs
- [ ] Config file at `~/.config/nsm/config.toml` is read and respected
- [ ] Status indicators show color-coded health
- [ ] Search/filter works with `/` key
- [ ] Column sorting works with `s` key
- [ ] Keyboard navigation: ↑↓/jk, Enter, Tab, Space, q all function correctly
- [ ] Empty state shown when no Node processes are running
- [ ] Log tab shows fallback message when no log source is detected
- [ ] Builds as single binary for macOS (arm64, x86_64) and Linux (x86_64)
- [ ] `cargo install nsm` works
