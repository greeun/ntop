// process_list widget — Left panel with process tree

use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Row, Table};
use ratatui::Frame;

use crate::process::HealthStatus;
use crate::tui::app::{App, FocusPanel, SortColumn};

/// Render the process list in the left panel.
pub fn render_process_list(f: &mut Frame, area: Rect, app: &mut App) {
    // Column headers with sort indicators
    let sort_indicator = |col: SortColumn| -> &'static str {
        if app.sort_column == col {
            if app.sort_ascending { " ^" } else { " v" }
        } else {
            ""
        }
    };

    let header_cells = vec![
        format!(" "),
        format!("PID{}", sort_indicator(SortColumn::Pid)),
        format!("NAME{}", sort_indicator(SortColumn::Name)),
        format!("PORT{}", sort_indicator(SortColumn::Port)),
        format!("THR{}", sort_indicator(SortColumn::Threads)),
        format!("CPU{}", sort_indicator(SortColumn::Cpu)),
        format!("MEM{}", sort_indicator(SortColumn::Memory)),
        format!("USER{}", sort_indicator(SortColumn::User)),
        format!("STS{}", sort_indicator(SortColumn::Status)),
        format!("UPTIME{}", sort_indicator(SortColumn::Uptime)),
    ];

    let header = Row::new(header_cells)
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(0);

    // Calculate available width for adaptive column sizes
    let inner_width = if area.width > 2 { area.width - 2 } else { area.width };

    let widths = if inner_width > 100 {
        vec![
            Constraint::Length(3),   // checkbox/health
            Constraint::Length(7),   // PID
            Constraint::Min(12),     // NAME (flexible)
            Constraint::Length(7),   // PORT
            Constraint::Length(4),   // THR
            Constraint::Length(7),   // CPU
            Constraint::Length(9),   // MEM
            Constraint::Length(8),   // USER
            Constraint::Length(7),   // STATUS
            Constraint::Length(10),  // UPTIME
        ]
    } else {
        vec![
            Constraint::Length(2),
            Constraint::Length(6),
            Constraint::Min(8),
            Constraint::Length(6),
            Constraint::Length(4),
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(8),
        ]
    };

    let rows: Vec<Row> = app
        .flat_list
        .iter()
        .enumerate()
        .map(|(idx, (proc_info, depth))| {
            let is_selected_row = idx == app.selected_index;
            let is_multi_selected = app.selected_pids.contains(&proc_info.pid);

            // Health indicator
            let health = proc_info.health();
            let health_dot = match health {
                HealthStatus::Healthy => Span::styled("●", Style::default().fg(Color::Green)),
                HealthStatus::Warning => Span::styled("●", Style::default().fg(Color::Yellow)),
                HealthStatus::Critical => Span::styled("●", Style::default().fg(Color::Red)),
            };

            // Checkbox
            let checkbox = if is_multi_selected { "[x]" } else { "[ ]" };
            let checkbox_span = Span::styled(
                format!("{}", checkbox),
                Style::default().fg(if is_multi_selected { Color::Cyan } else { Color::DarkGray }),
            );

            // Tree prefix
            let has_children = !proc_info.children.is_empty();
            let is_expanded = app.expanded_pids.contains(&proc_info.pid);
            let tree_prefix = if *depth == 0 {
                if has_children {
                    if is_expanded { "▾ " } else { "▸ " }
                } else {
                    "  "
                }
            } else {
                let indent = "  ".repeat(*depth);
                // For children: use └─ prefix
                if has_children {
                    if is_expanded {
                        &format!("{}├▾ ", indent)
                    } else {
                        &format!("{}├▸ ", indent)
                    }
                } else {
                    &format!("{}└─ ", indent)
                }
            };

            let base_name = if proc_info.framework.to_string() != "Generic" {
                format!("{} ({})", proc_info.display_name(), proc_info.framework)
            } else {
                proc_info.display_name()
            };
            let name_display = format!("{}{}", tree_prefix, base_name);

            // Ports
            let ports_str = if proc_info.ports.is_empty() {
                "-".to_string()
            } else {
                proc_info
                    .ports
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            };

            // Row style
            let row_style = if is_selected_row {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let first_cell = Line::from(vec![checkbox_span, Span::raw(" "), health_dot]);

            Row::new(vec![
                first_cell.to_string(),
                proc_info.pid.to_string(),
                name_display,
                ports_str,
                proc_info.threads.to_string(),
                format!("{:.1}%", proc_info.cpu_percent),
                proc_info.memory_display(),
                proc_info.user.clone(),
                proc_info.status.clone(),
                proc_info.uptime_display(),
            ])
            .style(row_style)
        })
        .collect();

    let title = if app.filter_active || !app.filter_text.is_empty() {
        format!(" Processes (filter: {}) ", app.filter_text)
    } else {
        " Processes ".to_string()
    };

    let border_color = if app.focus == FocusPanel::ProcessList {
        Color::Cyan
    } else {
        Color::Gray
    };

    let table = Table::new(rows, &widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(title),
        )
        .column_spacing(1)
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(table, area, &mut app.table_state);
}
