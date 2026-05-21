// TUI rendering and key handling

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Layout};
use ratatui::Frame;

use crate::process::killer::{KillSignal, ProcessKiller};
use crate::process::tree::TreeBuilder;
use crate::tui::app::{App, DialogKind, FocusPanel};
use crate::tui::widgets::{
    detail_panel, empty_state, help_dialog, kill_dialog, process_list, signal_picker, status_bar,
};

/// Main render function — lays out all panels and dialogs.
pub fn render(f: &mut Frame, app: &mut App) {
    let area = f.area();

    // Vertical layout: top bar(1) + main content(min 5) + bottom bar(1)
    let [top_bar, main_content, bottom_bar] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(5),
        Constraint::Length(1),
    ])
    .areas(area);

    // Top status bar
    status_bar::render_top_bar(f, top_bar, app);

    // Bottom bar
    status_bar::render_bottom_bar(f, bottom_bar, app);

    // Main content area
    if app.flat_list.is_empty() {
        // Empty state
        empty_state::render_empty_state(f, main_content, app.tick_count);
    } else {
        // Vertical split: 55% process list (top), 45% detail panel (bottom)
        let [top_panel, bottom_panel] = Layout::vertical([
            Constraint::Percentage(55),
            Constraint::Percentage(45),
        ])
        .areas(main_content);

        process_list::render_process_list(f, top_panel, app);
        detail_panel::render_detail_panel(f, bottom_panel, app);
    }

    // Modal dialogs rendered on top
    if let Some(ref dialog) = app.dialog {
        match dialog {
            DialogKind::SignalPicker => {
                signal_picker::render_signal_picker(f, area, app.signal_picker_index);
            }
            DialogKind::Help => {
                help_dialog::render_help_dialog(f, area, app);
            }
            _ => {
                let process = app.selected_process().cloned();
                let tree_process = process.as_ref().and_then(|p| {
                    app.find_process_in_trees(p.pid).cloned()
                });
                kill_dialog::render_kill_dialog(
                    f,
                    area,
                    dialog,
                    process.as_ref(),
                    tree_process.as_ref(),
                );
            }
        }
    }
}

/// Handle a key event, dispatching based on current mode.
pub fn handle_key(app: &mut App, key: KeyEvent) {
    // Dialog mode
    if let Some(ref dialog) = app.dialog.clone() {
        handle_dialog_key(app, key, dialog);
        return;
    }

    // Filter mode
    if app.filter_active {
        handle_filter_key(app, key);
        return;
    }

    // Normal mode
    handle_normal_key(app, key);
}

/// Handle keys when a dialog is open.
fn handle_dialog_key(app: &mut App, key: KeyEvent, dialog: &DialogKind) {
    match dialog {
        DialogKind::KillConfirm => match key.code {
            KeyCode::Enter => {
                if let Some(proc) = app.selected_process() {
                    let pid = proc.pid;
                    ProcessKiller::send_signal(pid, KillSignal::Term);
                }
                app.dialog = None;
                app.needs_rescan = true;
            }
            KeyCode::Esc => {
                app.dialog = None;
            }
            _ => {}
        },

        DialogKind::KillTreeConfirm => match key.code {
            KeyCode::Enter => {
                if let Some(proc) = app.selected_process() {
                    let pid = proc.pid;
                    if let Some(tree_proc) = app.find_process_in_trees(pid) {
                        let pids = TreeBuilder::collect_pids(tree_proc);
                        ProcessKiller::kill_tree(&pids, KillSignal::Term);
                    }
                }
                app.dialog = None;
                app.needs_rescan = true;
            }
            KeyCode::Esc => {
                app.dialog = None;
            }
            _ => {}
        },

        DialogKind::SignalPicker => {
            let signal_count = KillSignal::all().len();
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if app.signal_picker_index > 0 {
                        app.signal_picker_index -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if app.signal_picker_index < signal_count - 1 {
                        app.signal_picker_index += 1;
                    }
                }
                KeyCode::Enter => {
                    let signal = app.selected_kill_signal();
                    if let Some(proc) = app.selected_process() {
                        let pid = proc.pid;
                        ProcessKiller::send_signal(pid, signal);
                    }
                    app.dialog = None;
                    app.needs_rescan = true;
                }
                KeyCode::Esc => {
                    app.dialog = None;
                }
                _ => {}
            }
        }

        DialogKind::Help => match key.code {
            KeyCode::Esc | KeyCode::Char('H') | KeyCode::Char('q') => {
                app.dialog = None;
                app.help_scroll = 0;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.help_scroll = app.help_scroll.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.help_scroll = (app.help_scroll + 1).min(app.help_max_scroll);
            }
            KeyCode::PageUp => {
                app.help_scroll = app.help_scroll.saturating_sub(10);
            }
            KeyCode::PageDown => {
                app.help_scroll = (app.help_scroll + 10).min(app.help_max_scroll);
            }
            KeyCode::Home => {
                app.help_scroll = 0;
            }
            KeyCode::End => {
                app.help_scroll = app.help_max_scroll;
            }
            _ => {}
        },

        DialogKind::ForceKillPrompt => match key.code {
            KeyCode::Enter => {
                if let Some(proc) = app.selected_process() {
                    let pid = proc.pid;
                    ProcessKiller::force_kill(pid);
                }
                app.dialog = None;
                app.needs_rescan = true;
            }
            KeyCode::Esc => {
                app.dialog = None;
            }
            _ => {}
        },
    }
}

/// Handle keys when the filter input is active.
fn handle_filter_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.filter_active = false;
            app.filter_text.clear();
        }
        KeyCode::Enter => {
            app.filter_active = false;
            // filter_text remains applied
        }
        KeyCode::Backspace => {
            app.filter_text.pop();
        }
        KeyCode::Char(c) => {
            app.filter_text.push(c);
        }
        _ => {}
    }
}

/// Handle keys in normal mode.
fn handle_normal_key(app: &mut App, key: KeyEvent) {
    // Ctrl+C always quits
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        app.should_quit = true;
        return;
    }

    // Global keys (available in both panels)
    match key.code {
        KeyCode::Char('q') => {
            app.should_quit = true;
            return;
        }
        KeyCode::Char('/') => {
            app.filter_active = true;
            app.filter_text.clear();
            return;
        }
        KeyCode::Char('s') => {
            app.toggle_sort();
            return;
        }
        KeyCode::Char('r') => {
            app.sort_ascending = !app.sort_ascending;
            return;
        }
        KeyCode::Char('+') => {
            app.refresh_secs = (app.refresh_secs + 1).min(60);
            app.refresh_changed = true;
            return;
        }
        KeyCode::Char('-') => {
            app.refresh_secs = (app.refresh_secs.saturating_sub(1)).max(1);
            app.refresh_changed = true;
            return;
        }
        KeyCode::Char('x') => {
            if app.selected_process().is_some() {
                app.dialog = Some(DialogKind::KillConfirm);
            }
            return;
        }
        KeyCode::Char('K') => {
            if app.selected_process().is_some() {
                app.dialog = Some(DialogKind::KillTreeConfirm);
            }
            return;
        }
        KeyCode::Char('H') => {
            app.dialog = Some(DialogKind::Help);
            return;
        }
        KeyCode::Char('S') => {
            if app.selected_process().is_some() {
                app.signal_picker_index = 0;
                app.dialog = Some(DialogKind::SignalPicker);
            }
            return;
        }
        KeyCode::Char('e') => {
            app.toggle_expand_all();
            return;
        }
        _ => {}
    }

    // Panel-specific keys
    match app.focus {
        FocusPanel::ProcessList => handle_list_key(app, key),
        FocusPanel::DetailPanel => handle_detail_key(app, key),
    }
}

/// Handle keys when the process list has focus.
fn handle_list_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.should_quit = true;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.move_up();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.move_down();
        }
        KeyCode::PageUp => {
            for _ in 0..10 {
                app.move_up();
            }
        }
        KeyCode::PageDown => {
            for _ in 0..10 {
                app.move_down();
            }
        }
        KeyCode::Home => {
            app.selected_index = 0;
            app.table_state.select(Some(0));
        }
        KeyCode::End => {
            if !app.flat_list.is_empty() {
                app.selected_index = app.flat_list.len() - 1;
                app.table_state.select(Some(app.selected_index));
            }
        }
        KeyCode::Enter => {
            app.toggle_expand();
        }
        KeyCode::Right | KeyCode::Char('l') => {
            app.expand_selected();
        }
        KeyCode::Left | KeyCode::Char('h') => {
            app.collapse_selected();
        }
        KeyCode::Char(' ') => {
            app.toggle_select();
        }
        KeyCode::Tab => {
            app.focus = FocusPanel::DetailPanel;
        }
        _ => {}
    }
}

/// Handle keys when the detail panel has focus.
fn handle_detail_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.focus = FocusPanel::ProcessList;
        }
        KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => {
            app.next_tab();
        }
        KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => {
            app.prev_tab();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.detail_scroll = app.detail_scroll.saturating_sub(1);
            app.log_scroll = app.log_scroll.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.detail_scroll = app.detail_scroll.saturating_add(1);
            app.log_scroll = app.log_scroll.saturating_add(1);
            app.clamp_detail_scroll();
        }
        KeyCode::PageUp => {
            app.detail_scroll = app.detail_scroll.saturating_sub(10);
            app.log_scroll = app.log_scroll.saturating_sub(10);
        }
        KeyCode::PageDown => {
            app.detail_scroll = app.detail_scroll.saturating_add(10);
            app.log_scroll = app.log_scroll.saturating_add(10);
            app.clamp_detail_scroll();
        }
        KeyCode::Home => {
            app.detail_scroll = 0;
            app.log_scroll = 0;
        }
        _ => {}
    }
}
