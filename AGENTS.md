# AGENTS

## ToolScout

Use the local `toolscout` skill when an operation may otherwise lead to
incidental custom code for commodity local tool work.

Do not invoke ToolScout at task start. Invoke it only immediately before writing
incidental code that would reimplement a commodity local operation, or before
installing a dependency for that operation.

For in-scope operations:

1. Load `SKILL.md`.
2. Run `toolscout advise --task "<operation>" --intent avoid_custom_code --category <category> --json`.
3. Prefer the recommended installed tool when the decision is `use_existing_tool`.
4. Re-detect a recalled tool before reusing a remembered recipe.
5. Write code when the user requested implementation, custom logic is required,
   or no suitable mature local tool is available.

Shared runtime paths:

```text
Binary: /Users/licat/.local/bin/toolscout
Skill: /Users/licat/.codex/skills/toolscout
Memory: /Users/licat/.config/toolscout/tool-memory
```

`tool-memory` is shared runtime infrastructure, not authoritative Vault memory.
Use the default memory home at `~/.config/toolscout/tool-memory`; treat
`TOOLSCOUT_MEMORY_HOME` as an optional explicit override only.
