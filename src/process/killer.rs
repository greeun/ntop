// Process termination and signal sending

use std::thread;
use std::time::Duration;

use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;

/// Represents the type of signal to send to a process.
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
    /// Convert to the corresponding nix Signal type.
    pub fn to_nix_signal(&self) -> Signal {
        match self {
            KillSignal::Term => Signal::SIGTERM,
            KillSignal::Kill => Signal::SIGKILL,
            KillSignal::Hup => Signal::SIGHUP,
            KillSignal::Int => Signal::SIGINT,
            KillSignal::Usr1 => Signal::SIGUSR1,
            KillSignal::Usr2 => Signal::SIGUSR2,
        }
    }

    /// Get the signal name (e.g., "SIGTERM").
    pub fn name(&self) -> &'static str {
        match self {
            KillSignal::Term => "SIGTERM",
            KillSignal::Kill => "SIGKILL",
            KillSignal::Hup => "SIGHUP",
            KillSignal::Int => "SIGINT",
            KillSignal::Usr1 => "SIGUSR1",
            KillSignal::Usr2 => "SIGUSR2",
        }
    }

    /// Get a human-readable description of the signal.
    pub fn description(&self) -> &'static str {
        match self {
            KillSignal::Term => "Graceful termination request",
            KillSignal::Kill => "Force kill (cannot be caught)",
            KillSignal::Hup => "Hangup / reload configuration",
            KillSignal::Int => "Interrupt (like Ctrl+C)",
            KillSignal::Usr1 => "User-defined (Node.js: activate debugger)",
            KillSignal::Usr2 => "User-defined signal",
        }
    }

    /// Return a slice of all available kill signals.
    pub fn all() -> &'static [KillSignal] {
        &[
            KillSignal::Term,
            KillSignal::Kill,
            KillSignal::Hup,
            KillSignal::Int,
            KillSignal::Usr1,
            KillSignal::Usr2,
        ]
    }

    /// Parse a signal name string (case-insensitive).
    /// Accepts both "SIGTERM" and "TERM" forms.
    pub fn from_str(s: &str) -> Option<Self> {
        let upper = s.to_uppercase();
        let normalized = upper.strip_prefix("SIG").unwrap_or(&upper);
        match normalized {
            "TERM" => Some(KillSignal::Term),
            "KILL" => Some(KillSignal::Kill),
            "HUP" => Some(KillSignal::Hup),
            "INT" => Some(KillSignal::Int),
            "USR1" => Some(KillSignal::Usr1),
            "USR2" => Some(KillSignal::Usr2),
            _ => None,
        }
    }
}

/// Result of attempting to send a signal to a process.
#[derive(Debug)]
pub enum KillResult {
    Success,
    AlreadyDead,
    PermissionDenied,
    Error(String),
}

/// Result of a graceful kill attempt (SIGTERM + polling).
#[derive(Debug)]
pub enum GracefulResult {
    Terminated,
    TimedOut,
    AlreadyDead,
    PermissionDenied,
    Error(String),
}

/// Provides static methods for sending signals and killing processes.
pub struct ProcessKiller;

impl ProcessKiller {
    /// Send a signal to a process by PID.
    pub fn send_signal(pid: u32, signal: KillSignal) -> KillResult {
        let nix_pid = Pid::from_raw(pid as i32);
        match signal::kill(nix_pid, signal.to_nix_signal()) {
            Ok(()) => KillResult::Success,
            Err(nix::errno::Errno::ESRCH) => KillResult::AlreadyDead,
            Err(nix::errno::Errno::EPERM) => KillResult::PermissionDenied,
            Err(e) => KillResult::Error(e.to_string()),
        }
    }

    /// Check whether a process is still alive.
    pub fn is_alive(pid: u32) -> bool {
        let nix_pid = Pid::from_raw(pid as i32);
        signal::kill(nix_pid, None).is_ok()
    }

    /// Attempt a graceful kill: send SIGTERM, then poll every 200ms until the
    /// process exits or the timeout expires.
    pub fn graceful_kill(pid: u32, timeout: Duration) -> GracefulResult {
        // First, send SIGTERM
        let nix_pid = Pid::from_raw(pid as i32);
        match signal::kill(nix_pid, Signal::SIGTERM) {
            Ok(()) => {}
            Err(nix::errno::Errno::ESRCH) => return GracefulResult::AlreadyDead,
            Err(nix::errno::Errno::EPERM) => return GracefulResult::PermissionDenied,
            Err(e) => return GracefulResult::Error(e.to_string()),
        }

        // Poll every 200ms until the process is dead or timeout is reached
        let poll_interval = Duration::from_millis(200);
        let mut elapsed = Duration::ZERO;

        while elapsed < timeout {
            thread::sleep(poll_interval);
            elapsed += poll_interval;

            if !Self::is_alive(pid) {
                return GracefulResult::Terminated;
            }
        }

        GracefulResult::TimedOut
    }

    /// Force-kill a process by sending SIGKILL.
    pub fn force_kill(pid: u32) -> KillResult {
        Self::send_signal(pid, KillSignal::Kill)
    }

    /// Send a signal to multiple PIDs in reverse order (children first).
    pub fn kill_tree(pids: &[u32], signal: KillSignal) -> Vec<(u32, KillResult)> {
        pids.iter()
            .rev()
            .map(|&pid| (pid, Self::send_signal(pid, signal)))
            .collect()
    }
}
