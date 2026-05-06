use nsm::process::{FrameworkKind, HealthStatus, ProcessInfo};
use std::time::Duration;

#[test]
fn test_framework_kind_display() {
    assert_eq!(format!("{}", FrameworkKind::NextJs), "Next.js");
    assert_eq!(format!("{}", FrameworkKind::Express), "Express");
    assert_eq!(format!("{}", FrameworkKind::Fastify), "Fastify");
    assert_eq!(format!("{}", FrameworkKind::NestJs), "NestJs");
    assert_eq!(format!("{}", FrameworkKind::Nuxt), "Nuxt");
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
