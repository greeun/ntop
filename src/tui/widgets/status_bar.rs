// status_bar widget — Top bar with system stats, bottom bar with keybindings

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::tui::app::{App, DialogKind};

/// Render the top status bar.
pub fn render_top_bar(f: &mut Frame, area: Rect, app: &App) {
    let version = env!("CARGO_PKG_VERSION");
    let node_count = app.flat_list.len();
    let mem_used_mb = app.system_memory_used / (1024 * 1024);
    let mem_total_mb = app.system_memory_total / (1024 * 1024);

    let spans = vec![
        Span::styled(
            format!(" ntop v{}", version),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
        Span::styled("CPU: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{:.1}%", app.system_cpu),
            Style::default().fg(if app.system_cpu > 80.0 {
                Color::Red
            } else if app.system_cpu > 50.0 {
                Color::Yellow
            } else {
                Color::Green
            }),
        ),
        Span::styled("  MEM: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{}/{}MB", mem_used_mb, mem_total_mb),
            Style::default().fg(Color::White),
        ),
        Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Nodes: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{}", node_count),
            Style::default().fg(Color::White),
        ),
        Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Refresh: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{}s", app.refresh_secs),
            Style::default().fg(Color::White),
        ),
        Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[H]", Style::default().fg(Color::Yellow)),
        Span::styled("elp", Style::default().fg(Color::Gray)),
    ];

    let paragraph = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));

    f.render_widget(paragraph, area);
}

/// Render the bottom bar with context-sensitive keybinding hints.
pub fn render_bottom_bar(f: &mut Frame, area: Rect, app: &App) {
    let spans = if app.dialog.is_some() {
        match app.dialog.as_ref().unwrap() {
            DialogKind::Help => {
                vec![
                    key_hint("Esc", "Close"),
                ]
            }
            DialogKind::KillConfirm | DialogKind::KillTreeConfirm | DialogKind::ForceKillPrompt => {
                vec![
                    key_hint("Enter", "Confirm"),
                    separator(),
                    key_hint("Esc", "Cancel"),
                ]
            }
            DialogKind::SignalPicker => {
                vec![
                    key_hint("Up/Down", "Select"),
                    separator(),
                    key_hint("Enter", "Send"),
                    separator(),
                    key_hint("Esc", "Cancel"),
                ]
            }
        }
    } else if app.filter_active {
        vec![
            Span::styled(" /", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{}", app.filter_text),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
            Span::styled("█", Style::default().fg(Color::Yellow)),
            Span::styled("  ", Style::default()),
            separator(),
            key_hint("Enter", "Apply"),
            separator(),
            key_hint("Esc", "Cancel"),
        ]
    } else {
        vec![
            key_hint("q", "Quit"),
            separator(),
            key_hint("Up/Down", "Navigate"),
            separator(),
            key_hint("Enter", "Expand"),
            separator(),
            key_hint("Tab", "Detail Tab"),
            separator(),
            key_hint("Space", "Select"),
            separator(),
            key_hint("/", "Filter"),
            separator(),
            key_hint("s", "Sort"),
            separator(),
            key_hint("x", "Kill"),
            separator(),
            key_hint("K", "Tree Kill"),
            separator(),
            key_hint("S", "Signal"),
        ]
    };

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));
    f.render_widget(paragraph, area);
}

fn key_hint(key: &str, desc: &str) -> Span<'static> {
    // We build a combined string to avoid lifetime issues
    let text = format!(" [{}] {} ", key, desc);
    Span::styled(text, Style::default().fg(Color::Yellow))
}

fn separator() -> Span<'static> {
    Span::styled("|", Style::default().fg(Color::DarkGray))
}
