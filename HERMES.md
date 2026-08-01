# HERMES

## ToolScout

For Hermes Agent, use ToolScout as a pre-code gate, not a task router.

Do not invoke ToolScout at task start. Invoke it only immediately before writing
incidental code that would reimplement a commodity local operation, or before
installing a dependency for that operation.

When this rule applies:

1. Load the `toolscout` skill.
2. Run `toolscout advise --task "<operation>" --intent avoid_custom_code --category <category> --json`.
3. Use the recommended existing tool when the decision is `use_existing_tool`.
4. Re-detect recalled tools before using remembered recipes.
5. Write code only when explicitly requested, custom logic is needed, or no
   suitable mature local tool is available.

Installed paths:

```text
Binary: /Users/licat/.local/bin/toolscout
Skill: /Users/licat/.hermes/skills/devops/toolscout
Memory: /Users/licat/.config/toolscout/tool-memory
```

Do not copy ToolScout tool-memory into Hermes runtime memory. Tool-memory uses
the default home `~/.config/toolscout/tool-memory`, is shared runtime
infrastructure, and remains low-authority.
