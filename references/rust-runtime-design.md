# Rust Runtime Design

This document describes the architecture of the Rust runtime core in `tool-first-agent`.

## Overview

The Rust runtime core is a single binary (`tool-first`) that provides:

- **CLI** — for debugging, scripting, and low-frequency calls
- **MCP server** — for agent high-frequency calls over stdio JSON-RPC 2.0

```
CLI  = debugging, scripts, low-frequency entry point
MCP  = Agent high-frequency entry point
```

## Module Map

| Module | Responsibility |
|--------|---------------|
| `main.rs` | CLI parsing (clap), command dispatch |
| `config.rs` | Load `config.yaml`, resolve adapter name |
| `resolver.rs` | `TOOL_FIRST_MEMORY_HOME` resolution, `.tool-memory-home` / `.tool-memory-redirect` markers, conflict detection |
| `registry.rs` | Load and query `registry/tools.yaml` |
| `detect.rs` | Detect installed tools via `which`, known paths, version checks |
| `memory.rs` | `MemoryRecord` struct, `MemoryAdapter` trait, adapter factory |
| `adapters/file.rs` | File adapter: one-record-per-file, atomic writes, in-memory search |
| `adapters/sqlite.rs` | SQLite adapter: rusqlite + bundled SQLite |
| `mcp.rs` | MCP stdio server (JSON-RPC 2.0 over stdin/stdout) |

## Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing |
| `serde` / `serde_json` / `serde_yaml` | Serialization |
| `chrono` | Timestamps |
| `uuid` | Unique record IDs |
| `dirs` | Platform-specific home/config directories |
| `glob` | File pattern matching |
| `which` | Find executables on PATH |
| `sha2` | PATH fingerprinting |
| `rusqlite` (bundled) | SQLite adapter |
| `tokio` | Async runtime (reserved for future MCP expansion) |

## Design Principles

1. **Single binary** — `tool-first` is a statically-linked Rust binary. No Python, no Node, no runtime dependencies.
2. **Zero config defaults** — works out of the box with sensible defaults.
3. **TOOL_FIRST_MEMORY_HOME first** — env var is always the highest priority.
4. **Atomic writes** — file adapter uses `.tmp` + `rename` for crash safety.
5. **No dangerous operations** — the core never executes registry commands on behalf of the agent. It only suggests.
6. **Shared state** — all agents point to the same tool-memory home. Records include `source_agent` for provenance.

## MCP Protocol

The MCP server implements a minimal JSON-RPC 2.0 over stdio:

```json
→ {"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"query_registry","arguments":{"category":"document"}}}
← {"jsonrpc":"2.0","id":1,"result":{"results":[...]}}
```

Supported methods:
- `initialize` — MCP handshake
- `tools/list` — list available tools
- `tools/call` — invoke a tool by name

## Future Extensions

- FTS5-based search in the SQLite adapter
- MCP over TCP/SSE (not just stdio)
- Plugin system for custom adapters
- WASM build for browser-based agents
