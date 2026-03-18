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

### Active

- [ ] 总览仪表盘——展示最近开发活动、配置状态概览、关键统计
- [ ] Claude Code 配置管理——查看、编辑 CLAUDE.md、commands、hooks、settings.json
- [ ] Cursor 配置管理——查看、编辑 rules、skills、MCP 配置
- [ ] 配置全局视图——一目了然哪些配置已启用、哪些可用
- [ ] 配置导入导出——在项目间迁移配置
- [ ] 配置模板——常用配置的预设模板
- [ ] 交互记录浏览——完整对话历史（提示词 + AI 回复）
- [ ] AI 操作记录——文件修改、命令执行等操作追踪
- [ ] 统计分析——时间、token 用量、成功率等指标
- [ ] 编程模式发现——分析交互数据，发现可改进的工作流模式

### Out of Scope

- 云端同步 — 本地优先，不引入服务端复杂度
- 多用户/团队协作 — 个人工具，团队功能通过开源社区自行扩展
- 实时协作编辑 — 不是 IDE，不做代码编辑
- 移动端 — 桌面开发场景，移动端无意义
- 付费功能 — 开源项目，不做商业化

## Context

- 当前项目已有 Claude Code 会话监控和 Cursor transcript 读取的基础能力
- 项目定位从"监控工具"转向"AI 辅助开发管理平台"
- Claude Code 的配置散落在 `.claude/` 目录下（CLAUDE.md、commands/、hooks/、settings.json）
- Cursor 的配置在 `.cursor/rules/`、`.cursor/skills/`、MCP 配置等位置
- 交互记录数据源：Claude Code 的 transcript 文件、Cursor 的 agent-transcripts/*.jsonl
- 目标用户：使用 Claude Code 和 Cursor 进行 AI 辅助开发的开发者
- 技术栈保持不变：Tauri 2 + React + Rust + SQLite

## Constraints

- **技术栈**: 继续使用 Tauri 2 + React + Axum + SQLite — 已有基础，无需迁移
- **数据源**: 仅读取本地文件，不依赖外部 API 获取交互数据 — 隐私优先
- **兼容性**: 需兼容 macOS、Windows、Linux — Tauri 跨平台特性
- **性能**: 大量 transcript 文件的解析不能阻塞 UI — 后台 worker 处理

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| 重新定位为 AI 辅助开发管理平台 | 从单纯监控扩展到配置管理+交互分析，提供更大价值 | — Pending |
| 统一管理 Claude Code 和 Cursor 配置 | 两者配置分散，统一视图降低认知负担 | — Pending |
| 读取本地文件而非 hook 上报 | 减少侵入性，不需要修改用户的开发环境配置 | — Pending |
| 仪表盘作为首要入口 | 用户打开就能看到全貌，降低使用门槛 | — Pending |

---
*Last updated: 2026-03-18 after initialization*
