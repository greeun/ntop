use std::io;
use std::time::Duration;

use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use sysinfo::System;

use ntop::cli::{Cli, Commands, ListFormat};
use ntop::config::Config;
use ntop::log::streamer::LogStreamer;
use ntop::process::framework::FrameworkDetector;
use ntop::process::killer::{GracefulResult, KillSignal, ProcessKiller};
use ntop::process::network::NetworkInspector;
use ntop::process::scanner::ProcessScanner;
use ntop::process::tree::TreeBuilder;
use ntop::process::ProcessInfo;
use ntop::tui::app::{App, DetailTab};
use ntop::tui::event::{AppEvent, EventHandler};
use ntop::tui::ui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = Config::load();

    match cli.command {
        None => run_tui(config).await?,
        Some(Commands::List { json, format }) => cmd_list(&config, json, format)?,
        Some(Commands::Kill {
            pid,
            tree,
            signal,
            all,
            no_confirm,
        }) => cmd_kill(&config, pid, tree, signal, all, no_confirm)?,
        Some(Commands::Info { pid }) => cmd_info(&config, pid)?,
        Some(Commands::Log { pid }) => cmd_log(&config, pid)?,
        Some(Commands::Config) => cmd_config()?,
    }

    Ok(())
}

/// Launch the TUI event loop.
async fn run_tui(config: Config) -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(config.clone());

    // Create a persistent sysinfo::System for accurate CPU readings
    let mut sys = System::new_all();
    sys.refresh_all();

    // Persistent ProcessScanner so per-process CPU deltas accumulate
    // across ticks. Re-creating it each tick would make every process
    // report 0.0% CPU forever (sysinfo computes CPU from refresh deltas).
    let mut scanner = ProcessScanner::new(&config);

    // Event handler with tick rate from config
    let tick_rate = config.refresh_duration();
    let mut events = EventHandler::new(tick_rate);

    // Initial scan
    do_scan(&mut app, &mut scanner, &mut sys);

    // Main event loop
    loop {
        // Render
        terminal.draw(|f| {
            ui::render(f, &mut app);
        })?;

        // Wait for next event
        match events.next().await {
            Some(AppEvent::Tick) => {
                app.tick_count += 1;

                // Rescan processes
                do_scan(&mut app, &mut scanner, &mut sys);

                // Poll logs if Log tab is active
                if app.active_tab == DetailTab::Log {
                    if let Some(ref mut streamer) = app.log_streamer {
                        streamer.poll_new_lines();
                    }
                }
            }
            Some(AppEvent::Key(key)) => {
                ui::handle_key(&mut app, key);

                if app.should_quit {
                    break;
                }

                if app.refresh_changed {
                    app.refresh_changed = false;
                    events = EventHandler::new(Duration::from_secs(app.refresh_secs));
                }

                if app.needs_rescan {
                    app.needs_rescan = false;
                    do_scan(&mut app, &mut scanner, &mut sys);
                }

                // If the selected process changed and Log tab is active,
                // update the log streamer
                if app.active_tab == DetailTab::Log {
                    update_log_streamer(&mut app);
                }
            }
            Some(AppEvent::Resize(_, _)) => {
                // Terminal will re-render on next iteration
            }
            None => {
                break;
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

/// Scan processes, detect frameworks, fetch ports, and update app state.
fn do_scan(app: &mut App, scanner: &mut ProcessScanner<'_>, sys: &mut System) {
    // Refresh system info for CPU/memory readings
    sys.refresh_cpu_usage();
    sys.refresh_memory();

    app.system_cpu = sys.global_cpu_usage();
    app.system_memory_used = sys.used_memory();
    app.system_memory_total = sys.total_memory();

    // Scan Node.js processes (scanner holds a persistent System for CPU deltas)
    let mut processes = scanner.scan();

    // Detect framework and fetch ports for each process
    let net_map = NetworkInspector::connections_by_pid();
    for proc in &mut processes {
        let (framework, version) = FrameworkDetector::detect(&proc.name, &proc.command, &proc.cwd);
        proc.framework = framework;
        proc.framework_version = version;

        // Get listening ports from network data
        if let Some(conns) = net_map.get(&proc.pid) {
            let ports: Vec<u16> = conns
                .iter()
                .filter(|c| c.state == "LISTEN")
                .map(|c| c.local_addr.port())
                .collect();
            if !ports.is_empty() {
                proc.ports = ports;
            }
        }
    }

    app.update_processes(processes);
}

/// Update the log streamer to match the currently selected process.
fn update_log_streamer(app: &mut App) {
    let current_pid = app.selected_process().map(|p| p.pid);
    let streamer_pid = app.log_streamer.as_ref().and_then(|_| {
        // We track which PID the streamer is for by checking if it matches
        app.selected_process().map(|p| p.pid)
    });

    if current_pid != streamer_pid || app.log_streamer.is_none() {
        if let Some(proc) = app.selected_process() {
            let cwd = proc.cwd.clone();
            if !cwd.is_empty() {
                app.log_streamer = Some(LogStreamer::detect_and_open(&cwd));
            } else {
                app.log_streamer = Some(LogStreamer::new());
            }
        } else {
            app.log_streamer = None;
        }
    }
}

// ─── CLI Command Handlers ──────────────────────────────────────────────

/// `ntop list` — scan processes and output in table/json/csv format.
fn cmd_list(config: &Config, json: bool, format: Option<ListFormat>) -> anyhow::Result<()> {
    let mut scanner = ProcessScanner::new(config);
    let mut processes = scanner.scan_blocking();

    // Detect frameworks and fetch ports
    let net_map = NetworkInspector::connections_by_pid();
    for proc in &mut processes {
        let (framework, version) = FrameworkDetector::detect(&proc.name, &proc.command, &proc.cwd);
        proc.framework = framework;
        proc.framework_version = version;

        if let Some(conns) = net_map.get(&proc.pid) {
            let ports: Vec<u16> = conns
                .iter()
                .filter(|c| c.state == "LISTEN")
                .map(|c| c.local_addr.port())
                .collect();
            if !ports.is_empty() {
                proc.ports = ports;
            }
        }
    }

    // Build trees for display
    let trees = TreeBuilder::build(processes);
    let flat = TreeBuilder::flatten_with_depth(&trees);

    let effective_format = if json {
        ListFormat::Json
    } else {
        format.unwrap_or(ListFormat::Table)
    };

    match effective_format {
        ListFormat::Table => print_table(&flat),
        ListFormat::Json => print_json(&flat)?,
        ListFormat::Csv => print_csv(&flat)?,
    }

    Ok(())
}

fn print_table(flat: &[(&ProcessInfo, usize)]) {
    if flat.is_empty() {
        println!("No Node.js processes found.");
        return;
    }

    println!(
        "{:<8} {:<20} {:<12} {:<10} {:<8} {:<10} {:<12}",
        "PID", "NAME", "FRAMEWORK", "PORT", "CPU", "MEM", "UPTIME"
    );
    println!("{}", "-".repeat(80));

    for (proc, depth) in flat {
        let indent = "  ".repeat(*depth);
        let ports_str = if proc.ports.is_empty() {
            "-".to_string()
        } else {
            proc.ports
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(",")
        };

        println!(
            "{:<8} {}{:<width$} {:<12} {:<10} {:<8} {:<10} {:<12}",
            proc.pid,
            indent,
            proc.display_name(),
            proc.framework,
            ports_str,
            format!("{:.1}%", proc.cpu_percent),
            proc.memory_display(),
            proc.uptime_display(),
            width = 20 - indent.len(),
        );
    }
}

fn print_json(flat: &[(&ProcessInfo, usize)]) -> anyhow::Result<()> {
    let items: Vec<serde_json::Value> = flat
        .iter()
        .map(|(proc, depth)| {
            serde_json::json!({
                "pid": proc.pid,
                "ppid": proc.ppid,
                "name": proc.name,
                "is_node": proc.is_node,
                "framework": proc.framework.to_string(),
                "framework_version": proc.framework_version,
                "ports": proc.ports,
                "cpu_percent": proc.cpu_percent,
                "memory_rss": proc.memory_rss,
                "memory_display": proc.memory_display(),
                "uptime_seconds": proc.uptime.as_secs(),
                "uptime_display": proc.uptime_display(),
                "user": proc.user,
                "status": proc.status,
                "depth": depth,
                "cwd": proc.cwd,
                "command": proc.command,
            })
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&items)?);
    Ok(())
}

fn print_csv(flat: &[(&ProcessInfo, usize)]) -> anyhow::Result<()> {
    let mut wtr = csv::Writer::from_writer(io::stdout());
    wtr.write_record(["PID", "PPID", "NAME", "FRAMEWORK", "PORTS", "CPU", "MEMORY", "UPTIME", "STATUS"])?;

    for (proc, _) in flat {
        let ports_str = proc
            .ports
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(";");

        wtr.write_record([
            &proc.pid.to_string(),
            &proc.ppid.to_string(),
            &proc.name,
            &proc.framework.to_string(),
            &ports_str,
            &format!("{:.1}", proc.cpu_percent),
            &proc.memory_rss.to_string(),
            &proc.uptime_display(),
            &proc.status,
        ])?;
    }

    wtr.flush()?;
    Ok(())
}

/// `ntop kill` — kill process(es) by PID, tree, or all.
fn cmd_kill(
    config: &Config,
    pid: Option<u32>,
    tree: bool,
    signal: Option<String>,
    all: bool,
    no_confirm: bool,
) -> anyhow::Result<()> {
    let kill_signal = signal
        .as_deref()
        .and_then(KillSignal::from_str)
        .unwrap_or(KillSignal::Term);

    if all {
        // Kill all Node.js processes
        let mut scanner = ProcessScanner::new(config);
        let processes = scanner.scan();

        if processes.is_empty() {
            println!("No Node.js processes found.");
            return Ok(());
        }

        if !no_confirm && config.general.confirm_before_kill {
            println!("About to kill {} Node.js process(es) with {}:", processes.len(), kill_signal.name());
            for p in &processes {
                println!("  PID {} ({})", p.pid, p.name);
            }
            println!("\nProceed? [y/N] ");
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Cancelled.");
                return Ok(());
            }
        }

        for p in &processes {
            let result = ProcessKiller::send_signal(p.pid, kill_signal);
            println!("  PID {}: {:?}", p.pid, result);
        }
    } else if let Some(target_pid) = pid {
        if tree {
            // Kill process tree
            let mut scanner = ProcessScanner::new(config);
            let processes = scanner.scan();
            let trees = TreeBuilder::build(processes);

            if let Some(node) = find_in_trees(&trees, target_pid) {
                let pids = TreeBuilder::collect_pids(node);

                if !no_confirm && config.general.confirm_before_kill {
                    println!("About to kill process tree (PID {}) with {}:", target_pid, kill_signal.name());
                    for p in &pids {
                        println!("  PID {}", p);
                    }
                    println!("\nProceed? [y/N] ");
                    let mut input = String::new();
                    io::stdin().read_line(&mut input)?;
                    if !input.trim().eq_ignore_ascii_case("y") {
                        println!("Cancelled.");
                        return Ok(());
                    }
                }

                let results = ProcessKiller::kill_tree(&pids, kill_signal);
                for (p, result) in results {
                    println!("  PID {}: {:?}", p, result);
                }
            } else {
                println!("Process {} not found.", target_pid);
            }
        } else {
            // Single process kill
            if !no_confirm && config.general.confirm_before_kill {
                println!("Kill process {} with {}? [y/N] ", target_pid, kill_signal.name());
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Cancelled.");
                    return Ok(());
                }
            }

            let timeout = config.graceful_duration();
            let result = ProcessKiller::graceful_kill(target_pid, timeout);
            match result {
                GracefulResult::Terminated => println!("Process {} terminated gracefully.", target_pid),
                GracefulResult::TimedOut => {
                    println!("Graceful kill timed out. Force killing...");
                    let force_result = ProcessKiller::force_kill(target_pid);
                    println!("Force kill: {:?}", force_result);
                }
                GracefulResult::AlreadyDead => println!("Process {} is already dead.", target_pid),
                GracefulResult::PermissionDenied => println!("Permission denied to kill process {}.", target_pid),
                GracefulResult::Error(e) => println!("Error killing process {}: {}", target_pid, e),
            }
        }
    } else {
        println!("Please specify a PID or use --all.");
    }

    Ok(())
}

/// Find a process node in the tree by PID.
fn find_in_trees(trees: &[ProcessInfo], pid: u32) -> Option<&ProcessInfo> {
    for tree in trees {
        if tree.pid == pid {
            return Some(tree);
        }
        if let Some(found) = find_in_trees(&tree.children, pid) {
            return Some(found);
        }
    }
    None
}

/// `ntop info` — display detailed info about a specific process.
fn cmd_info(config: &Config, pid: u32) -> anyhow::Result<()> {
    let mut scanner = ProcessScanner::new(config);
    let processes = scanner.scan_blocking();

    let proc = processes.iter().find(|p| p.pid == pid);

    match proc {
        Some(process) => {
            let (framework, version) =
                FrameworkDetector::detect(&process.name, &process.command, &process.cwd);
            let connections = NetworkInspector::connections_for_pid(pid);
            let ports: Vec<u16> = connections
                .iter()
                .filter(|c| c.state == "LISTEN")
                .map(|c| c.local_addr.port())
                .collect();

            println!("Process Information");
            println!("{}", "=".repeat(40));
            println!("  PID:       {}", process.pid);
            println!("  PPID:      {}", process.ppid);
            println!("  Name:      {}", process.name);
            println!("  Type:      {}", if process.is_node { "Node" } else { "Tree parent" });
            println!("  Framework: {}", framework);
            println!(
                "  Version:   {}",
                version.as_deref().unwrap_or("-")
            );
            println!(
                "  Ports:     {}",
                if ports.is_empty() {
                    "-".to_string()
                } else {
                    ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", ")
                }
            );
            println!("  CPU:       {:.1}%", process.cpu_percent);
            println!("  Memory:    {}", process.memory_display());
            println!("  Threads:   {}", process.threads);
            println!("  Uptime:    {}", process.uptime_display());
            println!("  User:      {}", process.user);
            println!("  Status:    {}", process.status);
            println!("  Health:    {}", process.health());
            println!("  CWD:       {}", process.cwd);
            println!("  Command:   {}", process.command);
            println!("  Open FDs:  {}", process.open_fds);

            if !process.env_vars.is_empty() {
                println!("\nEnvironment Variables ({})", process.env_vars.len());
                println!("{}", "-".repeat(40));
                for (k, v) in &process.env_vars {
                    println!("  {}={}", k, v);
                }
            }

            if !connections.is_empty() {
                println!("\nNetwork Connections");
                println!("{}", "-".repeat(40));
                for conn in &connections {
                    let remote = conn
                        .remote_addr
                        .map(|a| a.to_string())
                        .unwrap_or_else(|| "-".to_string());
                    println!(
                        "  {} -> {} [{}]",
                        conn.local_addr, remote, conn.state
                    );
                }
            }
        }
        None => {
            println!("Process {} not found.", pid);
        }
    }

    Ok(())
}

/// `ntop log` — stream log output for a process.
fn cmd_log(config: &Config, pid: u32) -> anyhow::Result<()> {
    let mut scanner = ProcessScanner::new(config);
    let processes = scanner.scan();

    let proc = processes.iter().find(|p| p.pid == pid);

    match proc {
        Some(process) => {
            if process.cwd.is_empty() {
                println!("Cannot determine working directory for process {}.", pid);
                return Ok(());
            }

            let mut streamer = LogStreamer::detect_and_open(&process.cwd);

            if !streamer.has_source() {
                println!("No log source found for process {} in {}", pid, process.cwd);
                return Ok(());
            }

            println!(
                "Streaming logs from: {}",
                streamer
                    .source_path()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            );
            println!("Press Ctrl+C to stop.\n");

            loop {
                let new_lines = streamer.poll_new_lines();
                for line in &new_lines {
                    println!("{}", line);
                }

                if !ProcessKiller::is_alive(pid) {
                    println!("\nProcess {} has exited.", pid);
                    break;
                }

                std::thread::sleep(Duration::from_millis(200));
            }
        }
        None => {
            println!("Process {} not found.", pid);
        }
    }

    Ok(())
}

/// `ntop config` — show config file path and current settings.
fn cmd_config() -> anyhow::Result<()> {
    let config = Config::load();
    let path = Config::config_path()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    println!("Config file: {}", path);
    println!();
    println!("Current settings:");
    println!("  [general]");
    println!("    refresh_interval:   {}s", config.general.refresh_interval);
    println!("    default_signal:     {}", config.general.default_signal);
    println!("    graceful_timeout:   {}s", config.general.graceful_timeout);
    println!("    confirm_before_kill: {}", config.general.confirm_before_kill);
    println!("  [display]");
    println!("    show_tree:          {}", config.display.show_tree);
    println!("    color_theme:        {}", config.display.color_theme);
    println!("    mask_env_values:    {}", config.display.mask_env_values);
    println!("  [filter]");
    println!("    include_bun:        {}", config.filter.include_bun);
    println!("    include_tsx:        {}", config.filter.include_tsx);
    println!("    include_ts_node:    {}", config.filter.include_ts_node);

    Ok(())
}
