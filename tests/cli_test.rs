use clap::Parser;
use ntop::cli::{Cli, Commands, ListFormat};

#[test]
fn test_default_launches_tui() {
    let cli = Cli::parse_from(["ntop"]);
    assert!(cli.command.is_none());
}

#[test]
fn test_list_command() {
    let cli = Cli::parse_from(["ntop", "list"]);
    assert!(matches!(cli.command, Some(Commands::List { .. })));
}

#[test]
fn test_list_json() {
    let cli = Cli::parse_from(["ntop", "list", "--json"]);
    match cli.command {
        Some(Commands::List { json, .. }) => assert!(json),
        _ => panic!("Expected Commands::List"),
    }
}

#[test]
fn test_list_format_csv() {
    let cli = Cli::parse_from(["ntop", "list", "--format", "csv"]);
    match cli.command {
        Some(Commands::List { format, .. }) => assert_eq!(format, Some(ListFormat::Csv)),
        _ => panic!("Expected Commands::List"),
    }
}

#[test]
fn test_kill_command() {
    let cli = Cli::parse_from(["ntop", "kill", "1234"]);
    match cli.command {
        Some(Commands::Kill { pid, .. }) => assert_eq!(pid, Some(1234)),
        _ => panic!("Expected Commands::Kill"),
    }
}

#[test]
fn test_kill_with_tree() {
    let cli = Cli::parse_from(["ntop", "kill", "--tree", "1234"]);
    match cli.command {
        Some(Commands::Kill { tree, pid, .. }) => {
            assert!(tree);
            assert_eq!(pid, Some(1234));
        }
        _ => panic!("Expected Commands::Kill"),
    }
}

#[test]
fn test_kill_with_signal() {
    let cli = Cli::parse_from(["ntop", "kill", "--signal", "SIGKILL", "1234"]);
    match cli.command {
        Some(Commands::Kill { signal, .. }) => assert_eq!(signal, Some("SIGKILL".to_string())),
        _ => panic!("Expected Commands::Kill"),
    }
}

#[test]
fn test_kill_all() {
    let cli = Cli::parse_from(["ntop", "kill", "--all"]);
    match cli.command {
        Some(Commands::Kill { all, .. }) => assert!(all),
        _ => panic!("Expected Commands::Kill"),
    }
}

#[test]
fn test_info_command() {
    let cli = Cli::parse_from(["ntop", "info", "5678"]);
    match cli.command {
        Some(Commands::Info { pid }) => assert_eq!(pid, 5678),
        _ => panic!("Expected Commands::Info"),
    }
}

#[test]
fn test_log_command() {
    let cli = Cli::parse_from(["ntop", "log", "9999"]);
    match cli.command {
        Some(Commands::Log { pid }) => assert_eq!(pid, 9999),
        _ => panic!("Expected Commands::Log"),
    }
}

#[test]
fn test_no_confirm_flag() {
    let cli = Cli::parse_from(["ntop", "kill", "--no-confirm", "1234"]);
    match cli.command {
        Some(Commands::Kill { no_confirm, .. }) => assert!(no_confirm),
        _ => panic!("Expected Commands::Kill"),
    }
}
