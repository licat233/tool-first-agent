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
- Runs one-step tool-use advice with `tool-first advise`
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
Before writing code → Load skill → Run `tool-first advise --task "..."` → Use recommended existing tool → Write code only if justified
```

### Quick Start

```bash
# Download pre-built binary (macOS, no Rust required)
curl -sL https://github.com/licat233/tool-first-agent/releases/download/v0.2.0/tool-first-universal-apple-darwin.tar.gz | tar xz
mv tool-first-universal /usr/local/bin/tool-first

# Initialize memory home once, then verify
tool-first memory init --json
tool-first doctor

# Ask for tool-first advice before writing code
tool-first advise --task "extract text from a docx file" --json

# Query and detect manually when needed
tool-first registry query --category document --json
tool-first tools detect --category document --json
tool-first tools detect --category document --record --json
```

> Linux users: build from source with `cargo build --release` (requires Rust 1.75+).

### CLI Commands

```bash
tool-first advise --task <text> --json    # One-step tool-first recommendation
tool-first memory resolve --json          # Resolve canonical memory home
tool-first memory init --json             # Initialize the chosen memory home after explicit intent
tool-first memory recall --task <text>    # Search tool-memory
tool-first memory record '<json>' --json  # Persist a record
tool-first memory check-conflicts --json  # Check for path conflicts
tool-first registry query --category <c>  # Query registry by category
tool-first registry query --task <text>   # Query registry by task
tool-first tools detect --category <c>    # Detect installed tools
tool-first tools detect --category <c> --record  # Persist availability records
tool-first doctor                          # Run diagnostics
tool-first mcp serve                       # Start MCP stdio server
```

### MCP Server

Start the MCP server for agent integration:

```bash
tool-first mcp serve
```

This exposes `advise_tool_use`, `resolve_memory_home`, `query_registry`,
`detect_candidates`, `recall_memory`, `record_memory`, `check_conflicts`,
and `doctor` as MCP tools over stdio JSON-RPC 2.0.

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

### Agent Installation Tutorial

Paste the following tutorial into Codex, Hermes Agent, or Claude Code. It tells
the agent how to install `tool-first-agent`, configure itself, and verify that
the tool-first gate is actually being used.

````text
You are installing and configuring `tool-first-agent` for this local AI agent.

Repository:
https://github.com/licat233/tool-first-agent

Current release:
v0.2.0

Supported agents:
- Codex
- Claude Code
- Hermes Agent

Primary goal:
Make this agent follow the tool-first principle. Before writing custom scripts,
installing tools, or handling files/data with ad-hoc code, the agent must first
ask `tool-first` whether an existing local tool or remembered recipe solves the
task.

Required first gate:

  tool-first advise --task "<describe the task>" --json

If the decision is `use_existing_tool`, use the recommended tool before writing
custom code.

If the decision is `verify_recalled_recipe`, re-detect the tool and reuse the
remembered command if still valid.

If `advise` is unavailable or ambiguous, fall back to:

  category -> registry query -> detect -> recall

Write custom code only when tools are missing, fail, or the task requires custom
logic. If writing code, state why: "No existing tool fits because ..."

Architecture rules:
- `SKILL.md` is the canonical execution rule source.
- `tool-memory` is shared runtime infrastructure.
- `tool-memory` is not current truth, not authoritative Vault memory, and not a
  replacement for AI memory governance.
- Do not create private tool-memory when `TOOL_FIRST_MEMORY_HOME` exists.
- Do not copy the full rules into high-authority Vault paths such as
  `01-Facts/`, `02-Rules/`, `03-Insights/`, or `05-Truth/`.
- Do not write guessed or hallucinated records into tool-memory.

## Step 1: Install the `tool-first` binary

Detect the platform:

  uname -s
  uname -m

For macOS, prefer the universal binary unless the user explicitly wants a
single-architecture binary:

  curl -sL https://github.com/licat233/tool-first-agent/releases/download/v0.2.0/tool-first-universal-apple-darwin.tar.gz | tar xz
  chmod +x tool-first-universal

Install it as `tool-first`.

Preferred install path:

  /usr/local/bin/tool-first

If `/usr/local/bin` requires approval or is not writable, ask the user before
using elevated permissions. If the user does not want a system install, install
to:

  ~/.local/bin/tool-first

and make sure `~/.local/bin` is on PATH.

Commands:

  mv tool-first-universal tool-first
  mkdir -p ~/.local/bin
  mv tool-first ~/.local/bin/tool-first
  tool-first --version

Optional single-architecture downloads:

Apple Silicon only:

  curl -sL https://github.com/licat233/tool-first-agent/releases/download/v0.2.0/tool-first-aarch64-apple-darwin.tar.gz | tar xz

Intel only:

  curl -sL https://github.com/licat233/tool-first-agent/releases/download/v0.2.0/tool-first-x86_64-apple-darwin.tar.gz | tar xz

If no prebuilt binary matches the platform, build from source:

  git clone https://github.com/licat233/tool-first-agent.git
  cd tool-first-agent
  cargo build --release
  cp target/release/tool-first ~/.local/bin/tool-first

## Step 2: Install the skill files

Install the repository into the current agent's skill directory.

Codex:

  mkdir -p ~/.codex/skills
  git clone https://github.com/licat233/tool-first-agent.git ~/.codex/skills/tool-first-agent

Claude Code:

  mkdir -p ~/.claude/skills
  git clone https://github.com/licat233/tool-first-agent.git ~/.claude/skills/tool-first-agent

Hermes Agent:

  mkdir -p ~/.hermes/skills/devops
  git clone https://github.com/licat233/tool-first-agent.git ~/.hermes/skills/devops/tool-first-agent

If the directory already exists, update it instead of cloning again:

  git -C <skill-directory>/tool-first-agent pull

## Step 3: Configure shared tool-memory

First check whether the user already has a canonical memory home:

  echo "$TOOL_FIRST_MEMORY_HOME"
  tool-first memory resolve --json

If `TOOL_FIRST_MEMORY_HOME` is already set, use it. Do not create another
private memory home.

If it is not set, ask the user where shared tool-memory should live. Recommend a
low-authority runtime path, for example:

  ~/AI-Runtime/tool-first-agent/tool-memory
  <Vault>/92-Logs/_shared/tool-memory
  <Vault>/08-Working-Memory/_runtime/tool-memory

After the user confirms the path, set it for shells:

  export TOOL_FIRST_MEMORY_HOME="<confirmed-path>"

For macOS GUI apps, also set launchd environment variables:

  launchctl setenv TOOL_FIRST_MEMORY_HOME "<confirmed-path>"

Set the agent name:

Codex:

  export TOOL_FIRST_AGENT_NAME="codex"
  launchctl setenv TOOL_FIRST_AGENT_NAME "codex"

Claude Code:

  export TOOL_FIRST_AGENT_NAME="claude-code"
  launchctl setenv TOOL_FIRST_AGENT_NAME "claude-code"

Hermes:

  export TOOL_FIRST_AGENT_NAME="hermes"
  launchctl setenv TOOL_FIRST_AGENT_NAME "hermes"

Initialize the memory home only after the path is confirmed:

  tool-first memory init --json
  tool-first doctor
  tool-first memory check-conflicts --json

## Step 4: Add the agent rule

This step is required. Without an agent rule, the binary can be installed but
the agent may still skip the tool-first gate.

Use this rule text:

  ## Tool-First Rule

  Before writing custom scripts, installing tools, or handling files/data with
  ad-hoc code, always check whether an existing local tool already solves the
  problem.

  1. Run the one-step gate first:
     tool-first advise --task "<description>" --json
  2. If the decision is use_existing_tool, use the recommended tool before
     writing custom code.
  3. If the decision is verify_recalled_recipe, re-detect the tool and reuse the
     remembered command if still valid.
  4. If advise is unavailable or ambiguous, fall back to category -> registry
     query -> detect -> recall.
  5. Write code only when tools are missing, fail, or the task requires custom
     logic.

  If writing code, briefly state why: "No existing tool fits because ..."

  tool-memory is shared runtime infrastructure, not authoritative memory.
  Do not create private tool-memory when TOOL_FIRST_MEMORY_HOME exists.
  Do not write guessed tool-memory records.
  SKILL.md is the sole execution rule source.

Add the rule to the correct file:

Codex:

  ~/.codex/AGENTS.md

Claude Code:

  ~/.claude/CLAUDE.md

Hermes Agent:

  ~/.hermes/SOUL.md

## Step 5: Configure MCP when supported

MCP is recommended for Hermes and optional for Claude Code or Codex surfaces
that can launch local MCP servers.

Hermes example:

  mcp_servers:
    tfa:
      command: "/absolute/path/to/tool-first"
      args: ["mcp", "serve"]
      env:
        TOOL_FIRST_MEMORY_HOME: "<confirmed-path>"
        TOOL_FIRST_AGENT_NAME: "hermes"
      timeout: 120
      connect_timeout: 60
      tools:
        include:
          - advise_tool_use
          - resolve_memory_home
          - query_registry
          - detect_candidates
          - recall_memory
          - record_memory
          - check_conflicts
          - doctor
        resources: false
        prompts: false

When MCP is available, prefer the MCP tool:

  advise_tool_use

Hermes may expose it as:

  mcp_tfa_advise_tool_use

## Step 6: Verify behavior

Run:

  tool-first --version
  tool-first doctor
  tool-first advise --task "extract fields from a json file" --json
  tool-first advise --task "resize a png image to 800px" --json

Expected behavior:
- JSON tasks should recommend tools such as `jq` or `yq` when available.
- Image resize tasks should recommend tools such as `magick` or `sips` when
  available.
- The agent should not write a custom script when `advise` recommends
  `use_existing_tool`.

Optional persistence check:

  tool-first tools detect --category data --record --json
  tool-first memory recall --task "json" --json

## Step 7: Final report

Report back with:

- Agent configured: Codex / Claude Code / Hermes Agent
- Binary path: output of `command -v tool-first`
- Binary version: output of `tool-first --version`
- Skill directory used
- Agent rule file updated
- TOOL_FIRST_MEMORY_HOME value
- Whether `.tool-memory-home` exists
- Whether `tool-first doctor` passed
- Whether `tool-first advise` returned a useful recommendation
- Whether MCP was configured, and the exposed tool name if applicable
- Any conflicts from `tool-first memory check-conflicts --json`
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
        ├── advice.rs                   # one-step tool-first recommendation
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
写代码之前 → 加载技能 → 运行 `tool-first advise --task "..."` → 使用推荐工具 → 只有合理时才写代码
```

### 快速开始

```bash
# 下载预编译二进制（macOS，无需 Rust 环境）
curl -sL https://github.com/licat233/tool-first-agent/releases/download/v0.2.0/tool-first-universal-apple-darwin.tar.gz | tar xz
mv tool-first-universal /usr/local/bin/tool-first

# 初始化 memory home 一次，然后验证
tool-first memory init --json
tool-first doctor

# 写代码前先询问工具优先建议
tool-first advise --task "extract text from a docx file" --json

# 必要时再手动查询和检测
tool-first registry query --category document --json
tool-first tools detect --category document --json
tool-first tools detect --category document --record --json
```

> Linux 用户：需要从源码编译 `cargo build --release`（需要 Rust 1.75+）。

### CLI 命令

```bash
tool-first advise --task <text> --json    # 一步式工具优先建议
tool-first memory resolve --json          # 解析 canonical memory home
tool-first memory init --json             # 明确确认后初始化 memory home
tool-first memory recall --task <text>    # 搜索工具记忆
tool-first memory record '<json>' --json  # 写入一条记录
tool-first memory check-conflicts --json  # 检查路径冲突
tool-first registry query --category <c>  # 按类别查询注册表
tool-first tools detect --category <c>    # 检测已安装工具
tool-first tools detect --category <c> --record  # 写入 availability 记录
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
