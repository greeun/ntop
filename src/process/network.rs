// Network/port detection for processes

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

/// Inspects network connections by parsing `lsof` output to map PIDs to their
/// listening ports and active TCP connections.
pub struct NetworkInspector;

impl NetworkInspector {
    /// Get all TCP connections for a specific PID.
    pub fn connections_for_pid(pid: u32) -> Vec<NetworkConnection> {
        let all = Self::parse_lsof();
        all.get(&pid).cloned().unwrap_or_default()
    }

    /// Get only the LISTEN ports for a specific PID.
    pub fn listening_ports_for_pid(pid: u32) -> Vec<u16> {
        Self::connections_for_pid(pid)
            .iter()
            .filter(|c| c.state == "LISTEN")
            .map(|c| c.local_addr.port())
            .collect()
    }

    /// Scan all TCP connections (LISTEN + ESTABLISHED) grouped by PID.
    pub fn connections_by_pid() -> HashMap<u32, Vec<NetworkConnection>> {
        Self::parse_lsof()
    }

    /// Run `lsof -iTCP -sTCP:LISTEN,ESTABLISHED -nP -F pcnT` and parse the
    /// field-mode output into a map of PID -> connections.
    ///
    /// lsof `-F pcnT` field output format:
    /// - Lines starting with 'p' = PID (process set marker)
    /// - Lines starting with 'c' = command name (ignored here)
    /// - Lines starting with 'n' = connection name, e.g. "127.0.0.1:3000" or
    ///   "127.0.0.1:3000->192.168.1.1:5000"
    /// - Lines starting with "TST=" = TCP state (e.g. "TST=LISTEN")
    pub fn parse_lsof() -> HashMap<u32, Vec<NetworkConnection>> {
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
                // Flush any pending connection from the previous file descriptor
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
                // A new file-descriptor / connection entry starts with 'n'.
                // First, flush any previously accumulated connection for the
                // same PID (there can be multiple 'n' lines per process).
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

        // Flush the last accumulated connection
        if let (Some(pid), Some(ref name)) = (current_pid, &current_name) {
            if let Some(conn) = Self::parse_connection(name, current_state.as_deref(), pid) {
                result.entry(pid).or_default().push(conn);
            }
        }

        result
    }

    /// Parse a single connection entry from the lsof name field.
    ///
    /// The `name` field can look like:
    /// - `"127.0.0.1:3000"` (listening socket)
    /// - `"*:3000"` (listening on all interfaces)
    /// - `"[::1]:3000"` (IPv6 loopback)
    /// - `"[::]:3000"` (IPv6 wildcard)
    /// - `"127.0.0.1:3000->192.168.1.1:5000"` (established connection)
    pub fn parse_connection(
        name: &str,
        state: Option<&str>,
        pid: u32,
    ) -> Option<NetworkConnection> {
        let state_str = state.unwrap_or("UNKNOWN").to_string();

        // Split on "->" to separate local from remote
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

    /// Parse an address string from lsof into a `SocketAddr`.
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
            // Find the closing bracket
            let close_bracket = s.find(']')?;
            let ip_str = &s[1..close_bracket];
            // After the closing bracket, expect ":port"
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
        // Find the last colon to split host and port (handles e.g. "127.0.0.1:3000")
        let last_colon = s.rfind(':')?;
        let host_str = &s[..last_colon];
        let port_str = &s[last_colon + 1..];
        let port: u16 = port_str.parse().ok()?;
        let ip: Ipv4Addr = host_str.parse().ok()?;
        Some(SocketAddr::new(IpAddr::V4(ip), port))
    }
}
