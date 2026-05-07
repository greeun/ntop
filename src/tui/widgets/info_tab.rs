// info_tab widget — Key-value process information display

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

use crate::process::ProcessInfo;

/// Render the Info tab with key-value pairs about the process.
pub fn render_info_tab(f: &mut Frame, area: Rect, process: &ProcessInfo, scroll: u16) {
    let ports_str = if process.ports.is_empty() {
        "-".to_string()
    } else {
        process
            .ports
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    };

    let version_str = process
        .framework_version
        .as_deref()
        .unwrap_or("-");

    let fields: Vec<(&str, String)> = vec![
        ("PID", process.pid.to_string()),
        ("PPID", process.ppid.to_string()),
        ("Name", process.name.clone()),
        ("Framework", process.framework.to_string()),
        ("Version", version_str.to_string()),
        ("Ports", ports_str),
        ("CPU", format!("{:.1}%", process.cpu_percent)),
        ("Memory", process.memory_display()),
        ("Memory (VMS)", format_bytes(process.memory_vms)),
        ("Threads", process.threads.to_string()),
        ("Uptime", process.uptime_display()),
        ("User", process.user.clone()),
        ("Status", process.status.clone()),
        ("Health", process.health().to_string()),
        ("Open FDs", process.open_fds.to_string()),
        ("CWD", process.cwd.clone()),
        ("Command", process.command.clone()),
    ];

    let lines: Vec<Line> = fields
        .into_iter()
        .map(|(key, val)| {
            Line::from(vec![
                Span::styled(
                    format!("  {:>12}: ", key),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(val, Style::default().fg(Color::White)),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));

    f.render_widget(paragraph, area);
}

fn format_bytes(bytes: u64) -> String {
    let b = bytes as f64;
    if b >= 1_073_741_824.0 {
        format!("{:.1} GB", b / 1_073_741_824.0)
    } else if b >= 1_048_576.0 {
        format!("{:.1} MB", b / 1_048_576.0)
    } else if b >= 1_024.0 {
        format!("{:.1} KB", b / 1_024.0)
    } else {
        format!("{} B", bytes)
    }
}
