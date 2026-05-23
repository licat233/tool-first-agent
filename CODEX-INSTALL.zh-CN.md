# Codex 安装与接入指南

这份文档面向 Codex 使用者，说明如何把 `tool-first-agent` 接入 Codex，让 Codex 在写脚本之前优先检查和使用本机已有工具。

## 适用目标

接入后，Codex 在遇到下面这类任务时，应优先使用本机工具，而不是立即编写脚本：

- 文档读取与转换，例如 Markdown、DOCX、PDF、HTML、EPUB
- JSON、YAML、CSV、XML、SQLite 等数据处理
- 文件搜索、批量替换、压缩与解压
- 图片、音频、视频处理
- Web 抓取、API 调用、命令行工具安装
- 任何“一两条命令可能就能完成”的工具型任务

## 重要说明

Codex 的 skill 支持取决于你使用的是 Codex CLI、Codex Desktop，还是自定义的 Codex-like Agent。

因此最稳的接入方式是：

1. 把 `tool-first-agent` 安装到 Codex 可以访问的位置。
2. 在 Codex 的全局指令或项目指令中加入 Tool-First Rule。
3. 让 Codex 在相关任务开始前阅读并遵守 `SKILL.md`。
4. 需要时直接调用本仓库脚本查询、检测、注册工具。

如果你的 Codex 环境原生支持 skills，可以把本仓库作为一个普通 skill 安装；如果不支持，也可以通过指令文件手动引用。

## 推荐安装位置

推荐使用统一 Agent skill 目录：

```text
~/.agents/skills/tool-first-agent
```

这样 Hermes、Claude Code、Codex 或其他 Agent 都可以引用同一份 skill。

安装命令：

```bash
mkdir -p ~/.agents/skills
git clone https://github.com/licat233/tool-first-agent.git \
  ~/.agents/skills/tool-first-agent
chmod +x ~/.agents/skills/tool-first-agent/scripts/*
```

如果你希望只给 Codex 使用，也可以安装到 Codex skills 目录：

```bash
mkdir -p ~/.codex/skills
git clone https://github.com/licat233/tool-first-agent.git \
  ~/.codex/skills/tool-first-agent
chmod +x ~/.codex/skills/tool-first-agent/scripts/*
```

如果你已经把它安装在 Hermes 目录，也可以继续使用现有位置：

```text
~/.hermes/skills/devops/tool-first-agent
```

后续规则里的路径改成你的实际安装路径即可。

## 接入 Codex 全局指令

Codex 通常可以通过全局 instructions、profile/system prompt、项目级说明文件，或 Codex Desktop 的自定义规则来增加行为约束。

推荐加入下面这段规则：

```markdown
## Tool-First Agent Rule

Before writing custom scripts, installing new utilities, or handling files/data
with ad-hoc code, read and follow:

`~/.agents/skills/tool-first-agent/SKILL.md`

This applies to file conversion, document parsing, text search, compression,
image/media processing, JSON/CSV/YAML/XML/SQLite work, web scraping, API
interaction, package/tool installation, and similar tool-shaped tasks.

Use existing local tools when the task can be solved in 1-3 commands. Do not
write replacement scripts for tasks already covered by the `tool-first-agent`
registry or verified Hindsight `tool-inventory` memories.

Write custom code only when no suitable tool exists, the tool fails, or the task
requires custom logic.
```

如果安装到 Codex skills 目录，则改成：

```markdown
`~/.codex/skills/tool-first-agent/SKILL.md`
```

如果安装在 Hermes 目录，则改成：

```markdown
`~/.hermes/skills/devops/tool-first-agent/SKILL.md`
```

## 项目级接入方式

如果你只想让某个项目使用这个规则，可以把同样规则放进项目级指令文件中，例如：

- 项目的 agent instructions
- 项目的 `AGENTS.md`
- 项目的 `README`/开发约定中专门给 Agent 的规则区
- Codex Desktop 当前 workspace 的自定义说明

适合场景：

- 某个项目经常处理文档、数据、媒体文件
- 项目有固定工具链，希望 Codex 优先复用
- 不想影响 Codex 在其他项目中的行为

## 验证 Codex 是否会使用

在 Codex 中发起一个测试任务，例如：

```text
请把这个 Markdown 文件转换成 HTML。开始前先检查本机是否已有合适工具，不要直接写脚本。
```

理想行为是 Codex 会先：

1. 阅读 `tool-first-agent/SKILL.md`。
2. 查询 `registry/tools.yaml`。
3. 检测相关工具，例如 `qlmarkdown_cli`、`pandoc`。
4. 优先使用可用工具。
5. 只有工具不可用或不满足需求时，才考虑写脚本。

## 常用命令

如果安装在 `~/.agents/skills/tool-first-agent`：

```bash
python3 ~/.agents/skills/tool-first-agent/scripts/query-registry.py \
  --category document \
  --task "markdown html fast"
```

检测某类工具是否可用：

```bash
python3 ~/.agents/skills/tool-first-agent/scripts/detect-tools.py \
  --category document
```

检测具体工具：

```bash
python3 ~/.agents/skills/tool-first-agent/scripts/detect-tools.py \
  --tool qlmarkdown_cli \
  --json
```

从 Hindsight 回忆工具经验：

```bash
bash ~/.agents/skills/tool-first-agent/scripts/recall-tool-memory.sh \
  "convert markdown to html" \
  document
```

如果安装在 `~/.codex/skills/tool-first-agent`，把命令里的路径替换为：

```text
~/.codex/skills/tool-first-agent
```

## 安装新工具后的注册流程

如果你安装了一个新工具，例如 `qlmarkdown_cli`，可以注册到本地 registry：

```bash
python3 ~/.agents/skills/tool-first-agent/scripts/register-tool.py qlmarkdown_cli \
  --category document \
  --priority 25 \
  --handle "Fast batch conversion from Markdown to HTML" \
  --command markdown_to_html='qlmarkdown_cli {input} -o {output}' \
  --fallback pandoc
```

这个动作只会更新 `registry/tools.yaml`，并检测工具是否可用。

如果希望同时把“工具可用”写入 Hindsight：

```bash
python3 ~/.agents/skills/tool-first-agent/scripts/register-tool.py qlmarkdown_cli \
  --category document \
  --priority 25 \
  --handle "Fast batch conversion from Markdown to HTML" \
  --command markdown_to_html='qlmarkdown_cli {input} -o {output}' \
  --fallback pandoc \
  --retain
```

注意：`--retain` 只记录可用性。真实任务中成功执行过的命令，才应该作为 verified recipe 写入 Hindsight。

## 记录真实成功经验

当 Codex 使用某个工具完成真实任务后，可以记录成功经验：

```bash
python3 ~/.agents/skills/tool-first-agent/scripts/retain-tool-memory.py \
  --record-type recipe \
  --category document \
  --task markdown_to_html \
  --tool qlmarkdown_cli \
  --command-template 'qlmarkdown_cli {input} -o {output}' \
  --status verified_success
```

这样后续 Hermes、Claude Code、Codex 或其他 Agent 都可以通过 Hindsight 共享这条经验。

## 推荐工作流

Codex 遇到工具型任务时，应按这个顺序工作：

1. 判断任务类别，例如 document、pdf、data、image、media、search、archive、web。
2. 阅读 `tool-first-agent/SKILL.md`。
3. 查询 registry，找到候选工具。
4. 只检测相关类别或具体工具，不扫描整台电脑。
5. 如果已有工具能在 1-3 条命令内完成任务，优先使用工具。
6. 如果工具失败，记录失败原因。
7. 如果工具成功，记录 verified recipe。
8. 只有没有合适工具、工具失败、或任务需要自定义逻辑时，才编写脚本。

## 不推荐的行为

Codex 不应该：

- 每次任务都扫描整台电脑
- 还没查 registry 就写脚本
- 为 `jq`、`pandoc`、`rg`、`ffmpeg` 已经能做的事情写替代脚本
- 把未验证的命令模板写成 verified recipe
- 把普通会话记忆和工具经验混在一起不加 tag
- 为简单命令行任务创建大段临时代码

## 与 Hermes 和 Claude Code 共用

如果 Hermes、Claude Code、Codex 都使用 Hindsight 的 `default` bank，可以通过 tag 共享工具经验：

```text
tool-inventory
tool-category-<category>
```

这样任意一个 Agent 验证过的工具经验，其他 Agent 都可以复用。

## 最小可用版本

如果你只想做最小接入，只需要两步：

1. 安装仓库：

```bash
mkdir -p ~/.agents/skills
git clone https://github.com/licat233/tool-first-agent.git \
  ~/.agents/skills/tool-first-agent
chmod +x ~/.agents/skills/tool-first-agent/scripts/*
```

2. 在 Codex 的全局或项目指令中加入：

```markdown
Before writing custom scripts, installing utilities, or handling files/data with
ad-hoc code, read and follow:

`~/.agents/skills/tool-first-agent/SKILL.md`
```

这已经能让 Codex 在关键时刻知道要先使用 `tool-first-agent`。
