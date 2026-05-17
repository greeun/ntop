// Process termination and signal sending

use std::thread;
use std::time::Duration;

/// Represents the type of signal to send to a process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KillSignal {
    Term,
    Kill,
    #[cfg(unix)]
    Hup,
    Int,
    #[cfg(unix)]
    Usr1,
    #[cfg(unix)]
    Usr2,
}

impl KillSignal {
    /// Get the signal name (e.g., "SIGTERM").
    pub fn name(&self) -> &'static str {
        match self {
            KillSignal::Term => "SIGTERM",
            KillSignal::Kill => "SIGKILL",
            #[cfg(unix)]
            KillSignal::Hup => "SIGHUP",
            KillSignal::Int => "SIGINT",
            #[cfg(unix)]
            KillSignal::Usr1 => "SIGUSR1",
            #[cfg(unix)]
            KillSignal::Usr2 => "SIGUSR2",
        }
    }

    /// Get a human-readable description of the signal.
    pub fn description(&self) -> &'static str {
        match self {
            KillSignal::Term => "Graceful termination request",
            KillSignal::Kill => "Force kill (cannot be caught)",
            #[cfg(unix)]
            KillSignal::Hup => "Hangup / reload configuration",
            KillSignal::Int => "Interrupt (like Ctrl+C)",
            #[cfg(unix)]
            KillSignal::Usr1 => "User-defined (Node.js: activate debugger)",
            #[cfg(unix)]
            KillSignal::Usr2 => "User-defined signal",
        }
    }

    /// Return a slice of all available kill signals for the current platform.
    pub fn all() -> &'static [KillSignal] {
        #[cfg(unix)]
        {
            &[
                KillSignal::Term,
                KillSignal::Kill,
                KillSignal::Hup,
                KillSignal::Int,
                KillSignal::Usr1,
                KillSignal::Usr2,
            ]
        }
        #[cfg(windows)]
        {
            &[
                KillSignal::Term,
                KillSignal::Kill,
                KillSignal::Int,
            ]
        }
    }

    /// Parse a signal name string (case-insensitive).
    /// Accepts both "SIGTERM" and "TERM" forms.
    pub fn from_str(s: &str) -> Option<Self> {
        let upper = s.to_uppercase();
        let normalized = upper.strip_prefix("SIG").unwrap_or(&upper);
        match normalized {
            "TERM" => Some(KillSignal::Term),
            "KILL" => Some(KillSignal::Kill),
            #[cfg(unix)]
            "HUP" => Some(KillSignal::Hup),
            "INT" => Some(KillSignal::Int),
            #[cfg(unix)]
            "USR1" => Some(KillSignal::Usr1),
            #[cfg(unix)]
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

// ─── Unix implementation ──────────────────────────────────────────────

#[cfg(unix)]
mod unix_impl {
    use super::*;
    use nix::sys::signal::{self, Signal};
    use nix::unistd::Pid;

    impl KillSignal {
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
            let nix_pid = Pid::from_raw(pid as i32);
            match signal::kill(nix_pid, Signal::SIGTERM) {
                Ok(()) => {}
                Err(nix::errno::Errno::ESRCH) => return GracefulResult::AlreadyDead,
                Err(nix::errno::Errno::EPERM) => return GracefulResult::PermissionDenied,
                Err(e) => return GracefulResult::Error(e.to_string()),
            }

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

        pub fn force_kill(pid: u32) -> KillResult {
            Self::send_signal(pid, KillSignal::Kill)
        }

        pub fn kill_tree(pids: &[u32], signal: KillSignal) -> Vec<(u32, KillResult)> {
            pids.iter()
                .rev()
                .map(|&pid| (pid, Self::send_signal(pid, signal)))
                .collect()
        }
    }
}

// ─── Windows implementation ───────────────────────────────────────────

#[cfg(windows)]
mod windows_impl {
    use super::*;
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0};
    use windows_sys::Win32::System::Threading::{
        OpenProcess, TerminateProcess, WaitForSingleObject,
        PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE,
    };

    const SYNCHRONIZE: u32 = 0x00100000;

    fn open_process(pid: u32, access: u32) -> Option<HANDLE> {
        let handle = unsafe { OpenProcess(access, 0, pid) };
        if handle.is_null() {
            None
        } else {
            Some(handle)
        }
    }

    impl ProcessKiller {
        pub fn send_signal(pid: u32, signal: KillSignal) -> KillResult {
            match signal {
                KillSignal::Term | KillSignal::Int => {
                    // On Windows, graceful termination uses GenerateConsoleCtrlEvent
                    // which only works for console processes in the same group.
                    // Fall back to taskkill for broader compatibility.
                    let output = std::process::Command::new("taskkill")
                        .args(["/PID", &pid.to_string()])
                        .output();
                    match output {
                        Ok(o) if o.status.success() => KillResult::Success,
                        Ok(o) => {
                            let stderr = String::from_utf8_lossy(&o.stderr);
                            if stderr.contains("not found") {
                                KillResult::AlreadyDead
                            } else if stderr.contains("Access") {
                                KillResult::PermissionDenied
                            } else {
                                KillResult::Error(stderr.to_string())
                            }
                        }
                        Err(e) => KillResult::Error(e.to_string()),
                    }
                }
                KillSignal::Kill => {
                    let handle = match open_process(pid, PROCESS_TERMINATE) {
                        Some(h) => h,
                        None => {
                            let err = std::io::Error::last_os_error();
                            return match err.raw_os_error() {
                                Some(5) => KillResult::PermissionDenied,
                                Some(87) => KillResult::AlreadyDead,
                                _ => KillResult::Error(err.to_string()),
                            };
                        }
                    };
                    let result = unsafe { TerminateProcess(handle, 1) };
                    unsafe { CloseHandle(handle) };
                    if result != 0 {
                        KillResult::Success
                    } else {
                        let err = std::io::Error::last_os_error();
                        KillResult::Error(err.to_string())
                    }
                }
            }
        }

        pub fn is_alive(pid: u32) -> bool {
            let handle = match open_process(pid, PROCESS_QUERY_INFORMATION | SYNCHRONIZE) {
                Some(h) => h,
                None => return false,
            };
            let result = unsafe { WaitForSingleObject(handle, 0) };
            unsafe { CloseHandle(handle) };
            result != WAIT_OBJECT_0
        }

        pub fn graceful_kill(pid: u32, timeout: Duration) -> GracefulResult {
            match Self::send_signal(pid, KillSignal::Term) {
                KillResult::Success => {}
                KillResult::AlreadyDead => return GracefulResult::AlreadyDead,
                KillResult::PermissionDenied => return GracefulResult::PermissionDenied,
                KillResult::Error(e) => return GracefulResult::Error(e),
            }

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

        pub fn force_kill(pid: u32) -> KillResult {
            Self::send_signal(pid, KillSignal::Kill)
        }

        pub fn kill_tree(pids: &[u32], signal: KillSignal) -> Vec<(u32, KillResult)> {
            pids.iter()
                .rev()
                .map(|&pid| (pid, Self::send_signal(pid, signal)))
                .collect()
        }
    }
}
