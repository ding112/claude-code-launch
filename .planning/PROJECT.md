# Claude Code Launch

## What This Is

一个面向开源社区的 Tauri 桌面应用，帮助开发者统一管理 Claude Code 和 Cursor 的配置（CLAUDE.md、commands、hooks、rules、skills、MCP 等），并通过读取本地 transcript 文件记录和分析 AI 辅助开发的交互历史，让开发者发现自己的编程模式、改进工作流。

## Core Value

开发者打开应用就能看到 AI 辅助开发的全貌——配置状态、开发活动、交互模式——从而持续改进与 AI 协作的方式。

## Requirements

### Validated

- ✓ Tauri 2 桌面应用框架搭建 — existing
- ✓ 嵌入式 Axum HTTP 服务器（localhost:8787）— existing
- ✓ SQLite 数据存储 — existing
- ✓ React + shadcn 前端 UI 框架 — existing
- ✓ Claude Code 会话发现与监控 — existing
- ✓ Cursor agent-transcripts 读取与解析 — existing
- ✓ 事件接收与存储（POST /events）— existing
- ✓ LLM 评估集成（OpenAI、Anthropic、Ollama）— existing
- ✓ 设置页面（评估配置）— existing
- ✓ 安装向导（Node.js、npm、Claude Code 检测与安装）— existing
- ✓ 原子写入与备份清理 — v1.0
- ✓ JSONL 容错解析 — v1.0
- ✓ Claude Code 配置发现与只读查看 — v1.0
- ✓ Cursor 配置发现与只读查看 — v1.0
- ✓ 默认仪表盘展示开发活动概览 — v1.0
- ✓ AI 对话历史浏览（多来源支持）— v1.0
- ✓ Token 用量统计与图表展示 — v1.0

### Active

- [ ] 配置编辑功能——UI 上直接编辑配置文件
- [ ] MCP 配置管理——查看、编辑 MCP 配置
- [ ] 配置导入导出——在项目间迁移配置
- [ ] 配置模板——常用配置的预设模板
- [ ] AI 操作记录——文件修改、命令执行等操作追踪
- [ ] 编程模式发现——分析交互数据，发现可改进的工作流模式
- [ ] 成本估算——按模型定价计算 Token 成本

### Out of Scope

- 云端同步 — 本地优先，不引入服务端复杂度
- 多用户/团队协作 — 个人工具，团队功能通过开源社区自行扩展
- 实时协作编辑 — 不是 IDE，不做代码编辑
- 移动端 — 桌面开发场景，移动端无意义
- 付费功能 — 开源项目，不做商业化

## Context

- **Shipped v1.0** (2026-03-18): 5 phases, 16 requirements complete
- 当前项目已有 Claude Code 会话监控和 Cursor transcript 读取的基础能力
- 项目定位从"监控工具"转向"AI 辅助开发管理平台"
- Claude Code 的配置散落在 `.claude/` 目录下（CLAUDE.md、commands/、hooks/、settings.json）
- Cursor 的配置在 `.cursor/rules/`、`.cursor/skills/`、MCP 配置等位置
- 交互记录数据源：Claude Code 的 transcript 文件、Cursor 的 agent-transcripts/*.jsonl
- 目标用户：使用 Claude Code 和 Cursor 进行 AI 辅助开发的开发者
- 技术栈：Tauri 2 + React + Rust + SQLite (~12,700 LOC)

## Constraints

- **技术栈**: 继续使用 Tauri 2 + React + Axum + SQLite — 已有基础，无需迁移
- **数据源**: 仅读取本地文件，不依赖外部 API 获取交互数据 — 隐私优先
- **兼容性**: 需兼容 macOS、Windows、Linux — Tauri 跨平台特性
- **性能**: 大量 transcript 文件的解析不能阻塞 UI — 后台 worker 处理

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| 重新定位为 AI 辅助开发管理平台 | 从单纯监控扩展到配置管理+交互分析，提供更大价值 | ✓ Good — v1.0 成功交付 |
| 统一管理 Claude Code 和 Cursor 配置 | 两者配置分散，统一视图降低认知负担 | ✓ Good — ConfigPage 实现 |
| 读取本地文件而非 hook 上报 | 减少侵入性，不需要修改用户的开发环境配置 | ✓ Good — 发现历史功能 |
| 仪表盘作为首要入口 | 用户打开就能看到全貌，降低使用门槛 | ✓ Good — 默认路由 |
| 原子写入使用 tempfile crate | 跨平台支持，已有成熟实现 | ✓ Good — hooks.rs 集成 |
| JSONL 容错在 API 层验证 | 存储层保持原始，读取时过滤 | ✓ Good — skipped_lines 返回 |
| 复用 sessions 表做 Token 统计 | 无需新建表，SQL 聚合足够 | ✓ Good — 性能可接受 |

---
*Last updated: 2026-03-18 after v1.0 milestone*
