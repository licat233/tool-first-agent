# tool-first-agent update guide

This document explains how to propagate the reduced auto-trigger behavior across
Codex, Claude Code, and Hermes Agent.

本文说明如何把本次“减少误触发”的修复同步到 Codex、Claude Code 和
Hermes Agent。

## What Changed

The update has two parts:

1. `tool-first` CLI recommendation logic was tightened.
   - Normal conversation, explanations, code reading, repository summaries, and
     simple inspection should return `write_code_or_use_other_skill`.
   - Real tool tasks such as image resize, format conversion, extraction,
     compression, query, download, OCR, and parsing can still recommend local
     tools.
2. Agent rule text was narrowed.
   - The tool-first gate should run only before custom scripts, installing
     tools, or file/data conversion, extraction, transformation, or batch
     processing.
   - It should not run for normal chat, explanations, planning, code review,
     repository summaries, or simple commands such as `rg`, `sed`, `cat`, `ls`,
     and `git status`.

## Update the CLI

Build the new binary:

```bash
cd /Users/licat/Desktop/project/tool-first-agent
cargo build --release
```

Install it to the command path used by all agents:

```bash
cp target/release/tool-first ~/.local/bin/tool-first
```

If executing `~/.local/bin/tool-first` hangs on this machine, use a wrapper
instead:

```bash
mv ~/.local/bin/tool-first ~/.local/bin/tool-first.real-$(date +%Y%m%d)
printf '%s\n' '#!/bin/sh' 'exec /Users/licat/Desktop/project/tool-first-agent/target/release/tool-first "$@"' > ~/.local/bin/tool-first
chmod +x ~/.local/bin/tool-first
```

Verify:

```bash
tool-first --version
tool-first advise --task "看一下 README.md 总结项目" --json
tool-first advise --task "resize png image to 800px" --json
```

Expected behavior:

- README summary task: `decision` should be `write_code_or_use_other_skill`.
- Image resize task: `decision` should be `use_existing_tool` when `magick` or
  another registered image tool is available.

## Update Codex

Sync the skill:

```bash
cp /Users/licat/Desktop/project/tool-first-agent/SKILL.md ~/.codex/skills/tool-first-agent/SKILL.md
```

Update `~/.codex/AGENTS.md` so its Tool-First Rule uses this wording:

```markdown
## Tool-First Rule

Before writing custom scripts, installing new software, or doing file/data
conversion, extraction, transformation, or batch processing with ad-hoc code,
check if an existing local tool already solves the problem.

Do not run this gate for ordinary conversation, explanations, code reading,
planning, code review, repository summaries, or simple inspection commands such
as `rg`, `sed`, `cat`, `ls`, and `git status`.

1. **Run the one-step gate first only for in-scope tasks**:
   `tool-first advise --task "<description>" --json`
2. If the decision is `use_existing_tool`, use the recommended tool before
   writing custom code.
3. If the decision is `verify_recalled_recipe`, re-detect the tool and reuse the
   remembered command if still valid.
4. If `advise` is unavailable or ambiguous, fall back to category -> registry
   query -> detect -> recall.
5. **Write code only when** tools are missing, fail, or the task requires custom
   logic.

If writing code, briefly state why: "No existing tool fits because ..."

tool-memory is shared runtime infrastructure, not authoritative Vault memory.
Do not create private tool-memory when TOOL_FIRST_MEMORY_HOME exists.
SKILL.md is the sole execution rule source.
```

Restart Codex sessions that already loaded the old rule.

## Update Claude Code

Sync the skill:

```bash
cp /Users/licat/Desktop/project/tool-first-agent/SKILL.md ~/.claude/skills/tool-first-agent/SKILL.md
```

Update `~/.claude/CLAUDE.md` with the same Tool-First Rule block shown in the
Codex section.

Restart Claude Code sessions that already loaded the old rule.

## Update Hermes Agent

Sync the skill:

```bash
cp /Users/licat/Desktop/project/tool-first-agent/SKILL.md ~/.hermes/skills/devops/tool-first-agent/SKILL.md
```

Update `~/.hermes/SOUL.md` with the same Tool-First Rule block shown in the
Codex section.

Restart Hermes Agent sessions that already loaded the old rule.

## MCP Notes

If an agent uses `tool-first mcp serve`, no MCP config change is required when
the command path still resolves to `tool-first`.

If the MCP config points to an old absolute binary path, update it to one of:

```bash
/Users/licat/.local/bin/tool-first mcp serve
/Users/licat/Desktop/project/tool-first-agent/target/release/tool-first mcp serve
```

## Do Not

- Do not keep the old broad rule text that says to always run the gate before
  handling files/data.
- Do not copy `SKILL.md` into a Vault rule directory as a second source of truth.
- Do not create a private tool-memory home when `TOOL_FIRST_MEMORY_HOME` exists.
- Do not treat tool-memory as authoritative long-term memory.
