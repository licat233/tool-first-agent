<div align="center">

# tool-first-agent

**Rust runtime core + SKILL.md rule layer + shared tool-memory**

**Rust 运行时核心 + SKILL.md 规则层 + 共享工具记忆**

[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Hermes Agent](https://img.shields.io/badge/Hermes%20Agent-Compatible-purple.svg)](https://hermes-agent.nousresearch.com/docs)
[![Claude Code](https://img.shields.io/badge/Claude%20Code-Compatible-blue.svg)](https://docs.anthropic.com/en/docs/claude-code)
[![Codex](https://img.shields.io/badge/Codex-Compatible-blue.svg)](https://openai.com/codex)

[English](#english) · [中文](#中文)

</div>

---

<a id="english"></a>

## English

### Architecture

```text
tool-first-agent = Rust runtime core + SKILL.md rule layer + shared tool-memory
```

- `SKILL.md` is the canonical execution rule source for agents.
- The Rust runtime core provides fast local CLI and MCP access.
- `tool-memory` is shared runtime infrastructure resolved by `TOOL_FIRST_MEMORY_HOME`.
- `tool-memory` is not authoritative Vault memory and must not be promoted into high-authority memory automatically.

### Three-Layer Design

| Layer | Responsibility |
|-------|---------------|
| **SKILL.md** | "How the agent should behave" — sole execution rule source |
| **Rust runtime core** | "Fast, stable queries and writes" — CLI / MCP / registry / detection / memory |
| **shared tool-memory** | "Multi-agent tool experience" — resolved by `TOOL_FIRST_MEMORY_HOME` |

**Do not default-create `02-Rules/Tool-Inventory`.** The executable tool-first behavior belongs in `SKILL.md`. A Vault rule may optionally contain a short reference pointing to `SKILL.md`, but must not duplicate the full tool-first rules.

### What the Rust Core Does

- Reads `TOOL_FIRST_MEMORY_HOME`
- Parses `~/.config/tool-first-agent/config.yaml`
- Validates `.tool-memory-home` marker
- Handles `.tool-memory-redirect` for legacy paths
- Detects multiple memory home conflicts
- Queries `registry/tools.yaml`
- Detects candidate tools
- Recalls tool-memory records
- Writes verified records
- Provides CLI
- Provides MCP server

### What the Rust Core Does NOT Do

- Does not execute dangerous commands on behalf of the agent
- Does not auto-install software
- Does not modify user business files
- Does not maintain a second copy of rules
- Does not write into `02-Rules/Tool-Inventory`
- Does not treat tool-memory as Vault authority

### Why This Exists

AI agents often write custom scripts when `pandoc`, `jq`, `ffmpeg`, or `magick` would solve the task in one command. This skill fixes that:

1. **Routes tasks to categories** — `document`, `pdf`, `image`, `media`, `data`, `search`, `archive`, `dev`, `web`, `ai`
2. **Queries a local registry** for candidate tools per category
3. **Detects only those candidates** — no blind filesystem scans
4. **Recalls past experience** — what worked last time on this machine
5. **Retains new experience** — records successes and failures for future use

```text
Before writing code → Load skill → Classify task → Query registry → Detect candidates → Recall experience → Use existing tool
```

### Quick Start

```bash
# Download pre-built binary (macOS, no Rust required)
curl -sL https://github.com/licat233/tool-first-agent/releases/download/v0.1.0/tool-first-universal-apple-darwin.tar.gz | tar xz
mv tool-first-universal /usr/local/bin/tool-first

# Verify
tool-first doctor

# Query registry
tool-first registry query --category document --json

# Detect tools
tool-first tools detect --category document --json
```

> Linux users: build from source with `cargo build --release` (requires Rust 1.75+).

### CLI Commands

```bash
tool-first memory resolve --json          # Resolve canonical memory home
tool-first memory recall --task <text>    # Search tool-memory
tool-first memory record '<json>' --json  # Persist a record
tool-first memory check-conflicts --json  # Check for path conflicts
tool-first registry query --category <c>  # Query registry by category
tool-first registry query --task <text>   # Query registry by task
tool-first tools detect --category <c>    # Detect installed tools
tool-first doctor                          # Run diagnostics
tool-first mcp serve                       # Start MCP stdio server
```

### MCP Server

Start the MCP server for agent integration:

```bash
tool-first mcp serve
```

This exposes `resolve_memory_home`, `query_registry`, `detect_candidates`,
`recall_memory`, `record_memory`, `check_conflicts`, and `doctor` as MCP tools
over stdio JSON-RPC 2.0.

See [`references/mcp-integration.md`](references/mcp-integration.md) for Hermes config snippets.

### Supported Agents

| Agent | Config File | Skill Directory |
|-------|-------------|-----------------|
| **Hermes Agent** | `~/.hermes/SOUL.md` | `~/.hermes/skills/devops/tool-first-agent/` |
| **Claude Code** | `~/.claude/CLAUDE.md` | `~/.claude/skills/tool-first-agent/` |
| **Codex** | `~/.codex/AGENTS.md` | `~/.codex/skills/tool-first-agent/` |

### Memory Adapters

The file adapter stores each tool-memory record as an individual JSON file:

```text
<tool-memory-home>/
  .tool-memory-home              # canonical marker
  records/
    20260612-153000-hermes-pandoc-recipe-8f3a.json
    20260612-153102-claude-code-ffmpeg-media-a92d.json
```

- One record per file, atomic writes (`.tmp` + rename)
- Zero dependencies, fully inspectable, Git-friendly
- Filename: `{timestamp}-{agent}-{tool}-{task}-{uuid}.json`

Obsidian users should point `TOOL_FIRST_MEMORY_HOME` to a low-authority runtime path inside their vault (e.g. `<Vault>/92-Logs/_shared/tool-memory/`).

### Registry

`registry/tools.yaml` defines candidate tools organized by category:

```yaml
document:
  description: "Document extraction and format conversion"
  tools:
    pandoc:
      priority: 20
      detect_names: [pandoc]
      version_args: ["--version"]
      handles: ["Convert Markdown, DOCX, HTML, EPUB and many text document formats"]
      commands:
        extract_docx_text: "pandoc {input} -t plain"
        docx_to_markdown: "pandoc {input} -t markdown -o {output}"
      fallbacks: [markitdown, libreoffice, textutil]
```

10 categories: `document`, `pdf`, `image`, `media`, `data`, `search`, `archive`, `dev`, `web`, `ai`.

### Configuration

`~/.config/tool-first-agent/config.yaml`:

```yaml
memory_home: "~/.config/tool-first-agent/tool-memory"
canonical: true
authority: "runtime-infrastructure"

write_policy:
  allow_create_new_home: false
  append_only: true
  atomic_write: true
```

### Environment Variables

| Variable | Purpose |
|----------|---------|
| `TOOL_FIRST_MEMORY_HOME` | Canonical shared runtime tool-memory home (highest priority) |
| `TOOL_FIRST_MEMORY_CONFIG` | Override config file location |
| `TOOL_FIRST_AGENT_NAME` | Agent name for records (`hermes`, `claude-code`, `codex`) |

macOS GUI apps may not inherit shell env vars. Use `launchctl setenv`:

```bash
launchctl setenv TOOL_FIRST_MEMORY_HOME "/path/to/tool-memory"
```

### Install with an Agent

You can paste this prompt into any supported AI agent to install and configure `tool-first-agent`:

````text
You are installing `tool-first-agent` for this local agent environment.

Repository:
https://github.com/licat233/tool-first-agent

Supported agents:
- Codex
- Claude Code
- Hermes

Goal:
Install and configure `tool-first-agent` so this agent follows a tool-first workflow before writing custom scripts.

Important architecture rule:
`tool-memory` is shared runtime infrastructure.

It is not authoritative Vault memory.
It is not current truth.
It is not a replacement for AI memory governance.

Rule-source principle:
SKILL.md is the canonical execution rule source.
The Rust runtime core provides CLI and MCP access.
Do not create or duplicate full tool-first rules under 02-Rules/Tool-Inventory or any other Vault rule directory.

## Step 1: Download binary

Download the pre-built binary from GitHub Releases (no Rust required):

macOS (universal, Intel + Apple Silicon):
  curl -sL https://github.com/licat233/tool-first-agent/releases/download/v0.1.0/tool-first-universal-apple-darwin.tar.gz | tar xz
  mv tool-first-universal /usr/local/bin/tool-first

macOS (Apple Silicon only):
  curl -sL https://github.com/licat233/tool-first-agent/releases/download/v0.1.0/tool-first-aarch64-apple-darwin.tar.gz | tar xz
  mv tool-first-aarch64 /usr/local/bin/tool-first

macOS (Intel only):
  curl -sL https://github.com/licat233/tool-first-agent/releases/download/v0.1.0/tool-first-x86_64-apple-darwin.tar.gz | tar xz
  mv tool-first-x86_64 /usr/local/bin/tool-first

If no pre-built binary is available for your platform, build from source:
  git clone https://github.com/licat233/tool-first-agent.git
  cd tool-first-agent
  cargo build --release
  cp target/release/tool-first /usr/local/bin/

## Step 2: Install skill files

Clone the repo to the agent's skill directory:

  git clone https://github.com/licat233/tool-first-agent.git /path/to/agent/skill/dir/tool-first-agent

Or download and extract the source archive.

## Step 3: Configure the Tool-First Rule

THIS STEP IS REQUIRED. Without it, the agent will not auto-trigger tool-first behavior.

For Hermes Agent — add to ~/.hermes/SOUL.md:

  ## Tool-First Rule (G4)
  Before writing custom scripts, installing tools, or handling files/data, always check whether an existing local tool already solves the problem.
  1. Classify the task into a category: document, pdf, image, media, data, search, archive, dev, web, ai.
  2. Resolve the shared tool-memory home — check TOOL_FIRST_MEMORY_HOME.
  3. Query the registry: tool-first registry query --category <cat>
  4. Detect only those candidates: tool-first tools detect --category <cat>
  5. Recall past experience: tool-first memory recall --task "<description>"
  6. Use an existing tool when 1–3 commands can solve the task.
  7. Write code only when tools are missing, fail, or the task requires custom logic.
  If writing code, briefly state why: "No existing tool fits because …"
  tool-memory is shared runtime infrastructure, not authoritative Vault memory.
  SKILL.md is the sole execution rule source.

For Claude Code — add to ~/.claude/CLAUDE.md:

  ## Tool-First Rule
  Before writing custom scripts, installing new software, or handling files/data with ad-hoc code, check if an existing local tool already solves the problem.
  1. Classify the task into a category: document, pdf, image, media, data, search, archive, dev, web, ai.
  2. Resolve the shared tool-memory home — check TOOL_FIRST_MEMORY_HOME env var.
  3. Query the registry: tool-first registry query --category <cat> --json
  4. Detect only those candidates: tool-first tools detect --category <cat> --json
  5. Recall past experience: tool-first memory recall --task "<description>" --json
  6. Use an existing tool when 1–3 commands can solve the task.
  7. Write code only when tools are missing, fail, or the task requires custom logic.
  If writing code, briefly state why: "No existing tool fits because …"
  tool-memory is shared runtime infrastructure, not authoritative Vault memory.
  SKILL.md is the sole execution rule source.

For Codex — add to ~/.codex/AGENTS.md:

  Same rule text as Claude Code above.

## Step 4: Verify

Run `tool-first doctor` to verify the setup.
Run `tool-first memory resolve --json` to check memory home resolution.

Before writing custom code for file conversion, document processing, PDF handling,
image/media processing, archive operations, data transformation, search, web
extraction, or development utility tasks, first check whether an existing local
tool or verified recipe can solve the task.

If `TOOL_FIRST_MEMORY_HOME` is set:
1. Treat it as the canonical shared runtime tool-memory home.
2. Do not create a private tool-memory directory elsewhere.
3. If the directory does not exist, initialize it after confirming intent.

If `TOOL_FIRST_MEMORY_HOME` is not set:
1. Check `~/.config/tool-first-agent/config.yaml` for `memory_home`.
2. If not configured, ask the user where shared tool-memory should be stored.
3. Recommend a low-authority runtime/log path.

Recommended locations:
- ~/AI-Runtime/tool-first-agent/tool-memory
- <ARMORVault>/92-Logs/_shared/tool-memory
- <PAMAVault>/08-Working-Memory/_runtime/tool-memory

Do not place tool-memory in high-authority paths such as 01-Facts, 02-Rules, 03-Insights, 05-Truth.

Expected workflow:
1. Classify task type.
2. Query registry.
3. Resolve shared tool-memory home.
4. Recall verified recipes.
5. Detect only relevant tools.
6. Prefer verified recipes and existing tools.
7. Write custom code only if tools and recipes are unsuitable.
8. Record verified success, failure, or unsafe pattern into shared tool-memory.

Do not write hallucinated or guessed tool-memory records.

Final installation report must include:
- Which agent was configured
- Where tool-first-agent was installed
- Which tool-memory path is being used
- Whether TOOL_FIRST_MEMORY_HOME was detected
- Whether .tool-memory-home exists
- Which agent config file was updated (SOUL.md / CLAUDE.md / AGENTS.md)
````

### Project Structure

```text
tool-first-agent/
├── README.md                           # this file
├── SKILL.md                            # sole execution rule source
├── Cargo.toml                          # workspace root
├── memory_config.yaml                  # default config
├── references/                         # integration & architecture docs
├── registry/
│   └── tools.yaml                      # candidate tool definitions (10 categories, ~40 tools)
└── crates/tool-first/
    └── src/
        ├── main.rs                     # CLI entry point
        ├── config.rs                   # config loading + resolution
        ├── resolver.rs                 # TOOL_FIRST_MEMORY_HOME + markers
        ├── registry.rs                 # registry query
        ├── detect.rs                   # tool detection
        ├── memory.rs                   # MemoryRecord struct
        ├── file_store.rs               # file-based store (append-only, atomic writes)
        └── mcp.rs                      # MCP stdio server (JSON-RPC 2.0)
```

### Requirements

- macOS (Intel / Apple Silicon) or Linux
- No Rust installation required for users (pre-built binaries available)
- Rust 1.75+ only needed for building from source

### License

MIT

---

<a id="中文"></a>

## 中文

### 架构

```text
tool-first-agent = Rust 运行时核心 + SKILL.md 规则层 + 共享工具记忆
```

- `SKILL.md` 是 Agent 的唯一执行规则源。
- Rust 运行时核心提供快速的本地 CLI 和 MCP 访问。
- `tool-memory` 是通过 `TOOL_FIRST_MEMORY_HOME` 定位的共享运行时基础设施。
- `tool-memory` 不是权威 Vault 记忆，不得自动提升为高权威记忆。

### 三层设计

| 层 | 职责 |
|----|------|
| **SKILL.md** | "Agent 应该怎么做" — 唯一执行规则源 |
| **Rust 运行时核心** | "高频、稳定、快速的查询和写入" — CLI / MCP / 注册表 / 检测 / 记忆 |
| **共享工具记忆** | "多 Agent 共享工具经验" — 通过 `TOOL_FIRST_MEMORY_HOME` 定位 |

**不要默认创建 `02-Rules/Tool-Inventory`。** 可执行的 tool-first 行为属于 `SKILL.md`。

### 为什么需要这个项目

AI 助手经常在 `pandoc`、`jq`、`ffmpeg`、`magick` 等工具一条命令就能解决问题时，却去写自定义脚本。这个项目解决了这个问题：

1. **任务分类路由** — `document`、`pdf`、`image`、`media`、`data`、`search`、`archive`、`dev`、`web`、`ai` 十大类别
2. **查询本地注册表** — 按类别获取候选工具
3. **精准检测** — 只检测候选工具，不做盲目的文件系统扫描
4. **回忆历史经验** — 查询本机上次什么工具好用
5. **记录新经验** — 保存成功和失败记录供未来使用

```text
写代码之前 → 加载技能 → 分类任务 → 查询注册表 → 检测候选 → 回忆经验 → 使用现有工具
```

### 快速开始

```bash
# 下载预编译二进制（macOS，无需 Rust 环境）
curl -sL https://github.com/licat233/tool-first-agent/releases/download/v0.1.0/tool-first-universal-apple-darwin.tar.gz | tar xz
mv tool-first-universal /usr/local/bin/tool-first

# 验证
tool-first doctor

# 查询注册表
tool-first registry query --category document --json

# 检测已安装工具
tool-first tools detect --category document --json
```

> Linux 用户：需要从源码编译 `cargo build --release`（需要 Rust 1.75+）。

### CLI 命令

```bash
tool-first memory resolve --json          # 解析 canonical memory home
tool-first memory recall --task <text>    # 搜索工具记忆
tool-first memory record '<json>' --json  # 写入一条记录
tool-first memory check-conflicts --json  # 检查路径冲突
tool-first registry query --category <c>  # 按类别查询注册表
tool-first tools detect --category <c>    # 检测已安装工具
tool-first doctor                          # 运行诊断
tool-first mcp serve                       # 启动 MCP stdio 服务器
```

### 支持的 Agent

| Agent | 配置文件 | 技能目录 |
|-------|----------|----------|
| **Hermes Agent** | `~/.hermes/SOUL.md` | `~/.hermes/skills/devops/tool-first-agent/` |
| **Claude Code** | `~/.claude/CLAUDE.md` | `~/.claude/skills/tool-first-agent/` |
| **Codex** | `~/.codex/AGENTS.md` | `~/.codex/skills/tool-first-agent/` |

### 记忆适配器

文件适配器将每条工具记忆记录存储为独立 JSON 文件：

```text
<tool-memory-home>/
  .tool-memory-home              # canonical marker
  records/
    20260612-153000-hermes-pandoc-recipe-8f3a.json
    20260612-153102-claude-code-ffmpeg-media-a92d.json
```

- 每条记录一个文件，原子写入（`.tmp` + rename）
- 零依赖，可直接查看，Git 友好
- 文件名格式：`{时间戳}-{agent}-{工具}-{任务类型}-{uuid}.json`

Obsidian 用户应将 `TOOL_FIRST_MEMORY_HOME` 指向 vault 内的低权威 runtime 路径（如 `<Vault>/92-Logs/_shared/tool-memory/`）。

### 环境变量

| 变量 | 用途 |
|------|------|
| `TOOL_FIRST_MEMORY_HOME` | 共享工具记忆主目录（最高优先级） |
| `TOOL_FIRST_MEMORY_CONFIG` | 覆盖配置文件位置 |
| `TOOL_FIRST_AGENT_NAME` | 记录中的 Agent 名称（`hermes`、`claude-code`、`codex`） |

### 环境要求

- macOS（Intel / Apple Silicon）或 Linux
- 用户无需安装 Rust（可直接下载预编译二进制）
- 仅从源码编译时需要 Rust 1.75+

### 许可证

MIT
