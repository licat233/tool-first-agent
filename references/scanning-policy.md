# Scanning Policy

## Goal

Find whether known candidate tools are usable for the current task without scanning
the whole machine.

## Allowed Detection

Detection may use:

- `command -v <tool>`
- `which -a <tool>`
- Exact checks in known bin directories:
  - `~/.local/bin`
  - `~/.hermes/bin`
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
- First relevant task: scan only that category.
- Later tasks: use cache/memory when fresh.
- Tool failure: re-detect that tool.
- Tool install/uninstall: run a manual refresh script.

## Status Values

- `available`: executable found and optional version check succeeded.
- `present_unverified`: path found but version check failed or was unavailable.
- `missing`: no path found.
- `failed_once`: a prior execution failed; recheck before avoiding the tool.
- `stale`: cached result expired.
- `superseded`: newer memory replaces this record.
