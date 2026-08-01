# SOUL.md Integration — ToolScout Rule

Add this rule to `~/.hermes/SOUL.md` so Hermes Agent checks existing tools
before incidental code reimplements a commodity operation.

## Recommended Rule Text

```markdown
## ToolScout Rule (G4)

Do not invoke toolscout at task start. Invoke it only immediately before
writing incidental code that would reimplement a commodity local operation, or
before installing a dependency for that operation. Ordinary software
development does not trigger this rule.

1. **Load the `toolscout` skill** — it provides a registry of candidate
   tools, lazy category-based detection, and shared runtime tool-memory.
2. For an in-scope operation run:
   `toolscout advise --task "<operation>" --intent avoid_custom_code --category <category> --json`
3. If the decision is `use_existing_tool`, use the recommended tool before
   writing custom code.
4. If the decision is `verify_recalled_recipe`, re-detect the tool and reuse the
   remembered command if still valid.
5. Recall tool-memory only when candidates are unavailable or failed.
6. Write code when explicitly requested, custom logic is required, or no
   suitable mature tool is available.

If writing code, briefly state why: "No existing tool fits because …"

tool-memory is shared runtime infrastructure, not authoritative Vault memory.
Do not create private tool-memory when TOOLSCOUT_MEMORY_HOME exists.
Do not default-create 02-Rules/Tool-Inventory.
SKILL.md is the sole execution rule source.
```

## How It Works

- `~/.hermes/SOUL.md` is loaded by `agent/prompt_builder.py::load_soul_md()`
  and injected as the agent identity (slot #1 in the system prompt).
- The rule references `toolscout` by name, which triggers the
  skill-loading mechanism in the system prompt's "Skills" section.
- Without this rule, the skill is available but only loaded when explicitly
  requested (`/skill toolscout`).

## Installation

```bash
git clone https://github.com/licat233/toolscout.git
cd toolscout
cargo build --release
cp target/release/toolscout /usr/local/bin/
```

## Environment Variables

```bash
export TOOLSCOUT_AGENT_NAME="hermes"

# Optional custom path only:
# export TOOLSCOUT_MEMORY_HOME="/path/to/tool-memory"
```

## MCP Integration (Optional)

Optionally configure `toolscout mcp serve` as a Hermes MCP server.
See `references/mcp-integration.md` for the config snippet.
