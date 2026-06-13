# Agent Integration Guide

## Supported Agents

| Agent | Config File | Skill Directory | Agent Name |
|-------|-------------|-----------------|------------|
| Hermes Agent | `~/.hermes/SOUL.md` | `~/.hermes/skills/devops/tool-first-agent/` | `hermes` |
| Claude Code | `~/.claude/CLAUDE.md` | `~/.claude/skills/tool-first-agent/` | `claude-code` |
| Codex | `~/.codex/AGENTS.md` | `~/.codex/skills/tool-first-agent/` | `codex` |

## Shared Rules

All agents must:

1. Read `TOOL_FIRST_MEMORY_HOME` before any tool-memory operation.
2. Use the shared file-based runtime tool-memory home.
3. Not create private tool-memory when `TOOL_FIRST_MEMORY_HOME` exists.
4. Not treat tool-memory as authoritative Vault memory.
5. Not default-create `02-Rules/Tool-Inventory`.
6. Not copy full SKILL.md to a Vault rule directory.

Every record written by an agent must include `source_agent` identifying
which agent wrote it.

## Installation (All Agents)

```bash
git clone https://github.com/licat233/tool-first-agent.git
cd tool-first-agent
cargo build --release
cp target/release/tool-first /usr/local/bin/

# Verify
tool-first doctor
```

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `TOOL_FIRST_MEMORY_HOME` | Canonical shared runtime tool-memory home (highest priority) |
| `TOOL_FIRST_MEMORY_CONFIG` | Override config file location |
| `TOOL_FIRST_AGENT_NAME` | Agent name for records (`hermes`, `claude-code`, `codex`) |

```bash
# In shell profile (~/.zshrc, ~/.bashrc, etc.)
export TOOL_FIRST_MEMORY_HOME="/path/to/tool-memory"
export TOOL_FIRST_AGENT_NAME="claude-code"

# For macOS GUI apps:
launchctl setenv TOOL_FIRST_MEMORY_HOME "/path/to/tool-memory"
```

## Hermes Agent

Add the Tool-First Rule to `~/.hermes/SOUL.md`.
See `references/soul-rule-integration.md` for the full rule text.

Optionally configure MCP in `~/.hermes/config.yaml`.
See `references/mcp-integration.md` for the config snippet.

## Claude Code

Add the Tool-First Rule to `~/.claude/CLAUDE.md`.
See `references/claude-code-integration.md` for the full rule text.

## Codex

Add the Tool-First Rule to your Codex agent configuration:

```markdown
## Tool-First Rule

Before writing custom scripts, installing new software, or handling files/data
with ad-hoc code, check if an existing local tool already solves the problem.

1. **Run the one-step gate first**:
   `tool-first advise --task "<description>" --json`
2. If the decision is `use_existing_tool`, use the recommended tool before
   writing custom code.
3. If the decision is `verify_recalled_recipe`, re-detect the tool and reuse the
   remembered command if still valid.
4. If `advise` is unavailable or ambiguous, fall back to category -> registry
   query -> detect -> recall.
5. **Write code only when** tools are missing, fail, or the task requires custom logic.

If writing code, briefly state why: "No existing tool fits because …"

tool-memory is shared runtime infrastructure, not authoritative Vault memory.
Do not create private tool-memory when TOOL_FIRST_MEMORY_HOME exists.
SKILL.md is the sole execution rule source.
```

## Post-Installation Report

After installation, report:

- Which agent was configured
- Where tool-first-agent was installed
- Which tool-memory path is being used
- Whether `TOOL_FIRST_MEMORY_HOME` was detected
- Whether `.tool-memory-home` marker exists
- Any legacy or conflicting tool-memory paths found
- Which agent config file was updated
