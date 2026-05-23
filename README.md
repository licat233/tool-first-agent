# Tool First Agent

Tool First Agent is a Hermes/Codex-style agent skill that helps AI agents use
existing local tools before writing new scripts.

It is designed for a common agent failure mode: the agent sees a simple task like
"read this docx", "convert Markdown to HTML", or "filter this JSON", then spends
time and tokens writing custom code even though `pandoc`, `qlmarkdown_cli`, `jq`,
`ffmpeg`, `rg`, or another mature local tool could solve it directly.

This skill gives the agent a lightweight workflow:

1. Classify the task category.
2. Recall prior tool experience from Hindsight memory.
3. Query a local tool registry.
4. Detect only the relevant candidate tools.
5. Use an existing tool when it can solve the job.
6. Record verified successes and failures for future agents.

## What It Does

- Provides a tool-first decision rule for agents.
- Maintains a local registry of candidate tools in `registry/tools.yaml`.
- Performs lazy, category-based tool detection instead of full-machine scans.
- Supports Hindsight memory in the `default` bank using `tool-inventory` tags.
- Separates candidate knowledge from verified experience:
  - `tools.yaml` means "this tool may be useful for this kind of task".
  - Hindsight memory means "this tool actually worked or failed on this machine".
- Includes scripts for querying, detecting, registering, recalling, and retaining
  tool records.

## Target Environment

This skill is primarily designed for:

- macOS developer machines
- Hermes Agent
- Claude Code / Codex-style coding agents
- Hindsight memory provider
- Agents that can run local command-line tools

It can still be adapted to Linux or other agent runtimes because most of the
registry and detection logic is plain Python and shell. The bundled defaults,
however, include macOS-specific paths such as `/Applications/...` app bundle
executables.

## Use Cases

Use this skill when an agent is about to:

- Read or convert Office documents, Markdown, HTML, EPUB, or PDFs
- Convert Markdown to HTML with tools like `qlmarkdown_cli` or `pandoc`
- Extract text from PDFs
- Resize, convert, or OCR images
- Convert, compress, or inspect audio/video files
- Search code or files
- Process JSON, YAML, CSV, XML, or SQLite data
- Compress or extract archives
- Install a new utility
- Write a script for a task that might already have a command-line solution

The skill is especially useful in multi-agent setups where several agents share
memory and should learn from each other's successful tool choices.

## Repository Layout

```text
.
├── SKILL.md
├── registry/
│   └── tools.yaml
├── references/
│   ├── hindsight-memory-format.md
│   ├── registry-schema.md
│   └── scanning-policy.md
└── scripts/
    ├── detect-tools.py
    ├── import-registry-to-hindsight.py
    ├── query-registry.py
    ├── recall-tool-memory.sh
    ├── refresh-inventory.sh
    ├── register-tool.py
    ├── retain-tool-memory.py
    └── validate-tags.sh
```

## Installation

Clone or copy this folder into your Hermes skills directory, for example:

```bash
mkdir -p ~/.hermes/skills/devops
git clone https://github.com/licat233/tool-first-agent.git \
  ~/.hermes/skills/devops/tool-first-agent
```

Make sure the scripts are executable:

```bash
chmod +x ~/.hermes/skills/devops/tool-first-agent/scripts/*
```

## Agent Prompt Integration

Installing the skill is not enough by itself. The agent also needs a short
global rule that tells it when to load the skill.

Add a compact rule like this to your global `SOUL.md`, profile system prompt, or
equivalent agent instruction file:

```markdown
## Tool-First Rule

Before writing custom scripts, installing new utilities, or handling files/data
with ad-hoc code, load and follow the `tool-first-agent` skill.

This applies to file conversion, document parsing, text search, compression,
image/media processing, JSON/CSV/YAML/XML/SQLite work, web scraping, API
interaction, package/tool installation, and similar tool-shaped tasks.

Use existing local tools when the task can be solved in 1-3 commands. Do not
write replacement scripts for tasks already covered by the `tool-first-agent`
registry or verified Hindsight `tool-inventory` memories.

Write custom code only when no suitable tool exists, the tool fails, or the task
requires custom logic.
```

For Hermes, a practical global location is:

```text
~/.hermes/SOUL.md
```

If you use multiple Hermes profiles, avoid manually copying this rule into every
profile. Prefer a generated SOUL layout:

```text
~/.hermes/SOUL.md
  global rules, including the Tool-First Rule

~/.hermes/profiles/<profile>/SOUL.md.profile
  profile-specific rules

~/.hermes/profiles/<profile>/SOUL.md
  generated file = global rules + profile extension
```

Then regenerate profile prompts after changing global rules:

```bash
~/.hermes/scripts/sync-soul.sh --force
```

The important part is that the final system prompt for every active agent
contains the short Tool-First Rule. The detailed workflow and tool registry stay
inside this skill.

## Basic Workflow

Find candidate tools for a task:

```bash
python3 ~/.hermes/skills/devops/tool-first-agent/scripts/query-registry.py \
  --category document \
  --task "markdown html fast"
```

Detect only the tools in one category:

```bash
python3 ~/.hermes/skills/devops/tool-first-agent/scripts/detect-tools.py \
  --category document
```

Detect one specific tool:

```bash
python3 ~/.hermes/skills/devops/tool-first-agent/scripts/detect-tools.py \
  --tool qlmarkdown_cli \
  --json
```

Recall prior tool experience from Hindsight:

```bash
bash ~/.hermes/skills/devops/tool-first-agent/scripts/recall-tool-memory.sh \
  "convert markdown to html" \
  document
```

## Registering a New Tool

After installing a new tool, register it in `registry/tools.yaml`:

```bash
python3 ~/.hermes/skills/devops/tool-first-agent/scripts/register-tool.py qlmarkdown_cli \
  --category document \
  --priority 25 \
  --handle "Fast batch conversion from Markdown to HTML" \
  --command markdown_to_html='qlmarkdown_cli {input} -o {output}' \
  --fallback pandoc
```

By default, registration updates the local registry and runs detection. It does
not write a recipe to Hindsight.

To also retain the tool availability result in Hindsight:

```bash
python3 ~/.hermes/skills/devops/tool-first-agent/scripts/register-tool.py qlmarkdown_cli \
  --category document \
  --priority 25 \
  --handle "Fast batch conversion from Markdown to HTML" \
  --command markdown_to_html='qlmarkdown_cli {input} -o {output}' \
  --fallback pandoc \
  --retain
```

The `--retain` flag records availability only. A command template becomes a
verified recipe only after the tool has successfully completed a real task.

## Recording Verified Results

When a command succeeds in a real task, retain a verified recipe:

```bash
python3 ~/.hermes/skills/devops/tool-first-agent/scripts/retain-tool-memory.py \
  --record-type recipe \
  --category document \
  --task extract_text_from_docx \
  --tool pandoc \
  --command-template 'pandoc {input} -t plain' \
  --status verified_success
```

When a tool fails in a meaningful way, retain a scoped failure:

```bash
python3 ~/.hermes/skills/devops/tool-first-agent/scripts/retain-tool-memory.py \
  --record-type failure \
  --category document \
  --task convert_legacy_doc_to_docx \
  --tool libreoffice \
  --status failed_once \
  --failure-reason "headless conversion timed out"
```

## Hindsight Memory Model

This skill assumes Hindsight has a `default` bank. Tool memories are written to
that bank with tags:

```text
tool-inventory
tool-category-<category>
```

Example categories:

```text
tool-category-document
tool-category-pdf
tool-category-data
tool-category-media
```

The design intentionally avoids creating a separate memory bank. This is useful
for setups where the agent memory provider can only be configured with one bank.

## Categories

The default registry includes these categories:

- `document`
- `pdf`
- `image`
- `media`
- `data`
- `search`
- `archive`
- `dev`
- `web`
- `ai`

Each category contains candidate tools, command templates, detection names,
fallbacks, and short capability notes.

## Scanning Policy

This skill does not scan the whole machine.

Allowed detection methods include:

- `command -v <tool>`
- exact checks in known bin directories
- exact checks for declared macOS app bundle executables
- lightweight version commands

Disallowed for normal task execution:

- `find /`
- `find ~`
- broad recursive scans of `/Applications`
- package-manager full scans before every task

Full inventory refreshes are maintenance operations, not the default path.

## Design Philosophy

The core distinction is:

```text
registry/tools.yaml = candidate capability
Hindsight memory = verified local experience
```

This prevents agents from treating a theoretical command template as a proven
solution. The registry helps the agent discover what to try; Hindsight helps it
remember what actually worked.
