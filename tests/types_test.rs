use ntop::process::{FrameworkKind, HealthStatus, ProcessInfo, Runtime};
use std::time::Duration;

#[test]
fn test_framework_kind_display() {
    assert_eq!(format!("{}", FrameworkKind::NextJs), "Next.js");
    assert_eq!(format!("{}", FrameworkKind::Express), "Express");
    assert_eq!(format!("{}", FrameworkKind::Fastify), "Fastify");
    assert_eq!(format!("{}", FrameworkKind::NestJs), "NestJs");
    assert_eq!(format!("{}", FrameworkKind::Nuxt), "Nuxt.js");
    assert_eq!(format!("{}", FrameworkKind::Koa), "Koa");
    assert_eq!(format!("{}", FrameworkKind::Hapi), "Hapi");
    assert_eq!(format!("{}", FrameworkKind::Generic), "Generic");
}

#[test]
fn test_health_status_from_metrics() {
    assert_eq!(HealthStatus::from_cpu_mem(50.0, 50.0), HealthStatus::Healthy);
    assert_eq!(HealthStatus::from_cpu_mem(85.0, 50.0), HealthStatus::Warning);
    assert_eq!(HealthStatus::from_cpu_mem(50.0, 85.0), HealthStatus::Warning);
    assert_eq!(HealthStatus::from_cpu_mem(95.0, 50.0), HealthStatus::Critical);
    assert_eq!(HealthStatus::from_cpu_mem(50.0, 95.0), HealthStatus::Critical);
}

#[test]
fn test_health_status_zombie() {
    assert_eq!(
        HealthStatus::from_process_status("Zombie"),
        HealthStatus::Critical
    );
    assert_eq!(
        HealthStatus::from_process_status("Dead"),
        HealthStatus::Critical
    );
    assert_eq!(
        HealthStatus::from_process_status("Running"),
        HealthStatus::Healthy
    );
}

#[test]
fn test_process_info_default() {
    let info = ProcessInfo::new(1234, "node");
    assert_eq!(info.pid, 1234);
    assert_eq!(info.name, "node");
    assert_eq!(info.ppid, 0);
    assert_eq!(info.framework, FrameworkKind::Generic);
    assert_eq!(info.cpu_percent, 0.0);
    assert_eq!(info.memory_rss, 0);
    assert!(info.ports.is_empty());
    assert!(info.children.is_empty());
    assert!(info.env_vars.is_empty());
    assert_eq!(info.status, "Running");
    assert!(!info.is_node);
}

#[test]
fn test_process_info_uptime_display() {
    let mut info = ProcessInfo::new(1, "node");
    info.uptime = Duration::from_secs(3725); // 1h 2m 5s
    assert_eq!(info.uptime_display(), "1h 2m 5s");

    info.uptime = Duration::from_secs(65); // 1m 5s
    assert_eq!(info.uptime_display(), "1m 5s");

    info.uptime = Duration::from_secs(5); // 5s
    assert_eq!(info.uptime_display(), "5s");
}

#[test]
fn test_process_info_memory_display() {
    let mut info = ProcessInfo::new(1, "node");
    info.memory_rss = 134_217_728; // 128 MB
    assert_eq!(info.memory_display(), "128.0 MB");

    info.memory_rss = 1_073_741_824; // 1 GB
    assert_eq!(info.memory_display(), "1.0 GB");

    info.memory_rss = 512;
    assert_eq!(info.memory_display(), "512 B");

    info.memory_rss = 2048;
    assert_eq!(info.memory_display(), "2.0 KB");
}

// ─── display_name ────────────────────────────────────────────────────

#[test]
fn test_display_name_empty_command_returns_process_name() {
    let info = ProcessInfo::new(1, "my-server");
    assert_eq!(info.display_name(), "my-server");
}

#[test]
fn test_display_name_non_node_binary_no_args() {
    let mut info = ProcessInfo::new(1, "custom-server");
    info.command = "/usr/local/bin/custom-server".to_string();
    assert_eq!(info.display_name(), "custom-server");
}

#[test]
fn test_display_name_non_node_binary_with_subcommand() {
    let mut info = ProcessInfo::new(1, "bun");
    info.command = "bun run dev".to_string();
    assert_eq!(info.display_name(), "bun run dev");
}

#[test]
fn test_display_name_non_node_binary_with_script() {
    let mut info = ProcessInfo::new(1, "bun");
    info.command = "bun server.js".to_string();
    assert_eq!(info.display_name(), "bun server.js");
}

#[test]
fn test_display_name_node_with_modules_tool_and_subcommand() {
    let mut info = ProcessInfo::new(1, "node");
    info.command = "node node_modules/.bin/next start".to_string();
    assert_eq!(info.display_name(), "next start");
}

#[test]
fn test_display_name_node_with_modules_tool_only() {
    let mut info = ProcessInfo::new(1, "node");
    info.command = "node node_modules/.bin/next".to_string();
    assert_eq!(info.display_name(), "next");
}

#[test]
fn test_display_name_node_with_script() {
    let mut info = ProcessInfo::new(1, "node");
    info.command = "node server.js".to_string();
    assert_eq!(info.display_name(), "node server.js");
}

#[test]
fn test_display_name_node_stops_at_json_blob() {
    let mut info = ProcessInfo::new(1, "node");
    info.command = "node server.js {envdata}".to_string();
    assert_eq!(info.display_name(), "node server.js");
}

#[test]
fn test_display_name_node_no_args_returns_name() {
    let mut info = ProcessInfo::new(1, "node");
    info.command = "node".to_string();
    assert_eq!(info.display_name(), "node");
}

// ─── health ──────────────────────────────────────────────────────────

#[test]
fn test_health_zombie_status_overrides_healthy_metrics() {
    let mut info = ProcessInfo::new(1, "node");
    info.status = "Zombie".to_string();
    info.cpu_percent = 0.0;
    info.memory_rss = 0;
    assert_eq!(info.health(), HealthStatus::Critical);
}

#[test]
fn test_health_running_with_low_resources_is_healthy() {
    let mut info = ProcessInfo::new(1, "node");
    info.status = "Running".to_string();
    info.cpu_percent = 10.0;
    info.memory_rss = 64 * 1_048_576; // 64 MB
    assert_eq!(info.health(), HealthStatus::Healthy);
}

#[test]
fn test_health_high_cpu_is_warning() {
    let mut info = ProcessInfo::new(1, "node");
    info.status = "Running".to_string();
    info.cpu_percent = 85.0;
    info.memory_rss = 0;
    assert_eq!(info.health(), HealthStatus::Warning);
}

#[test]
fn test_health_very_high_cpu_is_critical() {
    let mut info = ProcessInfo::new(1, "node");
    info.status = "Running".to_string();
    info.cpu_percent = 95.0;
    info.memory_rss = 0;
    assert_eq!(info.health(), HealthStatus::Critical);
}

#[test]
fn test_framework_kind_serialization() {
    let original = FrameworkKind::NextJs;
    let json = serde_json::to_string(&original).unwrap();
    let deserialized: FrameworkKind = serde_json::from_str(&json).unwrap();
    assert_eq!(original, deserialized);

    // Test all variants roundtrip
    let variants = vec![
        FrameworkKind::NextJs,
        FrameworkKind::Express,
        FrameworkKind::Fastify,
        FrameworkKind::NestJs,
        FrameworkKind::Nuxt,
        FrameworkKind::Koa,
        FrameworkKind::Hapi,
        FrameworkKind::Generic,
    ];
    for variant in variants {
        let json = serde_json::to_string(&variant).unwrap();
        let back: FrameworkKind = serde_json::from_str(&json).unwrap();
        assert_eq!(variant, back);
    }
}

#[test]
fn test_runtime_display() {
    assert_eq!(format!("{}", Runtime::Node), "Node");
    assert_eq!(format!("{}", Runtime::Python), "Python");
    assert_eq!(format!("{}", Runtime::Java), "Java");
    assert_eq!(format!("{}", Runtime::Deno), "Deno");
    assert_eq!(format!("{}", Runtime::Bun), "Bun");
    assert_eq!(format!("{}", Runtime::Ruby), "Ruby");
    assert_eq!(format!("{}", Runtime::Php), "PHP");
    assert_eq!(format!("{}", Runtime::DotNet), ".NET");
}

#[test]
fn test_runtime_serialization() {
    let variants = vec![
        Runtime::Node, Runtime::Python, Runtime::Java, Runtime::Deno,
        Runtime::Bun, Runtime::Ruby, Runtime::Php, Runtime::DotNet,
    ];
    for v in variants {
        let json = serde_json::to_string(&v).unwrap();
        let back: Runtime = serde_json::from_str(&json).unwrap();
        assert_eq!(v, back);
    }
}

#[test]
fn test_new_framework_kind_display() {
    assert_eq!(format!("{}", FrameworkKind::FastApi), "FastAPI");
    assert_eq!(format!("{}", FrameworkKind::Flask), "Flask");
    assert_eq!(format!("{}", FrameworkKind::Django), "Django");
    assert_eq!(format!("{}", FrameworkKind::SpringBoot), "Spring Boot");
    assert_eq!(format!("{}", FrameworkKind::Rails), "Rails");
    assert_eq!(format!("{}", FrameworkKind::Laravel), "Laravel");
    assert_eq!(format!("{}", FrameworkKind::AspNet), "ASP.NET");
}
