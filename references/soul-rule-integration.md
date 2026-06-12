# SOUL.md Integration — Tool-First Rule

Add this rule to `~/.hermes/SOUL.md` so Hermes Agent checks existing tools
before writing custom code.

## Recommended Rule Text

```markdown
## Tool-First Rule (G4)

Before writing custom scripts, installing tools, or handling files/data,
**always check whether an existing local tool already solves the problem.**

1. **Load the `tool-first-agent` skill** — it provides a registry of candidate
   tools, lazy category-based detection, and shared runtime tool-memory.
2. **Classify the task** into a category: `document`, `pdf`, `image`, `media`,
   `data`, `search`, `archive`, `dev`, `web`, `ai`.
3. **Resolve the shared tool-memory home** — check `TOOL_FIRST_MEMORY_HOME`.
4. **Query the registry** for candidate tools in that category.
5. **Detect only those candidates** — do not perform blind filesystem scans.
6. **Recall past experience** from the memory backend for what worked before.
7. **Use an existing tool** when 1–3 commands can solve the task.
8. **Write code only when** tools are missing, fail, or the task requires
   custom logic.

If writing code, briefly state why: "No existing tool fits because …"

tool-memory is shared runtime infrastructure, not authoritative Vault memory.
Do not create private tool-memory when TOOL_FIRST_MEMORY_HOME exists.
Do not default-create 02-Rules/Tool-Inventory.
SKILL.md is the sole execution rule source.
```

## How It Works

- `~/.hermes/SOUL.md` is loaded by `agent/prompt_builder.py::load_soul_md()`
  and injected as the agent identity (slot #1 in the system prompt).
- The rule references `tool-first-agent` by name, which triggers the
  skill-loading mechanism in the system prompt's "Skills" section.
- Without this rule, the skill is available but only loaded when explicitly
  requested (`/skill tool-first-agent`).

## Installation

```bash
git clone https://github.com/licat233/tool-first-agent.git
cd tool-first-agent
cargo build --release
cp target/release/tool-first /usr/local/bin/
```

## Environment Variables

```bash
export TOOL_FIRST_MEMORY_HOME="/path/to/tool-memory"
export TOOL_FIRST_AGENT_NAME="hermes"
```

## MCP Integration (Optional)

Optionally configure `tool-first mcp serve` as a Hermes MCP server.
See `references/mcp-integration.md` for the config snippet.
