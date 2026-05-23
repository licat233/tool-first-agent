---
name: tool-first-inventory
description: |
  Use this skill before writing scripts, installing tools, or handling files/data
  when an existing local tool may already solve the task. It provides a tool-first
  workflow with lazy category-based detection, local registries, and Hindsight
  default-bank memories tagged as tool-inventory.
---

# Tool First Inventory

Use this skill when the user asks to process files, convert formats, search text,
handle JSON/CSV/XML/SQLite, work with PDF/Office documents, process images/audio/video,
compress archives, install a utility, or write a script for a task that may already
have a local tool.

## Core Rule

Before writing custom code:

1. Classify the task category.
2. Recall Hindsight tool memories tagged `tool-inventory`.
3. Query the local registry for candidate tools.
4. Detect only those candidate tools.
5. Use an existing tool when 1-3 commands can solve the task.
6. Write code only when tools are missing, fail, or the task requires custom logic.

Do not perform blind filesystem scans. Do not run `find /`, `find ~`, or scan every
executable on the machine. This skill discovers known candidate tools, not every
binary on disk.

## Fast Workflow

```bash
# 1. Find candidate tools for a category or task
python3 ~/.hermes/skills/devops/tool-first-inventory/scripts/query-registry.py \
  --category document --task "extract docx text"

# 2. Detect only those tools
python3 ~/.hermes/skills/devops/tool-first-inventory/scripts/detect-tools.py \
  --category document

# 3. Recall shared tool experience from Hindsight
bash ~/.hermes/skills/devops/tool-first-inventory/scripts/recall-tool-memory.sh \
  "extract docx text"
```

After a tool succeeds, retain the reusable result:

```bash
python3 ~/.hermes/skills/devops/tool-first-inventory/scripts/retain-tool-memory.py \
  --record-type recipe \
  --category document \
  --task extract_text_from_docx \
  --tool pandoc \
  --command-template 'pandoc {input} -t plain' \
  --status verified_success
```

Register a newly installed tool:

```bash
python3 ~/.hermes/skills/devops/tool-first-inventory/scripts/register-tool.py qlmarkdown_cli \
  --category document \
  --priority 25 \
  --handle "Fast batch conversion from Markdown to HTML" \
  --command markdown_to_html='qlmarkdown_cli {input} -o {output}' \
  --fallback pandoc
```

## Categories

Use these category names with scripts:

- `document`: Word, Markdown, HTML, EPUB, Office-like extraction/conversion
- `pdf`: PDF text extraction, metadata, rendering, split/merge
- `image`: image conversion, resize, OCR, metadata
- `media`: audio/video conversion, compression, probing
- `data`: JSON, YAML, CSV, TSV, XML, SQLite
- `search`: text search, file finding, fuzzy filtering
- `archive`: zip, tar, gzip, zstd, lz4, xz, 7z
- `dev`: git, language runtimes, package managers, build/test helpers
- `web`: curl-like access, web search/scraping/browser automation
- `ai`: local LLMs, coding agents, memory tools

## Decision Rules

Use existing tools for:

- Standard format conversion
- Text extraction
- Search/filter/transform tasks
- Compression/decompression
- Media/image conversions
- One-shot inspection or probing

Write code for:

- Multi-step workflows with state
- Business-specific rules
- Complex parsing beyond standard tools
- Error recovery and retries
- Structured output assembled from several sources
- Cases where the candidate tools are missing or failed

If writing code, state the reason briefly: "Existing tools do not fit because ...".

## Storage Model

- Local registry: `registry/tools.yaml`
- Optional shared local files: `~/.config/tool-inventory/`
- Hindsight bank: `default`
- Required Hindsight tag: `tool-inventory`
- Optional category tag: `tool-category-<category>`

Hindsight memories are experience records, not the source of truth. The registry
defines candidate tools and command templates; Hindsight records what worked on
this machine.

## References

- Read `references/scanning-policy.md` before changing detection behavior.
- Read `references/hindsight-memory-format.md` before changing memory writes.
- Read `references/registry-schema.md` before editing `registry/tools.yaml`.
