// TUI application state

use std::collections::HashSet;
use std::time::Instant;

use ratatui::widgets::TableState;

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

/// Which panel currently has keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPanel {
    ProcessList,
    DetailPanel,
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
    /// Last scan result, pre-filter. Used to re-apply the filter
    /// on every keystroke without waiting for the next scan tick.
    pub raw_processes: Vec<ProcessInfo>,
    /// The process trees (roots with nested children), filtered.
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
    /// Total content lines in the detail panel (set during render).
    pub detail_content_lines: u16,
    /// Visible height of the detail content area (set during render).
    pub detail_view_height: u16,
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
    /// PIDs seen in the previous scan. Used to detect freshly-started
    /// processes so they can inherit the default expand state.
    pub known_pids: HashSet<u32>,
    /// Whether newly-seen parents default to expanded. Starts true (so the
    /// first scan expands all) and flips with collapse-all/expand-all.
    pub default_expanded: bool,
    /// Scroll offset for the help dialog.
    pub help_scroll: u16,
    /// Max scroll offset for the help dialog (computed during render).
    pub help_max_scroll: u16,
    /// Refresh interval in seconds (adjustable at runtime).
    pub refresh_secs: u64,
    /// Whether refresh interval was changed and event loop needs restart.
    pub refresh_changed: bool,
    /// Whether a rescan is needed (e.g. after killing a process).
    pub needs_rescan: bool,
    /// Table state for scrolling the process list.
    pub table_state: TableState,
    /// Which panel currently has keyboard focus.
    pub focus: FocusPanel,
    /// When true, the list shows only Node servers (+ tree-context parents).
    pub node_only: bool,
}

impl App {
    pub fn new(config: Config) -> Self {
        let refresh = config.general.refresh_interval;
        Self {
            config,
            raw_processes: Vec::new(),
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
            detail_content_lines: 0,
            detail_view_height: 0,
            should_quit: false,
            system_cpu: 0.0,
            system_memory_used: 0,
            system_memory_total: 0,
            kill_in_progress: None,
            tick_count: 0,
            known_pids: HashSet::new(),
            default_expanded: true,
            help_scroll: 0,
            help_max_scroll: 0,
            refresh_secs: refresh,
            refresh_changed: false,
            needs_rescan: false,
            table_state: TableState::default().with_selected(Some(0)),
            focus: FocusPanel::ProcessList,
            node_only: false,
        }
    }

    /// True if `p` matches `filter` (case-insensitive substring on
    /// textual fields; digit substring on pid/ports). Empty filter
    /// matches everything.
    pub fn matches_filter(p: &ProcessInfo, filter: &str) -> bool {
        if filter.is_empty() {
            return true;
        }
        let f = filter.to_lowercase();
        p.name.to_lowercase().contains(&f)
            || p.command.to_lowercase().contains(&f)
            || p.pid.to_string().contains(&f)
            || p.framework.to_string().to_lowercase().contains(&f)
            || p.runtime.is_some_and(|r| r.to_string().to_lowercase().contains(&f))
            || p.ports.iter().any(|port| port.to_string().contains(&f))
    }

    /// Store a fresh scan result and rebuild the view from it. Any PID not
    /// seen in the previous scan is treated as freshly started: while
    /// `default_expanded` is set (the first scan and after expand-all), it
    /// inherits the expanded state so newly-launched servers appear expanded
    /// rather than collapsed. A prior collapse-all (`default_expanded=false`)
    /// keeps new processes collapsed.
    pub fn update_processes(&mut self, processes: Vec<ProcessInfo>) {
        if self.default_expanded {
            for p in &processes {
                if !self.known_pids.contains(&p.pid) {
                    self.expanded_pids.insert(p.pid);
                }
            }
        }
        self.known_pids = processes.iter().map(|p| p.pid).collect();
        self.raw_processes = processes;

        self.rebuild_view();
    }

    /// Re-apply the current filter to `raw_processes` and rebuild the
    /// tree, sort, and flat list. Call this whenever `filter_text`
    /// changes so results update on every keystroke (instead of waiting
    /// for the next scan tick).
    pub fn rebuild_view(&mut self) {
        let node_only = self.node_only;
        let processes: Vec<ProcessInfo> = self
            .raw_processes
            .iter()
            // Node-only keeps strictly Node servers (Deno/Bun are separate
            // runtimes and are hidden) plus tree-context parents (runtime None).
            .filter(|p| !node_only || p.is_node() || p.runtime.is_none())
            .filter(|p| Self::matches_filter(p, &self.filter_text))
            .cloned()
            .collect();

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
                    // Ports live on deep leaves (e.g. next-server), not on the
                    // shell/runner roots, so compare each subtree's lowest port
                    // — otherwise portless roots all tie and the tree won't sort.
                    let cmp = Self::subtree_port_key(a).cmp(&Self::subtree_port_key(b));
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
        self.sync_table_state();
    }

    /// Lowest non-zero port found in `p` or any descendant; 0 if none in the
    /// subtree. Lets port-sort order whole tree branches by the port that
    /// actually appears in them, rather than the (portless) root's own port.
    fn subtree_port_key(p: &ProcessInfo) -> u16 {
        let mut best = p.ports.first().copied().unwrap_or(0);
        for c in &p.children {
            let cp = Self::subtree_port_key(c);
            if cp != 0 && (best == 0 || cp < best) {
                best = cp;
            }
        }
        best
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
            if self.has_children_in_tree(pid) {
                if self.expanded_pids.contains(&pid) {
                    self.expanded_pids.remove(&pid);
                } else {
                    self.expanded_pids.insert(pid);
                }
                self.flat_list = Self::flatten_with_expand(&self.process_trees, &self.expanded_pids);
                if self.selected_index >= self.flat_list.len() && !self.flat_list.is_empty() {
                    self.selected_index = self.flat_list.len() - 1;
                }
                self.sync_table_state();
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
                    self.sync_table_state();
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
                self.sync_table_state();
            } else if depth > 0 {
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
            self.default_expanded = true;
        } else {
            self.expanded_pids.clear();
            self.default_expanded = false;
        }
        self.flat_list = Self::flatten_with_expand(&self.process_trees, &self.expanded_pids);
        if self.selected_index >= self.flat_list.len() && !self.flat_list.is_empty() {
            self.selected_index = self.flat_list.len() - 1;
        }
        self.sync_table_state();
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

    /// Toggle the Node-only view filter and rebuild.
    pub fn toggle_node_only(&mut self) {
        self.node_only = !self.node_only;
        self.rebuild_view();
    }

    /// Get the currently selected kill signal in the signal picker.
    pub fn selected_kill_signal(&self) -> KillSignal {
        let signals = KillSignal::all();
        signals[self.signal_picker_index % signals.len()]
    }

    /// Sync table_state.selected with selected_index.
    fn sync_table_state(&mut self) {
        if self.flat_list.is_empty() {
            self.table_state.select(None);
        } else {
            self.table_state.select(Some(self.selected_index));
        }
    }

    /// Called when the selection changes to update log streamer.
    fn on_selection_changed(&mut self) {
        self.log_scroll = 0;
        self.detail_scroll = 0;
        self.sync_table_state();
    }

    /// Max scroll offset for the detail panel.
    pub fn detail_max_scroll(&self) -> u16 {
        self.detail_content_lines.saturating_sub(self.detail_view_height)
    }

    /// Clamp detail/log scroll to valid range.
    pub fn clamp_detail_scroll(&mut self) {
        let max = self.detail_max_scroll();
        self.detail_scroll = self.detail_scroll.min(max);
        self.log_scroll = self.log_scroll.min(max);
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
