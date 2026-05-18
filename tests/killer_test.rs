use ntop::process::killer::KillSignal;

#[test]
fn test_kill_signal_name() {
    assert_eq!(KillSignal::Term.name(), "SIGTERM");
    assert_eq!(KillSignal::Kill.name(), "SIGKILL");
    assert_eq!(KillSignal::Int.name(), "SIGINT");
    #[cfg(unix)]
    {
        assert_eq!(KillSignal::Hup.name(), "SIGHUP");
        assert_eq!(KillSignal::Usr1.name(), "SIGUSR1");
        assert_eq!(KillSignal::Usr2.name(), "SIGUSR2");
    }
}

#[test]
fn test_kill_signal_description_non_empty() {
    for sig in KillSignal::all() {
        assert!(!sig.description().is_empty(), "{:?} has empty description", sig);
    }
}

#[test]
fn test_kill_signal_all_no_duplicates() {
    let all = KillSignal::all();
    let names: Vec<&str> = all.iter().map(|s| s.name()).collect();
    let unique: std::collections::HashSet<&str> = names.iter().copied().collect();
    assert_eq!(names.len(), unique.len(), "KillSignal::all() contains duplicates");
}

#[test]
fn test_kill_signal_from_str_with_sig_prefix() {
    assert!(matches!(KillSignal::from_str("SIGTERM"), Some(KillSignal::Term)));
    assert!(matches!(KillSignal::from_str("SIGKILL"), Some(KillSignal::Kill)));
    assert!(matches!(KillSignal::from_str("SIGINT"), Some(KillSignal::Int)));
    #[cfg(unix)]
    {
        assert!(matches!(KillSignal::from_str("SIGHUP"), Some(KillSignal::Hup)));
        assert!(matches!(KillSignal::from_str("SIGUSR1"), Some(KillSignal::Usr1)));
        assert!(matches!(KillSignal::from_str("SIGUSR2"), Some(KillSignal::Usr2)));
    }
}

#[test]
fn test_kill_signal_from_str_without_prefix() {
    assert!(matches!(KillSignal::from_str("TERM"), Some(KillSignal::Term)));
    assert!(matches!(KillSignal::from_str("KILL"), Some(KillSignal::Kill)));
    assert!(matches!(KillSignal::from_str("INT"), Some(KillSignal::Int)));
    #[cfg(unix)]
    {
        assert!(matches!(KillSignal::from_str("HUP"), Some(KillSignal::Hup)));
        assert!(matches!(KillSignal::from_str("USR1"), Some(KillSignal::Usr1)));
        assert!(matches!(KillSignal::from_str("USR2"), Some(KillSignal::Usr2)));
    }
}

#[test]
fn test_kill_signal_from_str_case_insensitive() {
    assert!(matches!(KillSignal::from_str("sigterm"), Some(KillSignal::Term)));
    assert!(matches!(KillSignal::from_str("sigkill"), Some(KillSignal::Kill)));
    assert!(matches!(KillSignal::from_str("term"), Some(KillSignal::Term)));
    assert!(matches!(KillSignal::from_str("kill"), Some(KillSignal::Kill)));
    assert!(matches!(KillSignal::from_str("SiGtErM"), Some(KillSignal::Term)));
}

#[test]
fn test_kill_signal_from_str_unknown_returns_none() {
    assert!(KillSignal::from_str("SIGFOO").is_none());
    assert!(KillSignal::from_str("").is_none());
    assert!(KillSignal::from_str("SIGALRM").is_none());
    assert!(KillSignal::from_str("9").is_none());
    assert!(KillSignal::from_str("SIG").is_none());
}
