# Runtime Adapters

ToolScout is installed as one shared local runtime for Codex, Claude Code, and
Hermes Agent.

## Installed Paths

```text
Repository:
/Users/licat/Documents/macbook 维护/toolscout

Binary:
/Users/licat/.local/bin/toolscout

Shared tool-memory:
/Users/licat/.config/toolscout/tool-memory
```

## Agent Skill Paths

```text
Codex:
/Users/licat/.codex/skills/toolscout

Claude Code:
/Users/licat/.claude/skills/toolscout

Hermes Agent:
/Users/licat/.hermes/skills/devops/toolscout
```

All three paths are symlinks to the repository checkout.

## Environment

```bash
export TOOLSCOUT_AGENT_NAME="codex" # codex | claude-code | hermes
```

ToolScout uses the default memory home:

```text
/Users/licat/.config/toolscout/tool-memory
```

Do not set `TOOLSCOUT_MEMORY_HOME` for normal installs. It is only for explicit
custom path overrides. Agent-specific MCP entries set `TOOLSCOUT_AGENT_NAME` to
the correct agent.

## MCP

Configured user-level MCP servers:

```text
Codex: toolscout -> /Users/licat/.local/bin/toolscout mcp serve
Claude Code: toolscout -> /Users/licat/.local/bin/toolscout mcp serve
Hermes Agent: mcp_servers.toolscout in ~/.hermes/config.yaml
```

## Smoke Tests

```bash
toolscout --version
toolscout doctor
toolscout advise --task "extract text from docx" --intent avoid_custom_code --category document --json
```
