// Process scanner - discovers running Node.js processes

use crate::config::Config;
use crate::process::ProcessInfo;
use std::collections::HashSet;
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

    fn collect_process_info(process: &sysinfo::Process, pid: u32) -> ProcessInfo {
        let name = process.name().to_string_lossy().to_string();
        let cmd_parts: Vec<String> = process
            .cmd()
            .iter()
            .map(|s| s.to_string_lossy().to_string())
            .collect();
        let command = cmd_parts.join(" ");
        let cwd = process
            .cwd()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let mut info = ProcessInfo::new(pid, &name);
        info.ppid = process.parent().map(|p| p.as_u32()).unwrap_or(0);
        info.command = command;
        info.cwd = cwd;
        info.cpu_percent = process.cpu_usage();
        // On macOS, prefer phys_footprint (what Activity Monitor shows) over
        // sysinfo's RSS, which excludes compressed memory and can underreport
        // by 100×+ for idle Node processes.
        info.memory_rss = crate::process::platform::phys_footprint(pid)
            .unwrap_or_else(|| process.memory());
        info.memory_vms = process.virtual_memory();
        info.status = format!("{:?}", process.status());
        info.uptime = Duration::from_secs(process.run_time());
        info.threads = crate::process::platform::thread_count(pid);
        info.open_fds = crate::process::platform::open_fd_count(pid);

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

        info.user = process
            .user_id()
            .map(|u| u.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        info
    }

    pub fn scan(&self) -> Vec<ProcessInfo> {
        let mut sys = System::new();
        sys.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing()
                .with_cpu()
                .with_memory()
                .with_cmd(UpdateKind::Always)
                .with_cwd(UpdateKind::Always)
                .with_environ(UpdateKind::Always)
                .with_user(UpdateKind::Always),
        );

        let mut results = Vec::new();
        let mut node_pids = HashSet::new();

        // First pass: collect node processes
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

            let info = Self::collect_process_info(process, pid.as_u32());
            node_pids.insert(pid.as_u32());
            results.push(info);
        }

        // Second pass: collect parent processes that aren't already node processes
        let parent_ppids: Vec<u32> = results
            .iter()
            .filter(|p| p.ppid != 0 && !node_pids.contains(&p.ppid))
            .map(|p| p.ppid)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        for ppid in parent_ppids {
            let sysinfo_pid = sysinfo::Pid::from_u32(ppid);
            if let Some(process) = sys.process(sysinfo_pid) {
                let info = Self::collect_process_info(process, ppid);
                results.push(info);
            }
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
