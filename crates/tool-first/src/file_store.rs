//! File-based tool-memory store.
//!
//! One record per JSON file under `<memory_home>/records/`.
//! Append-only, atomic writes (`.tmp` + rename).
//! `index.json` is a rebuildable cache, not the source of truth.

use crate::memory::MemoryRecord;
use crate::resolver;
use serde::Serialize;
use serde_json;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// Result of persisting a record.
#[derive(Debug, Serialize)]
pub struct RetainResult {
    pub saved: String,
    pub record: MemoryRecord,
}

/// Ensure the store is ready (dirs exist, marker exists).
pub fn ensure_ready(memory_home: &PathBuf) -> Result<(), String> {
    let records_dir = memory_home.join("records");
    fs::create_dir_all(&records_dir).map_err(|e| format!("Failed to create records dir: {e}"))?;
    resolver::ensure_marker(memory_home)?;
    Ok(())
}

/// Persist a record.
pub fn retain(memory_home: &PathBuf, record: &MemoryRecord) -> Result<RetainResult, String> {
    let records_dir = memory_home.join("records");
    fs::create_dir_all(&records_dir).map_err(|e| format!("Failed to create records dir: {e}"))?;
    let _ = resolver::ensure_marker(memory_home);

    let filename = generate_filename(record);
    let target = records_dir.join(&filename);
    let tmp = records_dir.join(format!(".tmp-{filename}"));

    let json = serde_json::to_string_pretty(record)
        .map_err(|e| format!("Failed to serialize record: {e}"))?;

    fs::write(&tmp, format!("{json}\n")).map_err(|e| format!("Failed to write temp file: {e}"))?;
    fs::rename(&tmp, &target).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("Failed to rename temp file: {e}")
    })?;

    Ok(RetainResult {
        saved: target.to_string_lossy().to_string(),
        record: record.clone(),
    })
}

/// Search records matching a query string.
pub fn recall(
    memory_home: &PathBuf,
    query: &str,
    category: Option<&str>,
    limit: usize,
) -> Vec<MemoryRecord> {
    let records = load_all(memory_home);
    let terms = tokenize(query);

    let mut scored: Vec<(usize, MemoryRecord)> = records
        .into_iter()
        .filter(|r| {
            if let Some(cat) = category {
                r.category.as_deref() == Some(cat)
                    || r.tags
                        .as_ref()
                        .map_or(false, |t| t.contains(&format!("tool-category-{cat}")))
            } else {
                true
            }
        })
        .filter_map(|r| {
            let score = score_record(&r, &terms);
            if score > 0 {
                Some((score, r))
            } else {
                None
            }
        })
        .collect();

    scored.sort_by(|a, b| {
        b.0.cmp(&a.0).then_with(|| {
            b.1.created_at
                .as_deref()
                .unwrap_or("")
                .cmp(a.1.created_at.as_deref().unwrap_or(""))
        })
    });

    scored.into_iter().take(limit).map(|(_, r)| r).collect()
}

/// Get availability records, optionally filtered by tool names.
pub fn get_availability(memory_home: &PathBuf, tools: Option<&[String]>) -> Vec<MemoryRecord> {
    load_all(memory_home)
        .into_iter()
        .filter(|r| r.record_type.as_deref() == Some("availability"))
        .filter(|r| tools.map_or(true, |ts| r.tool.as_ref().map_or(false, |t| ts.contains(t))))
        .collect()
}

/// Count records in the store.
pub fn count(memory_home: &PathBuf) -> usize {
    let records_dir = memory_home.join("records");
    if !records_dir.exists() {
        return 0;
    }
    fs::read_dir(&records_dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| {
                    e.path().extension().map_or(false, |ext| ext == "json")
                        && !e.file_name().to_string_lossy().starts_with(".tmp-")
                })
                .count()
        })
        .unwrap_or(0)
}

/// Return backend metadata.
pub fn backend_info(memory_home: &PathBuf) -> serde_json::Value {
    serde_json::json!({
        "adapter": "file",
        "memory_home": memory_home.to_string_lossy(),
        "records_dir": memory_home.join("records").to_string_lossy(),
        "record_count": count(memory_home),
        "TOOL_FIRST_MEMORY_HOME": std::env::var("TOOL_FIRST_MEMORY_HOME").unwrap_or_else(|_| "(not set)".to_string()),
    })
}

// ── internals ──────────────────────────────────────────────────────────

fn generate_filename(record: &MemoryRecord) -> String {
    let ts = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let agent = sanitize(&record.source_agent.clone().unwrap_or_default(), 40);
    let tool = sanitize(&record.tool.clone().unwrap_or_default(), 40);
    let rtype = sanitize(&record.record_type.clone().unwrap_or_default(), 40);
    let uid = &Uuid::new_v4().to_string()[..4];
    format!("{ts}-{agent}-{tool}-{rtype}-{uid}.json")
}

fn load_all(memory_home: &PathBuf) -> Vec<MemoryRecord> {
    let records_dir = memory_home.join("records");
    if !records_dir.exists() {
        return Vec::new();
    }
    let entries = match fs::read_dir(&records_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    let mut records = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.extension().map_or(false, |e| e == "json") {
            continue;
        }
        if path
            .file_name()
            .map_or(false, |n| n.to_string_lossy().starts_with(".tmp-"))
        {
            continue;
        }
        if let Ok(contents) = fs::read_to_string(&path) {
            if let Ok(record) = serde_json::from_str::<MemoryRecord>(&contents) {
                records.push(record);
            }
        }
    }
    records
}

fn sanitize(s: &str, max_len: usize) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .chars()
        .take(max_len)
        .collect()
}

fn tokenize(text: &str) -> Vec<String> {
    let mut tokens: Vec<String> = text
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    let expanded: Vec<String> = tokens
        .iter()
        .filter(|t| t.contains('_'))
        .flat_map(|t| t.split('_').map(|s| s.to_string()))
        .collect();
    tokens.extend(expanded);
    tokens.sort();
    tokens.dedup();
    tokens
}

fn score_record(record: &MemoryRecord, terms: &[String]) -> usize {
    let mut parts: Vec<String> = Vec::new();
    if let Some(ref v) = record.category {
        parts.push(v.clone());
    }
    if let Some(ref v) = record.tool {
        parts.push(v.clone());
    }
    if let Some(ref v) = record.task {
        parts.push(v.clone());
    }
    if let Some(ref v) = record.status {
        parts.push(v.clone());
    }
    if let Some(ref v) = record.path {
        parts.push(v.clone());
    }
    if let Some(ref v) = record.version {
        parts.push(v.clone());
    }
    if let Some(ref v) = record.command_template {
        parts.push(v.clone());
    }
    if let Some(ref v) = record.command {
        parts.push(v.clone());
    }
    if let Some(ref v) = record.failure_reason {
        parts.push(v.clone());
    }
    if let Some(ref v) = record.source_agent {
        parts.push(v.clone());
    }
    if let Some(ref tags) = record.tags {
        parts.extend(tags.clone());
    }
    let haystack = tokenize(&parts.join(" "));
    terms.iter().filter(|t| haystack.contains(t)).count()
}
