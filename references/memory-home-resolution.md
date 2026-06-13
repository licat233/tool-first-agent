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
| 1 | `TOOL_FIRST_MEMORY_HOME` env var | Highest priority. All agents use this. |
| 2 | `memory_home` in config.yaml | User-level config. |
| 3 | `file.base_dir` in config.yaml | Legacy compat. |
| 4 | Default | `~/.config/tool-first-agent/tool-memory` |

### Rules

1. If `TOOL_FIRST_MEMORY_HOME` is set, treat it as the canonical home.
2. Do not create private tool-memory elsewhere.
3. Do not silently fall back while it exists.
4. If the directory does not exist, initialize it after confirming intent with `tool-first memory init`.
5. Add `.tool-memory-home` marker if missing.

## Recommended Locations

### Vault-external

```
~/AI-Runtime/tool-first-agent/tool-memory
```

### ARMOR Enterprise Vault

```
<ARMORVault>/92-Logs/_shared/tool-memory/
```

### PAMA Personal Vault

```
<PAMAVault>/08-Working-Memory/_runtime/tool-memory/
```

### Prohibited

Do not place tool-memory in:
- `01-Facts/`
- `02-Rules/`
- `03-Insights/`
- `05-Truth/`

## .tool-memory-home Marker

```json
{
  "type": "tool-first-agent-memory-home",
  "version": "1.0",
  "canonical": true,
  "source": "TOOL_FIRST_MEMORY_HOME",
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
launchctl setenv TOOL_FIRST_MEMORY_HOME "/path/to/tool-memory"
```
