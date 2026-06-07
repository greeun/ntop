// Framework + runtime detection rules — the dictionary that drives tagging.
//
// To add a new framework: append a `Rule` to `FRAMEWORK_RULES`.
// To add a new runtime:   append a `Rule` to `RUNTIME_RULES`.
// Then add the variant to `FrameworkKind` / `Runtime` in `process/mod.rs`.
// No other code changes are required; `framework::classify` iterates these.

use super::{FrameworkKind, Runtime};

/// One detection rule = a (runtime, framework) tag plus the matchers that
/// fire it.
///
/// Three signal types, checked in reliability order within a tier:
/// 1. `name_exact`      — exact match against the normalized process name.
/// 2. `command_binary`  — exact match against the basename of the command's
///                        first whitespace-separated token.
/// 3. `command_contains`— substring search in the full command line. Use the
///                        most specific form possible to avoid false positives.
pub struct Rule {
    pub runtime: Runtime,
    pub framework: FrameworkKind,
    pub name_exact: &'static [&'static str],
    pub command_binary: &'static [&'static str],
    pub command_contains: &'static [&'static str],
}

/// Framework-specific rules. Resolved FIRST, as a tier, so a specific
/// framework substring (e.g. `fastapi`) beats a generic runtime name match
/// (e.g. `python`). Within this tier the three signal types apply in
/// priority order across all rules.
pub const FRAMEWORK_RULES: &[Rule] = &[
    Rule {
        runtime: Runtime::Node,
        framework: FrameworkKind::NextJs,
        name_exact: &["next-server", "next-router-worker", "next-router-page-worker"],
        command_binary: &["next-server", "next-router-worker", "next-router-page-worker"],
        command_contains: &["node_modules/.bin/next"],
    },
    Rule {
        runtime: Runtime::Node,
        framework: FrameworkKind::Nuxt,
        name_exact: &["nuxt", "nuxi"],
        command_binary: &["nuxt", "nuxi"],
        command_contains: &["node_modules/.bin/nuxt"],
    },
    Rule {
        runtime: Runtime::Node,
        framework: FrameworkKind::NestJs,
        name_exact: &[],
        command_binary: &[],
        command_contains: &["node_modules/.bin/nest"],
    },
    Rule {
        runtime: Runtime::Python,
        framework: FrameworkKind::Django,
        name_exact: &[],
        command_binary: &[],
        command_contains: &["manage.py", "django"],
    },
    Rule {
        runtime: Runtime::Python,
        framework: FrameworkKind::Flask,
        name_exact: &[],
        command_binary: &[],
        command_contains: &["flask"],
    },
    Rule {
        runtime: Runtime::Python,
        framework: FrameworkKind::FastApi,
        name_exact: &[],
        command_binary: &[],
        command_contains: &["fastapi"],
    },
    Rule {
        runtime: Runtime::Java,
        framework: FrameworkKind::SpringBoot,
        name_exact: &[],
        command_binary: &[],
        command_contains: &["org.springframework", "spring-boot"],
    },
    Rule {
        runtime: Runtime::Ruby,
        framework: FrameworkKind::Rails,
        name_exact: &["rails"],
        command_binary: &["rails"],
        command_contains: &["rails"],
    },
    Rule {
        runtime: Runtime::Php,
        framework: FrameworkKind::Laravel,
        name_exact: &[],
        command_binary: &[],
        command_contains: &["artisan"],
    },
    Rule {
        runtime: Runtime::DotNet,
        framework: FrameworkKind::AspNet,
        name_exact: &[],
        command_binary: &[],
        command_contains: &["Microsoft.AspNetCore", "aspnet"],
    },
];

/// Runtime-generic rules. Resolved SECOND, only when no framework rule fired.
/// These match the bare interpreter / VM by name.
pub const RUNTIME_RULES: &[Rule] = &[
    Rule {
        runtime: Runtime::Node,
        framework: FrameworkKind::Generic,
        name_exact: &["node"],
        command_binary: &["node"],
        command_contains: &[],
    },
    Rule {
        runtime: Runtime::Python,
        framework: FrameworkKind::Generic,
        name_exact: &["python", "python3", "uvicorn", "gunicorn", "hypercorn", "celery"],
        command_binary: &["python", "python3", "uvicorn", "gunicorn", "hypercorn", "celery"],
        command_contains: &[],
    },
    Rule {
        runtime: Runtime::Java,
        framework: FrameworkKind::Generic,
        name_exact: &["java"],
        command_binary: &["java"],
        command_contains: &[],
    },
    Rule {
        runtime: Runtime::Ruby,
        framework: FrameworkKind::Generic,
        name_exact: &["ruby", "puma", "unicorn", "rackup"],
        command_binary: &["ruby", "puma", "unicorn", "rackup"],
        command_contains: &[],
    },
    Rule {
        runtime: Runtime::Php,
        framework: FrameworkKind::Generic,
        name_exact: &["php", "php-fpm"],
        command_binary: &["php", "php-fpm"],
        command_contains: &[],
    },
    Rule {
        runtime: Runtime::DotNet,
        framework: FrameworkKind::Generic,
        name_exact: &["dotnet"],
        command_binary: &["dotnet"],
        command_contains: &[],
    },
    Rule {
        runtime: Runtime::Deno,
        framework: FrameworkKind::Generic,
        name_exact: &["deno"],
        command_binary: &["deno"],
        command_contains: &[],
    },
    Rule {
        runtime: Runtime::Bun,
        framework: FrameworkKind::Generic,
        name_exact: &["bun"],
        command_binary: &["bun"],
        command_contains: &[],
    },
];
