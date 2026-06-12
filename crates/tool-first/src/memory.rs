use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A tool-memory record. This is the canonical schema.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryRecord {
    pub namespace: Option<String>,
    pub memory_type: Option<String>,
    pub record_type: Option<String>,
    pub category: Option<String>,
    pub tool: Option<String>,
    pub task: Option<String>,
    pub status: Option<String>,
    pub scope: Option<String>,
    pub verified_at: Option<String>,
    pub created_at: Option<String>,
    pub confidence: Option<f64>,
    pub tags: Option<Vec<String>>,
    pub source_agent: Option<String>,
    pub os: Option<String>,
    pub arch: Option<String>,
    pub authority: Option<String>,
    pub path: Option<String>,
    pub version: Option<String>,
    pub command_template: Option<String>,
    pub command: Option<String>,
    pub failure_reason: Option<String>,
    pub notes: Option<String>,
    pub doc_id: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl MemoryRecord {
    /// Enrich with default infrastructure fields if missing.
    pub fn enrich(&mut self) {
        let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        if self.created_at.is_none() {
            self.created_at = Some(now.clone());
        }
        if self.verified_at.is_none() {
            self.verified_at = Some(now);
        }
        if self.authority.is_none() {
            self.authority = Some("runtime-infrastructure".to_string());
        }
        if self.os.is_none() {
            self.os = Some(detect_os());
        }
        if self.arch.is_none() {
            self.arch = Some(std::env::consts::ARCH.to_string());
        }
        if self.source_agent.is_none() {
            let agent =
                std::env::var("TOOL_FIRST_AGENT_NAME").unwrap_or_else(|_| "unknown".to_string());
            self.source_agent = Some(agent);
        }
        if self.namespace.is_none() {
            self.namespace = Some("agent_tool_inventory".to_string());
        }
        if self.memory_type.is_none() {
            self.memory_type = Some("tool_inventory".to_string());
        }
        if self.scope.is_none() {
            self.scope = Some("local_machine".to_string());
        }
        if self.tags.is_none() {
            let mut tags = vec!["tool-inventory".to_string()];
            if let Some(ref cat) = self.category {
                tags.push(format!("tool-category-{cat}"));
            }
            self.tags = Some(tags);
        }
    }
}

fn detect_os() -> String {
    match std::env::consts::OS {
        "macos" => "macos".to_string(),
        "linux" => "linux".to_string(),
        "windows" => "windows".to_string(),
        other => other.to_string(),
    }
}
