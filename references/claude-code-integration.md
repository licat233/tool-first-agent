# CLAUDE.md Integration — ToolScout Rule

Add this rule to `~/.claude/CLAUDE.md` so Claude Code checks existing tools
before incidental code reimplements a commodity operation. Do not install this skill into a project-local
`.claude/` directory unless the user explicitly asks for a project-specific
override.

## Recommended Rule Text

```markdown
## ToolScout Rule

Do not invoke toolscout at task start. Invoke it only immediately before
writing incidental code that would reimplement a commodity local operation, or
before installing a dependency for that operation. Ordinary software
development does not trigger this rule.

1. For an in-scope operation run:
   `toolscout advise --task "<operation>" --intent avoid_custom_code --category <category> --json`
2. If the decision is `use_existing_tool`, use the recommended tool before
   writing custom code.
3. If the decision is `verify_recalled_recipe`, re-detect the tool and reuse the
   remembered command if still valid.
4. Recall tool-memory only when candidates are unavailable or failed.
5. Write code when explicitly requested, custom logic is required, or no
   suitable mature tool is available.

If writing code, briefly state why: "No existing tool fits because …"

tool-memory is shared runtime infrastructure, not authoritative Vault memory.
Do not create private tool-memory when TOOLSCOUT_MEMORY_HOME exists.
Do not default-create 02-Rules/Tool-Inventory.
SKILL.md is the sole execution rule source.
```

## How It Works

- `~/.claude/CLAUDE.md` is the default user-level rule file for this tool's
  Claude Code integration.
- Claude Code may also read project-local files, but this installer should not
  create project-local `.claude/` files unless explicitly requested.
- Claude Code skills are listed in the system-reminder's "available skills"
  section. The `toolscout` skill is invoked via the `Skill` tool.
- Without a CLAUDE.md rule, the skill is available but only loaded when
  explicitly requested or when the user's message matches the skill description.

## Installation

```bash
git clone https://github.com/licat233/toolscout.git
cd toolscout
cargo build --release
cp target/release/toolscout /usr/local/bin/
mkdir -p ~/.claude/skills
git clone https://github.com/licat233/toolscout.git ~/.claude/skills/toolscout
```

## Environment Variables

```bash
export TOOLSCOUT_MEMORY_HOME="/path/to/tool-memory"
export TOOLSCOUT_AGENT_NAME="claude-code"

# For macOS GUI apps:
launchctl setenv TOOLSCOUT_MEMORY_HOME "/path/to/tool-memory"
launchctl setenv TOOLSCOUT_AGENT_NAME "claude-code"
```
