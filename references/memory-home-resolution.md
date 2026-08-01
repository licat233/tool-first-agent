# Memory Home Resolution

This document defines the canonical path resolution rules for the shared
runtime tool-memory home.

## tool-memory Is Shared Runtime Infrastructure

tool-memory stores tool availability, verified command recipes, failed attempts,
blocked command patterns, and environment-specific operational notes.

It is **not** current truth. It is **not** user-approved long-term memory. It is
**not** a replacement for Vault governance.

## Resolution Priority

| Priority | Source | Description |
|----------|--------|-------------|
| 1 | `TOOLSCOUT_MEMORY_HOME` env var | Optional explicit override. |
| 2 | `memory_home` in config.yaml | User-level config. |
| 3 | `file.base_dir` in config.yaml | Legacy compat. |
| 4 | Default | `~/.config/toolscout/tool-memory` |

### Rules

1. Use `~/.config/toolscout/tool-memory` for normal installation.
2. Do not ask users to choose a memory home during normal installation.
3. Do not create Obsidian- or Vault-specific memory homes by default.
4. If `TOOLSCOUT_MEMORY_HOME` is intentionally set, treat it as the canonical
   override and do not silently fall back.
5. If the chosen directory does not exist, initialize it with
   `toolscout memory init`.
6. Add `.tool-memory-home` marker if missing.

## Default Location

```
~/.config/toolscout/tool-memory
```

Custom paths are advanced overrides only. Obsidian users should still use the
default path unless they explicitly choose to keep runtime infrastructure inside
a Vault.

## Prohibited High-Authority Paths

Do not place tool-memory in:
- `01-Facts/`
- `02-Rules/`
- `03-Insights/`
- `05-Truth/`

## .tool-memory-home Marker

```json
{
  "type": "toolscout-memory-home",
  "version": "1.0",
  "canonical": true,
  "source": "TOOLSCOUT_MEMORY_HOME",
  "adapter": "file",
  "authority": "runtime-infrastructure",
  "vault_authority": "none",
  "description": "Canonical shared runtime tool-memory home for local agents. Not authoritative Vault memory."
}
```

## .tool-memory-redirect Marker

For old paths:

```json
{
  "redirect_to": "/path/to/new/tool-memory",
  "reason": "Canonical tool-memory home moved to shared runtime-infrastructure path.",
  "do_not_write_here": true
}
```

## macOS GUI Apps

```bash
launchctl setenv TOOLSCOUT_MEMORY_HOME "/path/to/tool-memory"
```
