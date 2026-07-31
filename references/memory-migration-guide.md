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

## Step 1: Choose Canonical Runtime Tool-Memory Home

| Option | Path | Best for |
|--------|------|----------|
| Vault-external | `~/AI-Runtime/toolscout/tool-memory` | Clean vault |
| ARMOR Vault | `<ARMORVault>/92-Logs/_shared/tool-memory/` | Multi-agent ARMOR |
| PAMA Vault | `<PAMAVault>/08-Working-Memory/_runtime/tool-memory/` | Personal PAMA |

Do **not** choose high-authority paths (`01-Facts/`, `02-Rules/`, `03-Insights/`, `05-Truth/`).

## Step 2: Set TOOLSCOUT_MEMORY_HOME

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
   "$TOOLSCOUT_MEMORY_HOME/records/" 2>/dev/null
```

## Step 5: Place Redirect Markers

```bash
OLD_PATH="$HOME/.config/tool-inventory/memory"
mkdir -p "$OLD_PATH"
cat > "$OLD_PATH/.tool-memory-redirect" <<EOF
{
  "redirect_to": "$TOOLSCOUT_MEMORY_HOME",
  "reason": "Canonical tool-memory home moved to shared runtime-infrastructure path.",
  "do_not_write_here": true
}
EOF
```

Repeat for each old path.

## Step 6: Update Config

Update `~/.config/toolscout/config.yaml`:

```yaml
memory_home: "/path/to/chosen/tool-memory"
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
