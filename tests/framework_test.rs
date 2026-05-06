use nsm::process::framework::FrameworkDetector;
use nsm::process::FrameworkKind;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_detect_nextjs_by_process_name() {
    assert_eq!(
        FrameworkDetector::detect_by_name("next-server"),
        Some(FrameworkKind::NextJs)
    );
    assert_eq!(
        FrameworkDetector::detect_by_name("next-router-worker"),
        Some(FrameworkKind::NextJs)
    );
}

#[test]
fn test_detect_framework_by_command() {
    assert_eq!(
        FrameworkDetector::detect_by_command("node node_modules/.bin/next start"),
        Some(FrameworkKind::NextJs)
    );
    assert_eq!(
        FrameworkDetector::detect_by_command("node node_modules/.bin/nuxt start"),
        Some(FrameworkKind::Nuxt)
    );
    assert_eq!(
        FrameworkDetector::detect_by_command("node server.js"),
        None
    );
}

#[test]
fn test_detect_framework_by_package_json() {
    // Test Next.js detection
    {
        let dir = TempDir::new().unwrap();
        let pkg = serde_json::json!({
            "name": "my-app",
            "dependencies": {
                "next": "14.0.0",
                "react": "18.0.0"
            }
        });
        fs::write(dir.path().join("package.json"), pkg.to_string()).unwrap();
        let (kind, version) = FrameworkDetector::detect_by_package_json(dir.path().to_str().unwrap());
        assert_eq!(kind, Some(FrameworkKind::NextJs));
        assert_eq!(version, Some("14.0.0".to_string()));
    }

    // Test Express detection
    {
        let dir = TempDir::new().unwrap();
        let pkg = serde_json::json!({
            "name": "my-app",
            "dependencies": {
                "express": "4.18.0"
            }
        });
        fs::write(dir.path().join("package.json"), pkg.to_string()).unwrap();
        let (kind, version) = FrameworkDetector::detect_by_package_json(dir.path().to_str().unwrap());
        assert_eq!(kind, Some(FrameworkKind::Express));
        assert_eq!(version, Some("4.18.0".to_string()));
    }

    // Test Fastify detection
    {
        let dir = TempDir::new().unwrap();
        let pkg = serde_json::json!({
            "name": "my-app",
            "dependencies": {
                "fastify": "4.0.0"
            }
        });
        fs::write(dir.path().join("package.json"), pkg.to_string()).unwrap();
        let (kind, version) = FrameworkDetector::detect_by_package_json(dir.path().to_str().unwrap());
        assert_eq!(kind, Some(FrameworkKind::Fastify));
        assert_eq!(version, Some("4.0.0".to_string()));
    }

    // Test NestJS detection
    {
        let dir = TempDir::new().unwrap();
        let pkg = serde_json::json!({
            "name": "my-app",
            "dependencies": {
                "@nestjs/core": "10.0.0"
            }
        });
        fs::write(dir.path().join("package.json"), pkg.to_string()).unwrap();
        let (kind, version) = FrameworkDetector::detect_by_package_json(dir.path().to_str().unwrap());
        assert_eq!(kind, Some(FrameworkKind::NestJs));
        assert_eq!(version, Some("10.0.0".to_string()));
    }

    // Test Koa detection
    {
        let dir = TempDir::new().unwrap();
        let pkg = serde_json::json!({
            "name": "my-app",
            "dependencies": {
                "koa": "2.14.0"
            }
        });
        fs::write(dir.path().join("package.json"), pkg.to_string()).unwrap();
        let (kind, version) = FrameworkDetector::detect_by_package_json(dir.path().to_str().unwrap());
        assert_eq!(kind, Some(FrameworkKind::Koa));
        assert_eq!(version, Some("2.14.0".to_string()));
    }

    // Test Hapi detection
    {
        let dir = TempDir::new().unwrap();
        let pkg = serde_json::json!({
            "name": "my-app",
            "dependencies": {
                "@hapi/hapi": "21.0.0"
            }
        });
        fs::write(dir.path().join("package.json"), pkg.to_string()).unwrap();
        let (kind, version) = FrameworkDetector::detect_by_package_json(dir.path().to_str().unwrap());
        assert_eq!(kind, Some(FrameworkKind::Hapi));
        assert_eq!(version, Some("21.0.0".to_string()));
    }

    // Test devDependencies detection
    {
        let dir = TempDir::new().unwrap();
        let pkg = serde_json::json!({
            "name": "my-app",
            "devDependencies": {
                "next": "13.0.0"
            }
        });
        fs::write(dir.path().join("package.json"), pkg.to_string()).unwrap();
        let (kind, version) = FrameworkDetector::detect_by_package_json(dir.path().to_str().unwrap());
        assert_eq!(kind, Some(FrameworkKind::NextJs));
        assert_eq!(version, Some("13.0.0".to_string()));
    }
}

#[test]
fn test_detect_combined_priority() {
    // Process name should take priority over command
    let dir = TempDir::new().unwrap();
    let (kind, _version) = FrameworkDetector::detect(
        "next-server",
        "node node_modules/.bin/nuxt start",
        dir.path().to_str().unwrap(),
    );
    assert_eq!(kind, FrameworkKind::NextJs);
}

#[test]
fn test_detect_fallback_to_generic() {
    let dir = TempDir::new().unwrap();
    let (kind, version) = FrameworkDetector::detect(
        "node",
        "node server.js",
        dir.path().to_str().unwrap(),
    );
    assert_eq!(kind, FrameworkKind::Generic);
    assert_eq!(version, None);
}
