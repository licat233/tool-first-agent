use crate::detect::{self, DetectionResult};
use crate::memory::MemoryRecord;
use crate::registry::{self, MatchedTool, Registry};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct ToolAdvice {
    pub task: String,
    pub category: Option<String>,
    pub candidates: Vec<MatchedTool>,
    pub detected: Vec<DetectionResult>,
    pub memory: Vec<MemoryRecord>,
    pub recommendation: Recommendation,
}

#[derive(Debug, Serialize)]
pub struct Recommendation {
    pub decision: String,
    pub tool: Option<String>,
    pub category: Option<String>,
    pub reason: String,
    pub command_templates: BTreeMap<String, String>,
}

pub fn advise(
    registry: &Registry,
    memory_home: &PathBuf,
    task: &str,
    category: Option<&str>,
    limit: usize,
) -> ToolAdvice {
    let resolved_category = category.map(String::from).or_else(|| infer_category(task));
    let candidates = registry::query(registry, resolved_category.as_deref(), Some(task));
    let detect_tools = tools_to_detect(&candidates, resolved_category.as_deref());
    let detected = detect::detect(registry, resolved_category.as_deref(), &detect_tools);
    let memory = crate::file_store::recall(memory_home, task, resolved_category.as_deref(), limit);
    let recommendation = recommend(&candidates, &detected, &memory);

    ToolAdvice {
        task: task.to_string(),
        category: resolved_category,
        candidates,
        detected,
        memory,
        recommendation,
    }
}

fn tools_to_detect(candidates: &[MatchedTool], category: Option<&str>) -> Vec<String> {
    if category.is_some() {
        return Vec::new();
    }

    let matched: Vec<String> = candidates
        .iter()
        .filter(|c| c.is_match)
        .map(|c| c.tool.clone())
        .collect();

    if !matched.is_empty() {
        return unique(matched);
    }

    unique(candidates.iter().take(8).map(|c| c.tool.clone()).collect())
}

fn recommend(
    candidates: &[MatchedTool],
    detected: &[DetectionResult],
    memory: &[MemoryRecord],
) -> Recommendation {
    let detected_by_tool: BTreeMap<&str, &DetectionResult> =
        detected.iter().map(|d| (d.tool.as_str(), d)).collect();

    let has_strict_match = candidates.iter().any(|c| c.is_match);
    for candidate in candidates
        .iter()
        .filter(|c| c.is_match || !has_strict_match)
    {
        if let Some(result) = detected_by_tool.get(candidate.tool.as_str()) {
            if result.status == "available" || result.status == "present_unverified" {
                return Recommendation {
                    decision: "use_existing_tool".to_string(),
                    tool: Some(candidate.tool.clone()),
                    category: Some(candidate.category.clone()),
                    reason: format!(
                        "{} is {} on this machine; use it before writing custom code.",
                        candidate.tool, result.status
                    ),
                    command_templates: candidate.commands.clone(),
                };
            }
        }
    }

    if let Some(record) = memory.iter().find(|r| {
        r.status.as_deref() == Some("verified_success")
            && (r.command_template.is_some() || r.command.is_some())
    }) {
        return Recommendation {
            decision: "verify_recalled_recipe".to_string(),
            tool: record.tool.clone(),
            category: record.category.clone(),
            reason: "A prior verified recipe exists in tool-memory; recheck availability before writing custom code.".to_string(),
            command_templates: memory_command_templates(record),
        };
    }

    if let Some(candidate) = candidates
        .iter()
        .find(|c| c.is_match)
        .or_else(|| candidates.first())
    {
        return Recommendation {
            decision: "tool_known_but_missing".to_string(),
            tool: Some(candidate.tool.clone()),
            category: Some(candidate.category.clone()),
            reason: format!(
                "{} matches the task but was not detected as available; use a fallback or ask before installing.",
                candidate.tool
            ),
            command_templates: candidate.commands.clone(),
        };
    }

    Recommendation {
        decision: "write_code_or_use_other_skill".to_string(),
        tool: None,
        category: None,
        reason: "No matching available tool or recalled recipe was found; custom code may be justified after checking relevant skills.".to_string(),
        command_templates: BTreeMap::new(),
    }
}

fn memory_command_templates(record: &MemoryRecord) -> BTreeMap<String, String> {
    let mut commands = BTreeMap::new();
    if let Some(command) = record.command_template.as_ref().or(record.command.as_ref()) {
        commands.insert("recalled".to_string(), command.clone());
    }
    commands
}

fn unique(values: Vec<String>) -> Vec<String> {
    let mut seen = Vec::new();
    for value in values {
        if !seen.contains(&value) {
            seen.push(value);
        }
    }
    seen
}

fn infer_category(task: &str) -> Option<String> {
    let text = task.to_lowercase();
    let rules: &[(&str, &[&str])] = &[
        (
            "document",
            &[
                "docx", "word", "markdown", "md", "html", "epub", "pptx", "slides",
            ],
        ),
        (
            "pdf",
            &[
                "pdf",
                "pdfs",
                "page count",
                "pdf text",
                "render pdf",
                "split pdf",
                "merge pdf",
            ],
        ),
        (
            "image",
            &[
                "image", "png", "jpg", "jpeg", "webp", "resize", "ocr", "exif",
            ],
        ),
        (
            "media",
            &[
                "video",
                "audio",
                "mp4",
                "mov",
                "mp3",
                "ffmpeg",
                "subtitle",
                "compress video",
            ],
        ),
        (
            "data",
            &["json", "yaml", "yml", "csv", "tsv", "xml", "sqlite", "sql"],
        ),
        (
            "search",
            &[
                "search",
                "grep",
                "find files",
                "ripgrep",
                "replace",
                "list files",
            ],
        ),
        (
            "archive",
            &[
                "zip",
                "unzip",
                "tar",
                "gz",
                "zstd",
                "archive",
                "compress files",
            ],
        ),
        (
            "dev",
            &["git", "test", "build", "npm", "cargo", "python", "node"],
        ),
        (
            "web",
            &[
                "url", "http", "website", "web page", "curl", "download", "scrape",
            ],
        ),
        (
            "ai",
            &["llm", "model", "claude", "codex", "hermes", "agent"],
        ),
    ];

    rules
        .iter()
        .find(|(_, keywords)| keywords.iter().any(|keyword| text.contains(keyword)))
        .map(|(category, _)| category.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::{Category, ToolSpec};

    #[test]
    fn advice_recommends_available_matching_tool() {
        let mut registry = Registry::new();
        let mut category = Category {
            description: None,
            tools: BTreeMap::new(),
        };
        category.tools.insert(
            "cargo".to_string(),
            ToolSpec {
                priority: Some(10),
                detect_names: vec!["cargo".to_string()],
                version_args: vec!["--version".to_string()],
                handles: vec!["Build and test Rust projects".to_string()],
                commands: BTreeMap::from([("test".to_string(), "cargo test".to_string())]),
                known_paths: Vec::new(),
                app_bundle_paths: Vec::new(),
                fallbacks: Vec::new(),
            },
        );
        registry.insert("dev".to_string(), category);

        let memory_home =
            std::env::temp_dir().join(format!("tool-first-advice-test-{}", uuid::Uuid::new_v4()));
        let advice = advise(
            &registry,
            &memory_home,
            "build and test rust project",
            Some("dev"),
            5,
        );

        assert_eq!(advice.recommendation.decision, "use_existing_tool");
        assert_eq!(advice.recommendation.tool.as_deref(), Some("cargo"));
    }

    #[test]
    fn infer_category_from_task_text() {
        assert_eq!(
            infer_category("extract fields from json"),
            Some("data".to_string())
        );
        assert_eq!(
            infer_category("resize png image"),
            Some("image".to_string())
        );
    }
}
