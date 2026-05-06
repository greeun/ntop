// signal_picker widget — Signal selection modal

use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::process::killer::KillSignal;
use crate::tui::widgets::kill_dialog::centered_rect;

/// Render the signal picker modal.
pub fn render_signal_picker(f: &mut Frame, area: Rect, selected_index: usize) {
    let popup_area = centered_rect(45, 50, area);

    // Clear the area behind the dialog
    f.render_widget(Clear, popup_area);

    let signals = KillSignal::all();

    let mut lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Select a signal to send:",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
    ];

    for (idx, signal) in signals.iter().enumerate() {
        let is_selected = idx == selected_index;
        let indicator = if is_selected { " ▸ " } else { "   " };

        let style = if is_selected {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        lines.push(Line::from(vec![
            Span::styled(indicator, style),
            Span::styled(
                format!("{:<10}", signal.name()),
                style,
            ),
            Span::styled(
                format!(" - {}", signal.description()),
                Style::default().fg(if is_selected {
                    Color::Gray
                } else {
                    Color::DarkGray
                }),
            ),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  [Enter] ", Style::default().fg(Color::Green)),
        Span::styled("Send  ", Style::default().fg(Color::White)),
        Span::styled("[Up/Down] ", Style::default().fg(Color::Yellow)),
        Span::styled("Select  ", Style::default().fg(Color::White)),
        Span::styled("[Esc] ", Style::default().fg(Color::Red)),
        Span::styled("Cancel", Style::default().fg(Color::White)),
    ]));

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Signal Picker ")
            .title_alignment(Alignment::Center),
    );

    f.render_widget(paragraph, popup_area);
}
