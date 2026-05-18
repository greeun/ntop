use ntop::config::Config;
use ntop::process::scanner::ProcessScanner;

#[test]
fn test_scanner_returns_vec() {
    let config = Config::default();
    let scanner = ProcessScanner::new(&config);
    let processes = scanner.scan();
    // scan() must return without panicking; every returned process must be valid
    for p in &processes {
        assert!(p.pid > 0);
        assert!(!p.name.is_empty());
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
