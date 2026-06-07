use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
};
use ratatui::Frame;

use crate::tui::app::App;
use crate::tui::widgets::kill_dialog::centered_rect;

pub fn render_help_dialog(f: &mut Frame, area: Rect, app: &mut App) {
    let popup_area = centered_rect(60, 70, area);
    f.render_widget(Clear, popup_area);

    let version = env!("CARGO_PKG_VERSION");

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  ntop v{}", version),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        section_header("Process List (default focus)"),
        key_line("Up/Down", "Move cursor"),
        key_line("PgUp/PgDn", "Page up/down"),
        key_line("Home/End", "Jump to first/last"),
        key_line("Enter", "Expand / collapse tree"),
        key_line("Left/Right", "Collapse / expand node"),
        key_line("Space", "Toggle multi-select"),
        key_line("Tab", "Move focus to detail panel"),
        Line::from(""),
        section_header("Detail Panel (Tab to enter)"),
        key_line("Tab", "Next detail tab"),
        key_line("Shift+Tab", "Previous detail tab"),
        key_line("Up/Down", "Scroll content"),
        key_line("PgUp/PgDn", "Page content"),
        key_line("Esc", "Return to process list"),
        Line::from(""),
        section_header("Filtering & Sort"),
        key_line("/", "Open filter input"),
        key_line("n", "Toggle Node-only view"),
        key_line("s", "Cycle sort column"),
        key_line("r", "Reverse sort direction"),
        Line::from(""),
        section_header("Process Control"),
        key_line("x", "Kill selected (SIGTERM)"),
        key_line("K", "Kill process tree"),
        key_line("S", "Open signal picker"),
        Line::from(""),
        section_header("General"),
        key_line("e", "Toggle expand/collapse all"),
        key_line("+/-", "Adjust refresh interval (1-60s, shown in status bar)"),
        key_line("H", "Show this help"),
        key_line("q / Ctrl+C", "Quit"),
        Line::from(""),
        section_header("This Dialog"),
        key_line("Up/Down", "Scroll one line"),
        key_line("PgUp/PgDn", "Scroll one page"),
        key_line("Home/End", "Jump to top/bottom"),
        key_line("Esc / H / q", "Close"),
    ];

    let total_lines = lines.len() as u16;
    // popup_area has 1-cell top/bottom border, so visible content height is height - 2.
    let inner_height = popup_area.height.saturating_sub(2);
    let max_scroll = total_lines.saturating_sub(inner_height);
    app.help_max_scroll = max_scroll;
    if app.help_scroll > max_scroll {
        app.help_scroll = max_scroll;
    }

    let title = if max_scroll > 0 {
        format!(" Help  [{}/{}] ", app.help_scroll + 1, max_scroll + 1)
    } else {
        " Help ".to_string()
    };

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((app.help_scroll, 0))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(title)
                .title_alignment(Alignment::Center),
        );

    f.render_widget(paragraph, popup_area);

    if max_scroll > 0 {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .style(Style::default().fg(Color::DarkGray));
        let mut state = ScrollbarState::new(max_scroll as usize).position(app.help_scroll as usize);
        f.render_stateful_widget(scrollbar, popup_area, &mut state);
    }
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
