// Framework detection logic

use super::FrameworkKind;
use std::fs;
use std::path::Path;

/// Maps known process names to their framework kind.
const NAME_MAP: &[(&str, FrameworkKind)] = &[
    ("next-server", FrameworkKind::NextJs),
    ("next-router-worker", FrameworkKind::NextJs),
    ("next-router-page-worker", FrameworkKind::NextJs),
];

/// Keywords found in command strings that indicate a framework.
const COMMAND_KEYWORDS: &[(&str, FrameworkKind)] = &[
    ("next", FrameworkKind::NextJs),
    ("nuxt", FrameworkKind::Nuxt),
    ("nest", FrameworkKind::NestJs),
];

/// Package dependency names mapped to framework kinds, checked in priority order.
const PACKAGE_DEPS: &[(&str, FrameworkKind)] = &[
    ("next", FrameworkKind::NextJs),
    ("nuxt", FrameworkKind::Nuxt),
    ("@nestjs/core", FrameworkKind::NestJs),
    ("express", FrameworkKind::Express),
    ("fastify", FrameworkKind::Fastify),
    ("koa", FrameworkKind::Koa),
    ("@hapi/hapi", FrameworkKind::Hapi),
];

/// Detects which Node.js framework a process is running.
///
/// Uses a priority-based approach:
/// 1. Process name matching
/// 2. Command string keyword matching
/// 3. package.json dependency scanning
/// 4. Falls back to Generic
pub struct FrameworkDetector;

impl FrameworkDetector {
    /// Full detection with priority: name -> command -> package.json -> Generic.
    ///
    /// Returns the detected framework kind and an optional version string
    /// (only available when detected via package.json).
    pub fn detect(name: &str, command: &str, cwd: &str) -> (FrameworkKind, Option<String>) {
        // Priority 1: process name
        if let Some(kind) = Self::detect_by_name(name) {
            return (kind, None);
        }

        // Priority 2: command keywords
        if let Some(kind) = Self::detect_by_command(command) {
            return (kind, None);
        }

        // Priority 3: package.json
        let (kind, version) = Self::detect_by_package_json(cwd);
        if let Some(kind) = kind {
            return (kind, version);
        }

        // Fallback
        (FrameworkKind::Generic, None)
    }

    /// Detect framework by matching the process name against known names.
    pub fn detect_by_name(name: &str) -> Option<FrameworkKind> {
        for (known_name, kind) in NAME_MAP {
            if name == *known_name {
                return Some(kind.clone());
            }
        }
        None
    }

    /// Detect framework by scanning the command string for known binary paths.
    ///
    /// Looks for patterns like `node_modules/.bin/<keyword>` in the command.
    pub fn detect_by_command(command: &str) -> Option<FrameworkKind> {
        for (keyword, kind) in COMMAND_KEYWORDS {
            let pattern = format!("node_modules/.bin/{}", keyword);
            if command.contains(&pattern) {
                return Some(kind.clone());
            }
        }
        None
    }

    /// Detect framework by reading and parsing the package.json in the given directory.
    ///
    /// Checks both `dependencies` and `devDependencies` against the known
    /// framework package list. Returns the framework kind and its version string
    /// if found.
    pub fn detect_by_package_json(cwd: &str) -> (Option<FrameworkKind>, Option<String>) {
        let pkg_path = Path::new(cwd).join("package.json");
        let content = match fs::read_to_string(&pkg_path) {
            Ok(c) => c,
            Err(_) => return (None, None),
        };

        let json: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => return (None, None),
        };

        // Check dependencies first, then devDependencies
        for (dep_name, kind) in PACKAGE_DEPS {
            if let Some(version) = json
                .get("dependencies")
                .and_then(|deps| deps.get(*dep_name))
                .and_then(|v| v.as_str())
            {
                return (Some(kind.clone()), Some(version.to_string()));
            }

            if let Some(version) = json
                .get("devDependencies")
                .and_then(|deps| deps.get(*dep_name))
                .and_then(|v| v.as_str())
            {
                return (Some(kind.clone()), Some(version.to_string()));
            }
        }

        (None, None)
    }
}
