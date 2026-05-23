# Hindsight Memory Format

All tool memories go into the `default` bank and must be tagged `tool-inventory`.
Use category tags as a secondary filter when possible.

## Availability

```json
{
  "namespace": "agent_tool_inventory",
  "memory_type": "tool_inventory",
  "record_type": "availability",
  "category": "document",
  "tool": "pandoc",
  "status": "available",
  "path": "~/.local/bin/pandoc",
  "version": "pandoc 3.1.12",
  "detection_method": "command_v",
  "checked_at": "2026-05-23T13:00:00+08:00",
  "confidence": 0.98,
  "tags": ["tool-inventory", "tool-category-document"]
}
```

## Recipe

```json
{
  "namespace": "agent_tool_inventory",
  "memory_type": "tool_inventory",
  "record_type": "recipe",
  "category": "document",
  "task": "extract_text_from_docx",
  "tool": "pandoc",
  "command_template": "pandoc {input} -t plain",
  "status": "verified_success",
  "scope": "local_machine",
  "verified_at": "2026-05-23T13:05:00+08:00",
  "confidence": 0.95,
  "tags": ["tool-inventory", "tool-category-document"]
}
```

## Failure

Failures must be scoped and low-confidence unless repeated.

```json
{
  "namespace": "agent_tool_inventory",
  "memory_type": "tool_inventory",
  "record_type": "failure",
  "category": "document",
  "task": "convert_legacy_doc_to_docx",
  "tool": "libreoffice",
  "status": "failed_once",
  "failure_reason": "headless conversion timed out",
  "verified_at": "2026-05-23T13:10:00+08:00",
  "confidence": 0.35,
  "tags": ["tool-inventory", "tool-category-document"]
}
```

## Rules

- Store one tool or recipe per memory.
- Do not store long command output.
- Do not store guesses as verified facts.
- Use stable document IDs when updating a known record.
- Treat memories without `tool-inventory` as non-authoritative for tool choice.
