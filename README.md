# Claude Code Launch

基于 Tauri 2 + React + TypeScript 的桌面应用，提供 Claude Code 一键安装与 AI Agent 会话监控，支持 Windows / macOS / Linux。

## 功能

### 一键安装

- **启动自动检测** — 应用启动后自动检查环境，已安装直接进入监控仪表盘
- **一键安装** — 自动串行执行环境检测 → npm install → 验证，无需逐步操作
- **Windows 自动安装 Node.js** — npm 不可用时自动下载 Node.js LTS MSI 静默安装
- **流式日志** — 安装过程实时输出子进程日志到界面

### Agent 监控

- **会话管理** — 按项目分组展示 Claude Code / Cursor 的 Agent 会话
- **事件时间线** — 查看会话中的工具调用、权限请求等事件
- **Transcript 同步** — 增量同步 Agent 对话记录，结构化展示
- **LLM 评估** — 使用 OpenAI / Anthropic / Ollama 评估 Agent 行为的风险和效率
- **Hooks 管理** — 一键配置 Claude Code / Cursor 的事件 Hook

## 技术栈

| 层 | 技术 |
|---|---|
| 前端 | React 19 + TypeScript + Tailwind CSS 4 + Vite |
| 后端 | Rust + Tauri 2 + Axum (HTTP Server) + SQLite |
| 通信 | Tauri Command（invoke）+ REST API + 事件（emit / listen） |
| Hooks | Python 3（report_event.py） |

## 快速开始

```bash
# 安装前端依赖
npm install

# 开发模式（同时启动前端 + Tauri）
npm run tauri dev

# 生产构建
npm run tauri build
```

## 用户配置

配置文件路径：`~/.config/claude-code-launch/config.json`

```json
{
  "db_path": "runtime/claude-code-launch.sqlite3"
}
```

可通过环境变量 `CLAUDE_CODE_LAUNCH_CONFIG_PATH` 覆盖配置文件路径。

## Hook 初始化

```bash
# 预览将要添加的 Hook 配置（不写入）
python3 hooks/init_hooks.py --target user --dry-run

# 写入用户级 Hook 配置
python3 hooks/init_hooks.py --target user
```

## 项目结构

```
├── src/                      # 前端（React）
│   ├── App.tsx               # 主入口：启动检测 → 安装/监控路由
│   ├── pages/                # 页面
│   │   ├── SessionsPage.tsx  # 会话监控
│   │   ├── SettingsPage.tsx  # 评估设置 + Hooks 配置
│   │   └── SetupPage.tsx     # 一键安装
│   ├── components/           # UI 组件
│   ├── hooks/                # 自定义 Hooks
│   └── types.ts              # 前后端共享类型
├── src-tauri/                # 后端（Rust / Tauri）
│   └── src/
│       ├── commands/         # Tauri Command（安装向导）
│       ├── services/         # 安装业务逻辑
│       ├── dao/              # 子进程执行、文件下载
│       ├── collection.rs     # Axum HTTP Server（事件采集）
│       ├── collection/       # 数据库、Handler、Transcript、评估队列
│       ├── evaluation.rs     # LLM 评估（OpenAI/Anthropic/Ollama）
│       ├── app_config.rs     # 配置加载
│       └── bin/local_api.rs  # 独立 HTTP Server（无 GUI）
└── hooks/                    # Python Hook 脚本
    ├── report_event.py       # 事件上报
    └── init_hooks.py         # Hook 初始化
```
