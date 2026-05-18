use ntop::log::streamer::LogStreamer;
use std::fs;
use std::io::Write;
use tempfile::TempDir;

#[test]
fn test_new_streamer_has_no_source() {
    let streamer = LogStreamer::new();
    assert!(!streamer.has_source());
    assert!(streamer.source_path().is_none());
    assert!(streamer.buffer().is_empty());
}

#[test]
fn test_find_log_file_returns_none_for_empty_dir() {
    let dir = TempDir::new().unwrap();
    let result = LogStreamer::find_log_file(dir.path().to_str().unwrap());
    assert!(result.is_none());
}

#[test]
fn test_find_log_file_finds_dot_log_in_dir() {
    let dir = TempDir::new().unwrap();
    let log_path = dir.path().join("app.log");
    fs::write(&log_path, "log content").unwrap();

    let found = LogStreamer::find_log_file(dir.path().to_str().unwrap());
    assert!(found.is_some());
    assert_eq!(found.unwrap(), log_path);
}

#[test]
fn test_find_log_file_finds_log_in_logs_subdir() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join("logs")).unwrap();
    let log_path = dir.path().join("logs").join("server.log");
    fs::write(&log_path, "server log").unwrap();

    let found = LogStreamer::find_log_file(dir.path().to_str().unwrap());
    assert!(found.is_some());
    assert_eq!(found.unwrap(), log_path);
}

#[test]
fn test_find_log_file_ignores_non_log_files() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("README.md"), "docs").unwrap();
    fs::write(dir.path().join("index.js"), "code").unwrap();

    let found = LogStreamer::find_log_file(dir.path().to_str().unwrap());
    assert!(found.is_none());
}

#[test]
fn test_find_log_file_picks_most_recently_modified() {
    let dir = TempDir::new().unwrap();
    let old_path = dir.path().join("old.log");
    let new_path = dir.path().join("new.log");

    fs::write(&old_path, "old").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(15));
    fs::write(&new_path, "new").unwrap();

    let found = LogStreamer::find_log_file(dir.path().to_str().unwrap()).unwrap();
    assert_eq!(found, new_path);
}

#[test]
fn test_detect_and_open_opens_log_file() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("app.log"), "existing content").unwrap();

    let streamer = LogStreamer::detect_and_open(dir.path().to_str().unwrap());
    assert!(streamer.has_source());
    assert!(streamer.source_path().is_some());
}

#[test]
fn test_detect_and_open_no_log_file() {
    let dir = TempDir::new().unwrap();
    let streamer = LogStreamer::detect_and_open(dir.path().to_str().unwrap());
    assert!(!streamer.has_source());
}

#[test]
fn test_poll_new_lines_reads_appended_content() {
    let dir = TempDir::new().unwrap();
    let log_path = dir.path().join("app.log");
    fs::write(&log_path, "").unwrap();

    let mut streamer = LogStreamer::detect_and_open(dir.path().to_str().unwrap());
    assert!(streamer.has_source());

    let mut file = fs::OpenOptions::new().append(true).open(&log_path).unwrap();
    writeln!(file, "line one").unwrap();
    writeln!(file, "line two").unwrap();
    drop(file);

    let new_lines = streamer.poll_new_lines();
    assert_eq!(new_lines, vec!["line one", "line two"]);
}

#[test]
fn test_poll_new_lines_empty_when_no_new_content() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("app.log"), "").unwrap();

    let mut streamer = LogStreamer::detect_and_open(dir.path().to_str().unwrap());
    let new_lines = streamer.poll_new_lines();
    assert!(new_lines.is_empty());
}

#[test]
fn test_poll_new_lines_no_file_returns_empty() {
    let mut streamer = LogStreamer::new();
    let new_lines = streamer.poll_new_lines();
    assert!(new_lines.is_empty());
}

#[test]
fn test_buffer_accumulates_across_multiple_polls() {
    let dir = TempDir::new().unwrap();
    let log_path = dir.path().join("app.log");
    fs::write(&log_path, "").unwrap();

    let mut streamer = LogStreamer::detect_and_open(dir.path().to_str().unwrap());

    let mut file = fs::OpenOptions::new().append(true).open(&log_path).unwrap();
    writeln!(file, "first").unwrap();
    drop(file);
    streamer.poll_new_lines();

    let mut file = fs::OpenOptions::new().append(true).open(&log_path).unwrap();
    writeln!(file, "second").unwrap();
    drop(file);
    streamer.poll_new_lines();

    assert_eq!(streamer.buffer().len(), 2);
    let lines: Vec<&str> = streamer.buffer().iter().map(|s| s.as_str()).collect();
    assert_eq!(lines, vec!["first", "second"]);
}

#[test]
fn test_buffer_capped_at_1000_lines() {
    let dir = TempDir::new().unwrap();
    let log_path = dir.path().join("app.log");
    fs::write(&log_path, "").unwrap();

    let mut streamer = LogStreamer::detect_and_open(dir.path().to_str().unwrap());

    let mut file = fs::OpenOptions::new().append(true).open(&log_path).unwrap();
    for i in 0..1100 {
        writeln!(file, "line {}", i).unwrap();
    }
    drop(file);

    streamer.poll_new_lines();
    assert_eq!(streamer.buffer().len(), 1000);
    // Buffer should contain the last 1000 lines
    assert_eq!(streamer.buffer().back().unwrap(), "line 1099");
}
