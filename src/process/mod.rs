pub mod scanner;
pub mod framework;
pub mod framework_rules;
pub mod tree;
pub mod killer;
pub mod network;
pub mod platform;

use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;

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

/// Web framework detected for a monitored process, across all runtimes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameworkKind {
    NextJs,
    Express,
    Fastify,
    NestJs,
    Nuxt,
    Koa,
    Hapi,
    FastApi,
    Flask,
    Django,
    SpringBoot,
    Rails,
    Laravel,
    AspNet,
    Generic,
}

impl fmt::Display for FrameworkKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameworkKind::NextJs => write!(f, "Next.js"),
            FrameworkKind::Express => write!(f, "Express"),
            FrameworkKind::Fastify => write!(f, "Fastify"),
            FrameworkKind::NestJs => write!(f, "NestJs"),
            FrameworkKind::Nuxt => write!(f, "Nuxt.js"),
            FrameworkKind::Koa => write!(f, "Koa"),
            FrameworkKind::Hapi => write!(f, "Hapi"),
            FrameworkKind::FastApi => write!(f, "FastAPI"),
            FrameworkKind::Flask => write!(f, "Flask"),
            FrameworkKind::Django => write!(f, "Django"),
            FrameworkKind::SpringBoot => write!(f, "Spring Boot"),
            FrameworkKind::Rails => write!(f, "Rails"),
            FrameworkKind::Laravel => write!(f, "Laravel"),
            FrameworkKind::AspNet => write!(f, "ASP.NET"),
            FrameworkKind::Generic => write!(f, "Generic"),
        }
    }
}

/// Health status of a process based on resource usage or process state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
}

impl HealthStatus {
    /// Determine health status from CPU and memory usage percentages.
    /// - Critical: CPU >= 90% or memory >= 90%
    /// - Warning: CPU >= 80% or memory >= 80%
    /// - Healthy: otherwise
    pub fn from_cpu_mem(cpu: f32, mem: f32) -> Self {
        if cpu >= 90.0 || mem >= 90.0 {
            HealthStatus::Critical
        } else if cpu >= 80.0 || mem >= 80.0 {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        }
    }

    /// Determine health status from process status string.
    /// - "Zombie" or "Dead" → Critical
    /// - "Sleeping" or "Idle" → Healthy
    /// - Otherwise → Healthy
    pub fn from_process_status(status: &str) -> Self {
        match status {
            "Zombie" | "Dead" => HealthStatus::Critical,
            _ => HealthStatus::Healthy,
        }
    }
}

impl fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "Healthy"),
            HealthStatus::Warning => write!(f, "Warning"),
            HealthStatus::Critical => write!(f, "Critical"),
        }
    }
}

/// Information about a running Node.js process.
#[derive(Debug, Clone)]
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
    /// `Some(runtime)` for processes the scanner classifies as servers.
    /// `None` for tree-context parents the second pass backfills (e.g.
    /// launchd, claude) so the tree view is rooted.
    pub runtime: Option<Runtime>,
}

impl ProcessInfo {
    /// Create a new ProcessInfo with sensible defaults.
    pub fn new(pid: u32, name: &str) -> Self {
        ProcessInfo {
            pid,
            ppid: 0,
            name: name.to_string(),
            command: String::new(),
            cwd: String::new(),
            framework: FrameworkKind::Generic,
            framework_version: None,
            ports: Vec::new(),
            cpu_percent: 0.0,
            memory_rss: 0,
            memory_vms: 0,
            threads: 0,
            uptime: Duration::from_secs(0),
            user: String::new(),
            status: String::from("Running"),
            open_fds: 0,
            children: Vec::new(),
            env_vars: Vec::new(),
            runtime: None,
        }
    }

    /// True if this process is a classified server (any runtime).
    pub fn is_server(&self) -> bool {
        self.runtime.is_some()
    }

    /// True if this process is a Node server specifically.
    pub fn is_node(&self) -> bool {
        self.runtime == Some(Runtime::Node)
    }

    /// Format the uptime duration as a human-readable string.
    /// e.g., "1h 2m 5s"
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

    /// Format memory_rss as a human-readable string.
    /// e.g., "128.0 MB", "1.5 GB"
    pub fn memory_display(&self) -> String {
        let bytes = self.memory_rss as f64;
        if bytes >= 1_073_741_824.0 {
            format!("{:.1} GB", bytes / 1_073_741_824.0)
        } else if bytes >= 1_048_576.0 {
            format!("{:.1} MB", bytes / 1_048_576.0)
        } else if bytes >= 1_024.0 {
            format!("{:.1} KB", bytes / 1_024.0)
        } else {
            format!("{} B", self.memory_rss)
        }
    }

    /// Derive a display name from the command for better identification.
    pub fn display_name(&self) -> String {
        if self.command.is_empty() {
            return self.name.clone();
        }

        let all_parts: Vec<&str> = self.command.split_whitespace().collect();
        if all_parts.is_empty() {
            return self.name.clone();
        }

        let binary = all_parts[0].rsplit('/').next().unwrap_or(all_parts[0]);

        fn is_subcommand(s: &str) -> bool {
            s.len() <= 20
                && s.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
                && s.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false)
        }

        fn is_script(s: &str) -> bool {
            let name = s.rsplit('/').next().unwrap_or(s);
            name.ends_with(".js") || name.ends_with(".ts") || name.ends_with(".mjs")
                || name.ends_with(".cjs") || name.ends_with(".sh")
        }

        fn is_package(s: &str) -> bool {
            s.starts_with('@') && s.len() <= 40
        }

        // Find node_modules tool if present
        let modules_tool = all_parts[1..].iter()
            .find(|s| s.contains("node_modules/"))
            .map(|s| s.rsplit('/').next().unwrap_or(s));

        // Collect clean args: stop at first JSON/env blob
        let mut clean_args: Vec<&str> = Vec::new();
        for s in &all_parts[1..] {
            if s.starts_with('{') || s.starts_with('"') || s.contains("\":") {
                break;
            }
            if s.contains("node_modules/") {
                continue;
            }
            let name = s.rsplit('/').next().unwrap_or(s);
            if is_subcommand(name) || is_script(name) || is_package(name) {
                clean_args.push(name);
                if clean_args.len() >= 2 {
                    break;
                }
            }
        }

        if binary != "node" {
            if clean_args.is_empty() {
                return binary.to_string();
            }
            return format!("{} {}", binary, clean_args.join(" "));
        }

        // node process — prefer node_modules tool name
        if let Some(tool) = modules_tool {
            if clean_args.is_empty() {
                return tool.to_string();
            }
            return format!("{} {}", tool, clean_args.join(" "));
        }

        if clean_args.is_empty() {
            return self.name.clone();
        }
        format!("node {}", clean_args.join(" "))
    }

    /// Get the health status of this process based on its metrics and status.
    pub fn health(&self) -> HealthStatus {
        let status_health = HealthStatus::from_process_status(&self.status);
        if status_health == HealthStatus::Critical {
            return HealthStatus::Critical;
        }
        HealthStatus::from_cpu_mem(self.cpu_percent, self.memory_rss as f32 / 1_048_576.0)
    }
}
