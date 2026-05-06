# NSM (Node Server Manager) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust CLI/TUI tool (`nsm`) that lists, inspects, and kills running Node.js/Next.js server processes with real-time monitoring, log streaming, and full signal control.

**Architecture:** Monolithic single binary. Core logic modules (scanner, killer, logger, network) are independent of the TUI layer and also power CLI subcommands. TUI uses ratatui + crossterm with an async event loop (tokio) for real-time refresh and log streaming.

**Tech Stack:** Rust, ratatui, crossterm, tokio, sysinfo, nix, clap, serde, toml, notify, dirs

---

## File Structure

```
nsm/
├── Cargo.toml
├── src/
│   ├── main.rs                  # Entry point: CLI dispatch or TUI launch
│   ├── lib.rs                   # Re-exports all public modules
│   ├── cli.rs                   # clap CLI definition (subcommands + args)
│   ├── config.rs                # Config file parsing (~/.config/nsm/config.toml)
│   ├── process/
│   │   ├── mod.rs               # ProcessInfo, FrameworkKind, HealthStatus types
│   │   ├── scanner.rs           # Scan system for Node.js processes via sysinfo
│   │   ├── framework.rs         # Detect framework from cmd/cwd/package.json
│   │   ├── tree.rs              # Build parent-child process tree from flat list
│   │   ├── killer.rs            # Send signals, graceful shutdown flow
│   │   └── network.rs           # Inspect listening ports + TCP connections per PID
│   ├── log/
│   │   ├── mod.rs               # Re-export
│   │   └── streamer.rs          # Detect log files in cwd, tail + stream lines
│   └── tui/
│       ├── mod.rs               # Re-export
│       ├── app.rs               # App state struct (selection, tabs, dialogs, filter)
│       ├── event.rs             # Async event loop: key events + tick timer
│       ├── ui.rs                # Top-level layout rendering (split panels, bars)
│       └── widgets/
│           ├── mod.rs           # Re-export all widgets
│           ├── process_list.rs  # Left panel: process list with tree + multi-select
│           ├── detail_panel.rs  # Right panel: tab container
│           ├── info_tab.rs      # Info tab content rendering
│           ├── log_tab.rs       # Log tab with scrollback buffer
│           ├── net_tab.rs       # Network connections table
│           ├── env_tab.rs       # Environment variables with masking
│           ├── kill_dialog.rs   # Kill confirmation modal overlay
│           ├── signal_picker.rs # Signal selection modal overlay
│           ├── status_bar.rs    # Top bar (system info) + bottom bar (keybindings)
│           └── empty_state.rs   # "No processes found" with spinner
├── tests/
│   ├── types_test.rs            # ProcessInfo, FrameworkKind serialization
│   ├── framework_test.rs        # Framework detection logic
│   ├── tree_test.rs             # Process tree building
│   ├── config_test.rs           # Config parsing + defaults
│   └── cli_test.rs              # CLI arg parsing
```

---

## Task 1: Project Scaffolding + Core Types

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`
- Create: `src/process/mod.rs`
- Test: `tests/types_test.rs`

- [ ] **Step 1: Initialize Cargo project**

```bash
cd /Users/uni4love/project/workspace/211-withwiz/node-server-manager
cargo init --name nsm
```

- [ ] **Step 2: Set up Cargo.toml with all dependencies**

Replace `Cargo.toml` with:

```toml
[package]
name = "nsm"
version = "0.1.0"
edition = "2021"
description = "Node Server Manager - TUI tool for monitoring and managing Node.js/Next.js processes"
license = "MIT"
repository = "https://github.com/withwiz/nsm"

[[bin]]
name = "nsm"
path = "src/main.rs"

[dependencies]
clap = { version = "4", features = ["derive"] }
crossterm = "0.28"
dirs = "6"
nix = { version = "0.29", features = ["signal", "process"] }
notify = "7"
ratatui = "0.29"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sysinfo = "0.33"
tokio = { version = "1", features = ["full"] }
toml = "0.8"
csv = "1"
chrono = "0.4"
unicode-width = "0.2"

[dev-dependencies]
tempfile = "3"
assert_cmd = "2"
predicates = "3"
```

- [ ] **Step 3: Write the failing test for core types**

Create `tests/types_test.rs`:

```rust
use nsm::process::{FrameworkKind, HealthStatus, ProcessInfo};
use std::time::Duration;

#[test]
fn test_framework_kind_display() {
    assert_eq!(FrameworkKind::NextJs.to_string(), "Next.js");
    assert_eq!(FrameworkKind::Express.to_string(), "Express");
    assert_eq!(FrameworkKind::Fastify.to_string(), "Fastify");
    assert_eq!(FrameworkKind::NestJs.to_string(), "NestJS");
    assert_eq!(FrameworkKind::Nuxt.to_string(), "Nuxt");
    assert_eq!(FrameworkKind::Koa.to_string(), "Koa");
    assert_eq!(FrameworkKind::Hapi.to_string(), "Hapi");
    assert_eq!(FrameworkKind::Generic.to_string(), "Node.js");
}

#[test]
fn test_health_status_from_metrics() {
    assert_eq!(HealthStatus::from_cpu_mem(50.0, 50.0), HealthStatus::Healthy);
    assert_eq!(HealthStatus::from_cpu_mem(85.0, 50.0), HealthStatus::Warning);
    assert_eq!(HealthStatus::from_cpu_mem(50.0, 85.0), HealthStatus::Warning);
}

#[test]
fn test_health_status_zombie() {
    assert_eq!(HealthStatus::from_process_status("Zombie"), HealthStatus::Critical);
}

#[test]
fn test_process_info_default() {
    let info = ProcessInfo::new(1234, "node".to_string());
    assert_eq!(info.pid, 1234);
    assert_eq!(info.name, "node");
    assert_eq!(info.framework, FrameworkKind::Generic);
    assert!(info.children.is_empty());
    assert!(info.ports.is_empty());
}

#[test]
fn test_process_info_uptime_display() {
    let mut info = ProcessInfo::new(1, "node".to_string());
    info.uptime = Duration::from_secs(3600 + 120 + 5);
    assert_eq!(info.uptime_display(), "1h 2m 5s");
}

#[test]
fn test_process_info_memory_display() {
    let mut info = ProcessInfo::new(1, "node".to_string());
    info.memory_rss = 134_217_728; // 128 MB
    assert_eq!(info.memory_display(), "128.0 MB");
}

#[test]
fn test_framework_kind_serialization() {
    let kind = FrameworkKind::NextJs;
    let json = serde_json::to_string(&kind).unwrap();
    assert_eq!(json, "\"NextJs\"");
    let deserialized: FrameworkKind = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, kind);
}
```

- [ ] **Step 4: Run test to verify it fails**

```bash
cargo test --test types_test 2>&1
```

Expected: compilation error — `nsm::process` module doesn't exist.

- [ ] **Step 5: Implement core types**

Create `src/process/mod.rs`:

```rust
pub mod scanner;
pub mod framework;
pub mod tree;
pub mod killer;
pub mod network;

use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameworkKind {
    NextJs,
    Express,
    Fastify,
    NestJs,
    Nuxt,
    Koa,
    Hapi,
    Generic,
}

impl fmt::Display for FrameworkKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NextJs => write!(f, "Next.js"),
            Self::Express => write!(f, "Express"),
            Self::Fastify => write!(f, "Fastify"),
            Self::NestJs => write!(f, "NestJS"),
            Self::Nuxt => write!(f, "Nuxt"),
            Self::Koa => write!(f, "Koa"),
            Self::Hapi => write!(f, "Hapi"),
            Self::Generic => write!(f, "Node.js"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
}

impl HealthStatus {
    pub fn from_cpu_mem(cpu_percent: f32, mem_percent: f32) -> Self {
        if cpu_percent > 80.0 || mem_percent > 80.0 {
            Self::Warning
        } else {
            Self::Healthy
        }
    }

    pub fn from_process_status(status: &str) -> Self {
        match status {
            "Zombie" | "Dead" => Self::Critical,
            _ => Self::Healthy,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub name: String,
    pub command: String,
    pub cwd: String,
    pub framework: FrameworkKind,
    pub framework_version: Option<String>,
    pub ports: Vec<u16>,
    pub cpu_percent: f32,
    pub memory_rss: u64,
    pub memory_vms: u64,
    pub threads: u32,
    pub uptime: Duration,
    pub user: String,
    pub status: String,
    pub open_fds: u32,
    pub children: Vec<ProcessInfo>,
    pub env_vars: Vec<(String, String)>,
}

impl ProcessInfo {
    pub fn new(pid: u32, name: String) -> Self {
        Self {
            pid,
            ppid: 0,
            name,
            command: String::new(),
            cwd: String::new(),
            framework: FrameworkKind::Generic,
            framework_version: None,
            ports: Vec::new(),
            cpu_percent: 0.0,
            memory_rss: 0,
            memory_vms: 0,
            threads: 0,
            uptime: Duration::ZERO,
            user: String::new(),
            status: "Running".to_string(),
            open_fds: 0,
            children: Vec::new(),
            env_vars: Vec::new(),
        }
    }

    pub fn uptime_display(&self) -> String {
        let total_secs = self.uptime.as_secs();
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;
        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }

    pub fn memory_display(&self) -> String {
        let bytes = self.memory_rss as f64;
        if bytes >= 1_073_741_824.0 {
            format!("{:.1} GB", bytes / 1_073_741_824.0)
        } else if bytes >= 1_048_576.0 {
            format!("{:.1} MB", bytes / 1_048_576.0)
        } else if bytes >= 1024.0 {
            format!("{:.1} KB", bytes / 1024.0)
        } else {
            format!("{} B", self.memory_rss)
        }
    }

    pub fn health(&self) -> HealthStatus {
        let status_health = HealthStatus::from_process_status(&self.status);
        if status_health == HealthStatus::Critical {
            return status_health;
        }
        HealthStatus::from_cpu_mem(self.cpu_percent, 0.0)
    }
}
```

Create stub files so the module tree compiles:

`src/process/scanner.rs`:
```rust
// will be implemented in Task 3
```

`src/process/framework.rs`:
```rust
// will be implemented in Task 4
```

`src/process/tree.rs`:
```rust
// will be implemented in Task 5
```

`src/process/killer.rs`:
```rust
// will be implemented in Task 7
```

`src/process/network.rs`:
```rust
// will be implemented in Task 6
```

Create `src/lib.rs`:
```rust
pub mod process;
pub mod config;
pub mod log;
pub mod cli;
pub mod tui;
```

Create stub `src/config.rs`:
```rust
// will be implemented in Task 2
```

Create stub `src/log/mod.rs`:
```rust
pub mod streamer;
```

Create stub `src/log/streamer.rs`:
```rust
// will be implemented in Task 8
```

Create stub `src/cli.rs`:
```rust
// will be implemented in Task 9
```

Create stub `src/tui/mod.rs`:
```rust
pub mod app;
pub mod event;
pub mod ui;
pub mod widgets;
```

Create stub `src/tui/app.rs`:
```rust
// will be implemented in Task 10
```

Create stub `src/tui/event.rs`:
```rust
// will be implemented in Task 10
```

Create stub `src/tui/ui.rs`:
```rust
// will be implemented in Task 11
```

Create stub `src/tui/widgets/mod.rs`:
```rust
pub mod process_list;
pub mod detail_panel;
pub mod info_tab;
pub mod log_tab;
pub mod net_tab;
pub mod env_tab;
pub mod kill_dialog;
pub mod signal_picker;
pub mod status_bar;
pub mod empty_state;
```

Create empty stub files for each widget:
- `src/tui/widgets/process_list.rs`
- `src/tui/widgets/detail_panel.rs`
- `src/tui/widgets/info_tab.rs`
- `src/tui/widgets/log_tab.rs`
- `src/tui/widgets/net_tab.rs`
- `src/tui/widgets/env_tab.rs`
- `src/tui/widgets/kill_dialog.rs`
- `src/tui/widgets/signal_picker.rs`
- `src/tui/widgets/status_bar.rs`
- `src/tui/widgets/empty_state.rs`

Create minimal `src/main.rs`:
```rust
fn main() {
    println!("nsm - Node Server Manager");
}
```

- [ ] **Step 6: Run test to verify it passes**

```bash
cargo test --test types_test 2>&1
```

Expected: all 7 tests PASS.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat: scaffold project with core types (ProcessInfo, FrameworkKind, HealthStatus)"
```

---

## Task 2: Config System

**Files:**
- Create: `src/config.rs`
- Test: `tests/config_test.rs`

- [ ] **Step 1: Write the failing test**

Create `tests/config_test.rs`:

```rust
use nsm::config::Config;
use std::time::Duration;

#[test]
fn test_default_config() {
    let config = Config::default();
    assert_eq!(config.general.refresh_interval, 1);
    assert_eq!(config.general.default_signal, "SIGTERM");
    assert_eq!(config.general.graceful_timeout, 10);
    assert!(config.general.confirm_before_kill);
    assert!(config.display.show_tree);
    assert_eq!(config.display.color_theme, "auto");
    assert!(config.display.mask_env_values);
    assert!(!config.filter.include_bun);
    assert!(!config.filter.include_tsx);
    assert!(!config.filter.include_ts_node);
}

#[test]
fn test_parse_toml_config() {
    let toml_str = r#"
[general]
refresh_interval = 2
default_signal = "SIGKILL"
graceful_timeout = 5
confirm_before_kill = false

[display]
show_tree = false
color_theme = "dark"
mask_env_values = false

[filter]
include_bun = true
include_tsx = true
include_ts_node = false
"#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(config.general.refresh_interval, 2);
    assert_eq!(config.general.default_signal, "SIGKILL");
    assert_eq!(config.general.graceful_timeout, 5);
    assert!(!config.general.confirm_before_kill);
    assert!(!config.display.show_tree);
    assert_eq!(config.display.color_theme, "dark");
    assert!(!config.display.mask_env_values);
    assert!(config.filter.include_bun);
    assert!(config.filter.include_tsx);
}

#[test]
fn test_partial_toml_uses_defaults() {
    let toml_str = r#"
[general]
refresh_interval = 3
"#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(config.general.refresh_interval, 3);
    assert_eq!(config.general.default_signal, "SIGTERM");
    assert!(config.display.show_tree);
}

#[test]
fn test_config_refresh_duration() {
    let config = Config::default();
    assert_eq!(config.refresh_duration(), Duration::from_secs(1));
}

#[test]
fn test_config_graceful_duration() {
    let mut config = Config::default();
    config.general.graceful_timeout = 15;
    assert_eq!(config.graceful_duration(), Duration::from_secs(15));
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test --test config_test 2>&1
```

Expected: FAIL — `Config` type doesn't exist yet.

- [ ] **Step 3: Implement config module**

Replace `src/config.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub general: GeneralConfig,
    pub display: DisplayConfig,
    pub filter: FilterConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    pub refresh_interval: u64,
    pub default_signal: String,
    pub graceful_timeout: u64,
    pub confirm_before_kill: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DisplayConfig {
    pub show_tree: bool,
    pub color_theme: String,
    pub mask_env_values: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FilterConfig {
    pub include_bun: bool,
    pub include_tsx: bool,
    pub include_ts_node: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            display: DisplayConfig::default(),
            filter: FilterConfig::default(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            refresh_interval: 1,
            default_signal: "SIGTERM".to_string(),
            graceful_timeout: 10,
            confirm_before_kill: true,
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_tree: true,
            color_theme: "auto".to_string(),
            mask_env_values: true,
        }
    }
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            include_bun: false,
            include_tsx: false,
            include_ts_node: false,
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("nsm")
            .join("config.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        match fs::read_to_string(&path) {
            Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn refresh_duration(&self) -> Duration {
        Duration::from_secs(self.general.refresh_interval)
    }

    pub fn graceful_duration(&self) -> Duration {
        Duration::from_secs(self.general.graceful_timeout)
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

```bash
cargo test --test config_test 2>&1
```

Expected: all 5 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/config.rs tests/config_test.rs
git commit -m "feat: add config system with TOML parsing and defaults"
```

---

## Task 3: Process Scanner

**Files:**
- Create: `src/process/scanner.rs`
- Test: `tests/scanner_test.rs`

- [ ] **Step 1: Write the failing test**

Create `tests/scanner_test.rs`:

```rust
use nsm::config::Config;
use nsm::process::scanner::ProcessScanner;

#[test]
fn test_scanner_returns_vec() {
    let config = Config::default();
    let scanner = ProcessScanner::new(&config);
    let processes = scanner.scan();
    // We can't assert specific processes exist, but the vec should be valid
    assert!(processes.len() >= 0);
    for p in &processes {
        assert!(p.pid > 0);
        assert!(!p.name.is_empty());
    }
}

#[test]
fn test_is_node_process_by_name() {
    assert!(ProcessScanner::is_node_process_name("node"));
    assert!(ProcessScanner::is_node_process_name("next-server"));
    assert!(ProcessScanner::is_node_process_name("next-router-worker"));
    assert!(!ProcessScanner::is_node_process_name("python"));
    assert!(!ProcessScanner::is_node_process_name("ruby"));
}

#[test]
fn test_is_node_process_by_command() {
    assert!(ProcessScanner::is_node_command("/usr/local/bin/node server.js"));
    assert!(ProcessScanner::is_node_command("/opt/homebrew/bin/node app.js"));
    assert!(ProcessScanner::is_node_command("node index.js"));
    assert!(!ProcessScanner::is_node_command("python app.py"));
}

#[test]
fn test_optional_runtime_filters() {
    let mut config = Config::default();
    config.filter.include_bun = true;
    assert!(ProcessScanner::is_node_process_name_with_config("bun", &config));

    config.filter.include_bun = false;
    assert!(!ProcessScanner::is_node_process_name_with_config("bun", &config));

    config.filter.include_tsx = true;
    assert!(ProcessScanner::is_node_process_name_with_config("tsx", &config));
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test --test scanner_test 2>&1
```

Expected: FAIL — `ProcessScanner` doesn't exist.

- [ ] **Step 3: Implement process scanner**

Replace `src/process/scanner.rs`:

```rust
use crate::config::Config;
use crate::process::{FrameworkKind, ProcessInfo};
use std::time::Duration;
use sysinfo::{ProcessRefreshKind, RefreshKind, System, UpdateKind};

pub struct ProcessScanner<'a> {
    config: &'a Config,
}

const NODE_PROCESS_NAMES: &[&str] = &[
    "node",
    "next-server",
    "next-router-worker",
    "next-router-page-worker",
];

const OPTIONAL_RUNTIMES: &[(&str, fn(&Config) -> bool)] = &[
    ("bun", |c: &Config| c.filter.include_bun),
    ("tsx", |c: &Config| c.filter.include_tsx),
    ("ts-node", |c: &Config| c.filter.include_ts_node),
];

impl<'a> ProcessScanner<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub fn scan(&self) -> Vec<ProcessInfo> {
        let mut sys = System::new();
        sys.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::new()
                .with_cpu()
                .with_memory()
                .with_cmd(UpdateKind::Always),
        );

        let mut results = Vec::new();

        for (pid, process) in sys.processes() {
            let name = process.name().to_string_lossy().to_string();
            let cmd_parts: Vec<String> = process.cmd().iter().map(|s| s.to_string_lossy().to_string()).collect();
            let command = cmd_parts.join(" ");

            let is_node = Self::is_node_process_name_with_config(&name, self.config)
                || Self::is_node_command(&command);

            if !is_node {
                continue;
            }

            let cwd = process
                .root()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

            let mut info = ProcessInfo::new(pid.as_u32(), name);
            info.ppid = process
                .parent()
                .map(|p| p.as_u32())
                .unwrap_or(0);
            info.command = command;
            info.cwd = cwd;
            info.cpu_percent = process.cpu_usage();
            info.memory_rss = process.memory();
            info.memory_vms = process.virtual_memory();
            info.status = format!("{:?}", process.status());
            info.uptime = Duration::from_secs(process.run_time());
            info.threads = process.tasks().map(|t| t.len() as u32).unwrap_or(0);

            let environ: Vec<(String, String)> = process
                .environ()
                .iter()
                .map(|s| {
                    let s = s.to_string_lossy().to_string();
                    if let Some(pos) = s.find('=') {
                        (s[..pos].to_string(), s[pos + 1..].to_string())
                    } else {
                        (s, String::new())
                    }
                })
                .collect();
            info.env_vars = environ;

            let user = process
                .user_id()
                .map(|u| u.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            info.user = user;

            results.push(info);
        }

        results
    }

    pub fn is_node_process_name(name: &str) -> bool {
        NODE_PROCESS_NAMES.iter().any(|&n| name == n)
    }

    pub fn is_node_process_name_with_config(name: &str, config: &Config) -> bool {
        if Self::is_node_process_name(name) {
            return true;
        }
        for &(runtime_name, filter_fn) in OPTIONAL_RUNTIMES {
            if name == runtime_name && filter_fn(config) {
                return true;
            }
        }
        false
    }

    pub fn is_node_command(command: &str) -> bool {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if let Some(first) = parts.first() {
            let binary = first.rsplit('/').next().unwrap_or(first);
            binary == "node"
        } else {
            false
        }
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

```bash
cargo test --test scanner_test 2>&1
```

Expected: all 4 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/process/scanner.rs tests/scanner_test.rs
git commit -m "feat: add process scanner using sysinfo"
```

---

## Task 4: Framework Detection

**Files:**
- Create: `src/process/framework.rs`
- Test: `tests/framework_test.rs`

- [ ] **Step 1: Write the failing test**

Create `tests/framework_test.rs`:

```rust
use nsm::process::framework::FrameworkDetector;
use nsm::process::FrameworkKind;

#[test]
fn test_detect_nextjs_by_process_name() {
    assert_eq!(
        FrameworkDetector::detect_by_name("next-server"),
        Some(FrameworkKind::NextJs)
    );
    assert_eq!(
        FrameworkDetector::detect_by_name("next-router-worker"),
        Some(FrameworkKind::NextJs)
    );
}

#[test]
fn test_detect_framework_by_command() {
    assert_eq!(
        FrameworkDetector::detect_by_command("node /app/node_modules/.bin/next dev"),
        Some(FrameworkKind::NextJs)
    );
    assert_eq!(
        FrameworkDetector::detect_by_command("node node_modules/.bin/nuxt dev"),
        Some(FrameworkKind::Nuxt)
    );
    assert_eq!(
        FrameworkDetector::detect_by_command("node server.js"),
        None
    );
}

#[test]
fn test_detect_framework_by_package_json() {
    use std::fs;
    let dir = tempfile::tempdir().unwrap();
    let pkg = dir.path().join("package.json");

    fs::write(&pkg, r#"{"dependencies":{"next":"14.0.0"}}"#).unwrap();
    let (kind, version) = FrameworkDetector::detect_by_package_json(dir.path().to_str().unwrap());
    assert_eq!(kind, Some(FrameworkKind::NextJs));
    assert_eq!(version, Some("14.0.0".to_string()));

    fs::write(&pkg, r#"{"dependencies":{"express":"4.18.0"}}"#).unwrap();
    let (kind, _) = FrameworkDetector::detect_by_package_json(dir.path().to_str().unwrap());
    assert_eq!(kind, Some(FrameworkKind::Express));

    fs::write(&pkg, r#"{"dependencies":{"fastify":"4.0.0"}}"#).unwrap();
    let (kind, _) = FrameworkDetector::detect_by_package_json(dir.path().to_str().unwrap());
    assert_eq!(kind, Some(FrameworkKind::Fastify));

    fs::write(&pkg, r#"{"dependencies":{"@nestjs/core":"10.0.0"}}"#).unwrap();
    let (kind, _) = FrameworkDetector::detect_by_package_json(dir.path().to_str().unwrap());
    assert_eq!(kind, Some(FrameworkKind::NestJs));

    fs::write(&pkg, r#"{"dependencies":{"koa":"2.0.0"}}"#).unwrap();
    let (kind, _) = FrameworkDetector::detect_by_package_json(dir.path().to_str().unwrap());
    assert_eq!(kind, Some(FrameworkKind::Koa));

    fs::write(&pkg, r#"{"dependencies":{"@hapi/hapi":"21.0.0"}}"#).unwrap();
    let (kind, _) = FrameworkDetector::detect_by_package_json(dir.path().to_str().unwrap());
    assert_eq!(kind, Some(FrameworkKind::Hapi));
}

#[test]
fn test_detect_combined_priority() {
    // process name takes priority over command
    let result = FrameworkDetector::detect("next-server", "node server.js", "");
    assert_eq!(result.0, FrameworkKind::NextJs);
}

#[test]
fn test_detect_fallback_to_generic() {
    let result = FrameworkDetector::detect("node", "node app.js", "/nonexistent/path");
    assert_eq!(result.0, FrameworkKind::Generic);
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test --test framework_test 2>&1
```

Expected: FAIL — `FrameworkDetector` doesn't exist.

- [ ] **Step 3: Implement framework detection**

Replace `src/process/framework.rs`:

```rust
use crate::process::FrameworkKind;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct FrameworkDetector;

const NAME_MAP: &[(&str, FrameworkKind)] = &[
    ("next-server", FrameworkKind::NextJs),
    ("next-router-worker", FrameworkKind::NextJs),
    ("next-router-page-worker", FrameworkKind::NextJs),
];

const COMMAND_KEYWORDS: &[(&str, FrameworkKind)] = &[
    ("next", FrameworkKind::NextJs),
    ("nuxt", FrameworkKind::Nuxt),
    ("nest", FrameworkKind::NestJs),
];

const PACKAGE_DEPS: &[(&str, FrameworkKind)] = &[
    ("next", FrameworkKind::NextJs),
    ("nuxt", FrameworkKind::Nuxt),
    ("@nestjs/core", FrameworkKind::NestJs),
    ("express", FrameworkKind::Express),
    ("fastify", FrameworkKind::Fastify),
    ("koa", FrameworkKind::Koa),
    ("@hapi/hapi", FrameworkKind::Hapi),
];

impl FrameworkDetector {
    pub fn detect(name: &str, command: &str, cwd: &str) -> (FrameworkKind, Option<String>) {
        if let Some(kind) = Self::detect_by_name(name) {
            let (_, version) = Self::detect_by_package_json(cwd);
            return (kind, version);
        }

        if let Some(kind) = Self::detect_by_command(command) {
            let (_, version) = Self::detect_by_package_json(cwd);
            return (kind, version);
        }

        let (kind, version) = Self::detect_by_package_json(cwd);
        if let Some(kind) = kind {
            return (kind, version);
        }

        (FrameworkKind::Generic, None)
    }

    pub fn detect_by_name(name: &str) -> Option<FrameworkKind> {
        NAME_MAP
            .iter()
            .find(|&&(n, _)| n == name)
            .map(|&(_, ref k)| k.clone())
    }

    pub fn detect_by_command(command: &str) -> Option<FrameworkKind> {
        let lower = command.to_lowercase();
        for &(keyword, ref kind) in COMMAND_KEYWORDS {
            if lower.contains(&format!("node_modules/.bin/{}", keyword))
                || lower.contains(&format!("node_modules/{}/", keyword))
                || lower
                    .split_whitespace()
                    .any(|part| part.rsplit('/').next() == Some(keyword))
            {
                return Some(kind.clone());
            }
        }
        None
    }

    pub fn detect_by_package_json(cwd: &str) -> (Option<FrameworkKind>, Option<String>) {
        let pkg_path = Path::new(cwd).join("package.json");
        let contents = match fs::read_to_string(&pkg_path) {
            Ok(c) => c,
            Err(_) => return (None, None),
        };

        let parsed: serde_json::Value = match serde_json::from_str(&contents) {
            Ok(v) => v,
            Err(_) => return (None, None),
        };

        let deps_sections = ["dependencies", "devDependencies"];
        let mut all_deps: HashMap<String, String> = HashMap::new();

        for section in &deps_sections {
            if let Some(deps) = parsed.get(section).and_then(|v| v.as_object()) {
                for (key, val) in deps {
                    if let Some(version) = val.as_str() {
                        all_deps.insert(key.clone(), version.to_string());
                    }
                }
            }
        }

        for &(dep_name, ref kind) in PACKAGE_DEPS {
            if let Some(version) = all_deps.get(dep_name) {
                return (Some(kind.clone()), Some(version.clone()));
            }
        }

        (None, None)
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

```bash
cargo test --test framework_test 2>&1
```

Expected: all 5 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/process/framework.rs tests/framework_test.rs
git commit -m "feat: add framework auto-detection (Next.js, Express, Fastify, NestJS, Nuxt, Koa, Hapi)"
```

---

## Task 5: Process Tree Builder

**Files:**
- Create: `src/process/tree.rs`
- Test: `tests/tree_test.rs`

- [ ] **Step 1: Write the failing test**

Create `tests/tree_test.rs`:

```rust
use nsm::process::tree::TreeBuilder;
use nsm::process::ProcessInfo;

fn make_process(pid: u32, ppid: u32, name: &str) -> ProcessInfo {
    let mut p = ProcessInfo::new(pid, name.to_string());
    p.ppid = ppid;
    p
}

#[test]
fn test_build_tree_simple() {
    let flat = vec![
        make_process(100, 1, "node"),
        make_process(101, 100, "next-server"),
        make_process(102, 101, "next-router-worker"),
        make_process(200, 1, "node"),
    ];
    let trees = TreeBuilder::build(flat);
    assert_eq!(trees.len(), 2);

    let first = &trees[0];
    assert_eq!(first.pid, 100);
    assert_eq!(first.children.len(), 1);
    assert_eq!(first.children[0].pid, 101);
    assert_eq!(first.children[0].children.len(), 1);
    assert_eq!(first.children[0].children[0].pid, 102);

    let second = &trees[1];
    assert_eq!(second.pid, 200);
    assert!(second.children.is_empty());
}

#[test]
fn test_build_tree_no_processes() {
    let trees = TreeBuilder::build(vec![]);
    assert!(trees.is_empty());
}

#[test]
fn test_build_tree_all_roots() {
    let flat = vec![
        make_process(100, 1, "node"),
        make_process(200, 1, "node"),
        make_process(300, 1, "node"),
    ];
    let trees = TreeBuilder::build(flat);
    assert_eq!(trees.len(), 3);
    for tree in &trees {
        assert!(tree.children.is_empty());
    }
}

#[test]
fn test_flatten_tree() {
    let flat = vec![
        make_process(100, 1, "node"),
        make_process(101, 100, "next-server"),
        make_process(102, 101, "next-router-worker"),
    ];
    let trees = TreeBuilder::build(flat);
    let flattened = TreeBuilder::flatten_with_depth(&trees);
    assert_eq!(flattened.len(), 3);
    assert_eq!(flattened[0].1, 0); // root depth
    assert_eq!(flattened[1].1, 1); // child depth
    assert_eq!(flattened[2].1, 2); // grandchild depth
}

#[test]
fn test_collect_subtree_pids() {
    let flat = vec![
        make_process(100, 1, "node"),
        make_process(101, 100, "next-server"),
        make_process(102, 101, "next-router-worker"),
    ];
    let trees = TreeBuilder::build(flat);
    let pids = TreeBuilder::collect_pids(&trees[0]);
    assert_eq!(pids, vec![100, 101, 102]);
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test --test tree_test 2>&1
```

Expected: FAIL — `TreeBuilder` doesn't exist.

- [ ] **Step 3: Implement tree builder**

Replace `src/process/tree.rs`:

```rust
use crate::process::ProcessInfo;
use std::collections::HashMap;

pub struct TreeBuilder;

impl TreeBuilder {
    pub fn build(mut flat: Vec<ProcessInfo>) -> Vec<ProcessInfo> {
        if flat.is_empty() {
            return vec![];
        }

        let pids: std::collections::HashSet<u32> = flat.iter().map(|p| p.pid).collect();

        let mut children_map: HashMap<u32, Vec<ProcessInfo>> = HashMap::new();
        let mut roots: Vec<ProcessInfo> = Vec::new();

        flat.sort_by_key(|p| p.pid);

        for process in flat {
            if pids.contains(&process.ppid) {
                children_map
                    .entry(process.ppid)
                    .or_default()
                    .push(process);
            } else {
                roots.push(process);
            }
        }

        for root in &mut roots {
            Self::attach_children(root, &mut children_map);
        }

        roots
    }

    fn attach_children(
        parent: &mut ProcessInfo,
        children_map: &mut HashMap<u32, Vec<ProcessInfo>>,
    ) {
        if let Some(mut children) = children_map.remove(&parent.pid) {
            for child in &mut children {
                Self::attach_children(child, children_map);
            }
            parent.children = children;
        }
    }

    pub fn flatten_with_depth(trees: &[ProcessInfo]) -> Vec<(&ProcessInfo, usize)> {
        let mut result = Vec::new();
        for tree in trees {
            Self::flatten_recursive(tree, 0, &mut result);
        }
        result
    }

    fn flatten_recursive<'a>(
        node: &'a ProcessInfo,
        depth: usize,
        result: &mut Vec<(&'a ProcessInfo, usize)>,
    ) {
        result.push((node, depth));
        for child in &node.children {
            Self::flatten_recursive(child, depth + 1, result);
        }
    }

    pub fn collect_pids(node: &ProcessInfo) -> Vec<u32> {
        let mut pids = vec![node.pid];
        for child in &node.children {
            pids.extend(Self::collect_pids(child));
        }
        pids
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

```bash
cargo test --test tree_test 2>&1
```

Expected: all 5 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/process/tree.rs tests/tree_test.rs
git commit -m "feat: add process tree builder with flatten and PID collection"
```

---

## Task 6: Network Inspector

**Files:**
- Create: `src/process/network.rs`

- [ ] **Step 1: Implement network inspector**

This module shells out to `lsof` (macOS) or reads `/proc/net/tcp` (Linux) to map PIDs to network connections. Unit-testing live network state is unreliable, so we write integration-level code with manual verification.

Replace `src/process/network.rs`:

```rust
use std::collections::HashMap;
use std::net::SocketAddr;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct NetworkConnection {
    pub local_addr: SocketAddr,
    pub remote_addr: Option<SocketAddr>,
    pub state: String,
    pub pid: u32,
}

pub struct NetworkInspector;

impl NetworkInspector {
    pub fn connections_for_pid(pid: u32) -> Vec<NetworkConnection> {
        Self::connections_by_pid()
            .remove(&pid)
            .unwrap_or_default()
    }

    pub fn listening_ports_for_pid(pid: u32) -> Vec<u16> {
        Self::connections_for_pid(pid)
            .iter()
            .filter(|c| c.state == "LISTEN")
            .map(|c| c.local_addr.port())
            .collect()
    }

    pub fn connections_by_pid() -> HashMap<u32, Vec<NetworkConnection>> {
        if cfg!(target_os = "macos") {
            Self::parse_lsof()
        } else {
            Self::parse_lsof() // lsof works on Linux too; /proc/net/tcp is an alternative
        }
    }

    fn parse_lsof() -> HashMap<u32, Vec<NetworkConnection>> {
        let output = Command::new("lsof")
            .args(["-iTCP", "-sTCP:LISTEN,ESTABLISHED", "-nP", "-F", "pcnT"])
            .output();

        let output = match output {
            Ok(o) => o,
            Err(_) => return HashMap::new(),
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut result: HashMap<u32, Vec<NetworkConnection>> = HashMap::new();
        let mut current_pid: Option<u32> = None;
        let mut current_name: Option<String> = None;
        let mut current_state: Option<String> = None;

        for line in stdout.lines() {
            if let Some(pid_str) = line.strip_prefix('p') {
                current_pid = pid_str.parse().ok();
            } else if let Some(name_str) = line.strip_prefix('n') {
                current_name = Some(name_str.to_string());
            } else if let Some(state_str) = line.strip_prefix("TST=") {
                current_state = Some(state_str.to_string());

                if let (Some(pid), Some(ref name)) = (current_pid, &current_name) {
                    if let Some(conn) = Self::parse_connection(name, current_state.as_deref(), pid)
                    {
                        result.entry(pid).or_default().push(conn);
                    }
                }
                current_state = None;
            }
        }

        result
    }

    fn parse_connection(name: &str, state: Option<&str>, pid: u32) -> Option<NetworkConnection> {
        let parts: Vec<&str> = name.splitn(2, "->").collect();
        let local_str = parts[0];
        let remote_str = parts.get(1).copied();

        let local_addr = Self::parse_addr(local_str)?;
        let remote_addr = remote_str.and_then(Self::parse_addr);
        let state = state.unwrap_or("UNKNOWN").to_string();

        Some(NetworkConnection {
            local_addr,
            remote_addr,
            state,
            pid,
        })
    }

    fn parse_addr(s: &str) -> Option<SocketAddr> {
        let s = s.trim();
        if let Ok(addr) = s.parse::<SocketAddr>() {
            return Some(addr);
        }
        // lsof often outputs like "127.0.0.1:3000" or "*:3000" or "[::1]:3000"
        if s.starts_with('*') {
            let port_str = s.strip_prefix("*:")?;
            let port: u16 = port_str.parse().ok()?;
            return Some(SocketAddr::from(([0, 0, 0, 0], port)));
        }
        // Try to parse as "ip:port" where ip might be IPv4
        if let Some(colon_pos) = s.rfind(':') {
            let ip_part = &s[..colon_pos];
            let port_part = &s[colon_pos + 1..];
            if let (Ok(ip), Ok(port)) = (ip_part.parse::<std::net::IpAddr>(), port_part.parse::<u16>()) {
                return Some(SocketAddr::new(ip, port));
            }
        }
        None
    }
}
```

- [ ] **Step 2: Verify it compiles**

```bash
cargo build 2>&1
```

Expected: compiles without errors.

- [ ] **Step 3: Commit**

```bash
git add src/process/network.rs
git commit -m "feat: add network inspector using lsof for port/connection mapping"
```

---

## Task 7: Process Killer

**Files:**
- Create: `src/process/killer.rs`

- [ ] **Step 1: Implement process killer**

Replace `src/process/killer.rs`:

```rust
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KillSignal {
    Term,
    Kill,
    Hup,
    Int,
    Usr1,
    Usr2,
}

impl KillSignal {
    pub fn to_nix_signal(self) -> Signal {
        match self {
            Self::Term => Signal::SIGTERM,
            Self::Kill => Signal::SIGKILL,
            Self::Hup => Signal::SIGHUP,
            Self::Int => Signal::SIGINT,
            Self::Usr1 => Signal::SIGUSR1,
            Self::Usr2 => Signal::SIGUSR2,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Term => "SIGTERM",
            Self::Kill => "SIGKILL",
            Self::Hup => "SIGHUP",
            Self::Int => "SIGINT",
            Self::Usr1 => "SIGUSR1",
            Self::Usr2 => "SIGUSR2",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Term => "Graceful termination request",
            Self::Kill => "Force kill (cannot be caught)",
            Self::Hup => "Hangup / reload configuration",
            Self::Int => "Interrupt (like Ctrl+C)",
            Self::Usr1 => "User-defined (Node.js: activate debugger)",
            Self::Usr2 => "User-defined signal",
        }
    }

    pub fn all() -> &'static [KillSignal] {
        &[
            Self::Term,
            Self::Kill,
            Self::Hup,
            Self::Int,
            Self::Usr1,
            Self::Usr2,
        ]
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "SIGTERM" | "TERM" => Some(Self::Term),
            "SIGKILL" | "KILL" => Some(Self::Kill),
            "SIGHUP" | "HUP" => Some(Self::Hup),
            "SIGINT" | "INT" => Some(Self::Int),
            "SIGUSR1" | "USR1" => Some(Self::Usr1),
            "SIGUSR2" | "USR2" => Some(Self::Usr2),
            _ => None,
        }
    }
}

pub struct ProcessKiller;

#[derive(Debug)]
pub enum KillResult {
    Success,
    AlreadyDead,
    PermissionDenied,
    Error(String),
}

impl ProcessKiller {
    pub fn send_signal(pid: u32, signal: KillSignal) -> KillResult {
        let nix_pid = Pid::from_raw(pid as i32);
        match signal::kill(nix_pid, signal.to_nix_signal()) {
            Ok(()) => KillResult::Success,
            Err(nix::errno::Errno::ESRCH) => KillResult::AlreadyDead,
            Err(nix::errno::Errno::EPERM) => KillResult::PermissionDenied,
            Err(e) => KillResult::Error(e.to_string()),
        }
    }

    pub fn is_alive(pid: u32) -> bool {
        let nix_pid = Pid::from_raw(pid as i32);
        signal::kill(nix_pid, None).is_ok()
    }

    pub fn graceful_kill(pid: u32, timeout: Duration) -> GracefulResult {
        match Self::send_signal(pid, KillSignal::Term) {
            KillResult::AlreadyDead => return GracefulResult::AlreadyDead,
            KillResult::PermissionDenied => return GracefulResult::PermissionDenied,
            KillResult::Error(e) => return GracefulResult::Error(e),
            KillResult::Success => {}
        }

        let start = Instant::now();
        while start.elapsed() < timeout {
            if !Self::is_alive(pid) {
                return GracefulResult::Terminated;
            }
            std::thread::sleep(Duration::from_millis(200));
        }

        GracefulResult::TimedOut
    }

    pub fn force_kill(pid: u32) -> KillResult {
        Self::send_signal(pid, KillSignal::Kill)
    }

    pub fn kill_tree(pids: &[u32], signal: KillSignal) -> Vec<(u32, KillResult)> {
        // Kill children first (reverse order), then parent
        pids.iter()
            .rev()
            .map(|&pid| (pid, Self::send_signal(pid, signal)))
            .collect()
    }
}

#[derive(Debug)]
pub enum GracefulResult {
    Terminated,
    TimedOut,
    AlreadyDead,
    PermissionDenied,
    Error(String),
}
```

- [ ] **Step 2: Verify it compiles**

```bash
cargo build 2>&1
```

Expected: compiles without errors.

- [ ] **Step 3: Commit**

```bash
git add src/process/killer.rs
git commit -m "feat: add process killer with signal sending and graceful shutdown"
```

---

## Task 8: Log Streamer

**Files:**
- Create: `src/log/streamer.rs`

- [ ] **Step 1: Implement log streamer**

Replace `src/log/streamer.rs`:

```rust
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};

const LOG_PATTERNS: &[&str] = &[
    "*.log",
    ".next/server/app/**/*.log",
    ".next/trace",
    "logs/*.log",
    "log/*.log",
    "npm-debug.log",
    "yarn-error.log",
];

const MAX_BUFFER_LINES: usize = 1000;

pub struct LogStreamer {
    file: Option<BufReader<File>>,
    path: Option<PathBuf>,
    buffer: VecDeque<String>,
}

impl LogStreamer {
    pub fn new() -> Self {
        Self {
            file: None,
            path: None,
            buffer: VecDeque::with_capacity(MAX_BUFFER_LINES),
        }
    }

    pub fn detect_and_open(cwd: &str) -> Self {
        let mut streamer = Self::new();

        if let Some(path) = Self::find_log_file(cwd) {
            if let Ok(file) = File::open(&path) {
                let mut reader = BufReader::new(file);
                // Seek to end so we only get new lines
                let _ = reader.seek(SeekFrom::End(0));
                streamer.file = Some(reader);
                streamer.path = Some(path);
            }
        }

        streamer
    }

    #[cfg(target_os = "linux")]
    pub fn detect_and_open_with_proc(cwd: &str, pid: u32) -> Self {
        let mut streamer = Self::detect_and_open(cwd);
        if streamer.file.is_none() {
            let stdout_path = PathBuf::from(format!("/proc/{}/fd/1", pid));
            if let Ok(file) = File::open(&stdout_path) {
                let mut reader = BufReader::new(file);
                let _ = reader.seek(SeekFrom::End(0));
                streamer.file = Some(reader);
                streamer.path = Some(stdout_path);
            }
        }
        streamer
    }

    pub fn find_log_file(cwd: &str) -> Option<PathBuf> {
        let base = Path::new(cwd);

        for pattern in LOG_PATTERNS {
            if let Ok(entries) = glob::glob(&base.join(pattern).to_string_lossy()) {
                let mut files: Vec<PathBuf> = entries.filter_map(Result::ok).collect();
                files.sort_by(|a, b| {
                    let a_mod = a.metadata().and_then(|m| m.modified()).ok();
                    let b_mod = b.metadata().and_then(|m| m.modified()).ok();
                    b_mod.cmp(&a_mod) // most recent first
                });
                if let Some(path) = files.into_iter().next() {
                    return Some(path);
                }
            }
        }

        None
    }

    pub fn poll_new_lines(&mut self) -> Vec<String> {
        let mut new_lines = Vec::new();

        if let Some(ref mut reader) = self.file {
            let mut line = String::new();
            while reader.read_line(&mut line).unwrap_or(0) > 0 {
                let trimmed = line.trim_end().to_string();
                new_lines.push(trimmed.clone());

                self.buffer.push_back(trimmed);
                if self.buffer.len() > MAX_BUFFER_LINES {
                    self.buffer.pop_front();
                }

                line.clear();
            }
        }

        new_lines
    }

    pub fn buffer(&self) -> &VecDeque<String> {
        &self.buffer
    }

    pub fn has_source(&self) -> bool {
        self.file.is_some()
    }

    pub fn source_path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
}
```

Note: add `glob` dependency to `Cargo.toml`:

Add to `[dependencies]` in `Cargo.toml`:
```toml
glob = "0.3"
```

- [ ] **Step 2: Verify it compiles**

```bash
cargo build 2>&1
```

Expected: compiles without errors.

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml src/log/streamer.rs src/log/mod.rs
git commit -m "feat: add log streamer with file detection and tail streaming"
```

---

## Task 9: CLI Parser + Subcommands

**Files:**
- Create: `src/cli.rs`
- Test: `tests/cli_test.rs`

- [ ] **Step 1: Write the failing test**

Create `tests/cli_test.rs`:

```rust
use clap::Parser;
use nsm::cli::{Cli, Commands, ListFormat};

#[test]
fn test_default_launches_tui() {
    let cli = Cli::parse_from(["nsm"]);
    assert!(cli.command.is_none());
}

#[test]
fn test_list_command() {
    let cli = Cli::parse_from(["nsm", "list"]);
    assert!(matches!(cli.command, Some(Commands::List { .. })));
}

#[test]
fn test_list_json() {
    let cli = Cli::parse_from(["nsm", "list", "--json"]);
    if let Some(Commands::List { json, .. }) = cli.command {
        assert!(json);
    } else {
        panic!("Expected List command");
    }
}

#[test]
fn test_list_format_csv() {
    let cli = Cli::parse_from(["nsm", "list", "--format", "csv"]);
    if let Some(Commands::List { format, .. }) = cli.command {
        assert_eq!(format, Some(ListFormat::Csv));
    } else {
        panic!("Expected List command");
    }
}

#[test]
fn test_kill_command() {
    let cli = Cli::parse_from(["nsm", "kill", "1234"]);
    if let Some(Commands::Kill { pid, .. }) = cli.command {
        assert_eq!(pid, 1234);
    } else {
        panic!("Expected Kill command");
    }
}

#[test]
fn test_kill_with_tree() {
    let cli = Cli::parse_from(["nsm", "kill", "--tree", "1234"]);
    if let Some(Commands::Kill { pid, tree, .. }) = cli.command {
        assert_eq!(pid, 1234);
        assert!(tree);
    } else {
        panic!("Expected Kill command");
    }
}

#[test]
fn test_kill_with_signal() {
    let cli = Cli::parse_from(["nsm", "kill", "--signal", "SIGKILL", "1234"]);
    if let Some(Commands::Kill { signal, .. }) = cli.command {
        assert_eq!(signal, Some("SIGKILL".to_string()));
    } else {
        panic!("Expected Kill command");
    }
}

#[test]
fn test_kill_all() {
    let cli = Cli::parse_from(["nsm", "kill", "--all"]);
    if let Some(Commands::Kill { all, .. }) = cli.command {
        assert!(all);
    } else {
        panic!("Expected Kill command");
    }
}

#[test]
fn test_info_command() {
    let cli = Cli::parse_from(["nsm", "info", "5678"]);
    if let Some(Commands::Info { pid }) = cli.command {
        assert_eq!(pid, 5678);
    } else {
        panic!("Expected Info command");
    }
}

#[test]
fn test_log_command() {
    let cli = Cli::parse_from(["nsm", "log", "9999"]);
    if let Some(Commands::Log { pid }) = cli.command {
        assert_eq!(pid, 9999);
    } else {
        panic!("Expected Log command");
    }
}

#[test]
fn test_no_confirm_flag() {
    let cli = Cli::parse_from(["nsm", "kill", "--no-confirm", "1234"]);
    if let Some(Commands::Kill { no_confirm, .. }) = cli.command {
        assert!(no_confirm);
    } else {
        panic!("Expected Kill command");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test --test cli_test 2>&1
```

Expected: FAIL — `Cli` struct doesn't exist.

- [ ] **Step 3: Implement CLI parser**

Replace `src/cli.rs`:

```rust
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(
    name = "nsm",
    version,
    about = "Node Server Manager - Monitor and manage Node.js/Next.js processes"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List all running Node.js processes
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Output format (table, csv)
        #[arg(long)]
        format: Option<ListFormat>,
    },

    /// Kill a Node.js process
    Kill {
        /// Process ID to kill (required unless --all is used)
        #[arg(required_unless_present = "all")]
        pid: Option<u32>,

        /// Kill the entire process tree
        #[arg(long)]
        tree: bool,

        /// Signal to send (default: SIGTERM)
        #[arg(long)]
        signal: Option<String>,

        /// Kill all Node.js processes
        #[arg(long)]
        all: bool,

        /// Skip confirmation prompt
        #[arg(long)]
        no_confirm: bool,
    },

    /// Show detailed info for a process
    Info {
        /// Process ID
        pid: u32,
    },

    /// Stream logs from a process
    Log {
        /// Process ID
        pid: u32,
    },

    /// Open or show config file location
    Config,
}

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum ListFormat {
    Table,
    Csv,
    Json,
}
```

Update `tests/cli_test.rs` — the `kill` command now has `pid` as `Option<u32>`:

Replace `test_kill_command`:
```rust
#[test]
fn test_kill_command() {
    let cli = Cli::parse_from(["nsm", "kill", "1234"]);
    if let Some(Commands::Kill { pid, .. }) = cli.command {
        assert_eq!(pid, Some(1234));
    } else {
        panic!("Expected Kill command");
    }
}
```

Replace `test_kill_with_tree`:
```rust
#[test]
fn test_kill_with_tree() {
    let cli = Cli::parse_from(["nsm", "kill", "--tree", "1234"]);
    if let Some(Commands::Kill { pid, tree, .. }) = cli.command {
        assert_eq!(pid, Some(1234));
        assert!(tree);
    } else {
        panic!("Expected Kill command");
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

```bash
cargo test --test cli_test 2>&1
```

Expected: all 11 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/cli.rs tests/cli_test.rs
git commit -m "feat: add CLI parser with list, kill, info, log, config subcommands"
```

---

## Task 10: TUI App Skeleton + Event Loop

**Files:**
- Create: `src/tui/app.rs`
- Create: `src/tui/event.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Implement App state**

Replace `src/tui/app.rs`:

```rust
use crate::config::Config;
use crate::log::streamer::LogStreamer;
use crate::process::killer::KillSignal;
use crate::process::tree::TreeBuilder;
use crate::process::ProcessInfo;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailTab {
    Info,
    Log,
    Net,
    Env,
}

impl DetailTab {
    pub fn next(&self) -> Self {
        match self {
            Self::Info => Self::Log,
            Self::Log => Self::Net,
            Self::Net => Self::Env,
            Self::Env => Self::Info,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Self::Info => Self::Env,
            Self::Log => Self::Info,
            Self::Net => Self::Log,
            Self::Env => Self::Net,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Info => "Info",
            Self::Log => "Log",
            Self::Net => "Net",
            Self::Env => "Env",
        }
    }

    pub fn all() -> &'static [DetailTab] {
        &[Self::Info, Self::Log, Self::Net, Self::Env]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogKind {
    KillConfirm,
    KillTreeConfirm,
    SignalPicker,
    ForceKillPrompt,
}

pub struct App {
    pub config: Config,
    pub process_trees: Vec<ProcessInfo>,
    pub flat_list: Vec<(ProcessInfo, usize)>, // (process, depth)
    pub selected_index: usize,
    pub selected_pids: HashSet<u32>,
    pub active_tab: DetailTab,
    pub expanded_pids: HashSet<u32>,
    pub dialog: Option<DialogKind>,
    pub signal_picker_index: usize,
    pub filter_text: String,
    pub filter_active: bool,
    pub sort_column: SortColumn,
    pub sort_ascending: bool,
    pub log_streamer: Option<LogStreamer>,
    pub log_scroll: usize,
    pub should_quit: bool,
    pub system_cpu: f32,
    pub system_memory_used: u64,
    pub system_memory_total: u64,
    pub kill_in_progress: Option<(u32, std::time::Instant)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Pid,
    Name,
    Cpu,
    Memory,
    Port,
}

impl SortColumn {
    pub fn next(&self) -> Self {
        match self {
            Self::Pid => Self::Name,
            Self::Name => Self::Cpu,
            Self::Cpu => Self::Memory,
            Self::Memory => Self::Port,
            Self::Port => Self::Pid,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Pid => "PID",
            Self::Name => "NAME",
            Self::Cpu => "CPU",
            Self::Memory => "MEM",
            Self::Port => "PORT",
        }
    }
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            process_trees: Vec::new(),
            flat_list: Vec::new(),
            selected_index: 0,
            selected_pids: HashSet::new(),
            active_tab: DetailTab::Info,
            expanded_pids: HashSet::new(),
            dialog: None,
            signal_picker_index: 0,
            filter_text: String::new(),
            filter_active: false,
            sort_column: SortColumn::Pid,
            sort_ascending: true,
            log_streamer: None,
            log_scroll: 0,
            should_quit: false,
            system_cpu: 0.0,
            system_memory_used: 0,
            system_memory_total: 0,
            kill_in_progress: None,
        }
    }

    pub fn update_processes(&mut self, mut trees: Vec<ProcessInfo>) {
        self.sort_trees(&mut trees);
        self.process_trees = trees;
        self.rebuild_flat_list();
    }

    fn sort_trees(&self, trees: &mut [ProcessInfo]) {
        let ascending = self.sort_ascending;
        trees.sort_by(|a, b| {
            let cmp = match self.sort_column {
                SortColumn::Pid => a.pid.cmp(&b.pid),
                SortColumn::Name => a.framework.to_string().cmp(&b.framework.to_string()),
                SortColumn::Cpu => a.cpu_percent.partial_cmp(&b.cpu_percent).unwrap_or(std::cmp::Ordering::Equal),
                SortColumn::Memory => a.memory_rss.cmp(&b.memory_rss),
                SortColumn::Port => {
                    let a_port = a.ports.first().copied().unwrap_or(0);
                    let b_port = b.ports.first().copied().unwrap_or(0);
                    a_port.cmp(&b_port)
                }
            };
            if ascending { cmp } else { cmp.reverse() }
        });
    }

    fn rebuild_flat_list(&mut self) {
        self.flat_list.clear();

        for tree in &self.process_trees {
            self.flatten_node(tree, 0);
        }

        if self.selected_index >= self.flat_list.len() && !self.flat_list.is_empty() {
            self.selected_index = self.flat_list.len() - 1;
        }
    }

    fn flatten_node(&mut self, node: &ProcessInfo, depth: usize) {
        if self.matches_filter(node) {
            self.flat_list.push((node.clone(), depth));

            if self.expanded_pids.contains(&node.pid) {
                for child in &node.children {
                    self.flatten_node(child, depth + 1);
                }
            }
        }
    }

    fn matches_filter(&self, process: &ProcessInfo) -> bool {
        if self.filter_text.is_empty() {
            return true;
        }
        let filter = self.filter_text.to_lowercase();
        process.name.to_lowercase().contains(&filter)
            || process.framework.to_string().to_lowercase().contains(&filter)
            || process.pid.to_string().contains(&filter)
            || process.ports.iter().any(|p| p.to_string().contains(&filter))
    }

    pub fn selected_process(&self) -> Option<&(ProcessInfo, usize)> {
        self.flat_list.get(self.selected_index)
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.on_selection_changed();
        }
    }

    pub fn move_down(&mut self) {
        if self.selected_index + 1 < self.flat_list.len() {
            self.selected_index += 1;
            self.on_selection_changed();
        }
    }

    pub fn toggle_expand(&mut self) {
        if let Some((process, _)) = self.selected_process() {
            let pid = process.pid;
            if self.expanded_pids.contains(&pid) {
                self.expanded_pids.remove(&pid);
            } else {
                self.expanded_pids.insert(pid);
            }
            self.rebuild_flat_list();
        }
    }

    pub fn toggle_select(&mut self) {
        if let Some((process, _)) = self.selected_process() {
            let pid = process.pid;
            if self.selected_pids.contains(&pid) {
                self.selected_pids.remove(&pid);
            } else {
                self.selected_pids.insert(pid);
            }
        }
    }

    pub fn next_tab(&mut self) {
        self.active_tab = self.active_tab.next();
        self.on_tab_changed();
    }

    pub fn toggle_sort(&mut self) {
        self.sort_column = self.sort_column.next();
        let trees = self.process_trees.clone();
        self.update_processes(trees);
    }

    fn on_selection_changed(&mut self) {
        self.log_streamer = None;
        self.log_scroll = 0;
        if self.active_tab == DetailTab::Log {
            self.init_log_streamer();
        }
    }

    fn on_tab_changed(&mut self) {
        if self.active_tab == DetailTab::Log && self.log_streamer.is_none() {
            self.init_log_streamer();
        }
    }

    fn init_log_streamer(&mut self) {
        if let Some((process, _)) = self.selected_process() {
            let cwd = process.cwd.clone();
            self.log_streamer = Some(LogStreamer::detect_and_open(&cwd));
        }
    }

    pub fn selected_kill_signal(&self) -> KillSignal {
        KillSignal::all()[self.signal_picker_index]
    }
}
```

- [ ] **Step 2: Implement event handler**

Replace `src/tui/event.rs`:

```rust
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;
use tokio::sync::mpsc;

pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    Resize(u16, u16),
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<AppEvent>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        let _tx = tx.clone();
        tokio::spawn(async move {
            loop {
                if event::poll(tick_rate).unwrap_or(false) {
                    match event::read() {
                        Ok(Event::Key(key)) => {
                            if _tx.send(AppEvent::Key(key)).is_err() {
                                break;
                            }
                        }
                        Ok(Event::Resize(w, h)) => {
                            if _tx.send(AppEvent::Resize(w, h)).is_err() {
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                if _tx.send(AppEvent::Tick).is_err() {
                    break;
                }
            }
        });

        Self { rx }
    }

    pub async fn next(&mut self) -> Option<AppEvent> {
        self.rx.recv().await
    }
}
```

- [ ] **Step 3: Implement main.rs with CLI dispatch**

Replace `src/main.rs`:

```rust
mod cli;
mod config;
mod log;
mod process;
mod tui;

use clap::Parser;
use cli::{Cli, Commands, ListFormat};
use config::Config;
use process::killer::{KillSignal, ProcessKiller};
use process::scanner::ProcessScanner;
use process::tree::TreeBuilder;
use process::network::NetworkInspector;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = Config::load();

    match cli.command {
        None => run_tui(config).await,
        Some(Commands::List { json, format }) => cmd_list(&config, json, format),
        Some(Commands::Kill { pid, tree, signal, all, no_confirm }) => {
            cmd_kill(&config, pid, tree, signal, all, no_confirm)
        }
        Some(Commands::Info { pid }) => cmd_info(&config, pid),
        Some(Commands::Log { pid }) => cmd_log(&config, pid),
        Some(Commands::Config) => cmd_config(),
    }
}

async fn run_tui(config: Config) -> anyhow::Result<()> {
    use crossterm::{
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::prelude::*;
    use std::io;
    use tui::app::App;
    use tui::event::{AppEvent, EventHandler};

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(config.clone());
    let mut events = EventHandler::new(config.refresh_duration());

    // Initial scan
    let scanner = ProcessScanner::new(&config);
    let processes = scanner.scan();
    let net_map = NetworkInspector::connections_by_pid();
    let mut enriched = enrich_processes(processes, &net_map);
    let trees = TreeBuilder::build(enriched);
    app.update_processes(trees);

    loop {
        terminal.draw(|f| tui::ui::render(f, &mut app))?;

        if let Some(event) = events.next().await {
            match event {
                AppEvent::Key(key) => {
                    tui::ui::handle_key(&mut app, key);
                }
                AppEvent::Tick => {
                    let processes = scanner.scan();
                    let net_map = NetworkInspector::connections_by_pid();
                    let enriched = enrich_processes(processes, &net_map);
                    let trees = TreeBuilder::build(enriched);
                    app.update_processes(trees);

                    if let Some(ref mut streamer) = app.log_streamer {
                        streamer.poll_new_lines();
                    }
                }
                AppEvent::Resize(_, _) => {}
            }
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn enrich_processes(
    mut processes: Vec<process::ProcessInfo>,
    net_map: &std::collections::HashMap<u32, Vec<process::network::NetworkConnection>>,
) -> Vec<process::ProcessInfo> {
    for p in &mut processes {
        if let Some(conns) = net_map.get(&p.pid) {
            p.ports = conns
                .iter()
                .filter(|c| c.state == "LISTEN")
                .map(|c| c.local_addr.port())
                .collect();
        }
        let (fw, ver) = process::framework::FrameworkDetector::detect(&p.name, &p.command, &p.cwd);
        p.framework = fw;
        p.framework_version = ver;
    }
    processes
}

fn cmd_list(config: &Config, json: bool, format: Option<ListFormat>) -> anyhow::Result<()> {
    let scanner = ProcessScanner::new(config);
    let processes = scanner.scan();
    let net_map = NetworkInspector::connections_by_pid();
    let enriched = enrich_processes(processes, &net_map);
    let trees = TreeBuilder::build(enriched);
    let flat = TreeBuilder::flatten_with_depth(&trees);

    if json || matches!(format, Some(ListFormat::Json)) {
        let infos: Vec<&process::ProcessInfo> = flat.iter().map(|(p, _)| *p).collect();
        println!("{}", serde_json::to_string_pretty(&infos)?);
        return Ok(());
    }

    if matches!(format, Some(ListFormat::Csv)) {
        println!("PID,FRAMEWORK,PORT,CPU,MEM,UPTIME,CWD");
        for (p, _) in &flat {
            let ports = p.ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(";");
            println!(
                "{},{},{},{:.1}%,{},{},{}",
                p.pid, p.framework, ports, p.cpu_percent, p.memory_display(), p.uptime_display(), p.cwd
            );
        }
        return Ok(());
    }

    // Default table format
    println!(
        " {:<8} {:<12} {:<8} {:<7} {:<10} {:<10} {}",
        "PID", "FRAMEWORK", "PORT", "CPU", "MEM", "UPTIME", "CWD"
    );
    for (p, _) in &flat {
        let ports = p.ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(",");
        println!(
            " {:<8} {:<12} {:<8} {:<7} {:<10} {:<10} {}",
            p.pid,
            p.framework,
            if ports.is_empty() { "-".to_string() } else { ports },
            format!("{:.1}%", p.cpu_percent),
            p.memory_display(),
            p.uptime_display(),
            p.cwd
        );
    }

    Ok(())
}

fn cmd_kill(
    config: &Config,
    pid: Option<u32>,
    tree: bool,
    signal: Option<String>,
    all: bool,
    no_confirm: bool,
) -> anyhow::Result<()> {
    let sig = signal
        .as_deref()
        .and_then(KillSignal::from_str)
        .unwrap_or(KillSignal::Term);

    if all {
        let scanner = ProcessScanner::new(config);
        let processes = scanner.scan();
        if processes.is_empty() {
            println!("No Node.js processes found.");
            return Ok(());
        }
        if !no_confirm {
            println!("Kill {} Node.js processes with {}? (y/N)", processes.len(), sig.name());
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Cancelled.");
                return Ok(());
            }
        }
        for p in &processes {
            let result = ProcessKiller::send_signal(p.pid, sig);
            println!("  PID {} ({}): {:?}", p.pid, p.framework, result);
        }
        return Ok(());
    }

    let pid = pid.expect("PID required when --all is not used");

    if tree {
        let scanner = ProcessScanner::new(config);
        let processes = scanner.scan();
        let trees = TreeBuilder::build(processes);
        if let Some(node) = find_node(&trees, pid) {
            let pids = TreeBuilder::collect_pids(node);
            if !no_confirm {
                println!("Kill process tree ({} processes) with {}?", pids.len(), sig.name());
                for &p in &pids {
                    println!("  PID {}", p);
                }
                print!("(y/N) ");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Cancelled.");
                    return Ok(());
                }
            }
            let results = ProcessKiller::kill_tree(&pids, sig);
            for (pid, result) in results {
                println!("  PID {}: {:?}", pid, result);
            }
        } else {
            eprintln!("Process {} not found.", pid);
        }
    } else {
        if !no_confirm {
            println!("Kill PID {} with {}? (y/N)", pid, sig.name());
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Cancelled.");
                return Ok(());
            }
        }
        let result = ProcessKiller::send_signal(pid, sig);
        println!("PID {}: {:?}", pid, result);
    }

    Ok(())
}

fn cmd_info(config: &Config, pid: u32) -> anyhow::Result<()> {
    let scanner = ProcessScanner::new(config);
    let processes = scanner.scan();
    let net_map = NetworkInspector::connections_by_pid();
    let enriched = enrich_processes(processes, &net_map);

    if let Some(p) = enriched.iter().find(|p| p.pid == pid) {
        println!("PID:        {}", p.pid);
        println!("Name:       {}", p.name);
        println!("Framework:  {}", p.framework);
        if let Some(ref v) = p.framework_version {
            println!("Version:    {}", v);
        }
        println!("Ports:      {:?}", p.ports);
        println!("CPU:        {:.1}%", p.cpu_percent);
        println!("Memory:     {} (RSS)", p.memory_display());
        println!("Uptime:     {}", p.uptime_display());
        println!("CWD:        {}", p.cwd);
        println!("Command:    {}", p.command);
        println!("User:       {}", p.user);
        println!("Status:     {}", p.status);
        println!("Threads:    {}", p.threads);
        println!("PPID:       {}", p.ppid);
    } else {
        eprintln!("Process {} not found (or not a Node.js process).", pid);
    }

    Ok(())
}

fn cmd_log(config: &Config, pid: u32) -> anyhow::Result<()> {
    let scanner = ProcessScanner::new(config);
    let processes = scanner.scan();

    if let Some(p) = processes.iter().find(|p| p.pid == pid) {
        let mut streamer = crate::log::streamer::LogStreamer::detect_and_open(&p.cwd);
        if !streamer.has_source() {
            eprintln!("No log source detected for PID {} (cwd: {})", pid, p.cwd);
            eprintln!("Use `nsm config` to set a custom log path.");
            return Ok(());
        }
        println!("Streaming logs from {:?} (Ctrl+C to stop)...", streamer.source_path());
        loop {
            let lines = streamer.poll_new_lines();
            for line in lines {
                println!("{}", line);
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    } else {
        eprintln!("Process {} not found.", pid);
    }

    Ok(())
}

fn cmd_config() -> anyhow::Result<()> {
    let path = Config::config_path();
    println!("Config file: {}", path.display());
    if path.exists() {
        println!("Status: exists");
    } else {
        println!("Status: not created yet (defaults are used)");
        println!("Create it with: mkdir -p {} && touch {}", path.parent().unwrap().display(), path.display());
    }
    Ok(())
}

fn find_node(trees: &[process::ProcessInfo], pid: u32) -> Option<&process::ProcessInfo> {
    for tree in trees {
        if tree.pid == pid {
            return Some(tree);
        }
        if let Some(found) = find_node(&tree.children, pid) {
            return Some(found);
        }
    }
    None
}
```

Add `anyhow` dependency to `Cargo.toml`:
```toml
anyhow = "1"
```

- [ ] **Step 4: Verify it compiles**

```bash
cargo build 2>&1
```

Expected: compiles (TUI rendering stubs will cause warnings but no errors).

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add TUI app skeleton, event loop, CLI dispatch, and all subcommands"
```

---

## Task 11: TUI Layout + Process List Widget

**Files:**
- Create: `src/tui/ui.rs`
- Create: `src/tui/widgets/process_list.rs`
- Create: `src/tui/widgets/status_bar.rs`
- Create: `src/tui/widgets/empty_state.rs`

- [ ] **Step 1: Implement status bars**

Replace `src/tui/widgets/status_bar.rs`:

```rust
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::tui::app::App;

pub fn render_top_bar(f: &mut Frame, area: Rect, app: &App) {
    let mem_used_mb = app.system_memory_used / 1_048_576;
    let mem_total_mb = app.system_memory_total / 1_048_576;
    let node_count = app.flat_list.len();

    let line = Line::from(vec![
        Span::styled(" nsm", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(format!(" v{}", env!("CARGO_PKG_VERSION"))),
        Span::raw("  │  "),
        Span::styled(format!("CPU: {:.1}%", app.system_cpu), Style::default().fg(Color::Cyan)),
        Span::raw("  "),
        Span::styled(format!("MEM: {}/{}MB", mem_used_mb, mem_total_mb), Style::default().fg(Color::Cyan)),
        Span::raw("  │  "),
        Span::styled(format!("Nodes: {}", node_count), Style::default().fg(Color::Yellow)),
        Span::raw("  │  "),
        Span::styled("[H]elp", Style::default().fg(Color::DarkGray)),
    ]);

    f.render_widget(Paragraph::new(line), area);
}

pub fn render_bottom_bar(f: &mut Frame, area: Rect, app: &App) {
    let hints = if app.dialog.is_some() {
        vec![
            ("[Enter] Confirm", Color::Green),
            ("[Esc] Cancel", Color::Red),
        ]
    } else if app.filter_active {
        vec![
            ("[Enter] Apply", Color::Green),
            ("[Esc] Cancel", Color::Red),
        ]
    } else {
        vec![
            ("[↑↓/jk] Move", Color::DarkGray),
            ("[Enter] Expand", Color::DarkGray),
            ("[Tab] Tab", Color::DarkGray),
            ("[Space] Select", Color::DarkGray),
            ("[k]ill", Color::Red),
            ("[K] Tree kill", Color::Red),
            ("[S]ignal", Color::Yellow),
            ("[/] Filter", Color::Cyan),
            ("[s]ort", Color::Cyan),
            ("[q]uit", Color::DarkGray),
        ]
    };

    let spans: Vec<Span> = hints
        .iter()
        .enumerate()
        .flat_map(|(i, (text, color))| {
            let mut v = vec![Span::styled(format!(" {} ", text), Style::default().fg(*color))];
            if i < hints.len() - 1 {
                v.push(Span::raw("│"));
            }
            v
        })
        .collect();

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}
```

- [ ] **Step 2: Implement empty state widget**

Replace `src/tui/widgets/empty_state.rs`:

```rust
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render_empty_state(f: &mut Frame, area: Rect) {
    let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let idx = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        / 100) as usize
        % spinner_chars.len();

    let lines = vec![
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {} No Node.js processes found. Waiting...", spinner_chars[idx]),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  Start a Node.js server and it will appear here automatically.",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    f.render_widget(
        Paragraph::new(lines).alignment(Alignment::Left),
        area,
    );
}
```

- [ ] **Step 3: Implement process list widget**

Replace `src/tui/widgets/process_list.rs`:

```rust
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::process::{HealthStatus, ProcessInfo};
use crate::tui::app::App;

pub fn render_process_list(f: &mut Frame, area: Rect, app: &mut App) {
    let header = Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            format!(" {:<7}", "PID"),
            col_style("PID", app),
        ),
        Span::styled(
            format!(" {:<12}", "FRAMEWORK"),
            col_style("NAME", app),
        ),
        Span::styled(
            format!(" {:<7}", "PORT"),
            col_style("PORT", app),
        ),
        Span::styled(
            format!(" {:<6}", "CPU"),
            col_style("CPU", app),
        ),
        Span::styled(
            format!(" {:<8}", "MEM"),
            col_style("MEM", app),
        ),
        Span::styled(
            format!(" {}", "UPTIME"),
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let items: Vec<ListItem> = app
        .flat_list
        .iter()
        .enumerate()
        .map(|(i, (process, depth))| {
            let selected = app.selected_pids.contains(&process.pid);
            let health = process.health();
            let health_color = match health {
                HealthStatus::Healthy => Color::Green,
                HealthStatus::Warning => Color::Yellow,
                HealthStatus::Critical => Color::Red,
            };

            let checkbox = if selected { "[x]" } else { "[ ]" };

            let tree_prefix = if *depth == 0 {
                if !process.children.is_empty() {
                    if app.expanded_pids.contains(&process.pid) {
                        "▾ ".to_string()
                    } else {
                        "▸ ".to_string()
                    }
                } else {
                    "  ".to_string()
                }
            } else {
                let mut prefix = String::new();
                for _ in 0..*depth {
                    prefix.push_str("  ");
                }
                prefix.push_str("└─ ");
                prefix
            };

            let ports = if process.ports.is_empty() {
                "-".to_string()
            } else {
                process.ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(",")
            };

            let line = Line::from(vec![
                Span::styled(format!("{} ", checkbox), Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("● "),
                    Style::default().fg(health_color),
                ),
                Span::raw(tree_prefix),
                Span::styled(
                    format!("{:<7}", process.pid),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!(" {:<12}", process.framework.to_string()),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    format!(" {:<7}", ports),
                    Style::default().fg(Color::Magenta),
                ),
                Span::styled(
                    format!(" {:<6}", format!("{:.1}%", process.cpu_percent)),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!(" {:<8}", process.memory_display()),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!(" {}", process.uptime_display()),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let title = if app.filter_active || !app.filter_text.is_empty() {
        format!(" Processes [filter: {}] ", app.filter_text)
    } else {
        " Processes ".to_string()
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default();
    state.select(Some(app.selected_index));

    f.render_stateful_widget(list, area, &mut state);
}

fn col_style(col: &str, app: &App) -> Style {
    let is_active = match (col, app.sort_column) {
        ("PID", crate::tui::app::SortColumn::Pid) => true,
        ("NAME", crate::tui::app::SortColumn::Name) => true,
        ("CPU", crate::tui::app::SortColumn::Cpu) => true,
        ("MEM", crate::tui::app::SortColumn::Memory) => true,
        ("PORT", crate::tui::app::SortColumn::Port) => true,
        _ => false,
    };

    if is_active {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}
```

- [ ] **Step 4: Implement top-level UI layout + key handler**

Replace `src/tui/ui.rs`:

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::tui::app::{App, DialogKind};
use crate::tui::widgets::{
    detail_panel, empty_state, kill_dialog, process_list, signal_picker, status_bar,
};

pub fn render(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // top bar
            Constraint::Min(5),   // main content
            Constraint::Length(1), // bottom bar
        ])
        .split(f.area());

    status_bar::render_top_bar(f, chunks[0], app);
    status_bar::render_bottom_bar(f, chunks[2], app);

    if app.flat_list.is_empty() && !app.filter_active {
        empty_state::render_empty_state(f, chunks[1]);
    } else {
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60),
                Constraint::Percentage(40),
            ])
            .split(chunks[1]);

        process_list::render_process_list(f, main_chunks[0], app);
        detail_panel::render_detail_panel(f, main_chunks[1], app);
    }

    // Render modal dialogs on top
    if let Some(dialog) = &app.dialog {
        let dialog_area = centered_rect(60, 50, f.area());
        match dialog {
            DialogKind::KillConfirm | DialogKind::KillTreeConfirm | DialogKind::ForceKillPrompt => {
                kill_dialog::render_kill_dialog(f, dialog_area, app);
            }
            DialogKind::SignalPicker => {
                signal_picker::render_signal_picker(f, dialog_area, app);
            }
        }
    }
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    // Handle dialog inputs first
    if let Some(dialog) = app.dialog {
        handle_dialog_key(app, key, dialog);
        return;
    }

    // Handle filter input mode
    if app.filter_active {
        match key.code {
            KeyCode::Esc => {
                app.filter_active = false;
                app.filter_text.clear();
                let trees = app.process_trees.clone();
                app.update_processes(trees);
            }
            KeyCode::Enter => {
                app.filter_active = false;
            }
            KeyCode::Backspace => {
                app.filter_text.pop();
                let trees = app.process_trees.clone();
                app.update_processes(trees);
            }
            KeyCode::Char(c) => {
                app.filter_text.push(c);
                let trees = app.process_trees.clone();
                app.update_processes(trees);
            }
            _ => {}
        }
        return;
    }

    // Normal mode
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
        }
        KeyCode::Up | KeyCode::Char('k') => app.move_up(),
        KeyCode::Down | KeyCode::Char('j') => app.move_down(),
        KeyCode::Enter => app.toggle_expand(),
        KeyCode::Tab => app.next_tab(),
        KeyCode::Char(' ') => app.toggle_select(),
        KeyCode::Char('/') => {
            app.filter_active = true;
            app.filter_text.clear();
        }
        KeyCode::Char('s') => app.toggle_sort(),
        KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::NONE) => {
            // 'k' without modifier is move up (handled above via Char('k'))
            // lowercase 'k' for kill is handled only if not also navigation
        }
        KeyCode::Char('x') => {
            if !app.selected_pids.is_empty() || app.selected_process().is_some() {
                app.dialog = Some(DialogKind::KillConfirm);
            }
        }
        KeyCode::Char('K') => {
            if app.selected_process().is_some() {
                app.dialog = Some(DialogKind::KillTreeConfirm);
            }
        }
        KeyCode::Char('S') => {
            if app.selected_process().is_some() {
                app.dialog = Some(DialogKind::SignalPicker);
            }
        }
        _ => {}
    }
}

fn handle_dialog_key(app: &mut App, key: KeyEvent, dialog: DialogKind) {
    match key.code {
        KeyCode::Esc => {
            app.dialog = None;
        }
        KeyCode::Enter => {
            match dialog {
                DialogKind::KillConfirm => {
                    execute_kill(app, false);
                    app.dialog = None;
                }
                DialogKind::KillTreeConfirm => {
                    execute_kill(app, true);
                    app.dialog = None;
                }
                DialogKind::SignalPicker => {
                    let signal = app.selected_kill_signal();
                    execute_signal(app, signal);
                    app.dialog = None;
                }
                DialogKind::ForceKillPrompt => {
                    if let Some((process, _)) = app.selected_process() {
                        let pid = process.pid;
                        use crate::process::killer::ProcessKiller;
                        ProcessKiller::force_kill(pid);
                    }
                    app.dialog = None;
                    app.kill_in_progress = None;
                }
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if dialog == DialogKind::SignalPicker && app.signal_picker_index > 0 {
                app.signal_picker_index -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if dialog == DialogKind::SignalPicker {
                use crate::process::killer::KillSignal;
                if app.signal_picker_index + 1 < KillSignal::all().len() {
                    app.signal_picker_index += 1;
                }
            }
        }
        _ => {}
    }
}

fn execute_kill(app: &mut App, tree: bool) {
    use crate::process::killer::{KillSignal, ProcessKiller};
    use crate::process::tree::TreeBuilder;

    if !app.selected_pids.is_empty() {
        let pids: Vec<u32> = app.selected_pids.iter().copied().collect();
        for pid in pids {
            ProcessKiller::send_signal(pid, KillSignal::Term);
        }
        app.selected_pids.clear();
    } else if let Some((process, _)) = app.selected_process() {
        let pid = process.pid;
        if tree {
            let pids = TreeBuilder::collect_pids(process);
            ProcessKiller::kill_tree(&pids, KillSignal::Term);
        } else {
            ProcessKiller::send_signal(pid, KillSignal::Term);
        }
    }
}

fn execute_signal(app: &mut App, signal: crate::process::killer::KillSignal) {
    use crate::process::killer::ProcessKiller;

    if let Some((process, _)) = app.selected_process() {
        let pid = process.pid;
        ProcessKiller::send_signal(pid, signal);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
```

- [ ] **Step 5: Verify it compiles**

```bash
cargo build 2>&1
```

Expected: compiles (detail_panel and other widgets are still stubs).

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: add TUI layout with process list, status bars, and empty state"
```

---

## Task 12: Detail Panel + Info Tab

**Files:**
- Create: `src/tui/widgets/detail_panel.rs`
- Create: `src/tui/widgets/info_tab.rs`

- [ ] **Step 1: Implement detail panel (tab container)**

Replace `src/tui/widgets/detail_panel.rs`:

```rust
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::tui::app::{App, DetailTab};
use crate::tui::widgets::{env_tab, info_tab, log_tab, net_tab};

pub fn render_detail_panel(f: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // tabs
            Constraint::Min(1),   // content
        ])
        .split(area);

    let tab_titles: Vec<Span> = DetailTab::all()
        .iter()
        .map(|t| {
            if *t == app.active_tab {
                Span::styled(
                    format!(" {} ", t.label()),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(
                    format!(" {} ", t.label()),
                    Style::default().fg(Color::DarkGray),
                )
            }
        })
        .collect();

    let selected_idx = DetailTab::all()
        .iter()
        .position(|t| *t == app.active_tab)
        .unwrap_or(0);

    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL).title(" Detail "))
        .select(selected_idx)
        .highlight_style(Style::default().fg(Color::Yellow));

    f.render_widget(tabs, chunks[0]);

    match app.active_tab {
        DetailTab::Info => info_tab::render_info_tab(f, chunks[1], app),
        DetailTab::Log => log_tab::render_log_tab(f, chunks[1], app),
        DetailTab::Net => net_tab::render_net_tab(f, chunks[1], app),
        DetailTab::Env => env_tab::render_env_tab(f, chunks[1], app),
    }
}
```

- [ ] **Step 2: Implement info tab**

Replace `src/tui/widgets/info_tab.rs`:

```rust
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::App;

pub fn render_info_tab(f: &mut Frame, area: Rect, app: &App) {
    let content = if let Some((process, _)) = app.selected_process() {
        let ports_str = if process.ports.is_empty() {
            "-".to_string()
        } else {
            process.ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", ")
        };

        let version_str = process
            .framework_version
            .as_deref()
            .unwrap_or("-");

        vec![
            info_line("PID", &process.pid.to_string()),
            info_line("PPID", &process.ppid.to_string()),
            info_line("Name", &process.name),
            info_line("Framework", &process.framework.to_string()),
            info_line("Version", version_str),
            info_line("Ports", &ports_str),
            info_line("CPU", &format!("{:.1}%", process.cpu_percent)),
            info_line("Memory", &format!("{} (RSS)", process.memory_display())),
            info_line("Threads", &process.threads.to_string()),
            info_line("Uptime", &process.uptime_display()),
            info_line("User", &process.user),
            info_line("Status", &process.status),
            info_line("FDs", &process.open_fds.to_string()),
            Line::from(""),
            info_line("CWD", &process.cwd),
            Line::from(""),
            Line::from(Span::styled(" Command:", Style::default().fg(Color::DarkGray))),
            Line::from(Span::styled(
                format!("   {}", process.command),
                Style::default().fg(Color::White),
            )),
        ]
    } else {
        vec![Line::from(Span::styled(
            " Select a process to view details",
            Style::default().fg(Color::DarkGray),
        ))]
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::LEFT));

    f.render_widget(paragraph, area);
}

fn info_line(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!(" {:<12}", label), Style::default().fg(Color::DarkGray)),
        Span::styled(value.to_string(), Style::default().fg(Color::White)),
    ])
}
```

- [ ] **Step 3: Verify it compiles**

```bash
cargo build 2>&1
```

- [ ] **Step 4: Commit**

```bash
git add src/tui/widgets/detail_panel.rs src/tui/widgets/info_tab.rs
git commit -m "feat: add detail panel with tabbed navigation and info tab"
```

---

## Task 13: Log, Net, Env Tabs

**Files:**
- Create: `src/tui/widgets/log_tab.rs`
- Create: `src/tui/widgets/net_tab.rs`
- Create: `src/tui/widgets/env_tab.rs`

- [ ] **Step 1: Implement log tab**

Replace `src/tui/widgets/log_tab.rs`:

```rust
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::App;

pub fn render_log_tab(f: &mut Frame, area: Rect, app: &App) {
    let content = if let Some(ref streamer) = app.log_streamer {
        if !streamer.has_source() {
            vec![
                Line::from(""),
                Line::from(Span::styled(
                    " No log source detected.",
                    Style::default().fg(Color::Yellow),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    " Use `nsm config` to set a custom log path for this process.",
                    Style::default().fg(Color::DarkGray),
                )),
            ]
        } else {
            let source_line = Line::from(Span::styled(
                format!(" Source: {:?}", streamer.source_path().unwrap_or(std::path::Path::new("-"))),
                Style::default().fg(Color::DarkGray),
            ));

            let mut lines = vec![source_line, Line::from("")];

            let buffer = streamer.buffer();
            let visible_height = area.height.saturating_sub(4) as usize;
            let start = buffer.len().saturating_sub(visible_height + app.log_scroll);
            let end = buffer.len().saturating_sub(app.log_scroll);

            for line in buffer.range(start..end) {
                lines.push(Line::from(Span::styled(
                    format!(" {}", line),
                    Style::default().fg(Color::White),
                )));
            }

            if buffer.is_empty() {
                lines.push(Line::from(Span::styled(
                    " Waiting for log output...",
                    Style::default().fg(Color::DarkGray),
                )));
            }

            lines
        }
    } else {
        vec![Line::from(Span::styled(
            " Select a process to view logs",
            Style::default().fg(Color::DarkGray),
        ))]
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::LEFT))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}
```

- [ ] **Step 2: Implement net tab**

Replace `src/tui/widgets/net_tab.rs`:

```rust
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::process::network::NetworkInspector;
use crate::tui::app::App;

pub fn render_net_tab(f: &mut Frame, area: Rect, app: &App) {
    let content = if let Some((process, _)) = app.selected_process() {
        let connections = NetworkInspector::connections_for_pid(process.pid);

        if connections.is_empty() {
            vec![
                Line::from(""),
                Line::from(Span::styled(
                    " No active network connections.",
                    Style::default().fg(Color::DarkGray),
                )),
            ]
        } else {
            let mut lines = vec![
                Line::from(vec![
                    Span::styled(
                        format!(" {:<24} {:<24} {}", "LOCAL", "REMOTE", "STATE"),
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(Span::styled(
                    " ─".repeat(30),
                    Style::default().fg(Color::DarkGray),
                )),
            ];

            for conn in &connections {
                let remote = conn
                    .remote_addr
                    .map(|a| a.to_string())
                    .unwrap_or_else(|| "-".to_string());

                let state_color = match conn.state.as_str() {
                    "LISTEN" => Color::Green,
                    "ESTABLISHED" => Color::Cyan,
                    "TIME_WAIT" | "CLOSE_WAIT" => Color::Yellow,
                    _ => Color::White,
                };

                lines.push(Line::from(vec![
                    Span::styled(
                        format!(" {:<24}", conn.local_addr),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(
                        format!(" {:<24}", remote),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(
                        format!(" {}", conn.state),
                        Style::default().fg(state_color),
                    ),
                ]));
            }

            lines
        }
    } else {
        vec![Line::from(Span::styled(
            " Select a process to view network info",
            Style::default().fg(Color::DarkGray),
        ))]
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::LEFT));

    f.render_widget(paragraph, area);
}
```

- [ ] **Step 3: Implement env tab**

Replace `src/tui/widgets/env_tab.rs`:

```rust
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::App;

pub fn render_env_tab(f: &mut Frame, area: Rect, app: &App) {
    let content = if let Some((process, _)) = app.selected_process() {
        if process.env_vars.is_empty() {
            vec![
                Line::from(""),
                Line::from(Span::styled(
                    " No environment variables available.",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(Span::styled(
                    " (May require elevated permissions to read)",
                    Style::default().fg(Color::DarkGray),
                )),
            ]
        } else {
            let mask = app.config.display.mask_env_values;
            let sensitive_keys = [
                "PASSWORD", "SECRET", "TOKEN", "KEY", "API_KEY", "PRIVATE",
                "CREDENTIALS", "AUTH",
            ];

            let mut lines = Vec::new();
            for (key, value) in &process.env_vars {
                let display_value = if mask
                    && sensitive_keys
                        .iter()
                        .any(|s| key.to_uppercase().contains(s))
                {
                    "********".to_string()
                } else {
                    value.clone()
                };

                lines.push(Line::from(vec![
                    Span::styled(
                        format!(" {}", key),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled("=", Style::default().fg(Color::DarkGray)),
                    Span::styled(display_value, Style::default().fg(Color::White)),
                ]));
            }
            lines
        }
    } else {
        vec![Line::from(Span::styled(
            " Select a process to view environment",
            Style::default().fg(Color::DarkGray),
        ))]
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::LEFT));

    f.render_widget(paragraph, area);
}
```

- [ ] **Step 4: Verify it compiles**

```bash
cargo build 2>&1
```

- [ ] **Step 5: Commit**

```bash
git add src/tui/widgets/log_tab.rs src/tui/widgets/net_tab.rs src/tui/widgets/env_tab.rs
git commit -m "feat: add log, network, and environment detail tabs"
```

---

## Task 14: Kill Dialog + Signal Picker

**Files:**
- Create: `src/tui/widgets/kill_dialog.rs`
- Create: `src/tui/widgets/signal_picker.rs`

- [ ] **Step 1: Implement kill dialog**

Replace `src/tui/widgets/kill_dialog.rs`:

```rust
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::tui::app::{App, DialogKind};

pub fn render_kill_dialog(f: &mut Frame, area: Rect, app: &App) {
    f.render_widget(Clear, area);

    let dialog = app.dialog.unwrap_or(DialogKind::KillConfirm);

    let mut lines = vec![Line::from("")];

    match dialog {
        DialogKind::KillConfirm => {
            if !app.selected_pids.is_empty() {
                lines.push(Line::from(Span::styled(
                    format!("  Kill {} selected processes with SIGTERM?", app.selected_pids.len()),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(""));
                for &pid in &app.selected_pids {
                    if let Some((p, _)) = app.flat_list.iter().find(|(p, _)| p.pid == pid) {
                        let ports = p.ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(",");
                        lines.push(Line::from(Span::styled(
                            format!("    PID {} ({}) :{}", p.pid, p.framework, ports),
                            Style::default().fg(Color::White),
                        )));
                    }
                }
            } else if let Some((process, _)) = app.selected_process() {
                let ports = process.ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(",");
                lines.push(Line::from(Span::styled(
                    "  Kill process with SIGTERM?",
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    format!("    PID:  {}", process.pid),
                    Style::default().fg(Color::White),
                )));
                lines.push(Line::from(Span::styled(
                    format!("    Name: {} ({})", process.name, process.framework),
                    Style::default().fg(Color::White),
                )));
                lines.push(Line::from(Span::styled(
                    format!("    Port: {}", if ports.is_empty() { "-".to_string() } else { ports }),
                    Style::default().fg(Color::White),
                )));
            }
        }
        DialogKind::KillTreeConfirm => {
            if let Some((process, _)) = app.selected_process() {
                use crate::process::tree::TreeBuilder;
                let pids = TreeBuilder::collect_pids(process);
                lines.push(Line::from(Span::styled(
                    format!("  Kill process tree ({} processes) with SIGTERM?", pids.len()),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(""));
                for pid in &pids {
                    lines.push(Line::from(Span::styled(
                        format!("    PID {}", pid),
                        Style::default().fg(Color::White),
                    )));
                }
            }
        }
        DialogKind::ForceKillPrompt => {
            lines.push(Line::from(Span::styled(
                "  Process did not exit. Force kill with SIGKILL?",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )));
        }
        _ => {}
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("    [Enter] ", Style::default().fg(Color::Green)),
        Span::raw("Confirm  "),
        Span::styled("[Esc] ", Style::default().fg(Color::Red)),
        Span::raw("Cancel"),
    ]));

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Confirm ")
        .style(Style::default().bg(Color::Black));

    f.render_widget(Paragraph::new(lines).block(block), area);
}
```

- [ ] **Step 2: Implement signal picker**

Replace `src/tui/widgets/signal_picker.rs`:

```rust
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::process::killer::KillSignal;
use crate::tui::app::App;

pub fn render_signal_picker(f: &mut Frame, area: Rect, app: &App) {
    f.render_widget(Clear, area);

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Select signal to send:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    for (i, signal) in KillSignal::all().iter().enumerate() {
        let is_selected = i == app.signal_picker_index;
        let prefix = if is_selected { " ▸ " } else { "   " };
        let style = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(format!("{:<12}", signal.name()), style),
            Span::styled(
                signal.description(),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("    [↑↓] ", Style::default().fg(Color::Cyan)),
        Span::raw("Select  "),
        Span::styled("[Enter] ", Style::default().fg(Color::Green)),
        Span::raw("Send  "),
        Span::styled("[Esc] ", Style::default().fg(Color::Red)),
        Span::raw("Cancel"),
    ]));

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Signal ")
        .style(Style::default().bg(Color::Black));

    f.render_widget(Paragraph::new(lines).block(block), area);
}
```

- [ ] **Step 3: Update widgets/mod.rs**

Replace `src/tui/widgets/mod.rs`:

```rust
pub mod process_list;
pub mod detail_panel;
pub mod info_tab;
pub mod log_tab;
pub mod net_tab;
pub mod env_tab;
pub mod kill_dialog;
pub mod signal_picker;
pub mod status_bar;
pub mod empty_state;
```

- [ ] **Step 4: Verify it compiles and build succeeds**

```bash
cargo build 2>&1
```

Expected: clean build.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add kill confirmation dialog and signal picker modal"
```

---

## Task 15: Update lib.rs + Fix Compilation + Integration Test

**Files:**
- Modify: `src/lib.rs`
- Modify: various files for compilation fixes

- [ ] **Step 1: Ensure lib.rs re-exports correctly**

Replace `src/lib.rs`:

```rust
pub mod cli;
pub mod config;
pub mod log;
pub mod process;
pub mod tui;
```

- [ ] **Step 2: Run full build and fix any remaining compilation errors**

```bash
cargo build 2>&1
```

Fix any remaining issues (import paths, missing derives, etc.) iteratively until the build succeeds.

- [ ] **Step 3: Run all tests**

```bash
cargo test 2>&1
```

Expected: all tests pass (types_test, config_test, scanner_test, framework_test, tree_test, cli_test).

- [ ] **Step 4: Test the CLI subcommands manually**

```bash
cargo run -- list
cargo run -- list --json
cargo run -- config
```

Verify output is reasonable (may show 0 processes if none are running).

- [ ] **Step 5: Launch the TUI briefly**

```bash
cargo run
```

Verify the TUI opens, shows the layout (or empty state), and `q` exits cleanly.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: integration fixes, all modules connected, TUI and CLI functional"
```

---

## Task 16: End-to-end Test with Live Node Process

This is a manual verification task — start a real Node.js server and verify the tool works.

- [ ] **Step 1: Start a test Node server**

In a separate terminal:
```bash
node -e "require('http').createServer((req,res) => res.end('ok')).listen(9876, () => console.log('listening on 9876'))"
```

- [ ] **Step 2: Test TUI detection**

```bash
cargo run
```

Verify:
- The Node.js process appears in the list with PID, port 9876
- Framework shows "Node.js" (generic)
- Info tab shows correct details
- Net tab shows port 9876 as LISTEN
- Navigation (↑↓, Tab) works
- `q` exits

- [ ] **Step 3: Test CLI detection**

```bash
cargo run -- list
cargo run -- list --json
```

Verify the test Node process appears in output.

- [ ] **Step 4: Test kill from CLI**

```bash
cargo run -- kill --no-confirm <PID>
```

Verify the Node process is terminated.

- [ ] **Step 5: Commit any fixes**

```bash
git add -A
git commit -m "fix: end-to-end testing adjustments"
```

---

## Task 17: Release Configuration

**Files:**
- Modify: `Cargo.toml` (metadata)
- Create: `.github/workflows/release.yml` (optional, for CI)

- [ ] **Step 1: Finalize Cargo.toml metadata**

Ensure `Cargo.toml` has complete metadata:

```toml
[package]
name = "nsm"
version = "0.1.0"
edition = "2021"
description = "Node Server Manager - TUI tool for monitoring and managing Node.js/Next.js processes"
license = "MIT"
repository = "https://github.com/withwiz/nsm"
keywords = ["node", "process", "tui", "monitor", "nextjs"]
categories = ["command-line-utilities", "development-tools"]
readme = "README.md"
```

- [ ] **Step 2: Verify cargo install works**

```bash
cargo install --path .
nsm --version
nsm list
```

- [ ] **Step 3: Build release binary**

```bash
cargo build --release
ls -la target/release/nsm
```

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "chore: finalize Cargo.toml metadata for release"
```

---

## Spec Coverage Check

| Spec Requirement | Task |
|---|---|
| TUI with split-panel layout | Task 11 |
| Real-time process list refresh | Task 10, 11 |
| Node.js process detection | Task 3 |
| Framework auto-detection | Task 4 |
| Process tree display | Task 5, 11 |
| Info tab | Task 12 |
| Log tab streaming | Task 8, 13 |
| Net tab | Task 6, 13 |
| Env tab with masking | Task 13 |
| Single kill with signal | Task 7, 14 |
| Tree kill | Task 7, 14 |
| Multi-select kill | Task 11 (select), 14 (dialog) |
| Signal picker | Task 14 |
| Graceful shutdown flow | Task 7 |
| Kill confirmation dialog | Task 14 |
| `nsm list` (table/JSON/CSV) | Task 10 |
| `nsm kill` | Task 10 |
| `nsm info` | Task 10 |
| `nsm log` | Task 10 |
| Config file | Task 2 |
| Status indicators (color) | Task 11 |
| Search/filter `/` | Task 10 (key), 11 (display) |
| Column sorting `s` | Task 10 (key), 11 (display) |
| Keyboard navigation | Task 10 |
| Empty state | Task 11 |
| Log fallback message | Task 13 |
| Single binary build | Task 17 |
| cargo install | Task 17 |
