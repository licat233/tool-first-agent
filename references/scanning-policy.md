# Scanning Policy

## Goal

Find whether known candidate tools are usable for the current task without
scanning the whole machine.

## Allowed Detection

Detection may use:

- `command -v <tool>`
- `which -a <tool>`
- Exact checks in known bin directories:
  - `~/.local/bin`
  - `~/.hermes/bin`
  - `~/.cargo/bin`
  - `/opt/homebrew/bin`
  - `/usr/local/bin`
  - `/usr/bin`
  - `/bin`
- Exact checks for declared macOS app bundle binaries.
- Lightweight version commands declared in the registry.

## Disallowed Detection

Do not use:

- `find /`
- `find ~`
- Broad recursive scans of `/Applications`
- Broad scans of all executable files
- Package manager full scans during task execution

Full scans are maintenance operations only.

## Trigger Rules

- Agent startup: no scan.
- Ordinary conversation, explanations, code reading/review, and normal software
  development: no scan.
- Before incidental code would reimplement a commodity local operation: scan
  only that category and at most five candidates.
- Later tasks: use cache/memory when fresh.
- Tool failure: re-detect that tool.
- Tool install/uninstall: run `tool-first tools detect --category <cat>`.
- Recall tool-memory only after relevant registered candidates are unavailable,
  or when explicitly requested.
