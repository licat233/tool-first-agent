# Tool First Agent 中文说明

Tool First Agent 是一个给 Hermes、Claude Code、Codex 这类 AI Agent 使用的 skill。

它解决的问题很简单：当 Agent 遇到文件转换、文档解析、数据处理、图片/音视频处理、搜索、压缩等任务时，应该优先使用电脑上已经安装好的成熟工具，而不是一上来就自己写大量脚本。

典型例子：

- 读取或转换文档时，先考虑 `pandoc`、`qlmarkdown_cli`、`textutil` 等工具。
- 处理 JSON 时，先考虑 `jq`。
- 搜索文件时，先考虑 `rg`。
- 处理图片或视频时，先考虑 `imagemagick`、`ffmpeg`。
- 解压缩时，先考虑系统已有的压缩工具。

目标不是禁止写代码，而是让 Agent 先判断：“这个任务是不是已有工具一两条命令就能完成？”

## 核心功能

- 给 Agent 提供一条 “tool-first” 决策规则。
- 使用 `registry/tools.yaml` 维护本机候选工具清单。
- 按任务类别懒加载检测工具，而不是每次扫描整台电脑。
- 支持 Hindsight memory，在 `default` bank 中用 `tool-inventory` tag 记录工具经验。
- 区分“候选工具”和“验证经验”：
  - `tools.yaml` 表示这个工具可能适合某类任务。
  - Hindsight 记忆表示这个工具在本机真实成功或失败过。
- 提供查询、检测、注册、回忆、写入记忆等脚本。

## 适用环境

主要面向：

- macOS 开发者电脑
- Hermes Agent
- Claude Code / Codex 风格的代码 Agent
- Hindsight memory provider
- 可以执行本地命令行工具的 Agent

它也可以改造到 Linux 或其他 Agent 环境中使用。默认 registry 里会包含一些 macOS 路径和工具检测方式。

## 适用场景

当 Agent 准备做下面这些事情时，应该先使用这个 skill：

- 读取或转换 Office、Markdown、HTML、EPUB、PDF 等文档
- Markdown 转 HTML，例如使用 `qlmarkdown_cli` 或 `pandoc`
- 从 PDF 中提取文本
- 图片格式转换、压缩、OCR
- 音视频转换、压缩、信息检查
- 搜索代码或文件
- 处理 JSON、YAML、CSV、XML、SQLite
- 压缩或解压文件
- 安装新的命令行工具
- 为一个可能已有命令行工具能解决的任务编写脚本

在多 Agent 场景下尤其有用：多个 Agent 可以共享 Hindsight 记忆，从彼此已经验证过的工具经验中受益。

## 目录结构

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

## 安装方式

把仓库克隆到 Hermes skills 目录：

```bash
mkdir -p ~/.hermes/skills/devops
git clone https://github.com/licat233/tool-first-agent.git \
  ~/.hermes/skills/devops/tool-first-agent
```

确保脚本可执行：

```bash
chmod +x ~/.hermes/skills/devops/tool-first-agent/scripts/*
```

## 接入 Agent 的 System Prompt

只安装 skill 还不够。Agent 的 System Prompt、`SOUL.md` 或类似规则文件里，需要明确告诉它什么时候调用这个 skill。

建议添加下面这段全局规则：

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

如果使用 Hermes，可以把这段规则放在：

```text
~/.hermes/SOUL.md
```

如果 Hermes 有多个 profile，不建议把这段规则手动复制到每个 profile。更推荐使用生成式结构：

```text
~/.hermes/SOUL.md
  全局规则，包括 Tool-First Rule

~/.hermes/profiles/<profile>/SOUL.md.profile
  profile 专属规则

~/.hermes/profiles/<profile>/SOUL.md
  生成文件 = 全局规则 + profile 扩展
```

全局规则修改后，重新生成各 profile 的最终 prompt：

```bash
~/.hermes/scripts/sync-soul.sh --force
```

关键点是：每个实际运行中的 Agent，最终 System Prompt 里都应该包含简短的 Tool-First Rule。详细流程和工具注册表留在这个 skill 内部。

## 基本用法

按任务类别查询候选工具：

```bash
python3 ~/.hermes/skills/devops/tool-first-agent/scripts/query-registry.py \
  --category document \
  --task "markdown html fast"
```

只检测某一类工具：

```bash
python3 ~/.hermes/skills/devops/tool-first-agent/scripts/detect-tools.py \
  --category document
```

检测某个具体工具：

```bash
python3 ~/.hermes/skills/devops/tool-first-agent/scripts/detect-tools.py \
  --tool qlmarkdown_cli \
  --json
```

从 Hindsight 中回忆以前的工具经验：

```bash
bash ~/.hermes/skills/devops/tool-first-agent/scripts/recall-tool-memory.sh \
  "convert markdown to html" \
  document
```

## 注册新工具

安装新工具后，可以把它注册到 `registry/tools.yaml`：

```bash
python3 ~/.hermes/skills/devops/tool-first-agent/scripts/register-tool.py qlmarkdown_cli \
  --category document \
  --priority 25 \
  --handle "Fast batch conversion from Markdown to HTML" \
  --command markdown_to_html='qlmarkdown_cli {input} -o {output}' \
  --fallback pandoc
```

默认情况下，注册只会更新本地 registry，并执行检测。它不会把命令模板直接写入 Hindsight。

如果希望同时把“这个工具可用”的结果写入 Hindsight：

```bash
python3 ~/.hermes/skills/devops/tool-first-agent/scripts/register-tool.py qlmarkdown_cli \
  --category document \
  --priority 25 \
  --handle "Fast batch conversion from Markdown to HTML" \
  --command markdown_to_html='qlmarkdown_cli {input} -o {output}' \
  --fallback pandoc \
  --retain
```

注意：`--retain` 只记录工具可用性。某个命令模板只有在真实任务中成功执行后，才应该作为 verified recipe 写入 Hindsight。

## 记录真实成功或失败

真实任务中命令成功后，记录为 verified recipe：

```bash
python3 ~/.hermes/skills/devops/tool-first-agent/scripts/retain-tool-memory.py \
  --record-type recipe \
  --category document \
  --task extract_text_from_docx \
  --tool pandoc \
  --command-template 'pandoc {input} -t plain' \
  --status verified_success
```

如果工具在某种场景下失败，也可以记录失败经验：

```bash
python3 ~/.hermes/skills/devops/tool-first-agent/scripts/retain-tool-memory.py \
  --record-type failure \
  --category document \
  --task convert_legacy_doc_to_docx \
  --tool libreoffice \
  --status failed_once \
  --failure-reason "headless conversion timed out"
```

这样后续 Agent 就不会重复踩同一个坑。

## Hindsight 记忆模型

这个 skill 假设 Hindsight 使用 `default` bank。工具记忆通过 tag 与普通会话记忆区分：

```text
tool-inventory
tool-category-<category>
```

示例：

```text
tool-category-document
tool-category-pdf
tool-category-data
tool-category-media
```

这个设计特意避免创建额外 bank，适合只能配置一个 Hindsight bank 的 Agent 环境。

## 默认分类

默认 registry 包含这些分类：

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

每个分类里可以包含候选工具、命令模板、检测名称、fallback 工具和简短能力说明。

## 扫描策略

这个 skill 不会每次扫描整台电脑。

允许的检测方式：

- `command -v <tool>`
- 检查已声明的常见 bin 路径
- 检查明确声明的 macOS app bundle 可执行文件
- 轻量级版本命令

常规任务中不应该使用：

- `find /`
- `find ~`
- 递归扫描整个 `/Applications`
- 每次任务前都完整扫描包管理器

完整 inventory refresh 应该是维护操作，而不是每个任务的默认路径。

## 设计理念

核心区分是：

```text
registry/tools.yaml = 候选能力
Hindsight memory = 本机验证经验
```

这样可以避免 Agent 把“理论上可以用的命令模板”误认为“已经验证过的解决方案”。

registry 帮 Agent 知道可以尝试什么；Hindsight 帮 Agent 记住本机真实成功或失败过什么。
