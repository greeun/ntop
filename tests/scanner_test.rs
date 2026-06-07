use ntop::config::Config;
use ntop::process::framework::FrameworkDetector;
use ntop::process::scanner::ProcessScanner;
use ntop::process::Runtime;

#[test]
fn test_scanner_returns_vec() {
    let config = Config::default();
    let mut scanner = ProcessScanner::new(&config);
    let processes = scanner.scan();
    for p in &processes {
        assert!(p.pid > 0);
        assert!(!p.name.is_empty());
    }
}

#[test]
fn test_scanner_runtime_classification_is_consistent() {
    let config = Config::default();
    let mut scanner = ProcessScanner::new(&config);
    let processes = scanner.scan();

    for p in &processes {
        // is_server() and is_node() must agree with the runtime field.
        assert_eq!(p.is_server(), p.runtime.is_some());
        assert_eq!(p.is_node(), p.runtime == Some(Runtime::Node));
        // A classified server's runtime must match what classify() returns
        // for its own name/command (servers only — tree parents are None).
        if let Some(rt) = p.runtime {
            let classified = FrameworkDetector::classify(&p.name, &p.command, &config);
            assert_eq!(classified.map(|(r, _)| r), Some(rt));
        }
    }
}
