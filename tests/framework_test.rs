use ntop::config::Config;
use ntop::process::framework::FrameworkDetector;
use ntop::process::{FrameworkKind, Runtime};

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
    // Production Nest: compiled entrypoint at the Nest CLI default path.
    assert_eq!(
        FrameworkDetector::detect_by_command("node dist/main.js"),
        Some(FrameworkKind::NestJs)
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

// ─── classify: runtime + framework ───────────────────────────────────

fn cfg() -> Config { Config::default() }

#[test]
fn test_classify_node_generic() {
    assert_eq!(
        FrameworkDetector::classify("node", "node server.js", &cfg()),
        Some((Runtime::Node, FrameworkKind::Generic))
    );
}

#[test]
fn test_classify_nextjs() {
    assert_eq!(
        FrameworkDetector::classify("next-server", "next-server (v16)", &cfg()),
        Some((Runtime::Node, FrameworkKind::NextJs))
    );
}

#[test]
fn test_classify_nestjs_production() {
    // `node dist/main.js` — Nest CLI's default compiled entrypoint — is
    // tagged Nest even though the command line has no `nest` token.
    assert_eq!(
        FrameworkDetector::classify("node", "node dist/main.js", &cfg()),
        Some((Runtime::Node, FrameworkKind::NestJs))
    );
    // With JVM-style flags before the path, the substring still matches.
    assert_eq!(
        FrameworkDetector::classify(
            "node",
            "node --max-old-space-size=2048 dist/main.js",
            &cfg()
        ),
        Some((Runtime::Node, FrameworkKind::NestJs))
    );
}

#[test]
fn test_classify_nestjs_dev() {
    assert_eq!(
        FrameworkDetector::classify("node", "node node_modules/.bin/nest start", &cfg()),
        Some((Runtime::Node, FrameworkKind::NestJs))
    );
}

#[test]
fn test_classify_python_generic_uvicorn() {
    assert_eq!(
        FrameworkDetector::classify("uvicorn", "/usr/bin/python -m uvicorn app:app", &cfg()),
        Some((Runtime::Python, FrameworkKind::Generic))
    );
}

#[test]
fn test_classify_fastapi_beats_python_name() {
    assert_eq!(
        FrameworkDetector::classify("python", "python -m uvicorn main:app --factory fastapi", &cfg()),
        Some((Runtime::Python, FrameworkKind::FastApi))
    );
}

#[test]
fn test_classify_django() {
    assert_eq!(
        FrameworkDetector::classify("python", "python manage.py runserver", &cfg()),
        Some((Runtime::Python, FrameworkKind::Django))
    );
}

#[test]
fn test_classify_java_generic_and_spring() {
    assert_eq!(
        FrameworkDetector::classify("java", "java -jar app.jar", &cfg()),
        Some((Runtime::Java, FrameworkKind::Generic))
    );
    assert_eq!(
        FrameworkDetector::classify("java", "java org.springframework.boot.loader.JarLauncher", &cfg()),
        Some((Runtime::Java, FrameworkKind::SpringBoot))
    );
}

#[test]
fn test_classify_ruby_and_rails() {
    assert_eq!(
        FrameworkDetector::classify("ruby", "bin/rails server", &cfg()),
        Some((Runtime::Ruby, FrameworkKind::Rails))
    );
    assert_eq!(
        FrameworkDetector::classify("puma", "puma -C config/puma.rb", &cfg()),
        Some((Runtime::Ruby, FrameworkKind::Generic))
    );
}

#[test]
fn test_classify_php_and_laravel() {
    assert_eq!(
        FrameworkDetector::classify("php", "php artisan serve", &cfg()),
        Some((Runtime::Php, FrameworkKind::Laravel))
    );
    assert_eq!(
        FrameworkDetector::classify("php-fpm", "php-fpm", &cfg()),
        Some((Runtime::Php, FrameworkKind::Generic))
    );
}

#[test]
fn test_classify_dotnet_deno_bun() {
    assert_eq!(
        FrameworkDetector::classify("dotnet", "dotnet MyApp.dll", &cfg()),
        Some((Runtime::DotNet, FrameworkKind::Generic))
    );
    assert_eq!(
        FrameworkDetector::classify("deno", "deno run --allow-net server.ts", &cfg()),
        Some((Runtime::Deno, FrameworkKind::Generic))
    );
    assert_eq!(
        FrameworkDetector::classify("bun", "bun run start", &cfg()),
        Some((Runtime::Bun, FrameworkKind::Generic))
    );
}

#[test]
fn test_classify_non_server_is_none() {
    assert_eq!(FrameworkDetector::classify("bash", "bash deploy.sh", &cfg()), None);
    assert_eq!(FrameworkDetector::classify("ssh", "ssh user@host", &cfg()), None);
}

#[test]
fn test_classify_tsx_is_config_gated() {
    let mut c = Config::default();
    assert_eq!(FrameworkDetector::classify("tsx", "tsx watch src/index.ts", &c), None);
    c.filter.include_tsx = true;
    assert_eq!(
        FrameworkDetector::classify("tsx", "tsx watch src/index.ts", &c),
        Some((Runtime::Node, FrameworkKind::Generic))
    );
}

#[test]
fn test_classify_npx_mcp_in_nextjs_cwd_is_node_generic() {
    assert_eq!(
        FrameworkDetector::classify(
            "node",
            "node /Users/u/.npm/_npx/abc/node_modules/.bin/context7-mcp",
            &cfg()
        ),
        Some((Runtime::Node, FrameworkKind::Generic))
    );
}

#[test]
fn test_classify_flask_via_module_flag() {
    // `python -m flask run` is tagged Flask via the tightened `-m flask`
    // substring (a bare path containing "flask" must not trigger it).
    assert_eq!(
        FrameworkDetector::classify("python", "python -m flask run", &cfg()),
        Some((Runtime::Python, FrameworkKind::Flask))
    );
}

#[test]
fn test_classify_rails_via_command_substring() {
    // name "ruby" and first-token basename "ruby" (not "rails"), so the
    // `bin/rails` command substring is what must tag it Rails.
    assert_eq!(
        FrameworkDetector::classify("ruby", "ruby /app/bin/rails server", &cfg()),
        Some((Runtime::Ruby, FrameworkKind::Rails))
    );
}
