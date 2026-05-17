// log_tab widget — Scrollable log output display

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

use crate::tui::app::App;

/// Render the Log tab.
/// Returns total content line count.
pub fn render_log_tab(f: &mut Frame, area: Rect, app: &App) -> u16 {
    match &app.log_streamer {
        Some(streamer) if streamer.has_source() => {
            let source_path = streamer
                .source_path()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let buffer = streamer.buffer();

            if buffer.is_empty() {
                // Has source but no output yet
                let lines = vec![
                    Line::from(vec![
                        Span::styled("  Source: ", Style::default().fg(Color::Cyan)),
                        Span::styled(&source_path, Style::default().fg(Color::White)),
                    ]),
                    Line::from(""),
                    Line::from(Span::styled(
                        "  Waiting for log output...",
                        Style::default().fg(Color::DarkGray),
                    )),
                ];
                let paragraph = Paragraph::new(lines);
                f.render_widget(paragraph, area);
                3
            } else {
                // Build log lines with source header
                let mut lines: Vec<Line> = Vec::new();
                lines.push(Line::from(vec![
                    Span::styled("  Source: ", Style::default().fg(Color::Cyan)),
                    Span::styled(source_path, Style::default().fg(Color::White)),
                ]));
                lines.push(Line::from(Span::styled(
                    "  ─".repeat(area.width.saturating_sub(4) as usize / 2),
                    Style::default().fg(Color::DarkGray),
                )));

                for log_line in buffer.iter() {
                    let style = if log_line.contains("ERROR") || log_line.contains("error") {
                        Style::default().fg(Color::Red)
                    } else if log_line.contains("WARN") || log_line.contains("warn") {
                        Style::default().fg(Color::Yellow)
                    } else if log_line.contains("DEBUG") || log_line.contains("debug") {
                        Style::default().fg(Color::DarkGray)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    lines.push(Line::from(Span::styled(
                        format!("  {}", log_line),
                        style,
                    )));
                }

                let line_count = lines.len() as u16;
                let paragraph = Paragraph::new(lines)
                    .wrap(Wrap { trim: false })
                    .scroll((app.log_scroll, 0));

                f.render_widget(paragraph, area);
                line_count
            }
        }
        _ => {
            let lines = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "  No log source detected",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "  ntop looks for log files in the process working directory:",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(Span::styled(
                    "    *.log, logs/*.log, .next/trace, npm-debug.log, etc.",
                    Style::default().fg(Color::DarkGray),
                )),
            ];
            let line_count = lines.len() as u16;
            let paragraph = Paragraph::new(lines);
            f.render_widget(paragraph, area);
            line_count
        }
    }
}
