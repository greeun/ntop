// env_tab widget — Environment variables with sensitive value masking

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

use crate::config::Config;
use crate::process::ProcessInfo;

/// Sensitive key patterns — if a key contains any of these, mask the value.
const SENSITIVE_PATTERNS: &[&str] = &[
    "PASSWORD",
    "SECRET",
    "TOKEN",
    "KEY",
    "API_KEY",
    "PRIVATE",
    "CREDENTIALS",
    "AUTH",
];

/// Check if an environment variable key should be masked.
fn is_sensitive(key: &str) -> bool {
    let upper = key.to_uppercase();
    SENSITIVE_PATTERNS
        .iter()
        .any(|pattern| upper.contains(pattern))
}

/// Render the Env tab with KEY=VALUE pairs.
pub fn render_env_tab(f: &mut Frame, area: Rect, process: &ProcessInfo, config: &Config) {
    if process.env_vars.is_empty() {
        let msg = ratatui::widgets::Paragraph::new("  No environment variables available.")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(msg, area);
        return;
    }

    let should_mask = config.display.mask_env_values;

    let lines: Vec<Line> = process
        .env_vars
        .iter()
        .map(|(key, value)| {
            let display_value = if should_mask && is_sensitive(key) {
                "********".to_string()
            } else {
                value.clone()
            };

            Line::from(vec![
                Span::styled(
                    format!("  {}", key),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("=", Style::default().fg(Color::DarkGray)),
                Span::styled(display_value, Style::default().fg(Color::White)),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}
