# MCP Integration

`tool-first-agent` provides a built-in MCP server via `tool-first mcp serve`.

## How It Works

The MCP server runs as a stdio JSON-RPC 2.0 process. The host agent (Hermes,
Claude Code, Codex) launches it and communicates over stdin/stdout.

```bash
tool-first memory init --json  # run once for a new intended memory home
tool-first mcp serve
```

## Available MCP Tools

| Tool | Description | Input |
|------|-------------|-------|
| `advise_tool_use` | Recommend existing tools before code | `task`, `category?`, `limit?` |
| `resolve_memory_home` | Resolve canonical memory home | — |
| `query_registry` | Find candidate tools | `category?`, `task?` |
| `detect_candidates` | Detect installed tools | `category?`, `tools?` |
| `recall_memory` | Search tool-memory | `query`, `category?`, `limit?` |
| `record_memory` | Persist a record | `record_type`, `category`, `tool`, `status`, ... |
| `check_conflicts` | Check for multiple memory homes | — |
| `doctor` | Run diagnostics | — |

## Hermes Integration

Add to `~/.hermes/config.yaml`:

```yaml
mcp_servers:
  tool_first:
    command: "/path/to/tool-first"
    args: ["mcp", "serve"]
    env:
      TOOL_FIRST_MEMORY_HOME: "/path/to/tool-memory"
      TOOL_FIRST_AGENT_NAME: "hermes-mcp"
    timeout: 120
    connect_timeout: 60
    tools:
      include:
        - advise_tool_use
        - resolve_memory_home
        - query_registry
        - detect_candidates
        - recall_memory
        - record_memory
        - check_conflicts
        - doctor
      resources: false
      prompts: false
```

Hermes registers MCP tools as:

```text
mcp_tool_first_<tool_name>
```

## Claude Code Integration

Claude Code can launch the MCP server via a settings.json configuration.
See `references/agent-integration.md` for details.

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `TOOL_FIRST_MEMORY_HOME` | Canonical shared runtime tool-memory home |
| `TOOL_FIRST_MEMORY_CONFIG` | Override config file location |
| `TOOL_FIRST_AGENT_NAME` | Agent name for records |

## Smoke Test

```bash
# Verify the binary works
tool-first memory init --json
tool-first doctor

# Start MCP server and test a simple request
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | tool-first mcp serve
```
