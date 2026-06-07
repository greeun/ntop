// Process scanner - discovers running server processes across runtimes

use crate::config::Config;
use crate::process::framework::FrameworkDetector;
use crate::process::ProcessInfo;
use std::collections::HashSet;
use std::time::Duration;
use sysinfo::{ProcessRefreshKind, System, UpdateKind};

pub struct ProcessScanner<'a> {
    config: &'a Config,
    // sysinfo computes per-process CPU usage as the delta between two
    // refreshes, so we keep a long-lived System and refresh it in place.
    // Constructing a fresh System on every scan would make cpu_usage()
    // permanently 0.0.
    sys: System,
}


impl<'a> ProcessScanner<'a> {
    pub fn new(config: &'a Config) -> Self {
        let mut sys = System::new();
        // sysinfo needs 3+ refreshes (with ≥ MINIMUM_CPU_UPDATE_INTERVAL
        // between samples) before per-process cpu_usage() returns a real
        // value. The first two refreshes here prime the sampler so that
        // the next scan() (after at least one tick interval) already
        // produces accurate CPU readings.
        Self::refresh_processes(&mut sys);
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        Self::refresh_processes(&mut sys);
        Self { config, sys }
    }

    fn refresh_processes(sys: &mut System) {
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

    pub fn scan(&mut self) -> Vec<ProcessInfo> {
        Self::refresh_processes(&mut self.sys);

        let mut results = Vec::new();
        let mut server_pids = HashSet::new();

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
            server_pids.insert(pid.as_u32());
            results.push(info);
        }

        // Second pass: collect parent processes that aren't already server processes
        let parent_ppids: Vec<u32> = results
            .iter()
            .filter(|p| p.ppid != 0 && !server_pids.contains(&p.ppid))
            .map(|p| p.ppid)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        for ppid in parent_ppids {
            let sysinfo_pid = sysinfo::Pid::from_u32(ppid);
            if let Some(process) = self.sys.process(sysinfo_pid) {
                let info = Self::collect_process_info(process, ppid);
                results.push(info);
            }
        }

        results
    }

    /// One-shot scan with built-in CPU warmup for non-loop callers (CLI
    /// commands). Sleeps `MINIMUM_CPU_UPDATE_INTERVAL` after `new()`'s two
    /// priming refreshes, then does the third measurement refresh — this
    /// is the minimum sysinfo requires for `cpu_usage()` to return a real
    /// value instead of 0.0.
    pub fn scan_blocking(&mut self) -> Vec<ProcessInfo> {
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        self.scan()
    }

}
