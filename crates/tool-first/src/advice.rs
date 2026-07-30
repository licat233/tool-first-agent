use crate::detect::{self, DetectionResult};
use crate::memory::MemoryRecord;
use crate::registry::{self, MatchedTool, Registry};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;

const MAX_CANDIDATES: usize = 5;
pub const AVOID_CUSTOM_CODE_INTENT: &str = "avoid_custom_code";

#[derive(Debug, Default)]
pub struct AdviceOptions<'a> {
    pub category: Option<&'a str>,
    pub intent: Option<&'a str>,
    pub recall: bool,
    pub verbose: bool,
    pub memory_limit: usize,
}

#[derive(Debug, Serialize)]
pub struct ToolAdvice {
    pub task: String,
    pub applicable: bool,
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub candidates: Vec<MatchedTool>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub detected: Vec<DetectionResult>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub memory: Vec<MemoryRecord>,
    pub recommendation: Recommendation,
}

#[derive(Debug, Clone, Serialize)]
pub struct Recommendation {
    pub decision: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    pub reason: String,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub command_templates: BTreeMap<String, String>,
}

pub fn advise(
    registry: &Registry,
    memory_home: &Path,
    task: &str,
    options: AdviceOptions<'_>,
) -> ToolAdvice {
    let resolved_category = options
        .category
        .map(String::from)
        .or_else(|| infer_category(task));

    if !is_applicable(
        task,
        options.category,
        options.intent,
        resolved_category.as_deref(),
    ) {
        return not_applicable(
            task,
            resolved_category,
            "No commodity local operation requiring incidental custom code was identified.",
        );
    }

    let Some(category) = resolved_category else {
        return not_applicable(
            task,
            None,
            "A supported tool category is required before local tool discovery.",
        );
    };

    let candidates: Vec<MatchedTool> = registry::query(registry, Some(&category), Some(task))
        .into_iter()
        .take(MAX_CANDIDATES)
        .collect();

    if candidates.is_empty() {
        return not_applicable(
            task,
            Some(category),
            "The requested category has no registered local tool candidates.",
        );
    }

    let detect_tools: Vec<String> = candidates.iter().map(|c| c.tool.clone()).collect();
    let detected = detect::detect(registry, Some(&category), &detect_tools);

    let available_recommendation = recommend_available(&candidates, &detected);
    if let Some(recommendation) = available_recommendation
        .as_ref()
        .filter(|_| !options.recall)
    {
        return ToolAdvice {
            task: task.to_string(),
            applicable: true,
            category: Some(category),
            candidates: detail(options.verbose, candidates),
            detected: detail(options.verbose, detected),
            memory: Vec::new(),
            recommendation: recommendation.clone(),
        };
    }

    // Memory is a secondary source. Read it only after registered candidates
    // are unavailable, unless the caller explicitly requested recall.
    let memory = crate::file_store::recall(
        memory_home,
        task,
        Some(&category),
        options.memory_limit.max(1),
    );

    let recommendation = available_recommendation
        .unwrap_or_else(|| recommend_after_recall(&candidates, &memory, &category));
    ToolAdvice {
        task: task.to_string(),
        applicable: true,
        category: Some(category),
        candidates: detail(options.verbose, candidates),
        detected: detail(options.verbose, detected),
        memory: detail(options.verbose, memory),
        recommendation,
    }
}

fn is_applicable(
    task: &str,
    explicit_category: Option<&str>,
    intent: Option<&str>,
    resolved_category: Option<&str>,
) -> bool {
    if requests_software_implementation(task) {
        return false;
    }

    if let Some(intent) = intent {
        if intent != AVOID_CUSTOM_CODE_INTENT {
            return false;
        }
    }

    if explicit_category.is_some() {
        return true;
    }

    resolved_category.is_some() && has_commodity_action(task)
}

fn requests_software_implementation(task: &str) -> bool {
    let text = task.to_lowercase();
    let implementation_verbs = [
        "write",
        "implement",
        "develop",
        "code",
        "build",
        "create",
        "编写",
        "实现",
        "开发",
        "编码",
        "创建",
        "写一个",
    ];
    let software_nouns = [
        "script",
        "program",
        "library",
        "function",
        "api",
        " app",
        "application",
        " cli",
        "rust",
        "python",
        "javascript",
        "typescript",
        "代码",
        "脚本",
        "程序",
        "函数",
        "库",
        "应用",
        "工具",
        "接口",
    ];

    implementation_verbs.iter().any(|word| text.contains(word))
        && software_nouns.iter().any(|word| text.contains(word))
}

fn not_applicable(task: &str, category: Option<String>, reason: &str) -> ToolAdvice {
    ToolAdvice {
        task: task.to_string(),
        applicable: false,
        category,
        candidates: Vec::new(),
        detected: Vec::new(),
        memory: Vec::new(),
        recommendation: Recommendation {
            decision: "not_applicable".to_string(),
            tool: None,
            category: None,
            reason: reason.to_string(),
            command_templates: BTreeMap::new(),
        },
    }
}

fn recommend_available(
    candidates: &[MatchedTool],
    detected: &[DetectionResult],
) -> Option<Recommendation> {
    let detected_by_tool: BTreeMap<&str, &DetectionResult> =
        detected.iter().map(|d| (d.tool.as_str(), d)).collect();

    for candidate in candidates {
        if let Some(result) = detected_by_tool.get(candidate.tool.as_str()) {
            if result.status == "available" || result.status == "present_unverified" {
                return Some(Recommendation {
                    decision: "use_existing_tool".to_string(),
                    tool: Some(candidate.tool.clone()),
                    category: Some(candidate.category.clone()),
                    reason: format!(
                        "{} is {} and can be used instead of incidental custom code.",
                        candidate.tool, result.status
                    ),
                    command_templates: candidate.commands.clone(),
                });
            }
        }
    }
    None
}

fn recommend_after_recall(
    candidates: &[MatchedTool],
    memory: &[MemoryRecord],
    category: &str,
) -> Recommendation {
    if let Some(record) = memory.iter().find(|r| {
        r.status.as_deref() == Some("verified_success")
            && (r.command_template.is_some() || r.command.is_some())
    }) {
        return Recommendation {
            decision: "verify_recalled_recipe".to_string(),
            tool: record.tool.clone(),
            category: record.category.clone(),
            reason: "A prior verified recipe exists; recheck tool availability before using it."
                .to_string(),
            command_templates: memory_command_templates(record),
        };
    }

    if let Some(candidate) = candidates.first() {
        return Recommendation {
            decision: "known_tool_not_installed".to_string(),
            tool: Some(candidate.tool.clone()),
            category: Some(candidate.category.clone()),
            reason: format!(
                "{} fits the operation but was not detected; ask before installing it or use justified custom code.",
                candidate.tool
            ),
            command_templates: candidate.commands.clone(),
        };
    }

    Recommendation {
        decision: "custom_code_justified".to_string(),
        tool: None,
        category: Some(category.to_string()),
        reason: "No matching installed tool or recalled recipe was found.".to_string(),
        command_templates: BTreeMap::new(),
    }
}

fn has_commodity_action(task: &str) -> bool {
    let text = task.to_lowercase();
    let actions = [
        "convert",
        "extract",
        "resize",
        "compress",
        "decompress",
        "split",
        "merge",
        "query",
        "filter",
        "transform",
        "download",
        "scrape",
        "render",
        "ocr",
        "archive",
        "unzip",
        "zip",
        "parse",
        "转换",
        "转成",
        "提取",
        "调整尺寸",
        "压缩",
        "解压",
        "拆分",
        "合并",
        "查询",
        "过滤",
        "下载",
        "抓取",
        "渲染",
        "识别",
    ];

    actions.iter().any(|action| text.contains(action))
}

fn memory_command_templates(record: &MemoryRecord) -> BTreeMap<String, String> {
    let mut commands = BTreeMap::new();
    if let Some(command) = record.command_template.as_ref().or(record.command.as_ref()) {
        commands.insert("recalled".to_string(), command.clone());
    }
    commands
}

fn detail<T>(verbose: bool, values: Vec<T>) -> Vec<T> {
    if verbose {
        values
    } else {
        Vec::new()
    }
}

fn infer_category(task: &str) -> Option<String> {
    let text = task.to_lowercase();
    let rules: &[(&str, &[&str])] = &[
        (
            "document",
            &[
                "docx",
                "word",
                "markdown",
                "epub",
                "pptx",
                "slides",
                "文档",
                "幻灯片",
            ],
        ),
        (
            "pdf",
            &["pdf", "page count", "pdf text", "render pdf", "PDF"],
        ),
        (
            "image",
            &[
                "image", "png", "jpg", "jpeg", "webp", "resize", "ocr", "exif", "图片", "图像",
            ],
        ),
        (
            "media",
            &[
                "video", "audio", "mp4", "mov", "mp3", "ffmpeg", "subtitle", "视频", "音频", "字幕",
            ],
        ),
        (
            "data",
            &[
                "json", "yaml", "yml", "csv", "tsv", "xml", "sqlite", "sql", "数据",
            ],
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
                "搜索文件",
                "查找文本",
                "批量替换",
            ],
        ),
        (
            "archive",
            &["zip", "unzip", "tar", "zstd", "archive", "压缩包", "解压"],
        ),
        (
            "web",
            &[
                "url", "http", "website", "web page", "curl", "download", "scrape", "网页", "网站",
            ],
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
    use std::path::PathBuf;

    fn registry_with_tool(category_name: &str, tool: &str, handles: &[&str]) -> Registry {
        let mut registry = Registry::new();
        let mut category = Category {
            description: None,
            tools: BTreeMap::new(),
        };
        category.tools.insert(
            tool.to_string(),
            ToolSpec {
                priority: Some(10),
                detect_names: vec![tool.to_string()],
                version_args: vec!["--version".to_string()],
                handles: handles.iter().map(|s| s.to_string()).collect(),
                commands: BTreeMap::from([("run".to_string(), format!("{tool} {{input}}"))]),
                known_paths: Vec::new(),
                app_bundle_paths: Vec::new(),
                fallbacks: Vec::new(),
            },
        );
        registry.insert(category_name.to_string(), category);
        registry
    }

    fn options<'a>(category: Option<&'a str>) -> AdviceOptions<'a> {
        AdviceOptions {
            category,
            intent: Some(AVOID_CUSTOM_CODE_INTENT),
            memory_limit: 5,
            ..Default::default()
        }
    }

    #[test]
    fn ordinary_question_exits_without_candidates_or_detection() {
        let registry = registry_with_tool("ai", "claude", &["Claude Code assistant"]);
        let memory_home = PathBuf::from("/path/that/must/not/be/read");
        let advice = advise(
            &registry,
            &memory_home,
            "解释为什么天空是蓝色的",
            AdviceOptions::default(),
        );

        assert!(!advice.applicable);
        assert_eq!(advice.recommendation.decision, "not_applicable");
        assert!(advice.candidates.is_empty());
        assert!(advice.detected.is_empty());
        assert!(advice.memory.is_empty());
        assert!(serde_json::to_vec(&advice).unwrap().len() < 1_000);
    }

    #[test]
    fn ordinary_software_development_does_not_trigger() {
        let registry = registry_with_tool("dev", "sh", &["Run shell commands"]);
        let advice = advise(
            &registry,
            &PathBuf::from("/path/that/must/not/be/read"),
            "build a REST API in Rust",
            AdviceOptions::default(),
        );

        assert!(!advice.applicable);
        assert_eq!(advice.recommendation.decision, "not_applicable");
    }

    #[test]
    fn explicitly_requested_converter_implementation_does_not_trigger() {
        let registry = registry_with_tool("image", "sips", &["Convert and resize images"]);
        let advice = advise(
            &registry,
            &PathBuf::from("/path/that/must/not/be/read"),
            "请用 Rust 编写一个 PNG 转换程序",
            options(Some("image")),
        );

        assert!(!advice.applicable);
        assert_eq!(advice.recommendation.decision, "not_applicable");
    }

    #[test]
    fn explicit_category_checks_a_commodity_operation() {
        let registry = registry_with_tool("dev", "sh", &["Run shell commands"]);
        let memory_home =
            std::env::temp_dir().join(format!("tool-first-advice-test-{}", uuid::Uuid::new_v4()));
        let advice = advise(
            &registry,
            &memory_home,
            "execute a shell command",
            options(Some("dev")),
        );

        assert!(advice.applicable);
        assert_eq!(advice.recommendation.decision, "use_existing_tool");
        assert_eq!(advice.recommendation.tool.as_deref(), Some("sh"));
        assert!(
            advice.candidates.is_empty(),
            "compact output is the default"
        );
    }

    #[test]
    fn chinese_image_conversion_is_in_scope() {
        let registry = registry_with_tool("image", "sips", &["Convert and resize images"]);
        let memory_home =
            std::env::temp_dir().join(format!("tool-first-advice-test-{}", uuid::Uuid::new_v4()));
        let advice = advise(
            &registry,
            &memory_home,
            "把这张 PNG 图片转换成 JPG",
            AdviceOptions {
                memory_limit: 5,
                ..Default::default()
            },
        );

        assert!(advice.applicable);
        assert_eq!(advice.category.as_deref(), Some("image"));
        assert_eq!(advice.recommendation.decision, "use_existing_tool");
    }

    #[test]
    fn verbose_mode_includes_bounded_details() {
        let registry = registry_with_tool("dev", "sh", &["Run shell commands"]);
        let memory_home =
            std::env::temp_dir().join(format!("tool-first-advice-test-{}", uuid::Uuid::new_v4()));
        let advice = advise(
            &registry,
            &memory_home,
            "test rust project",
            AdviceOptions {
                verbose: true,
                ..options(Some("dev"))
            },
        );

        assert_eq!(advice.candidates.len(), 1);
        assert_eq!(advice.detected.len(), 1);
        assert!(advice.candidates.len() <= MAX_CANDIDATES);
    }
}
