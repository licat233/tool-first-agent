# Memory Migration Guide

This document describes how to migrate tool-memory from old paths to the
canonical shared file-based runtime tool-memory home.

## Old Paths

| Old Path | Source |
|----------|--------|
| `~/.config/tool-inventory/memory/` | Python default |
| `<Vault>/02-Rules/Tool-Inventory/` | Old Obsidian path |
| `<Vault>/05-Truth/Frameworks/Tool-Inventory/` | Old PAMA path |
| `~/.claude/tool-memory/` | Agent-specific |
| `~/.hermes/tool-memory/` | Agent-specific |

## Step 1: Use The Canonical Runtime Tool-Memory Home

Default canonical path:

```text
~/.config/toolscout/tool-memory
```

Do not choose an Obsidian- or Vault-specific destination by default. Use a
custom path only when the user explicitly requests it.

Do **not** choose high-authority paths (`01-Facts/`, `02-Rules/`, `03-Insights/`, `05-Truth/`).

## Step 2: Optional TOOLSCOUT_MEMORY_HOME Override

Most migrations do not need this environment variable. Set it only when the user
intentionally chooses a custom memory home:

```bash
# In your shell profile (~/.zshrc, ~/.bashrc, etc.)
export TOOLSCOUT_MEMORY_HOME="/path/to/chosen/tool-memory"

# For macOS GUI apps:
launchctl setenv TOOLSCOUT_MEMORY_HOME "/path/to/chosen/tool-memory"
```

## Step 3: Detect Old Memory Homes

```bash
# Old default path
ls ~/.config/tool-inventory/memory/records/ 2>/dev/null

# Old Obsidian paths
find ~/Obsidian -name "tool-memory.jsonl" 2>/dev/null
find ~/Obsidian -path "*/02-Rules/Tool-Inventory/*" 2>/dev/null

# Agent-specific paths
ls ~/.claude/tool-memory/ 2>/dev/null
ls ~/.hermes/tool-memory/ 2>/dev/null
```

Or use the built-in conflict detector:

```bash
toolscout memory check-conflicts --json
```

## Step 4: Merge Records

```bash
mkdir -p "$TOOLSCOUT_MEMORY_HOME/records"

# Copy from old file-based path
cp ~/.config/tool-inventory/memory/records/*.json \
   "${TOOLSCOUT_MEMORY_HOME:-$HOME/.config/toolscout/tool-memory}/records/" 2>/dev/null
```

## Step 5: Place Redirect Markers

```bash
OLD_PATH="$HOME/.config/tool-inventory/memory"
mkdir -p "$OLD_PATH"
cat > "$OLD_PATH/.tool-memory-redirect" <<EOF
{
  "redirect_to": "${TOOLSCOUT_MEMORY_HOME:-$HOME/.config/toolscout/tool-memory}",
  "reason": "Canonical tool-memory home moved to shared runtime-infrastructure path.",
  "do_not_write_here": true
}
EOF
```

Repeat for each old path.

## Step 6: Update Config

Update `~/.config/toolscout/config.yaml`:

```yaml
memory_home: "~/.config/toolscout/tool-memory"
canonical: true
authority: "runtime-infrastructure"

write_policy:
  allow_create_new_home: false
  append_only: true
  atomic_write: true
```

## Step 7: Rebuild and Verify

```bash
cd toolscout
cargo build --release

toolscout doctor
toolscout memory check-conflicts --json
```

## Migration Rules

- Do not automatically delete old data.
- Do not automatically overwrite conflict records.
- Conflict records should preserve `source_agent`, `source_path`, `migrated_at`.
- Migrated tool-memory is still runtime infrastructure, not authoritative Vault memory.
- Do not write migration results into `02-Rules/Tool-Inventory`.
