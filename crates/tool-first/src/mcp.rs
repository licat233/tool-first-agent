use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, BufRead, Write};

use crate::detect;
use crate::memory::MemoryRecord;
use crate::registry;
use crate::resolver;

/// Run the MCP stdio server (JSON-RPC 2.0 over stdin/stdout).
pub fn run_stdio_server() -> Result<(), String> {
    let cfg = crate::config::load();
    let memory_home = resolver::resolve_memory_home(&cfg);
    crate::file_store::ensure_ready(&memory_home)?;
    let reg = registry::load_registry().unwrap_or_default();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let response = handle_request(&request, &memory_home, &reg);

        let resp_json = serde_json::to_string(&response).unwrap_or_default();
        writeln!(stdout, "{resp_json}").ok();
        stdout.flush().ok();
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: Option<String>,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

fn handle_request(
    req: &JsonRpcRequest,
    memory_home: &std::path::PathBuf,
    reg: &registry::Registry,
) -> JsonRpcResponse {
    let result = match req.method.as_str() {
        "tools/list" => {
            serde_json::json!({
                "tools": [
                    {
                        "name": "resolve_memory_home",
                        "description": "Resolve the canonical tool-memory home directory."
                    },
                    {
                        "name": "query_registry",
                        "description": "Find candidate tools in registry by category and/or task.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "category": { "type": "string" },
                                "task": { "type": "string" }
                            }
                        }
                    },
                    {
                        "name": "detect_candidates",
                        "description": "Detect whether registered candidate tools are installed.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "category": { "type": "string" },
                                "tools": { "type": "array", "items": { "type": "string" } }
                            }
                        }
                    },
                    {
                        "name": "recall_memory",
                        "description": "Search retained tool-memory for prior recipes, failures, or availability.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "query": { "type": "string" },
                                "category": { "type": "string" },
                                "limit": { "type": "integer", "default": 10 }
                            },
                            "required": ["query"]
                        }
                    },
                    {
                        "name": "record_memory",
                        "description": "Persist a tool experience record.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "record_type": { "type": "string", "enum": ["availability", "recipe", "failure", "policy"] },
                                "category": { "type": "string" },
                                "tool": { "type": "string" },
                                "task": { "type": "string" },
                                "status": { "type": "string" },
                                "command_template": { "type": "string" },
                                "failure_reason": { "type": "string" },
                                "confidence": { "type": "number" },
                                "source_agent": { "type": "string" }
                            },
                            "required": ["record_type", "category", "tool", "status"]
                        }
                    },
                    {
                        "name": "check_conflicts",
                        "description": "Check for multiple tool-memory home candidates."
                    },
                    {
                        "name": "doctor",
                        "description": "Run diagnostic checks."
                    }
                ]
            })
        }

        "tools/call" => {
            let params = req.params.clone().unwrap_or_default();
            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let args = params
                .get("arguments")
                .cloned()
                .unwrap_or(serde_json::json!({}));

            match tool_name {
                "resolve_memory_home" => {
                    let has_marker = resolver::has_marker(memory_home);
                    serde_json::json!({
                        "memory_home": memory_home.to_string_lossy(),
                        "TOOL_FIRST_MEMORY_HOME": std::env::var("TOOL_FIRST_MEMORY_HOME").unwrap_or_else(|_| "(not set)".to_string()),
                        "has_marker": has_marker,
                    })
                }
                "query_registry" => {
                    let category = args.get("category").and_then(|v| v.as_str());
                    let task = args.get("task").and_then(|v| v.as_str());
                    let results = registry::query(reg, category, task);
                    serde_json::json!({ "results": results })
                }
                "detect_candidates" => {
                    let category = args.get("category").and_then(|v| v.as_str());
                    let tools: Vec<String> = args
                        .get("tools")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default();
                    let results = detect::detect(reg, category, &tools);
                    serde_json::json!({ "results": results })
                }
                "recall_memory" => {
                    let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
                    let category = args.get("category").and_then(|v| v.as_str());
                    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
                    let results = crate::file_store::recall(memory_home, query, category, limit);
                    serde_json::json!({
                        "query": query,
                        "results": results,
                    })
                }
                "record_memory" => {
                    let mut record = MemoryRecord {
                        record_type: args
                            .get("record_type")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        category: args
                            .get("category")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        tool: args.get("tool").and_then(|v| v.as_str()).map(String::from),
                        task: args.get("task").and_then(|v| v.as_str()).map(String::from),
                        status: args
                            .get("status")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        command_template: args
                            .get("command_template")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        failure_reason: args
                            .get("failure_reason")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        confidence: args.get("confidence").and_then(|v| v.as_f64()),
                        source_agent: args
                            .get("source_agent")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        ..Default::default()
                    };
                    record.enrich();
                    match crate::file_store::retain(memory_home, &record) {
                        Ok(result) => serde_json::json!({ "saved": result.saved }),
                        Err(e) => serde_json::json!({ "error": e }),
                    }
                }
                "check_conflicts" => {
                    let candidates = resolver::detect_memory_homes();
                    let conflict = candidates.iter().filter(|c| c.path.exists()).count() > 1;
                    serde_json::json!({
                        "candidates": candidates,
                        "conflict": conflict,
                    })
                }
                "doctor" => doctor(memory_home, reg),
                _ => {
                    return JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: req.id.clone(),
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32601,
                            message: format!("Unknown tool: {tool_name}"),
                        }),
                    };
                }
            }
        }

        "initialize" => {
            serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "tool-first", "version": "0.1.0" }
            })
        }

        "notifications/initialized" => serde_json::json!(null),

        _ => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req.id.clone(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: format!("Unknown method: {}", req.method),
                }),
            };
        }
    };

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: req.id.clone(),
        result: Some(result),
        error: None,
    }
}

fn doctor(memory_home: &std::path::PathBuf, reg: &registry::Registry) -> Value {
    let has_marker = resolver::has_marker(memory_home);
    let backend_info = crate::file_store::backend_info(memory_home);
    let conflicts = resolver::detect_memory_homes();
    let conflict_count = conflicts.iter().filter(|c| c.path.exists()).count();

    serde_json::json!({
        "memory_home": memory_home.to_string_lossy(),
        "TOOL_FIRST_MEMORY_HOME": std::env::var("TOOL_FIRST_MEMORY_HOME").unwrap_or_else(|_| "(not set)".to_string()),
        "adapter": "file",
        "has_marker": has_marker,
        "backend": backend_info,
        "registry_categories": reg.len(),
        "conflicts": {
            "count": conflict_count,
            "candidates": conflicts,
        },
    })
}
