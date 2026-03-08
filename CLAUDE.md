# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 开发命令

```bash
# 安装前端依赖
npm install

# 开发模式（同时启动前端 + Tauri）
npm run tauri dev

# 生产构建
npm run tauri build

# 仅检查 Rust 编译
cd src-tauri && cargo check

# 独立 HTTP Server（无 GUI）
cd src-tauri && cargo run --bin local_api
```

## 项目架构

基于 Tauri 2 + React 19 + TypeScript 的桌面应用，合并了 Claude Code 安装向导与 Agent 会话监控功能。

### 应用启动流程

1. `lib.rs` → `refresh_path_from_registry()` → `load_app_config()` → spawn HTTP Server → Tauri App
2. 前端 `App.tsx` → `check_prereqs` → Claude Code 已安装则进入 dashboard，否则进入 setup

### 前端架构（React + Tailwind CSS 4）

- **src/App.tsx** - 主入口，状态驱动路由（loading → setup → dashboard）
- **src/pages/** - 页面
  - `SessionsPage.tsx` - 会话列表 + 事件时间线 + Transcript + 评估
  - `SettingsPage.tsx` - LLM 评估配置 + Hooks 管理
  - `SetupPage.tsx` - 一键安装（detect → install → verify）
- **src/components/** - UI 组件
  - `TabSwitcher.tsx` - 三 Tab 切换（Sessions | Settings | Setup）
  - `LogPanel.tsx` - 安装日志面板
  - `Stepper.tsx` - 步骤指示器
  - `EventItemView.tsx` - 事件卡片
  - `Pager.tsx` - 分页
  - `AddCommandInput.tsx` - 命令输入
- **src/hooks/** - 自定义 Hooks
  - `usePrereqs.ts` / `useInstall.ts` / `useVerify.ts` - 安装向导
  - `useSessions.ts` / `useEvents.ts` / `useEvaluations.ts` - 监控数据
  - `useTranscript.ts` - Transcript 同步
  - `useEvalSettings.ts` / `useHooksConfig.ts` - 设置
- **src/types.ts** - 前后端共享类型（安装 + 监控）
- **src/api.ts** - REST API 客户端（与 Axum HTTP Server 通信）
- **src/constants.ts** - API 地址、风险样式、已知事件类型

### 后端架构（Rust / Tauri + Axum）

- **src-tauri/src/lib.rs** - Tauri 入口，模块声明，HTTP Server 启动
- **src-tauri/src/commands/** - Tauri Command（安装向导）
  - `check_prereqs` / `run_install` / `run_verify` / `append_log`
- **src-tauri/src/services/** - 安装业务逻辑
- **src-tauri/src/dao/** - 子进程执行、文件下载、PATH 刷新
- **src-tauri/src/models/** - 安装向导数据结构
- **src-tauri/src/collection.rs** - Axum HTTP Server 主模块（路由、AppState、类型）
- **src-tauri/src/collection/** - 子模块
  - `db.rs` - SQLite schema + 持久化
  - `handlers.rs` - HTTP handler
  - `hooks.rs` - Claude settings.json 读写
  - `transcript.rs` - Transcript 增量同步
  - `transcript_poller.rs` - 后台轮询
  - `eval_queue.rs` - 评估任务队列
- **src-tauri/src/evaluation.rs** - LLM 评估（OpenAI / Anthropic / Ollama）
- **src-tauri/src/app_config.rs** - 配置加载（~/.config/claude-code-launch/config.json）
- **src-tauri/src/overseer_models.rs** - JSON Schema 模型
- **src-tauri/src/bin/local_api.rs** - 独立 HTTP Server binary

### 前后端通信

- **Tauri Command** - `invoke()` 调用安装向导命令
- **REST API** - 前端通过 `api.ts` 与 Axum HTTP Server 通信（默认 localhost:8787）
- **Tauri Events** - `emit()` / `listen()` 传递日志事件（`launch-log`）

### 配置

- 配置文件：`~/.config/claude-code-launch/config.json`
- 环境变量：`CLAUDE_CODE_LAUNCH_CONFIG_PATH`（覆盖配置路径）、`CLAUDE_CODE_LAUNCH_PORT`（HTTP 端口，默认 8787）

### Python Hooks

- `hooks/report_event.py` - 事件上报脚本，被 Claude Code / Cursor 调用
- `hooks/init_hooks.py` - 将 report_event.py 注册到 `~/.claude/settings.json`

### Vite 配置

- Tailwind CSS 4 通过 `@tailwindcss/vite` 插件集成
- 开发服务器固定端口：1420
- 忽略 `src-tauri` 目录的 watch
