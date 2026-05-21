use ntop::process::{FrameworkKind, ProcessInfo};
use ntop::tui::app::App;

#[test]
fn test_matches_filter_empty_matches_all() {
    let p = ProcessInfo::new(1, "anything");
    assert!(App::matches_filter(&p, ""));
}

#[test]
fn test_matches_filter_name_case_insensitive() {
    let p = ProcessInfo::new(1, "Node");
    assert!(App::matches_filter(&p, "node"));
    assert!(App::matches_filter(&p, "NODE"));
    assert!(App::matches_filter(&p, "NoDe"));
}

#[test]
fn test_matches_filter_command_case_insensitive() {
    let mut p = ProcessInfo::new(1, "node");
    p.command = "node /Path/To/Next-Server".to_string();
    assert!(App::matches_filter(&p, "next-server"));
    assert!(App::matches_filter(&p, "NEXT-SERVER"));
    assert!(App::matches_filter(&p, "path"));
    assert!(App::matches_filter(&p, "PATH"));
    assert!(!App::matches_filter(&p, "express"));
}

#[test]
fn test_matches_filter_framework_case_insensitive() {
    let mut p = ProcessInfo::new(1, "node");
    p.framework = FrameworkKind::NextJs;
    assert!(App::matches_filter(&p, "next.js"));
    assert!(App::matches_filter(&p, "NEXT.JS"));
    assert!(App::matches_filter(&p, "Next.JS"));
}

#[test]
fn test_matches_filter_pid_substring() {
    let p = ProcessInfo::new(12345, "node");
    assert!(App::matches_filter(&p, "1234"));
    assert!(App::matches_filter(&p, "12345"));
    assert!(!App::matches_filter(&p, "99999"));
}

#[test]
fn test_matches_filter_ports_substring() {
    let mut p = ProcessInfo::new(1, "node");
    p.ports = vec![3000, 8080];
    assert!(App::matches_filter(&p, "3000"));
    assert!(App::matches_filter(&p, "8080"));
    assert!(App::matches_filter(&p, "300"));
    assert!(!App::matches_filter(&p, "9999"));
}

#[test]
fn test_matches_filter_no_match() {
    let mut p = ProcessInfo::new(1, "node");
    p.command = "node server.js".to_string();
    p.framework = FrameworkKind::Generic;
    assert!(!App::matches_filter(&p, "next"));
    assert!(!App::matches_filter(&p, "nuxt"));
    assert!(!App::matches_filter(&p, "9999"));
}
