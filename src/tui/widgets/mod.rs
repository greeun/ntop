pub mod process_list;
pub mod detail_panel;
pub mod info_tab;
pub mod log_tab;
pub mod net_tab;
pub mod env_tab;
pub mod help_dialog;
pub mod kill_dialog;
pub mod signal_picker;
pub mod status_bar;
pub mod empty_state;

use ratatui::text::Line;

/// Compute the number of terminal rows required to render `lines` inside
/// an area of `width` columns when `Wrap { trim: false }` is in effect.
///
/// Using `lines.len()` directly under-counts when long fields wrap,
/// which prevents the surrounding panel from scrolling past the first
/// viewport's worth of rows.
pub fn wrapped_line_count(lines: &[Line<'_>], width: u16) -> u16 {
    if width == 0 {
        return lines.len() as u16;
    }
    let w = width as usize;
    let mut total: usize = 0;
    for line in lines {
        let line_w = line.width().max(1);
        total += line_w.div_ceil(w);
    }
    total.min(u16::MAX as usize) as u16
}
