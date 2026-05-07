use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::tui::widgets::kill_dialog::centered_rect;

pub fn render_help_dialog(f: &mut Frame, area: Rect) {
    let popup_area = centered_rect(60, 70, area);
    f.render_widget(Clear, popup_area);

    let version = env!("CARGO_PKG_VERSION");

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  Node Server Manager v{}", version),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        section_header("Navigation"),
        key_line("Up / k", "Move cursor up"),
        key_line("Down / j", "Move cursor down"),
        key_line("Home", "Jump to first item"),
        key_line("End", "Jump to last item"),
        key_line("Enter", "Expand / collapse tree node"),
        key_line("Space", "Toggle multi-select"),
        Line::from(""),
        section_header("Tabs & Filtering"),
        key_line("Tab", "Next detail tab"),
        key_line("Shift+Tab", "Previous detail tab"),
        key_line("/", "Open filter input"),
        key_line("s", "Cycle sort column"),
        Line::from(""),
        section_header("Process Control"),
        key_line("x", "Kill selected process (SIGTERM)"),
        key_line("K", "Kill process tree (SIGTERM)"),
        key_line("S", "Open signal picker"),
        Line::from(""),
        section_header("Log View"),
        key_line("PageUp", "Scroll log up"),
        key_line("PageDown", "Scroll log down"),
        Line::from(""),
        section_header("General"),
        key_line("H", "Show this help"),
        key_line("q / Esc", "Quit"),
        key_line("Ctrl+C", "Force quit"),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Press ", Style::default().fg(Color::DarkGray)),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::styled(" to close", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false }).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Help ")
            .title_alignment(Alignment::Center),
    );

    f.render_widget(paragraph, popup_area);
}

fn section_header(title: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!("  -- {} --", title),
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ))
}

fn key_line(key: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("    {:14}", key),
            Style::default().fg(Color::Green),
        ),
        Span::styled(desc.to_string(), Style::default().fg(Color::White)),
    ])
}
