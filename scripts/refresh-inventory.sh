#!/usr/bin/env bash
set -euo pipefail

# Maintenance-only refresh. Do not run this for every task.

OUT_DIR="$HOME/.config/tool-inventory"
OUT_FILE="$OUT_DIR/inventory.txt"
TMPFILE="$(mktemp)"

mkdir -p "$OUT_DIR"

{
  echo "# Tool inventory maintenance snapshot"
  echo "# Generated: $(date '+%Y-%m-%d %H:%M:%S %z')"
  echo

  echo "=== BREW FORMULAE ==="
  brew list --formula 2>/dev/null | sort || true
  echo

  echo "=== BREW CASKS ==="
  brew list --cask 2>/dev/null | sort || true
  echo

  echo "=== UV TOOLS ==="
  uv tool list 2>/dev/null || true
  echo

  echo "=== PIP3 USER ==="
  pip3 list --user 2>/dev/null | tail -n +3 | awk '{print $1}' || true
  echo

  echo "=== NPM GLOBAL ==="
  npm list -g --depth=0 2>/dev/null | tail -n +2 | sed 's/├── //g; s/└── //g' || true
  echo

  echo "=== LOCAL BIN ==="
  ls "$HOME/.local/bin" 2>/dev/null | sort || true
  echo

  echo "=== HERMES BIN ==="
  ls "$HOME/.hermes/bin" 2>/dev/null | sort || true
  echo

  echo "=== LANGUAGE RUNTIMES ==="
  python3 --version 2>/dev/null || true
  node --version 2>/dev/null || true
  go version 2>/dev/null || true
  cargo --version 2>/dev/null || true
  rustc --version 2>/dev/null || true
  docker --version 2>/dev/null || true
} > "$TMPFILE"

mv "$TMPFILE" "$OUT_FILE"
echo "Wrote $OUT_FILE"
