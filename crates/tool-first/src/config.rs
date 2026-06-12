use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Loaded from `~/.config/tool-first-agent/config.yaml`
/// or overridden by TOOL_FIRST_MEMORY_CONFIG.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub memory_home: Option<String>,
    pub canonical: Option<bool>,
    pub authority: Option<String>,

    #[serde(default)]
    pub write_policy: WritePolicy,

    #[serde(default)]
    pub file: FileConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WritePolicy {
    pub allow_create_new_home: Option<bool>,
    pub append_only: Option<bool>,
    pub atomic_write: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileConfig {
    pub base_dir: Option<String>,
}

/// Load the config from disk. Returns `Default` if no file found.
pub fn load() -> Config {
    if let Some(path) = find_config_path() {
        if let Ok(contents) = std::fs::read_to_string(&path) {
            if let Ok(cfg) = serde_yaml::from_str::<Config>(&contents) {
                return cfg;
            }
        }
    }
    Config::default()
}

/// Search priority for config file:
///   1. TOOL_FIRST_MEMORY_CONFIG env
///   2. ./memory_config.yaml  (next to binary / project root)
///   3. ~/.config/tool-first-agent/config.yaml
///   4. ~/.config/tool-first-agent/memory_config.yaml
pub fn find_config_path() -> Option<PathBuf> {
    // 1. env override
    if let Ok(env_path) = std::env::var("TOOL_FIRST_MEMORY_CONFIG") {
        let p = PathBuf::from(shellexpand(&env_path));
        if p.is_file() {
            return Some(p);
        }
    }

    // 2. next to the crate root (for dev) or CWD
    if let Ok(cwd) = std::env::current_dir() {
        let p = cwd.join("memory_config.yaml");
        if p.is_file() {
            return Some(p);
        }
        let p2 = cwd
            .join("crates")
            .join("tool-first")
            .join("memory_config.yaml");
        if p2.is_file() {
            return Some(p2);
        }
    }

    // 3. ~/.config/tool-first-agent/config.yaml
    if let Some(config_dir) = dirs::config_dir() {
        let p = config_dir.join("tool-first-agent").join("config.yaml");
        if p.is_file() {
            return Some(p);
        }
        let p2 = config_dir
            .join("tool-first-agent")
            .join("memory_config.yaml");
        if p2.is_file() {
            return Some(p2);
        }
    }

    // 4. Agent skill dirs
    if let Some(home) = dirs::home_dir() {
        let candidates = [
            home.join(".hermes/skills/devops/tool-first-agent/memory_config.yaml"),
            home.join(".claude/skills/tool-first-agent/memory_config.yaml"),
            home.join(".codex/skills/tool-first-agent/memory_config.yaml"),
        ];
        for c in &candidates {
            if c.is_file() {
                return Some(c.clone());
            }
        }
    }

    None
}

fn shellexpand(s: &str) -> String {
    if s.starts_with("~/") || s == "~" {
        if let Some(home) = dirs::home_dir() {
            return format!("{}{}", home.display(), &s[1..]);
        }
    }
    s.to_string()
}
