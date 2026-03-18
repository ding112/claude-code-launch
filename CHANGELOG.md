# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2026-03-18

### Added

- **配置发现**: 自动发现 Claude Code 与 Cursor 的全局和项目级配置文件
  - 支持 CLAUDE.md、settings.json、commands、hooks 等配置类型
  - 支持 .cursorrules、.cursor/rules、mcp.json 等 Cursor 配置
  - 提供 JSON 格式化和语法高亮展示
- **仪表盘**: 默认首页展示开发活动概览
  - 今日会话数、总 Token 用量、活跃项目数统计
  - 近 30 天活动趋势图表（会话数 + Token 用量）
  - 最近会话列表和配置健康度状态
- **会话历史浏览**: 多来源 AI 对话历史管理
  - 支持 Claude Code 和 Cursor 会话来源
  - 事件时间线和 Transcript 查看
  - 会话归档功能
- **Token 用量统计**: 详细的 Token 使用分析
  - 按天聚合的 Token 用量图表（柱状图/折线图）
  - 会话级别 Token 明细表
  - 支持时间范围和来源筛选
- **事件上报**: Python Hook 脚本自动注册到 Claude Code
- **评估功能**: 可选的 LLM 评估模块（OpenAI/Anthropic/Ollama）

### Changed

- 使用结构化日志（tracing）替代 println!
- Transcript 数据写入采用原子写入确保数据完整性
- Transcript 解析增加 JSONL 容错处理

### Fixed

- 修复 Windows 平台 PATH 环境变量刷新问题
- 修复 Transcript 文件监控的资源泄漏

## [0.1.0] - 2026-03-07

### Added

- Claude Code 安装向导功能
  - 前置条件检查（Node.js、Git 等）
  - 多种安装方式支持（npm、Homebrew、手动下载）
  - 安装后验证
- 基础监控功能框架
- Tauri 2 + React 19 + TypeScript 技术栈搭建
- Tailwind CSS 4 样式系统
- shadcn/ui 组件库集成

[Unreleased]: https://github.com/dingjiyao/claude-code-launch/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/dingjiyao/claude-code-launch/compare/0.1.0...v1.0.0
[0.1.0]: https://github.com/dingjiyao/claude-code-launch/releases/tag/0.1.0