// net_tab widget — Network connections table

use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Row, Table};
use ratatui::Frame;

use crate::process::network::NetworkInspector;
use crate::process::ProcessInfo;

/// Render the Net tab with network connections for the selected process.
/// Returns total content line count.
pub fn render_net_tab(f: &mut Frame, area: Rect, process: &ProcessInfo, scroll: u16) -> u16 {
    let connections = NetworkInspector::connections_for_pid(process.pid);

    if connections.is_empty() {
        let msg = ratatui::widgets::Paragraph::new("  No active network connections found.")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(msg, area);
        return 1;
    }

    let header = Row::new(vec!["  LOCAL", "REMOTE", "STATE"])
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(0);

    let rows: Vec<Row> = connections
        .iter()
        .skip(scroll as usize)
        .map(|conn| {
            let local = format!("  {}", conn.local_addr);
            let remote = conn
                .remote_addr
                .map(|a| a.to_string())
                .unwrap_or_else(|| "-".to_string());

            let state_color = match conn.state.as_str() {
                "LISTEN" => Color::Green,
                "ESTABLISHED" => Color::Cyan,
                "CLOSE_WAIT" => Color::Yellow,
                "TIME_WAIT" => Color::DarkGray,
                _ => Color::White,
            };

            Row::new(vec![
                local,
                remote,
                conn.state.clone(),
            ])
            .style(Style::default().fg(state_color))
        })
        .collect();

    let widths = [
        Constraint::Percentage(40),
        Constraint::Percentage(40),
        Constraint::Percentage(20),
    ];

    let line_count = (connections.len() + 1) as u16; // +1 for header

    let table = Table::new(rows, widths)
        .header(header)
        .column_spacing(1);

    f.render_widget(table, area);
    line_count
}
