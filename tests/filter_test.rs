use ntop::process::{FrameworkKind, ProcessInfo, Runtime};
use ntop::tui::app::App;
use ntop::config::Config;

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

fn server(pid: u32, name: &str, runtime: Option<Runtime>) -> ProcessInfo {
    let mut p = ProcessInfo::new(pid, name);
    p.runtime = runtime;
    p
}

#[test]
fn test_matches_filter_runtime() {
    let mut p = ProcessInfo::new(1, "uvicorn");
    p.runtime = Some(Runtime::Python);
    assert!(App::matches_filter(&p, "python"));
    assert!(App::matches_filter(&p, "PYTHON"));
    assert!(!App::matches_filter(&p, "java"));
}

#[test]
fn test_node_only_toggle_hides_other_runtimes() {
    let mut app = App::new(Config::default());
    app.update_processes(vec![
        server(1, "node", Some(Runtime::Node)),
        server(2, "uvicorn", Some(Runtime::Python)),
        server(3, "java", Some(Runtime::Java)),
        server(4, "launchd", None), // tree-context parent
    ]);

    // Default: all rows visible.
    assert_eq!(app.flat_list.len(), 4);

    // Node-only: Node servers + tree parents remain; Python/Java hidden.
    app.toggle_node_only();
    assert!(app.node_only);
    let pids: Vec<u32> = app.flat_list.iter().map(|(p, _)| p.pid).collect();
    assert!(pids.contains(&1)); // node
    assert!(pids.contains(&4)); // tree parent
    assert!(!pids.contains(&2)); // python hidden
    assert!(!pids.contains(&3)); // java hidden

    // Toggling off restores everything.
    app.toggle_node_only();
    assert!(!app.node_only);
    assert_eq!(app.flat_list.len(), 4);
}
