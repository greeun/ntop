// Process scanner - discovers running Node.js processes

use crate::config::Config;
use crate::process::ProcessInfo;
use std::time::Duration;
use sysinfo::{ProcessRefreshKind, System, UpdateKind};

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
            ProcessRefreshKind::nothing()
                .with_cpu()
                .with_memory()
                .with_cmd(UpdateKind::Always),
        );

        let mut results = Vec::new();

        for (pid, process) in sys.processes() {
            let name = process.name().to_string_lossy().to_string();
            let cmd_parts: Vec<String> = process
                .cmd()
                .iter()
                .map(|s| s.to_string_lossy().to_string())
                .collect();
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

            let mut info = ProcessInfo::new(pid.as_u32(), &name);
            info.ppid = process.parent().map(|p| p.as_u32()).unwrap_or(0);
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
