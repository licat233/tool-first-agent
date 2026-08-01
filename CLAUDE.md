# CLAUDE

## ToolScout

For Claude Code, use ToolScout as a pre-code gate for commodity local tool work.

Do not invoke ToolScout for ordinary software development, conversation,
explanations, planning, code reading/review, repository inspection, or an
already-selected tool.

When an in-scope operation appears:

1. Load `SKILL.md`.
2. Run `toolscout advise --task "<operation>" --intent avoid_custom_code --category <category> --json`.
3. Use a recommended existing tool when available.
4. Re-detect recalled tools before using remembered recipes.
5. Write code only when explicitly requested, custom logic is needed, or no
   suitable mature tool is available.

Installed paths:

```text
Binary: /Users/licat/.local/bin/toolscout
Skill: /Users/licat/.claude/skills/toolscout
Memory: /Users/licat/.config/toolscout/tool-memory
```

`SKILL.md` is the sole execution rule source. Use the default memory home at
`~/.config/toolscout/tool-memory`; do not duplicate full ToolScout rules into
high-authority Vault memory.
