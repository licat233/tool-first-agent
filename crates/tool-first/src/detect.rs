use crate::registry::{Registry, ToolSpec};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::env;
use std::path::{Path, PathBuf};

/// Well-known bin directories to check.
const KNOWN_DIRS: &[&str] = &[
    "~/.local/bin",
    "~/.hermes/bin",
    "~/.cargo/bin",
    "/opt/homebrew/bin",
    "/usr/local/bin",
    "/usr/bin",
    "/bin",
];

/// Result of detecting a single tool.
#[derive(Debug, Serialize)]
pub struct DetectionResult {
    pub namespace: String,
    pub memory_type: String,
    pub record_type: String,
    pub category: String,
    pub tool: String,
    pub status: String,
    pub path: String,
    pub version: String,
    pub detection_method: String,
    pub checked_at: String,
    pub path_fingerprint: String,
    pub confidence: f64,
    pub tags: Vec<String>,
}

/// Detect all candidate tools in a category or a specific list of tool names.
pub fn detect(
    registry: &Registry,
    category: Option<&str>,
    tools: &[String],
) -> Vec<DetectionResult> {
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let fp = path_fingerprint();
    let mut results = Vec::new();

    let candidates = collect_candidates(registry, category, tools);

    for (cat, tool_name, spec) in candidates {
        let detect_names = if spec.detect_names.is_empty() {
            vec![tool_name.clone()]
        } else {
            spec.detect_names.clone()
        };

        let (path, method) =
            find_executable(&detect_names, &spec.known_paths, &spec.app_bundle_paths);

        let (version, version_ok) = if path.is_some() && !spec.version_args.is_empty() {
            get_version(path.as_deref().unwrap(), &spec.version_args)
        } else {
            (String::new(), path.is_some())
        };

        let (status, confidence) = if path.is_some() && version_ok {
            ("available".to_string(), 0.98)
        } else if path.is_some() {
            ("present_unverified".to_string(), 0.7)
        } else {
            ("missing".to_string(), 0.2)
        };

        results.push(DetectionResult {
            namespace: "agent_tool_inventory".to_string(),
            memory_type: "tool_inventory".to_string(),
            record_type: "availability".to_string(),
            category: cat,
            tool: tool_name,
            status,
            path: path.unwrap_or_default(),
            version,
            detection_method: method,
            checked_at: now.clone(),
            path_fingerprint: fp.clone(),
            confidence,
            tags: vec!["tool-inventory".to_string()],
        });
    }

    results
}

/// Collect all candidate (category, tool, spec) tuples.
fn collect_candidates<'a>(
    registry: &'a Registry,
    category: Option<&str>,
    tools: &[String],
) -> Vec<(String, String, &'a ToolSpec)> {
    let mut rows = Vec::new();
    for (cat, section) in registry {
        if let Some(filter_cat) = category {
            if cat != filter_cat {
                continue;
            }
        }
        for (tool_name, spec) in &section.tools {
            if !tools.is_empty() {
                let matches_name = tools.contains(tool_name);
                let matches_detect = spec.detect_names.iter().any(|d| tools.contains(d));
                if !matches_name && !matches_detect {
                    continue;
                }
            }
            rows.push((cat.clone(), tool_name.clone(), spec));
        }
    }
    rows
}

/// Find an executable for the given names.
fn find_executable(
    names: &[String],
    known_paths: &[String],
    app_paths: &[String],
) -> (Option<String>, String) {
    // 1. Try `which` (shutil.which equivalent)
    for name in names {
        if let Ok(path) = which::which(name) {
            return (
                Some(path.to_string_lossy().to_string()),
                "which".to_string(),
            );
        }
    }

    // 2. Try known_paths and app_bundle_paths
    for path_str in known_paths.iter().chain(app_paths.iter()) {
        let expanded = shellexpand(path_str);
        let p = PathBuf::from(&expanded);
        if p.exists() && is_executable(&p) {
            return (
                Some(p.to_string_lossy().to_string()),
                "known_path".to_string(),
            );
        }
    }

    // 3. Try well-known bin directories
    for dir_str in KNOWN_DIRS {
        let dir = PathBuf::from(shellexpand(dir_str));
        for name in names {
            let p = dir.join(name);
            if p.exists() && is_executable(&p) {
                return (
                    Some(p.to_string_lossy().to_string()),
                    "known_bin_dir".to_string(),
                );
            }
        }
    }

    (None, "not_found".to_string())
}

/// Check if a path is executable (Unix: has +x bit).
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

/// Get version string by running the tool with version args.
fn get_version(path: &str, args: &[String]) -> (String, bool) {
    match std::process::Command::new(path).args(args).output() {
        Ok(output) => {
            let text = if !output.stdout.is_empty() {
                String::from_utf8_lossy(&output.stdout)
            } else {
                String::from_utf8_lossy(&output.stderr)
            };
            let first_line = text
                .lines()
                .next()
                .unwrap_or("")
                .chars()
                .take(240)
                .collect();
            (first_line, output.status.success() || !text.is_empty())
        }
        Err(e) => (format!("{e}"), false),
    }
}

/// SHA-256 fingerprint of the PATH environment variable (truncated to 16 hex chars).
fn path_fingerprint() -> String {
    let path = env::var("PATH").unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(path.as_bytes());
    let result = format!("{:x}", hasher.finalize());
    result[..16].to_string()
}

fn shellexpand(s: &str) -> String {
    if s.starts_with("~/") || s == "~" {
        if let Some(home) = dirs::home_dir() {
            return format!("{}{}", home.display(), &s[1..]);
        }
    }
    s.to_string()
}
