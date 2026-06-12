use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Top-level structure of `registry/tools.yaml`.
pub type Registry = BTreeMap<String, Category>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub description: Option<String>,
    #[serde(default)]
    pub tools: BTreeMap<String, ToolSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub priority: Option<u32>,
    #[serde(default)]
    pub detect_names: Vec<String>,
    #[serde(default)]
    pub known_paths: Vec<String>,
    #[serde(default)]
    pub app_bundle_paths: Vec<String>,
    #[serde(default)]
    pub version_args: Vec<String>,
    #[serde(default)]
    pub handles: Vec<String>,
    #[serde(default)]
    pub commands: BTreeMap<String, String>,
    #[serde(default)]
    pub fallbacks: Vec<String>,
}

/// A matched tool from a registry query.
#[derive(Debug, Serialize)]
pub struct MatchedTool {
    pub category: String,
    pub tool: String,
    pub priority: u32,
    #[serde(rename = "match")]
    pub is_match: bool,
    pub detect_names: Vec<String>,
    pub known_paths: Vec<String>,
    pub app_bundle_paths: Vec<String>,
    pub handles: Vec<String>,
    pub commands: BTreeMap<String, String>,
    pub fallbacks: Vec<String>,
}

/// Load `registry/tools.yaml` from the project.
pub fn load_registry() -> Result<Registry, String> {
    let path = find_registry()?;
    load_from_path(&path)
}

pub fn load_from_path(path: &Path) -> Result<Registry, String> {
    let contents =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read registry: {e}"))?;
    serde_yaml::from_str(&contents).map_err(|e| format!("Failed to parse registry YAML: {e}"))
}

/// Find the registry file relative to the binary or project root.
fn find_registry() -> Result<PathBuf, String> {
    // Try relative to executable
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            // In target/debug/ -> go up to project root
            let candidates = [
                parent.join("../../../registry/tools.yaml"),
                parent.join("../../registry/tools.yaml"),
                parent.join("../registry/tools.yaml"),
                parent.join("registry/tools.yaml"),
            ];
            for c in &candidates {
                if c.is_file() {
                    return Ok(c.clone());
                }
            }
        }
    }

    // Try CWD
    if let Ok(cwd) = std::env::current_dir() {
        let candidates = [
            cwd.join("registry/tools.yaml"),
            cwd.join("crates/tool-first/registry/tools.yaml"),
            cwd.join("../registry/tools.yaml"),
        ];
        for c in &candidates {
            if c.is_file() {
                return Ok(c.clone());
            }
        }
    }

    Err("registry/tools.yaml not found. Run from the project root or set the path.".to_string())
}

/// Query the registry for tools in a category, optionally filtering by task text.
pub fn query(registry: &Registry, category: Option<&str>, task: Option<&str>) -> Vec<MatchedTool> {
    let mut output = Vec::new();

    for cat_name in registry.keys() {
        if let Some(filter_cat) = category {
            if cat_name != filter_cat {
                continue;
            }
        }
        if let Some(section) = registry.get(cat_name) {
            let mut rows: Vec<MatchedTool> = section
                .tools
                .iter()
                .map(|(name, spec)| {
                    let is_match = task.map(|t| matches_task(spec, t)).unwrap_or(true);
                    MatchedTool {
                        category: (*cat_name).clone(),
                        tool: name.clone(),
                        priority: spec.priority.unwrap_or(999),
                        is_match,
                        detect_names: spec.detect_names.clone(),
                        known_paths: spec.known_paths.clone(),
                        app_bundle_paths: spec.app_bundle_paths.clone(),
                        handles: spec.handles.clone(),
                        commands: spec.commands.clone(),
                        fallbacks: spec.fallbacks.clone(),
                    }
                })
                .collect();

            rows.sort_by(|a, b| {
                a.is_match
                    .cmp(&b.is_match)
                    .reverse()
                    .then(a.priority.cmp(&b.priority))
                    .then(a.tool.cmp(&b.tool))
            });

            output.extend(rows);
        }
    }

    output
}

/// Check whether all tokens from `text` appear in the tool's handles/commands.
fn matches_task(spec: &ToolSpec, text: &str) -> bool {
    let haystack = {
        let mut parts: Vec<&str> = spec.handles.iter().map(|s| s.as_str()).collect();
        for (k, v) in &spec.commands {
            parts.push(k);
            parts.push(v);
        }
        parts.join(" ").to_lowercase()
    };
    text.to_lowercase()
        .split_whitespace()
        .all(|token| haystack.contains(token))
}
