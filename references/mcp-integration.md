# MCP Integration

`toolscout` provides a built-in MCP server via `toolscout mcp serve`.

## How It Works

The MCP server runs as a stdio JSON-RPC 2.0 process. The host agent (Hermes,
Claude Code, Codex) launches it and communicates over stdin/stdout.

```bash
toolscout memory init --json  # run once for a new intended memory home
toolscout mcp serve
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
  toolscout:
    command: "/path/to/toolscout"
    args: ["mcp", "serve"]
    env:
      TOOLSCOUT_MEMORY_HOME: "/path/to/tool-memory"
      TOOLSCOUT_AGENT_NAME: "hermes"
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
mcp_toolscout_<tool_name>
```

## Claude Code Integration

Claude Code supports MCP natively. Add the server at user scope so it is
available across all projects:

```bash
claude mcp add toolscout \
  --scope user \
  -e TOOLSCOUT_MEMORY_HOME="/path/to/tool-memory" \
  -e TOOLSCOUT_AGENT_NAME="claude-code" \
  -- /path/to/toolscout mcp serve
```

Verify:

```bash
claude mcp get toolscout
```

Remove:

```bash
claude mcp remove toolscout -s user
```

## Codex Integration

Codex also supports MCP natively:

```bash
codex mcp add toolscout \
  --env TOOLSCOUT_MEMORY_HOME="/path/to/tool-memory" \
  --env TOOLSCOUT_AGENT_NAME="codex" \
  -- /path/to/toolscout mcp serve
```

Verify:

```bash
codex mcp get toolscout
```

Remove:

```bash
codex mcp remove toolscout
```

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `TOOLSCOUT_MEMORY_HOME` | Canonical shared runtime tool-memory home |
| `TOOLSCOUT_MEMORY_CONFIG` | Override config file location |
| `TOOLSCOUT_AGENT_NAME` | Agent name for records |

## Smoke Test

```bash
# Verify the binary works
toolscout memory init --json
toolscout doctor

# Start MCP server and test a simple request
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | toolscout mcp serve
```
