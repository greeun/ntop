use ntop::process::network::NetworkInspector;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

// ─── parse_addr ───────────────────────────────────────────────────────

#[test]
fn test_parse_addr_wildcard() {
    let addr = NetworkInspector::parse_addr("*:3000").unwrap();
    assert_eq!(addr.port(), 3000);
    assert_eq!(addr.ip(), IpAddr::V4(Ipv4Addr::UNSPECIFIED));
}

#[test]
fn test_parse_addr_ipv4_loopback() {
    let addr = NetworkInspector::parse_addr("127.0.0.1:5000").unwrap();
    assert_eq!(addr.port(), 5000);
    assert_eq!(addr.ip(), IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
}

#[test]
fn test_parse_addr_ipv4_unspecified() {
    let addr = NetworkInspector::parse_addr("0.0.0.0:8080").unwrap();
    assert_eq!(addr.port(), 8080);
    assert_eq!(addr.ip(), IpAddr::V4(Ipv4Addr::UNSPECIFIED));
}

#[test]
fn test_parse_addr_ipv6_loopback() {
    let addr = NetworkInspector::parse_addr("[::1]:3000").unwrap();
    assert_eq!(addr.port(), 3000);
    assert_eq!(addr.ip(), IpAddr::V6(Ipv6Addr::LOCALHOST));
}

#[test]
fn test_parse_addr_ipv6_unspecified() {
    let addr = NetworkInspector::parse_addr("[::]:8080").unwrap();
    assert_eq!(addr.port(), 8080);
    assert_eq!(addr.ip(), IpAddr::V6(Ipv6Addr::UNSPECIFIED));
}

#[test]
fn test_parse_addr_invalid_returns_none() {
    assert!(NetworkInspector::parse_addr("notanaddress").is_none());
    assert!(NetworkInspector::parse_addr("").is_none());
    assert!(NetworkInspector::parse_addr("127.0.0.1").is_none()); // no port
}

#[test]
fn test_parse_addr_invalid_port_returns_none() {
    assert!(NetworkInspector::parse_addr("*:99999").is_none()); // out of u16 range
    assert!(NetworkInspector::parse_addr("*:notaport").is_none());
    assert!(NetworkInspector::parse_addr("[::1]:notaport").is_none());
}

// ─── parse_connection (unix only) ────────────────────────────────────

#[cfg(unix)]
#[test]
fn test_parse_connection_listen() {
    let conn =
        NetworkInspector::parse_connection("127.0.0.1:3000", Some("LISTEN"), 42).unwrap();
    assert_eq!(conn.pid, 42);
    assert_eq!(conn.state, "LISTEN");
    assert_eq!(conn.local_addr.port(), 3000);
    assert!(conn.remote_addr.is_none());
}

#[cfg(unix)]
#[test]
fn test_parse_connection_established_with_remote() {
    let conn = NetworkInspector::parse_connection(
        "127.0.0.1:3000->10.0.0.1:50123",
        Some("ESTABLISHED"),
        99,
    )
    .unwrap();
    assert_eq!(conn.pid, 99);
    assert_eq!(conn.state, "ESTABLISHED");
    assert_eq!(conn.local_addr.port(), 3000);
    let remote = conn.remote_addr.unwrap();
    assert_eq!(remote.port(), 50123);
}

#[cfg(unix)]
#[test]
fn test_parse_connection_wildcard_listen() {
    let conn = NetworkInspector::parse_connection("*:8080", Some("LISTEN"), 10).unwrap();
    assert_eq!(conn.local_addr.port(), 8080);
    assert_eq!(conn.local_addr.ip(), IpAddr::V4(Ipv4Addr::UNSPECIFIED));
}

#[cfg(unix)]
#[test]
fn test_parse_connection_none_state_defaults_to_unknown() {
    let conn = NetworkInspector::parse_connection("*:8080", None, 10).unwrap();
    assert_eq!(conn.state, "UNKNOWN");
}

#[cfg(unix)]
#[test]
fn test_parse_connection_invalid_addr_returns_none() {
    let result = NetworkInspector::parse_connection("notanaddr", Some("LISTEN"), 1);
    assert!(result.is_none());
}

#[cfg(unix)]
#[test]
fn test_parse_connection_ipv6_listen() {
    let conn =
        NetworkInspector::parse_connection("[::]:3000", Some("LISTEN"), 7).unwrap();
    assert_eq!(conn.local_addr.port(), 3000);
    assert_eq!(conn.local_addr.ip(), IpAddr::V6(Ipv6Addr::UNSPECIFIED));
    assert!(conn.remote_addr.is_none());
}
