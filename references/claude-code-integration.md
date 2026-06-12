# CLAUDE.md Integration — Tool-First Rule

Add this rule to `~/.claude/CLAUDE.md` so Claude Code checks existing tools
before writing custom code.

## Recommended Rule Text

```markdown
## Tool-First Rule

Before writing custom scripts, installing new software, or handling files/data
with ad-hoc code, check if an existing local tool already solves the problem.

1. **Classify the task** into a category: `document`, `pdf`, `image`, `media`,
   `data`, `search`, `archive`, `dev`, `web`, `ai`.
2. **Resolve the shared tool-memory home** — check `TOOL_FIRST_MEMORY_HOME` env var.
3. **Query the registry** for candidate tools:
   `tool-first registry query --category <cat> --json`
4. **Detect only those candidates** — do not perform blind filesystem scans:
   `tool-first tools detect --category <cat> --json`
5. **Recall past experience** from tool-memory:
   `tool-first memory recall --task "<description>" --json`
6. **Use an existing tool** when 1–3 commands can solve the task.
7. **Write code only when** tools are missing, fail, or the task requires custom logic.

If writing code, briefly state why: "No existing tool fits because …"

tool-memory is shared runtime infrastructure, not authoritative Vault memory.
Do not create private tool-memory when TOOL_FIRST_MEMORY_HOME exists.
Do not default-create 02-Rules/Tool-Inventory.
SKILL.md is the sole execution rule source.
```

## How It Works

- `~/.claude/CLAUDE.md` is automatically loaded by Claude Code as global
  instructions for every conversation.
- Claude Code skills are listed in the system-reminder's "available skills"
  section. The `tool-first-agent` skill is invoked via the `Skill` tool.
- Without a CLAUDE.md rule, the skill is available but only loaded when
  explicitly requested or when the user's message matches the skill description.

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
export TOOL_FIRST_AGENT_NAME="claude-code"

# For macOS GUI apps:
launchctl setenv TOOL_FIRST_MEMORY_HOME "/path/to/tool-memory"
launchctl setenv TOOL_FIRST_AGENT_NAME "claude-code"
```
