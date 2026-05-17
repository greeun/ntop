// detail_panel widget — Right panel tab container

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Tabs};
use ratatui::Frame;

use crate::tui::app::{App, DetailTab, FocusPanel};
use crate::tui::widgets::{env_tab, info_tab, log_tab, net_tab};

/// Render the detail panel with tab bar and active tab content.
pub fn render_detail_panel(f: &mut Frame, area: Rect, app: &mut App) {
    let border_color = if app.focus == FocusPanel::DetailPanel {
        Color::Cyan
    } else {
        Color::Gray
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(" Details ");

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height < 3 || inner.width < 4 {
        return;
    }

    // Split into tab bar + content area
    let [tab_area, content_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(inner);

    // Render tab bar
    let tab_titles: Vec<String> = DetailTab::all()
        .iter()
        .map(|t| format!(" {} ", t.label()))
        .collect();

    let selected_idx = DetailTab::all()
        .iter()
        .position(|t| *t == app.active_tab)
        .unwrap_or(0);

    let tabs = Tabs::new(tab_titles)
        .select(selected_idx)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )
        .divider("|");

    f.render_widget(tabs, tab_area);

    app.detail_view_height = content_area.height;

    // Render active tab content
    if let Some(process) = app.selected_process().cloned() {
        let lines = match app.active_tab {
            DetailTab::Info => info_tab::render_info_tab(f, content_area, &process, app.detail_scroll),
            DetailTab::Log => log_tab::render_log_tab(f, content_area, app),
            DetailTab::Net => net_tab::render_net_tab(f, content_area, &process, app.detail_scroll),
            DetailTab::Env => env_tab::render_env_tab(f, content_area, &process, &app.config, app.detail_scroll),
        };
        app.detail_content_lines = lines;
    } else {
        app.detail_content_lines = 0;
        let msg = ratatui::widgets::Paragraph::new("Select a process to view details")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(msg, content_area);
    }
}
