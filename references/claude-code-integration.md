# CLAUDE.md Integration — Tool-First Rule

Add this rule to `~/.claude/CLAUDE.md` so Claude Code checks existing tools
before writing custom code. Do not install this skill into a project-local
`.claude/` directory unless the user explicitly asks for a project-specific
override.

## Recommended Rule Text

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
Do not default-create 02-Rules/Tool-Inventory.
SKILL.md is the sole execution rule source.
```

## How It Works

- `~/.claude/CLAUDE.md` is the default user-level rule file for this tool's
  Claude Code integration.
- Claude Code may also read project-local files, but this installer should not
  create project-local `.claude/` files unless explicitly requested.
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
mkdir -p ~/.claude/skills
git clone https://github.com/licat233/tool-first-agent.git ~/.claude/skills/tool-first-agent
```

## Environment Variables

```bash
export TOOL_FIRST_MEMORY_HOME="/path/to/tool-memory"
export TOOL_FIRST_AGENT_NAME="claude-code"

# For macOS GUI apps:
launchctl setenv TOOL_FIRST_MEMORY_HOME "/path/to/tool-memory"
launchctl setenv TOOL_FIRST_AGENT_NAME "claude-code"
```
