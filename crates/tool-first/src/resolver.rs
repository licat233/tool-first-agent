use crate::config::Config;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// The canonical `.tool-memory-home` marker filename.
pub const MARKER_FILENAME: &str = ".tool-memory-home";

/// Content of the `.tool-memory-home` marker file.
#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryHomeMarker {
    #[serde(rename = "type")]
    pub marker_type: String,
    pub version: String,
    pub canonical: bool,
    pub source: String,
    pub adapter: String,
    pub authority: String,
    pub vault_authority: String,
    pub description: String,
}

impl Default for MemoryHomeMarker {
    fn default() -> Self {
        Self {
            marker_type: "tool-first-agent-memory-home".to_string(),
            version: "1.0".to_string(),
            canonical: true,
            source: "TOOL_FIRST_MEMORY_HOME".to_string(),
            adapter: "file".to_string(),
            authority: "runtime-infrastructure".to_string(),
            vault_authority: "none".to_string(),
            description: "Canonical shared runtime tool-memory home for local agents. Not authoritative Vault memory.".to_string(),
        }
    }
}

/// Content of the `.tool-memory-redirect` marker file.
#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryRedirectMarker {
    pub redirect_to: String,
    pub reason: String,
    pub do_not_write_here: bool,
}

/// Resolve the canonical tool-memory home directory.
///
/// Priority:
///   1. `TOOL_FIRST_MEMORY_HOME` env var (highest)
///   2. `config.memory_home`
///   3. `config.file.base_dir`
///   4. `~/.config/tool-first-agent/tool-memory` (default)
pub fn resolve_memory_home(cfg: &Config) -> PathBuf {
    // 1. env var
    if let Ok(env_home) = std::env::var("TOOL_FIRST_MEMORY_HOME") {
        return follow_redirects(PathBuf::from(shellexpand(&env_home)));
    }

    // 2. config.memory_home
    if let Some(ref home) = cfg.memory_home {
        return follow_redirects(PathBuf::from(shellexpand(home)));
    }

    // 3. config.file.base_dir
    if let Some(ref base) = cfg.file.base_dir {
        return follow_redirects(PathBuf::from(shellexpand(base)));
    }

    // 4. default
    follow_redirects(default_memory_home())
}

pub fn default_memory_home() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("tool-first-agent")
        .join("tool-memory")
}

/// Return the path to the `.tool-memory-home` marker.
pub fn marker_path(memory_home: &PathBuf) -> PathBuf {
    memory_home.join(MARKER_FILENAME)
}

/// Check whether the `.tool-memory-home` marker exists.
pub fn has_marker(memory_home: &PathBuf) -> bool {
    marker_path(memory_home).exists()
}

/// Write the `.tool-memory-home` marker if missing.
/// Returns `true` if the marker was created, `false` if it already existed.
pub fn ensure_marker(memory_home: &PathBuf) -> Result<bool, String> {
    let path = marker_path(memory_home);
    if path.exists() {
        return Ok(false);
    }
    std::fs::create_dir_all(memory_home)
        .map_err(|e| format!("Failed to create memory home dir: {e}"))?;
    let marker = MemoryHomeMarker::default();
    let json = serde_json::to_string_pretty(&marker)
        .map_err(|e| format!("Failed to serialize marker: {e}"))?;
    std::fs::write(&path, format!("{json}\n"))
        .map_err(|e| format!("Failed to write marker: {e}"))?;
    Ok(true)
}

/// Check for `.tool-memory-redirect` in a directory.
/// Returns the redirect target if found.
fn follow_redirects(mut path: PathBuf) -> PathBuf {
    for _ in 0..8 {
        match check_redirect(&path) {
            Some(next) => {
                let next_path = PathBuf::from(shellexpand(&next));
                if next_path == path {
                    break;
                }
                path = next_path;
            }
            None => break,
        }
    }
    path
}

pub fn check_redirect(dir: &PathBuf) -> Option<String> {
    let path = dir.join(".tool-memory-redirect");
    if !path.exists() {
        return None;
    }
    let contents = std::fs::read_to_string(&path).ok()?;
    let marker: MemoryRedirectMarker = serde_json::from_str(&contents).ok()?;
    Some(marker.redirect_to)
}

/// Detect all known tool-memory home candidates and return them.
/// This is used to check for conflicts.
pub fn detect_memory_homes() -> Vec<MemoryHomeCandidate> {
    let mut candidates = Vec::new();

    // Check TOOL_FIRST_MEMORY_HOME
    if let Ok(env_home) = std::env::var("TOOL_FIRST_MEMORY_HOME") {
        let path = PathBuf::from(shellexpand(&env_home));
        candidates.push(MemoryHomeCandidate {
            path: path.clone(),
            source: "TOOL_FIRST_MEMORY_HOME".to_string(),
            has_marker: has_marker(&path),
            is_canonical: true,
        });
    }

    // Check config file
    let cfg = crate::config::load();
    let config_candidates: Vec<(&str, Option<&String>)> = vec![
        ("config.memory_home", cfg.memory_home.as_ref()),
        ("config.file.base_dir", cfg.file.base_dir.as_ref()),
    ];

    for (source, opt_val) in config_candidates {
        if let Some(val) = opt_val {
            let path = PathBuf::from(shellexpand(val));
            if !candidates.iter().any(|c| c.path == path) {
                candidates.push(MemoryHomeCandidate {
                    path: path.clone(),
                    source: source.to_string(),
                    has_marker: has_marker(&path),
                    is_canonical: false,
                });
            }
        }
    }

    // Check default
    let default = default_memory_home();
    if !candidates.iter().any(|c| c.path == default) {
        candidates.push(MemoryHomeCandidate {
            path: default.clone(),
            source: "default".to_string(),
            has_marker: has_marker(&default),
            is_canonical: false,
        });
    }

    // Check agent-specific legacy paths
    if let Some(home) = dirs::home_dir() {
        let legacy_paths = [
            (
                home.join(".config/tool-inventory/memory"),
                "legacy:~/.config/tool-inventory/memory",
            ),
            (
                home.join(".claude/tool-memory"),
                "legacy:~/.claude/tool-memory",
            ),
            (
                home.join(".hermes/tool-memory"),
                "legacy:~/.hermes/tool-memory",
            ),
        ];
        for (path, source) in &legacy_paths {
            if path.exists() && !candidates.iter().any(|c| c.path == *path) {
                candidates.push(MemoryHomeCandidate {
                    path: path.clone(),
                    source: source.to_string(),
                    has_marker: has_marker(path),
                    is_canonical: false,
                });
            }
        }
    }

    candidates
}

#[derive(Debug, Serialize)]
pub struct MemoryHomeCandidate {
    pub path: PathBuf,
    pub source: String,
    pub has_marker: bool,
    pub is_canonical: bool,
}

/// Expand leading `~` to home directory.
fn shellexpand(s: &str) -> String {
    if s.starts_with("~/") || s == "~" {
        if let Some(home) = dirs::home_dir() {
            return format!("{}{}", home.display(), &s[1..]);
        }
    }
    s.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_memory_home(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "tool-first-resolver-test-{name}-{}",
            uuid::Uuid::new_v4()
        ))
    }

    #[test]
    fn resolve_memory_home_follows_redirect_marker() {
        let old_home = temp_memory_home("old");
        let new_home = temp_memory_home("new");
        std::fs::create_dir_all(&old_home).unwrap();
        std::fs::write(
            old_home.join(".tool-memory-redirect"),
            serde_json::json!({
                "redirect_to": new_home.to_string_lossy(),
                "reason": "test migration",
                "do_not_write_here": true
            })
            .to_string(),
        )
        .unwrap();

        assert_eq!(follow_redirects(old_home), new_home);
    }
}
