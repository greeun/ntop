// TUI application state

use std::collections::HashSet;
use std::time::Instant;

use crate::config::Config;
use crate::log::streamer::LogStreamer;
use crate::process::killer::KillSignal;
use crate::process::tree::TreeBuilder;
use crate::process::ProcessInfo;

/// Which detail tab is active in the detail panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailTab {
    Info,
    Log,
    Net,
    Env,
}

impl DetailTab {
    pub fn next(&self) -> Self {
        match self {
            DetailTab::Info => DetailTab::Log,
            DetailTab::Log => DetailTab::Net,
            DetailTab::Net => DetailTab::Env,
            DetailTab::Env => DetailTab::Info,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            DetailTab::Info => DetailTab::Env,
            DetailTab::Log => DetailTab::Info,
            DetailTab::Net => DetailTab::Log,
            DetailTab::Env => DetailTab::Net,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            DetailTab::Info => "Info",
            DetailTab::Log => "Log",
            DetailTab::Net => "Net",
            DetailTab::Env => "Env",
        }
    }

    pub fn all() -> &'static [DetailTab] {
        &[DetailTab::Info, DetailTab::Log, DetailTab::Net, DetailTab::Env]
    }
}

/// What dialog, if any, is currently displayed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogKind {
    KillConfirm,
    KillTreeConfirm,
    SignalPicker,
    ForceKillPrompt,
    Help,
}

/// Column by which the process list can be sorted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Pid,
    Name,
    Port,
    Threads,
    Cpu,
    Memory,
    User,
    Status,
    Uptime,
}

impl SortColumn {
    pub fn next(&self) -> Self {
        match self {
            SortColumn::Pid => SortColumn::Name,
            SortColumn::Name => SortColumn::Port,
            SortColumn::Port => SortColumn::Threads,
            SortColumn::Threads => SortColumn::Cpu,
            SortColumn::Cpu => SortColumn::Memory,
            SortColumn::Memory => SortColumn::User,
            SortColumn::User => SortColumn::Status,
            SortColumn::Status => SortColumn::Uptime,
            SortColumn::Uptime => SortColumn::Pid,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            SortColumn::Pid => "PID",
            SortColumn::Name => "NAME",
            SortColumn::Port => "PORT",
            SortColumn::Threads => "THR",
            SortColumn::Cpu => "CPU",
            SortColumn::Memory => "MEM",
            SortColumn::User => "USER",
            SortColumn::Status => "STATUS",
            SortColumn::Uptime => "UPTIME",
        }
    }
}

/// Main application state for the TUI.
pub struct App {
    pub config: Config,
    /// The process trees (roots with nested children).
    pub process_trees: Vec<ProcessInfo>,
    /// Flattened list of (ProcessInfo, depth) for display.
    pub flat_list: Vec<(ProcessInfo, usize)>,
    /// Currently highlighted row index.
    pub selected_index: usize,
    /// PIDs that are multi-selected (via Space).
    pub selected_pids: HashSet<u32>,
    /// Active detail tab on the bottom panel.
    pub active_tab: DetailTab,
    /// PIDs whose children are expanded in tree view.
    pub expanded_pids: HashSet<u32>,
    /// Currently open dialog, if any.
    pub dialog: Option<DialogKind>,
    /// Index in the signal picker list.
    pub signal_picker_index: usize,
    /// Current filter text.
    pub filter_text: String,
    /// Whether the filter input is active.
    pub filter_active: bool,
    /// Sort column.
    pub sort_column: SortColumn,
    /// Sort direction.
    pub sort_ascending: bool,
    /// Log streamer for the selected process.
    pub log_streamer: Option<LogStreamer>,
    /// Scroll offset for the log tab.
    pub log_scroll: u16,
    /// Scroll offset for the detail panel.
    pub detail_scroll: u16,
    /// Whether the app should quit.
    pub should_quit: bool,
    /// System-wide CPU usage percentage.
    pub system_cpu: f32,
    /// System memory used in bytes.
    pub system_memory_used: u64,
    /// System total memory in bytes.
    pub system_memory_total: u64,
    /// In-progress kill: (pid, when it started).
    pub kill_in_progress: Option<(u32, Instant)>,
    /// Tick counter for animations (e.g. spinner).
    pub tick_count: u64,
    /// Whether this is the first process load (for default expand-all).
    pub first_load: bool,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            process_trees: Vec::new(),
            flat_list: Vec::new(),
            selected_index: 0,
            selected_pids: HashSet::new(),
            active_tab: DetailTab::Info,
            expanded_pids: HashSet::new(),
            dialog: None,
            signal_picker_index: 0,
            filter_text: String::new(),
            filter_active: false,
            sort_column: SortColumn::Pid,
            sort_ascending: true,
            log_streamer: None,
            log_scroll: 0,
            detail_scroll: 0,
            should_quit: false,
            system_cpu: 0.0,
            system_memory_used: 0,
            system_memory_total: 0,
            kill_in_progress: None,
            tick_count: 0,
            first_load: true,
        }
    }

    /// Replace the process trees and rebuild the flat list.
    pub fn update_processes(&mut self, mut processes: Vec<ProcessInfo>) {
        // Apply filter
        if !self.filter_text.is_empty() {
            let filter_lower = self.filter_text.to_lowercase();
            processes.retain(|p| {
                p.name.to_lowercase().contains(&filter_lower)
                    || p.command.to_lowercase().contains(&filter_lower)
                    || p.pid.to_string().contains(&filter_lower)
                    || p.framework.to_string().to_lowercase().contains(&filter_lower)
                    || p.ports.iter().any(|port| port.to_string().contains(&filter_lower))
            });
        }

        // Build trees first, then sort at each level
        self.process_trees = TreeBuilder::build(processes);

        let asc = self.sort_ascending;
        let sort_cmp: Box<dyn Fn(&ProcessInfo, &ProcessInfo) -> std::cmp::Ordering> =
            match self.sort_column {
                SortColumn::Pid => Box::new(move |a: &ProcessInfo, b: &ProcessInfo| {
                    let cmp = a.pid.cmp(&b.pid);
                    if asc { cmp } else { cmp.reverse() }
                }),
                SortColumn::Name => Box::new(move |a: &ProcessInfo, b: &ProcessInfo| {
                    let cmp = a.name.to_lowercase().cmp(&b.name.to_lowercase());
                    if asc { cmp } else { cmp.reverse() }
                }),
                SortColumn::Port => Box::new(move |a: &ProcessInfo, b: &ProcessInfo| {
                    let cmp = a.ports.first().copied().unwrap_or(0).cmp(&b.ports.first().copied().unwrap_or(0));
                    if asc { cmp } else { cmp.reverse() }
                }),
                SortColumn::Threads => Box::new(move |a: &ProcessInfo, b: &ProcessInfo| {
                    let cmp = a.threads.cmp(&b.threads);
                    if asc { cmp } else { cmp.reverse() }
                }),
                SortColumn::Cpu => Box::new(move |a: &ProcessInfo, b: &ProcessInfo| {
                    let cmp = a.cpu_percent.partial_cmp(&b.cpu_percent).unwrap_or(std::cmp::Ordering::Equal);
                    if asc { cmp } else { cmp.reverse() }
                }),
                SortColumn::Memory => Box::new(move |a: &ProcessInfo, b: &ProcessInfo| {
                    let cmp = a.memory_rss.cmp(&b.memory_rss);
                    if asc { cmp } else { cmp.reverse() }
                }),
                SortColumn::User => Box::new(move |a: &ProcessInfo, b: &ProcessInfo| {
                    let cmp = a.user.cmp(&b.user);
                    if asc { cmp } else { cmp.reverse() }
                }),
                SortColumn::Status => Box::new(move |a: &ProcessInfo, b: &ProcessInfo| {
                    let cmp = a.status.cmp(&b.status);
                    if asc { cmp } else { cmp.reverse() }
                }),
                SortColumn::Uptime => Box::new(move |a: &ProcessInfo, b: &ProcessInfo| {
                    let cmp = a.uptime.cmp(&b.uptime);
                    if asc { cmp } else { cmp.reverse() }
                }),
            };
        TreeBuilder::sort_recursive(&mut self.process_trees, &sort_cmp);

        // First load: expand all by default
        if self.first_load {
            self.first_load = false;
            Self::collect_all_pids(&self.process_trees, &mut self.expanded_pids);
        }

        // Flatten for display, respecting expanded state
        self.flat_list = Self::flatten_with_expand(&self.process_trees, &self.expanded_pids);

        // Clamp selected index
        if !self.flat_list.is_empty() {
            if self.selected_index >= self.flat_list.len() {
                self.selected_index = self.flat_list.len() - 1;
            }
        } else {
            self.selected_index = 0;
        }
    }

    /// Flatten trees, only expanding children whose parent PID is in expanded_pids.
    fn flatten_with_expand(trees: &[ProcessInfo], expanded: &HashSet<u32>) -> Vec<(ProcessInfo, usize)> {
        let mut result = Vec::new();
        for tree in trees {
            Self::flatten_recursive(tree, 0, expanded, &mut result);
        }
        result
    }

    fn flatten_recursive(
        node: &ProcessInfo,
        depth: usize,
        expanded: &HashSet<u32>,
        result: &mut Vec<(ProcessInfo, usize)>,
    ) {
        // Clone the node without children for the flat list
        let mut display_node = node.clone();
        // Keep children info for tree indicators but don't recurse unless expanded
        let has_children = !node.children.is_empty();
        // We store children count info in the clone for the tree connector rendering
        if !expanded.contains(&node.pid) {
            display_node.children = node.children.iter().map(|c| {
                let mut stub = ProcessInfo::new(c.pid, &c.name);
                stub.children = Vec::new();
                stub
            }).collect();
        }
        result.push((display_node, depth));

        if has_children && expanded.contains(&node.pid) {
            for child in &node.children {
                Self::flatten_recursive(child, depth + 1, expanded, result);
            }
        }
    }

    /// Get the currently selected process, if any.
    pub fn selected_process(&self) -> Option<&ProcessInfo> {
        self.flat_list.get(self.selected_index).map(|(p, _)| p)
    }

    /// Move selection up.
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.on_selection_changed();
        }
    }

    /// Move selection down.
    pub fn move_down(&mut self) {
        if !self.flat_list.is_empty() && self.selected_index < self.flat_list.len() - 1 {
            self.selected_index += 1;
            self.on_selection_changed();
        }
    }

    /// Toggle expand/collapse of the selected process's children.
    pub fn toggle_expand(&mut self) {
        if let Some((proc, _)) = self.flat_list.get(self.selected_index) {
            let pid = proc.pid;
            // Check if this process has children in the tree
            if self.has_children_in_tree(pid) {
                if self.expanded_pids.contains(&pid) {
                    self.expanded_pids.remove(&pid);
                } else {
                    self.expanded_pids.insert(pid);
                }
                // Rebuild flat list
                self.flat_list = Self::flatten_with_expand(&self.process_trees, &self.expanded_pids);
                if self.selected_index >= self.flat_list.len() && !self.flat_list.is_empty() {
                    self.selected_index = self.flat_list.len() - 1;
                }
            }
        }
    }

    /// Expand the selected node (Right arrow). If already expanded, move to first child.
    pub fn expand_selected(&mut self) {
        if let Some((proc, _)) = self.flat_list.get(self.selected_index) {
            let pid = proc.pid;
            if self.has_children_in_tree(pid) {
                if self.expanded_pids.contains(&pid) {
                    if self.selected_index + 1 < self.flat_list.len() {
                        self.selected_index += 1;
                        self.on_selection_changed();
                    }
                } else {
                    self.expanded_pids.insert(pid);
                    self.flat_list = Self::flatten_with_expand(&self.process_trees, &self.expanded_pids);
                }
            }
        }
    }

    /// Collapse the selected node (Left arrow). If already collapsed, move to parent.
    pub fn collapse_selected(&mut self) {
        if let Some((proc, depth)) = self.flat_list.get(self.selected_index) {
            let pid = proc.pid;
            let depth = *depth;
            if self.expanded_pids.contains(&pid) {
                self.expanded_pids.remove(&pid);
                self.flat_list = Self::flatten_with_expand(&self.process_trees, &self.expanded_pids);
                if self.selected_index >= self.flat_list.len() && !self.flat_list.is_empty() {
                    self.selected_index = self.flat_list.len() - 1;
                }
            } else if depth > 0 {
                // Move to parent
                for i in (0..self.selected_index).rev() {
                    if let Some((_, d)) = self.flat_list.get(i) {
                        if *d < depth {
                            self.selected_index = i;
                            self.on_selection_changed();
                            break;
                        }
                    }
                }
            }
        }
    }

    /// Check if a PID has children anywhere in the tree.
    fn has_children_in_tree(&self, pid: u32) -> bool {
        fn find(trees: &[ProcessInfo], pid: u32) -> bool {
            for tree in trees {
                if tree.pid == pid {
                    return !tree.children.is_empty();
                }
                if find(&tree.children, pid) {
                    return true;
                }
            }
            false
        }
        find(&self.process_trees, pid)
    }

    fn collect_all_pids(trees: &[ProcessInfo], set: &mut HashSet<u32>) {
        for tree in trees {
            if !tree.children.is_empty() {
                set.insert(tree.pid);
                Self::collect_all_pids(&tree.children, set);
            }
        }
    }

    /// Toggle between expand-all and collapse-all.
    pub fn toggle_expand_all(&mut self) {
        if self.expanded_pids.is_empty() {
            Self::collect_all_pids(&self.process_trees, &mut self.expanded_pids);
        } else {
            self.expanded_pids.clear();
        }
        self.flat_list = Self::flatten_with_expand(&self.process_trees, &self.expanded_pids);
        if self.selected_index >= self.flat_list.len() && !self.flat_list.is_empty() {
            self.selected_index = self.flat_list.len() - 1;
        }
    }

    /// Toggle multi-select for the selected process.
    pub fn toggle_select(&mut self) {
        if let Some((proc, _)) = self.flat_list.get(self.selected_index) {
            let pid = proc.pid;
            if self.selected_pids.contains(&pid) {
                self.selected_pids.remove(&pid);
            } else {
                self.selected_pids.insert(pid);
            }
        }
    }

    /// Cycle to the next detail tab.
    pub fn next_tab(&mut self) {
        self.active_tab = self.active_tab.next();
        self.detail_scroll = 0;
    }

    /// Cycle to the previous detail tab.
    pub fn prev_tab(&mut self) {
        self.active_tab = self.active_tab.prev();
        self.detail_scroll = 0;
    }

    /// Toggle sort column and direction.
    pub fn toggle_sort(&mut self) {
        let next = self.sort_column.next();
        if next == self.sort_column {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_column = next;
            self.sort_ascending = true;
        }
    }

    /// Get the currently selected kill signal in the signal picker.
    pub fn selected_kill_signal(&self) -> KillSignal {
        let signals = KillSignal::all();
        signals[self.signal_picker_index % signals.len()]
    }

    /// Called when the selection changes to update log streamer.
    fn on_selection_changed(&mut self) {
        self.log_scroll = 0;
        self.detail_scroll = 0;
    }

    /// Find a process in the original trees by PID (for getting full children info).
    pub fn find_process_in_trees(&self, pid: u32) -> Option<&ProcessInfo> {
        fn find(trees: &[ProcessInfo], pid: u32) -> Option<&ProcessInfo> {
            for tree in trees {
                if tree.pid == pid {
                    return Some(tree);
                }
                if let Some(found) = find(&tree.children, pid) {
                    return Some(found);
                }
            }
            None
        }
        find(&self.process_trees, pid)
    }
}
