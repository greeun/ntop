use ntop::process::framework::FrameworkDetector;
use ntop::process::FrameworkKind;

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
    assert_eq!(
        FrameworkDetector::detect_by_name("next-router-page-worker"),
        Some(FrameworkKind::NextJs)
    );
}

#[test]
fn test_detect_nuxt_by_process_name() {
    assert_eq!(
        FrameworkDetector::detect_by_name("nuxt"),
        Some(FrameworkKind::Nuxt)
    );
    assert_eq!(
        FrameworkDetector::detect_by_name("nuxi"),
        Some(FrameworkKind::Nuxt)
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
        FrameworkDetector::detect_by_command("node node_modules/.bin/nest start"),
        Some(FrameworkKind::NestJs)
    );
    assert_eq!(
        FrameworkDetector::detect_by_command("node server.js"),
        None
    );
}

#[test]
fn test_detect_combined_priority() {
    // Process name takes priority over command keywords.
    let (kind, version) = FrameworkDetector::detect(
        "next-server",
        "node node_modules/.bin/nuxt start",
        "",
    );
    assert_eq!(kind, FrameworkKind::NextJs);
    assert_eq!(version, None);
}

#[test]
fn test_detect_nuxt_by_process_name_priority() {
    let (kind, _) = FrameworkDetector::detect(
        "nuxt",
        "node node_modules/.bin/next start",
        "",
    );
    assert_eq!(kind, FrameworkKind::Nuxt);
}

#[test]
fn test_detect_fallback_to_generic() {
    let (kind, version) = FrameworkDetector::detect(
        "node",
        "node server.js",
        "",
    );
    assert_eq!(kind, FrameworkKind::Generic);
    assert_eq!(version, None);
}

/// Regression: an npx-launched MCP server (context7-mcp) inheriting a
/// Next.js project's cwd must NOT be tagged as Next.js. Detection is
/// process-local and ignores the cwd.
#[test]
fn test_detect_npx_mcp_server_is_generic_even_in_nextjs_cwd() {
    let (kind, version) = FrameworkDetector::detect(
        "node",
        "node /Users/u/.npm/_npx/abc123/node_modules/.bin/context7-mcp",
        "/Users/u/some-nextjs-project",
    );
    assert_eq!(kind, FrameworkKind::Generic);
    assert_eq!(version, None);
}

/// Regression: cwd is never consulted, so a node process started inside
/// a Next.js project running an unrelated script stays Generic.
#[test]
fn test_detect_node_in_framework_cwd_is_generic() {
    let (kind, _) = FrameworkDetector::detect(
        "node",
        "node /tmp/scratch.js",
        "/some/nextjs/project",
    );
    assert_eq!(kind, FrameworkKind::Generic);
}

/// Regression: macOS truncates the `comm` field to 16 chars, so a
/// next-server worker that calls `process.title = "next-server (v16.2.4)"`
/// shows up as `name = "next-server (v16"`. Detection must still tag it
/// as Next.js by normalizing the name.
#[test]
fn test_detect_nextjs_from_truncated_macos_comm() {
    let (kind, _) = FrameworkDetector::detect(
        "next-server (v16",
        "next-server (v16.2.4)",
        "",
    );
    assert_eq!(kind, FrameworkKind::NextJs);
}

/// Regression: if sysinfo name is opaque (e.g. "node") but the command
/// starts with a known framework binary (process.title style), detect it.
#[test]
fn test_detect_nextjs_from_command_binary_when_name_is_node() {
    let (kind, _) = FrameworkDetector::detect(
        "node",
        "next-server (v16.2.4)",
        "",
    );
    assert_eq!(kind, FrameworkKind::NextJs);
}
