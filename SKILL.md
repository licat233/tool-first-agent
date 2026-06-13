---
name: tool-first-agent
description: |
  Use this skill before writing scripts, installing tools, or handling files/data
  when an existing local tool may already solve the task. Provides a tool-first
  workflow powered by a Rust runtime core, shared file-based tool-memory, and
  the SKILL.md rule layer.
---

# Tool First Agent

Use this skill when the user asks to process files, convert formats, search text,
handle JSON/CSV/XML/SQLite, work with PDF/Office documents, process images/audio/video,
compress archives, install a utility, or write a script for a task that may already
have a local tool.

## Architecture

```
tool-first-agent = Rust runtime core + SKILL.md rule layer + shared file-based tool-memory
```

```text
tool-first-agent/
├── SKILL.md                    ← you are here (sole execution rule source)
├── README.md                   ← installation & usage entry
├── Cargo.toml                  ← workspace root
├── memory_config.yaml          ← default config
├── registry/
│   └── tools.yaml              ← candidate tool definitions (10 categories)
├── references/
│   ├── memory-home-resolution.md
│   ├── memory-migration-guide.md
│   ├── agent-integration.md
│   ├── claude-code-integration.md
│   ├── soul-rule-integration.md
│   ├── mcp-integration.md
│   ├── rust-runtime-design.md
│   ├── tool-memory-format.md
│   ├── registry-schema.md
│   └── scanning-policy.md
└── crates/tool-first/
    └── src/
        ├── main.rs             # CLI entry point
        ├── config.rs           # config loading
        ├── resolver.rs         # TOOL_FIRST_MEMORY_HOME + markers
        ├── registry.rs         # registry query
        ├── detect.rs           # tool detection
        ├── memory.rs           # MemoryRecord struct
        ├── file_store.rs       # file-based store (append-only, atomic writes)
        └── mcp.rs              # MCP stdio server
```

## Core Rule

Before writing custom code:

1. **Check relevant skills first** — skills encode specialized knowledge, API endpoints, and proven workflows that outperform general-purpose approaches. On Hermes, use `skills_list` and `skill_view`. On Claude Code, skills are listed in the system-reminder's "available skills" section — invoke via the `Skill` tool. Do not perform blind filesystem scans before checking skills.
2. Run `tool-first advise --task "<description>" --json` as the one-step gate when the CLI is available.
3. If `advise` is unavailable, classify the task category manually.
4. **Resolve the shared tool-memory home** — check `TOOL_FIRST_MEMORY_HOME` env var.
5. Query the registry for candidate tools.
6. Detect only those candidate tools.
7. Recall past experience from tool-memory.
8. Use an existing tool when 1-3 commands can solve the task.
9. Write code only when tools are missing, fail, or the task requires custom logic.
10. Record verified success, failure, or unsafe pattern into shared tool-memory.

Do not perform blind filesystem scans. Do not run `find /`, `find ~`, or scan every
executable on the machine.

## tool-memory Is Shared Runtime Infrastructure

tool-memory stores tool availability, verified command recipes, failed attempts,
blocked command patterns, and environment-specific operational notes.

It is **not** current truth. It is **not** user-approved long-term memory. It is **not**
a replacement for Vault governance. It must **not** be promoted into high-authority
memory automatically.

All agents (Codex, Claude Code, Hermes) share one canonical runtime tool-memory home.
Each record includes `source_agent` to identify which agent wrote it.

## TOOL_FIRST_MEMORY_HOME

This environment variable is the highest priority path entry for tool-memory.

Resolution priority:
1. `TOOL_FIRST_MEMORY_HOME` env var
2. `memory_home` key in `~/.config/tool-first-agent/config.yaml`
3. `file.base_dir` in config
4. Default: `~/.config/tool-first-agent/tool-memory`

If `TOOL_FIRST_MEMORY_HOME` is set, all agents use it as the canonical home.
Do not create private tool-memory elsewhere. Do not silently fall back while it exists.

See `references/memory-home-resolution.md` for the full resolution rules.

## Tool Memory Storage

tool-first-agent uses a file-based shared tool-memory store.

One record per JSON file, append-only, atomic writes (`.tmp` + rename).

```text
<tool-memory-home>/
  .tool-memory-home              # canonical marker
  records/
    20260612-153000-hermes-pandoc-recipe-8f3a.json
    20260612-153102-claude-code-ffmpeg-media-a92d.json
```

Database adapters are intentionally not supported in the baseline design for:
- lower maintenance cost
- safer multi-agent writes
- easier manual inspection
- better Obsidian compatibility
- better Git backup and diff
- simpler migration
- no database locking issues

See `references/tool-memory-format.md` for the full record schema.

## Multi-Agent Sharing

Rules:
- Agents may share tool-memory.
- Agents may **not** create private tool-memory when `TOOL_FIRST_MEMORY_HOME` exists.
- Agents may **not** treat tool-memory as current truth.
- Agents may **not** use another agent's execution record as approved SOP.
- If a tool recipe should become a formal rule, create a proposal or update SKILL.md
  through normal project maintenance.

Each record must include `source_agent` (`hermes`, `claude-code`, `codex`).

## Rust Runtime CLI

Build: `cargo build --release`

```bash
# Resolve the canonical memory home
tool-first advise --task "<describe the task>" --json
tool-first memory resolve --json

# Initialize the resolved memory home only after explicit intent
tool-first memory init --json

# Query the registry for candidate tools
tool-first registry query --category document --json
tool-first registry query --task "extract docx text" --json

# Detect which candidate tools are installed
tool-first tools detect --category document --json

# Persist availability records when detection should be retained
tool-first tools detect --category document --record --json

# Recall past experience from tool-memory
tool-first memory recall --task "extract docx text" --json

# Record a tool experience
tool-first memory record '{"record_type":"recipe","category":"document","tool":"pandoc","task":"extract_text_from_docx","status":"verified_success","command_template":"pandoc {input} -t plain","source_agent":"claude-code"}' --json

# Check for memory home conflicts
tool-first memory check-conflicts --json

# Run diagnostics
tool-first doctor

# Start MCP server
tool-first mcp serve
```

### MCP Tools

| MCP Tool | Description |
|----------|-------------|
| `advise_tool_use` | Recommend existing local tools before writing custom code |
| `resolve_memory_home` | Resolve the canonical tool-memory home |
| `query_registry` | Find candidate tools by category/task |
| `detect_candidates` | Detect which tools are installed |
| `recall_memory` | Search retained tool-memory |
| `record_memory` | Persist a tool experience record |
| `check_conflicts` | Check for multiple memory home candidates |
| `doctor` | Run diagnostic checks |

## Categories

- `document`: Word, Markdown, HTML, EPUB, Office-like extraction/conversion
- `pdf`: PDF text extraction, metadata, rendering, split/merge
- `image`: image conversion, resize, OCR, metadata
- `media`: audio/video conversion, compression, probing
- `data`: JSON, YAML, CSV, TSV, XML, SQLite
- `search`: text search, file finding, fuzzy filtering
- `archive`: zip, tar, gzip, zstd, lz4, xz, 7z
- `dev`: git, language runtimes, package managers, build/test helpers
- `web`: curl-like access, web search/scraping/browser automation
- `ai`: local LLMs, coding agents, memory tools

## Decision Rules

Priority order (highest first):

1. **Skill covers the task** — load and follow the skill.
2. **Existing tool solves it** — use the tool for standard format conversion, text extraction, search/filter/transform, compression, media conversions, or one-shot inspection.
3. **Write code** — for multi-step workflows with state, business-specific rules, complex parsing, error recovery, or when skills and tools both fail.

If writing code, state the reason briefly: "Existing tools do not fit because ...".

## Prohibited Behaviors

- Do not blindly scan the entire filesystem (`find /`, `find ~`).
- Do not create private tool-memory when `TOOL_FIRST_MEMORY_HOME` exists.
- Do not write LLM guesses as `verified_success` tool-memory.
- Do not treat tool-memory as Vault current truth.
- Do not automatically write tool-memory into high-authority Vault directories
  (`01-Facts/`, `02-Rules/`, `03-Insights/`, `05-Truth/`).
- Do not default-create `02-Rules/Tool-Inventory`.
- Do not copy the full SKILL.md into a Vault rule directory to form a second
  rule source.
- Do not write hallucinated or guessed tool-memory records.
- Only write tool-memory records after actual detection or execution.

## Agent Integration

This skill supports multiple AI agents. See `references/agent-integration.md` for
the unified guide.

- **Hermes**: Add Tool-First Rule to `~/.hermes/SOUL.md` (see `references/soul-rule-integration.md`).
- **Claude Code**: Add Tool-First Rule to `~/.claude/CLAUDE.md` (see `references/claude-code-integration.md`).
- **Codex**: Add Tool-First Rule to your Codex agent config (see `references/agent-integration.md`).

## Pitfalls

- **Agent-specific rule required for auto-activation.** The skill ships as a passive reference — it only triggers when explicitly loaded or when a matching rule exists. For Hermes, add a SOUL.md rule. For Claude Code, add a CLAUDE.md rule.
- **TOOL_FIRST_MEMORY_HOME takes precedence.** If this env var is set, it overrides all config file settings. Do not create private memory homes when it exists.
- **Workspace vs installed copy.** If you develop in a separate workspace, remember to sync changes to the installed location:
  - Hermes: `cp -r . ~/.hermes/skills/devops/tool-first-agent/`
  - Claude Code: `cp -r . ~/.claude/skills/tool-first-agent/`
  - Codex: `cp -r . ~/.codex/skills/tool-first-agent/`
- **macOS GUI apps** may not inherit shell environment variables. Use `launchctl setenv TOOL_FIRST_MEMORY_HOME "/path/to/tool-memory"`.
- **Path migration verification.** After changing any config path: run `tool-first doctor` and `tool-first memory check-conflicts --json`.

## References

- **`references/memory-home-resolution.md`** — TOOL_FIRST_MEMORY_HOME resolution rules and marker specs.
- **`references/memory-migration-guide.md`** — Migrating from old memory paths.
- **`references/agent-integration.md`** — Unified multi-agent integration guide.
- **`references/claude-code-integration.md`** — CLAUDE.md rule for auto-activation.
- **`references/soul-rule-integration.md`** — SOUL.md rule for auto-activation.
- **`references/mcp-integration.md`** — MCP server integration guide.
- **`references/rust-runtime-design.md`** — Rust runtime architecture.
- **`references/tool-memory-format.md`** — Record schema and rules.
- **`references/registry-schema.md`** — tools.yaml format.
- **`references/scanning-policy.md`** — What detection methods are allowed.
