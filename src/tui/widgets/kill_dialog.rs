// kill_dialog widget — Kill confirmation modal

use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::process::ProcessInfo;
use crate::tui::app::DialogKind;

/// Render the kill confirmation dialog.
pub fn render_kill_dialog(
    f: &mut Frame,
    area: Rect,
    dialog: &DialogKind,
    process: Option<&ProcessInfo>,
    tree_process: Option<&ProcessInfo>,
) {
    let popup_area = centered_rect(50, 40, area);

    // Clear the area behind the dialog
    f.render_widget(Clear, popup_area);

    let (title, lines) = match dialog {
        DialogKind::KillConfirm => {
            let mut lines = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "  Are you sure you want to kill this process?",
                    Style::default().fg(Color::Yellow),
                )),
                Line::from(""),
            ];

            if let Some(proc) = process {
                lines.push(Line::from(vec![
                    Span::styled("    PID:  ", Style::default().fg(Color::Cyan)),
                    Span::styled(proc.pid.to_string(), Style::default().fg(Color::White)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("    Name: ", Style::default().fg(Color::Cyan)),
                    Span::styled(&proc.name, Style::default().fg(Color::White)),
                ]));
                if !proc.ports.is_empty() {
                    lines.push(Line::from(vec![
                        Span::styled("    Port: ", Style::default().fg(Color::Cyan)),
                        Span::styled(
                            proc.ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", "),
                            Style::default().fg(Color::White),
                        ),
                    ]));
                }
            }

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  Signal: SIGTERM (graceful)",
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  [Enter] ", Style::default().fg(Color::Green)),
                Span::styled("Confirm  ", Style::default().fg(Color::White)),
                Span::styled("[Esc] ", Style::default().fg(Color::Red)),
                Span::styled("Cancel", Style::default().fg(Color::White)),
            ]));

            (" Kill Process ", lines)
        }

        DialogKind::KillTreeConfirm => {
            let mut lines = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "  Kill process tree? This will terminate:",
                    Style::default().fg(Color::Yellow),
                )),
                Line::from(""),
            ];

            if let Some(proc) = tree_process {
                lines.push(Line::from(vec![
                    Span::styled("    Root: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        format!("PID {} ({})", proc.pid, proc.name),
                        Style::default().fg(Color::White),
                    ),
                ]));

                let child_count = count_descendants(proc);
                if child_count > 0 {
                    lines.push(Line::from(vec![
                        Span::styled("    Children: ", Style::default().fg(Color::Cyan)),
                        Span::styled(
                            format!("{} process(es)", child_count),
                            Style::default().fg(Color::Red),
                        ),
                    ]));

                    // List child PIDs
                    for child in &proc.children {
                        lines.push(Line::from(Span::styled(
                            format!("      - PID {} ({})", child.pid, child.name),
                            Style::default().fg(Color::DarkGray),
                        )));
                    }
                }
            }

            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  [Enter] ", Style::default().fg(Color::Green)),
                Span::styled("Confirm  ", Style::default().fg(Color::White)),
                Span::styled("[Esc] ", Style::default().fg(Color::Red)),
                Span::styled("Cancel", Style::default().fg(Color::White)),
            ]));

            (" Kill Process Tree ", lines)
        }

        DialogKind::ForceKillPrompt => {
            let mut lines = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "  Graceful kill timed out.",
                    Style::default().fg(Color::Red),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "  Send SIGKILL (force kill)?",
                    Style::default().fg(Color::Yellow),
                )),
            ];

            if let Some(proc) = process {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("    PID: ", Style::default().fg(Color::Cyan)),
                    Span::styled(proc.pid.to_string(), Style::default().fg(Color::White)),
                ]));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  WARNING: SIGKILL cannot be caught or ignored.",
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  [Enter] ", Style::default().fg(Color::Green)),
                Span::styled("Force Kill  ", Style::default().fg(Color::White)),
                Span::styled("[Esc] ", Style::default().fg(Color::Red)),
                Span::styled("Cancel", Style::default().fg(Color::White)),
            ]));

            (" Force Kill ", lines)
        }

        DialogKind::SignalPicker | DialogKind::Help => {
            return;
        }
    };

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false }).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red))
            .title(title)
            .title_alignment(Alignment::Center),
    );

    f.render_widget(paragraph, popup_area);
}

fn count_descendants(process: &ProcessInfo) -> usize {
    let mut count = process.children.len();
    for child in &process.children {
        count += count_descendants(child);
    }
    count
}

/// Create a centered rectangle within the given area.
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let width = area.width * percent_x / 100;
    let height = area.height * percent_y / 100;
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}
