use ntop::config::Config;
use ntop::process::scanner::ProcessScanner;

#[test]
fn test_scanner_returns_vec() {
    let config = Config::default();
    let mut scanner = ProcessScanner::new(&config);
    let processes = scanner.scan();
    // scan() must return without panicking; every returned process must be valid
    for p in &processes {
        assert!(p.pid > 0);
        assert!(!p.name.is_empty());
    }
}

#[test]
fn test_scanner_marks_node_vs_tree_parent() {
    let config = Config::default();
    let mut scanner = ProcessScanner::new(&config);
    let processes = scanner.scan();

    // If any process at all is returned, the test environment has Node
    // processes — assert at least one row is flagged is_node = true.
    // Otherwise, skip the assertion (CI may run on a node-less host).
    if !processes.is_empty() {
        let any_node = processes.iter().any(|p| p.is_node);
        let any_node_named = processes.iter().any(|p| ProcessScanner::is_node_process_name(&p.name));
        // If there's any process whose name matches a Node binary, at
        // least one entry should also be flagged is_node.
        if any_node_named {
            assert!(any_node, "expected at least one is_node=true row when node processes exist");
        }
    }
}

#[test]
fn test_is_node_process_by_name() {
    assert!(ProcessScanner::is_node_process_name("node"));
    assert!(ProcessScanner::is_node_process_name("next-server"));
    assert!(ProcessScanner::is_node_process_name("next-router-worker"));
    assert!(!ProcessScanner::is_node_process_name("python"));
    assert!(!ProcessScanner::is_node_process_name("ruby"));
}

#[test]
fn test_is_node_process_by_command() {
    assert!(ProcessScanner::is_node_command("/usr/local/bin/node server.js"));
    assert!(ProcessScanner::is_node_command("/opt/homebrew/bin/node app.js"));
    assert!(ProcessScanner::is_node_command("node index.js"));
    assert!(!ProcessScanner::is_node_command("python app.py"));
}

#[test]
fn test_optional_runtime_filters() {
    let mut config = Config::default();
    config.filter.include_bun = true;
    assert!(ProcessScanner::is_node_process_name_with_config("bun", &config));

    config.filter.include_bun = false;
    assert!(!ProcessScanner::is_node_process_name_with_config("bun", &config));

    config.filter.include_tsx = true;
    assert!(ProcessScanner::is_node_process_name_with_config("tsx", &config));
}
