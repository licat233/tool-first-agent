#!/usr/bin/env bash
set -euo pipefail

QUERY="${1:-}"
if [[ -z "$QUERY" ]]; then
  echo "Usage: recall-tool-memory.sh <query> [category]" >&2
  exit 1
fi

CATEGORY="${2:-}"

if ! command -v hindsight >/dev/null 2>&1; then
  echo "hindsight CLI not found" >&2
  exit 2
fi

if [[ -n "$CATEGORY" ]]; then
  hindsight memory recall default "$QUERY" --tags "tool-category-$CATEGORY" --output json
else
  hindsight memory recall default "$QUERY" --tags tool-inventory --output json
fi
