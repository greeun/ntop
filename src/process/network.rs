// Network/port detection for processes

use super::ProcessInfo;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::process::Command;

/// Represents a single TCP network connection associated with a process.
#[derive(Debug, Clone)]
pub struct NetworkConnection {
    pub local_addr: SocketAddr,
    pub remote_addr: Option<SocketAddr>,
    pub state: String,
    pub pid: u32,
}

/// Inspects network connections by parsing system command output to map PIDs to
/// their listening ports and active TCP connections.
pub struct NetworkInspector;

impl NetworkInspector {
    /// Get all TCP connections for a specific PID.
    pub fn connections_for_pid(pid: u32) -> Vec<NetworkConnection> {
        let all = Self::scan_connections();
        all.get(&pid).cloned().unwrap_or_default()
    }

    /// Extract the LISTEN-state ports from a slice of connections.
    pub fn listening_ports(conns: &[NetworkConnection]) -> Vec<u16> {
        conns
            .iter()
            .filter(|c| c.state == "LISTEN")
            .map(|c| c.local_addr.port())
            .collect()
    }

    /// Get only the LISTEN ports for a specific PID.
    pub fn listening_ports_for_pid(pid: u32) -> Vec<u16> {
        Self::listening_ports(&Self::connections_for_pid(pid))
    }

    /// Fill each process's `ports` with its LISTEN ports, looked up in a single
    /// `connections_by_pid()` scan. Only overwrites when ports are found, so a
    /// process with no listening sockets keeps its existing (empty) list.
    pub fn fill_listening_ports(procs: &mut [ProcessInfo]) {
        let net_map = Self::connections_by_pid();
        for proc in procs {
            if let Some(conns) = net_map.get(&proc.pid) {
                let ports = Self::listening_ports(conns);
                if !ports.is_empty() {
                    proc.ports = ports;
                }
            }
        }
    }

    /// Scan all TCP connections (LISTEN + ESTABLISHED) grouped by PID.
    pub fn connections_by_pid() -> HashMap<u32, Vec<NetworkConnection>> {
        Self::scan_connections()
    }

    #[cfg(unix)]
    fn scan_connections() -> HashMap<u32, Vec<NetworkConnection>> {
        Self::parse_lsof()
    }

    #[cfg(windows)]
    fn scan_connections() -> HashMap<u32, Vec<NetworkConnection>> {
        Self::parse_netstat()
    }

    /// Parse an address string into a `SocketAddr`.
    ///
    /// Handles the following forms:
    /// - `"*:port"` -> `0.0.0.0:port`
    /// - `"127.0.0.1:port"` -> IPv4 as-is
    /// - `"[::1]:port"` -> IPv6 as-is
    /// - `"[::]:port"` -> IPv6 unspecified
    pub fn parse_addr(s: &str) -> Option<SocketAddr> {
        let s = s.trim();

        // Handle IPv6 bracket notation: [addr]:port
        if s.starts_with('[') {
            let close_bracket = s.find(']')?;
            let ip_str = &s[1..close_bracket];
            let rest = &s[close_bracket + 1..];
            let port_str = rest.strip_prefix(':')?;
            let port: u16 = port_str.parse().ok()?;
            let ip: Ipv6Addr = ip_str.parse().ok()?;
            return Some(SocketAddr::new(IpAddr::V6(ip), port));
        }

        // Handle wildcard: *:port
        if let Some(port_str) = s.strip_prefix("*:") {
            let port: u16 = port_str.parse().ok()?;
            return Some(SocketAddr::new(
                IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                port,
            ));
        }

        // Handle plain IPv4: host:port
        let last_colon = s.rfind(':')?;
        let host_str = &s[..last_colon];
        let port_str = &s[last_colon + 1..];
        let port: u16 = port_str.parse().ok()?;
        let ip: Ipv4Addr = host_str.parse().ok()?;
        Some(SocketAddr::new(IpAddr::V4(ip), port))
    }
}

// ─── Unix: lsof-based implementation ─────────────────────────────────

#[cfg(unix)]
impl NetworkInspector {
    /// Run `lsof -iTCP -sTCP:LISTEN,ESTABLISHED -nP -F pcnT` and parse the
    /// field-mode output into a map of PID -> connections.
    fn parse_lsof() -> HashMap<u32, Vec<NetworkConnection>> {
        let output = Command::new("lsof")
            .args(["-iTCP", "-sTCP:LISTEN,ESTABLISHED", "-nP", "-F", "pcnT"])
            .output();

        let output = match output {
            Ok(o) => o,
            Err(_) => return HashMap::new(),
        };

        if !output.status.success() {
            return HashMap::new();
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        let mut result: HashMap<u32, Vec<NetworkConnection>> = HashMap::new();
        let mut current_pid: Option<u32> = None;
        let mut current_name: Option<String> = None;
        let mut current_state: Option<String> = None;

        for line in stdout.lines() {
            if line.is_empty() {
                continue;
            }

            if let Some(pid_str) = line.strip_prefix('p') {
                if let (Some(pid), Some(ref name)) = (current_pid, &current_name) {
                    if let Some(conn) =
                        Self::parse_connection(name, current_state.as_deref(), pid)
                    {
                        result.entry(pid).or_default().push(conn);
                    }
                }
                current_name = None;
                current_state = None;

                current_pid = pid_str.parse::<u32>().ok();
            } else if line.starts_with('c') {
                // Command name line -- ignored
            } else if let Some(name_str) = line.strip_prefix('n') {
                if let (Some(pid), Some(ref prev_name)) = (current_pid, &current_name) {
                    if let Some(conn) =
                        Self::parse_connection(prev_name, current_state.as_deref(), pid)
                    {
                        result.entry(pid).or_default().push(conn);
                    }
                }
                current_name = Some(name_str.to_string());
                current_state = None;
            } else if let Some(state_str) = line.strip_prefix("TST=") {
                current_state = Some(state_str.to_string());
            }
        }

        if let (Some(pid), Some(ref name)) = (current_pid, &current_name) {
            if let Some(conn) = Self::parse_connection(name, current_state.as_deref(), pid) {
                result.entry(pid).or_default().push(conn);
            }
        }

        result
    }

    /// Parse a single connection entry from the lsof name field.
    pub fn parse_connection(
        name: &str,
        state: Option<&str>,
        pid: u32,
    ) -> Option<NetworkConnection> {
        let state_str = state.unwrap_or("UNKNOWN").to_string();

        let (local_part, remote_part) = if let Some(idx) = name.find("->") {
            let (l, r) = name.split_at(idx);
            (l, Some(&r[2..]))
        } else {
            (name, None)
        };

        let local_addr = Self::parse_addr(local_part)?;
        let remote_addr = remote_part.and_then(Self::parse_addr);

        Some(NetworkConnection {
            local_addr,
            remote_addr,
            state: state_str,
            pid,
        })
    }
}

// ─── Windows: netstat-based implementation ────────────────────────────

#[cfg(windows)]
impl NetworkInspector {
    /// Run `netstat -ano` and parse TCP connections into a map of PID -> connections.
    fn parse_netstat() -> HashMap<u32, Vec<NetworkConnection>> {
        let output = Command::new("netstat")
            .args(["-ano", "-p", "TCP"])
            .output();

        let output = match output {
            Ok(o) => o,
            Err(_) => return HashMap::new(),
        };

        if !output.status.success() {
            return HashMap::new();
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut result: HashMap<u32, Vec<NetworkConnection>> = HashMap::new();

        for line in stdout.lines() {
            let line = line.trim();
            if !line.starts_with("TCP") {
                continue;
            }

            // Format: TCP    local_addr    remote_addr    state    pid
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 5 {
                continue;
            }

            let local_str = parts[1];
            let remote_str = parts[2];
            let state = parts[3];
            let pid_str = parts[4];

            let pid: u32 = match pid_str.parse() {
                Ok(p) => p,
                Err(_) => continue,
            };

            let local_addr = match Self::parse_netstat_addr(local_str) {
                Some(a) => a,
                None => continue,
            };

            let remote_addr = Self::parse_netstat_addr(remote_str);

            let normalized_state = match state {
                "LISTENING" => "LISTEN".to_string(),
                other => other.to_string(),
            };

            let conn = NetworkConnection {
                local_addr,
                remote_addr,
                state: normalized_state,
                pid,
            };

            result.entry(pid).or_default().push(conn);
        }

        result
    }

    /// Parse a Windows netstat address (e.g., "0.0.0.0:3000", "[::]:3000", "127.0.0.1:5000").
    fn parse_netstat_addr(s: &str) -> Option<SocketAddr> {
        let s = s.trim();

        // IPv6: [::]:port or [::1]:port
        if s.starts_with('[') {
            return Self::parse_addr(s);
        }

        // IPv4: host:port — but netstat uses last ':' as separator
        let last_colon = s.rfind(':')?;
        let host_str = &s[..last_colon];
        let port_str = &s[last_colon + 1..];
        let port: u16 = port_str.parse().ok()?;

        if host_str == "0.0.0.0" || host_str == "*" {
            return Some(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port));
        }

        let ip: Ipv4Addr = host_str.parse().ok()?;
        Some(SocketAddr::new(IpAddr::V4(ip), port))
    }

    pub fn parse_connection(
        name: &str,
        state: Option<&str>,
        pid: u32,
    ) -> Option<NetworkConnection> {
        let state_str = state.unwrap_or("UNKNOWN").to_string();

        let (local_part, remote_part) = if let Some(idx) = name.find("->") {
            let (l, r) = name.split_at(idx);
            (l, Some(&r[2..]))
        } else {
            (name, None)
        };

        let local_addr = Self::parse_addr(local_part)?;
        let remote_addr = remote_part.and_then(Self::parse_addr);

        Some(NetworkConnection {
            local_addr,
            remote_addr,
            state: state_str,
            pid,
        })
    }
}
