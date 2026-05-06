// Log streaming implementation

use glob::glob;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};

const LOG_PATTERNS: &[&str] = &[
    "*.log",
    ".next/server/app/**/*.log",
    ".next/trace",
    "logs/*.log",
    "log/*.log",
    "npm-debug.log",
    "yarn-error.log",
];

const MAX_BUFFER_LINES: usize = 1000;

pub struct LogStreamer {
    file: Option<BufReader<File>>,
    path: Option<PathBuf>,
    buffer: VecDeque<String>,
}

impl LogStreamer {
    /// Create an empty streamer with no file open.
    pub fn new() -> Self {
        Self {
            file: None,
            path: None,
            buffer: VecDeque::new(),
        }
    }

    /// Scan `cwd` for log files using LOG_PATTERNS, open the most recently
    /// modified one, and seek to the end so only new content is streamed.
    pub fn detect_and_open(cwd: &str) -> Self {
        let mut streamer = Self::new();

        if let Some(log_path) = Self::find_log_file(cwd) {
            if let Ok(file) = File::open(&log_path) {
                let mut reader = BufReader::new(file);
                // Seek to end so we only pick up new lines
                let _ = reader.seek(SeekFrom::End(0));
                streamer.path = Some(log_path);
                streamer.file = Some(reader);
            }
        }

        streamer
    }

    /// Search `cwd` for log files matching LOG_PATTERNS, sort by modification
    /// time (most recent first), and return the first match.
    pub fn find_log_file(cwd: &str) -> Option<PathBuf> {
        let base = Path::new(cwd);
        let mut candidates: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();

        for pattern in LOG_PATTERNS {
            let full_pattern = base.join(pattern);
            let pattern_str = full_pattern.to_string_lossy().to_string();

            if let Ok(entries) = glob(&pattern_str) {
                for entry in entries.flatten() {
                    if entry.is_file() {
                        if let Ok(metadata) = entry.metadata() {
                            if let Ok(modified) = metadata.modified() {
                                candidates.push((entry, modified));
                            }
                        }
                    }
                }
            }
        }

        // Sort by modification time, most recent first
        candidates.sort_by(|a, b| b.1.cmp(&a.1));
        candidates.into_iter().next().map(|(path, _)| path)
    }

    /// Read new lines from the file, add them to the buffer (capped at
    /// MAX_BUFFER_LINES), and return the new lines.
    pub fn poll_new_lines(&mut self) -> Vec<String> {
        let mut new_lines = Vec::new();

        if let Some(ref mut reader) = self.file {
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => break, // No more data
                    Ok(_) => {
                        let trimmed = line.trim_end_matches('\n').trim_end_matches('\r').to_string();
                        self.buffer.push_back(trimmed.clone());
                        new_lines.push(trimmed);

                        // Cap buffer at MAX_BUFFER_LINES
                        while self.buffer.len() > MAX_BUFFER_LINES {
                            self.buffer.pop_front();
                        }
                    }
                    Err(_) => break,
                }
            }
        }

        new_lines
    }

    /// Return a reference to the internal line buffer.
    pub fn buffer(&self) -> &VecDeque<String> {
        &self.buffer
    }

    /// Whether a log file is currently open.
    pub fn has_source(&self) -> bool {
        self.file.is_some()
    }

    /// Path of the open log file, if any.
    pub fn source_path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Try cwd first, then fall back to `/proc/<pid>/fd/1` on Linux.
    #[cfg(target_os = "linux")]
    pub fn detect_and_open_with_proc(cwd: &str, pid: u32) -> Self {
        let streamer = Self::detect_and_open(cwd);
        if streamer.has_source() {
            return streamer;
        }

        // Fall back to /proc/<pid>/fd/1 (stdout)
        let proc_path = PathBuf::from(format!("/proc/{}/fd/1", pid));
        if proc_path.exists() {
            if let Ok(file) = File::open(&proc_path) {
                let mut reader = BufReader::new(file);
                let _ = reader.seek(SeekFrom::End(0));
                return Self {
                    file: Some(reader),
                    path: Some(proc_path),
                    buffer: VecDeque::new(),
                };
            }
        }

        Self::new()
    }
}

impl Default for LogStreamer {
    fn default() -> Self {
        Self::new()
    }
}
