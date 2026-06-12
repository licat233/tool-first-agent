# Tool Memory Format

Tool experience records use a common JSON schema stored as individual files
under `<memory_home>/records/`.

## Required Fields

```json
{
  "namespace": "agent_tool_inventory",
  "memory_type": "tool_inventory",
  "record_type": "recipe",
  "category": "document",
  "tool": "pandoc",
  "task": "extract_text_from_docx",
  "status": "verified_success",
  "scope": "local_machine",
  "verified_at": "2026-06-12T15:30:00+00:00",
  "created_at": "2026-06-12T15:30:00+00:00",
  "confidence": 0.95,
  "tags": ["tool-inventory", "tool-category-document"],
  "source_agent": "claude-code",
  "os": "macos",
  "arch": "arm64",
  "authority": "runtime-infrastructure"
}
```

## Record Types

| Type | Purpose |
|------|---------|
| `availability` | Tool is present / missing / present-unverified |
| `recipe` | Command template worked for a task |
| `failure` | Command or tool failed for a reusable reason |
| `policy` | Durable rule for when to choose or avoid a tool |

## Status Values

| Status | Meaning |
|--------|---------|
| `available` | Executable found and version check succeeded |
| `present_unverified` | Path found but version check failed or unavailable |
| `missing` | No path found |
| `verified_success` | Command was tested and worked |
| `failed_once` | A prior execution failed; recheck before avoiding |
| `registry_candidate` | Imported from registry, not yet tested |
| `stale` | Cached result expired |
| `superseded` | Newer memory replaces this record |

## Priority

Status priority (highest first):

```
verified_success
> available
> registry_candidate
> failed_once
> blocked
```

## Additional Fields

| Field | Description |
|-------|-------------|
| `source_agent` | Which agent wrote this record (`hermes`, `claude-code`, `codex`) |
| `created_at` | ISO 8601 timestamp when the record was created |
| `os` | Operating system (`macos`, `linux`, `windows`) |
| `arch` | Architecture (`arm64`, `x86_64`, etc.) |
| `authority` | Always `runtime-infrastructure` for tool-memory |
| `command_template` | The verified command pattern |
| `failure_reason` | Why a command failed (for failure records) |

## Rules

- Store one tool or recipe per record.
- Verified successful recipes outrank registry candidates.
- Missing/failure records should include `failure_reason` when known.
- Only write `verified_success` after actual execution or detection.
- Do not write hallucinated memory.
- Do not write model guesses as verified tool-memory.
- Do not promote tool-memory into Vault authority automatically.

## File Store

Each record is stored as an individual JSON file under:

```
<tool-memory-home>/records/
```

Filename pattern: `{YYYYMMDD}-{HHMMSS}-{source_agent}-{tool}-{record_type}-{uuid4hex}.json`

Example: `20260612-153000-hermes-pandoc-recipe-8f3a.json`

Writes are atomic: data is written to a `.tmp` file first, then renamed.
