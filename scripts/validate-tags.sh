#!/usr/bin/env bash
set -euo pipefail

if ! command -v hindsight >/dev/null 2>&1; then
  echo "hindsight CLI not found" >&2
  exit 2
fi

echo "Checking tag list for tool-inventory..."
hindsight tag list default | grep "tool-inventory" || true

echo
echo "Testing recall with --tags tool-inventory..."
hindsight memory recall default "pdf conversion" --tags tool-inventory --output json
