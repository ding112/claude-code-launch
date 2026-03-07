# Claude Code Launch

基于 Tauri 2 + React + TypeScript 的桌面应用，提供 Claude Code 的一键安装与环境验证向导，支持 Windows / macOS / Linux。

## 功能

- **启动自动检测** — 应用启动后自动检查 npm、Git 等依赖项，并执行 `claude --version` 判断是否已安装
- **智能跳过** — 检测到已安装 Claude Code 时自动跳过安装步骤，直接进入验证
- **全平台 npm 安装** — 所有平台统一通过 `npm install -g @anthropic-ai/claude-code` 安装 Claude Code
- **Windows 自动安装 Node.js** — Windows 下 npm 不可用时自动下载 Node.js LTS MSI 静默安装，并设置国内镜像源
- **安装后自动验证** — 安装成功后自动执行验证步骤，无需手动触发
- **流式日志** — 安装与验证过程中实时输出子进程日志到界面

## 技术栈

| 层 | 技术 |
|---|---|
| 前端 | React 19 + TypeScript + Tailwind CSS + Vite |
| 后端 | Rust + Tauri 2 |
| 通信 | Tauri Command（invoke）+ 事件（emit / listen） |

## 快速开始

```bash
# 安装前端依赖
npm install

# 开发模式（同时启动前端 + Tauri）
npm run tauri dev

# 生产构建
npm run tauri build
```

## 项目结构

```
├── src/                  # 前端（React）
│   ├── App.tsx           # 主流程：检测 → 安装 → 验证
│   ├── components/       # UI 组件（Stepper、LogPanel）
│   ├── hooks/            # usePrereqs / useInstall / useVerify
│   └── types.ts          # 前后端共享类型定义
└── src-tauri/            # 后端（Rust / Tauri）
    └── src/
        ├── commands/     # Tauri Command 入口
        ├── services/     # 业务逻辑（prereq / install / node_install / verify）
        ├── dao/          # 子进程执行、文件下载与流式日志
        └── models/       # 数据结构
```

## 推荐开发环境

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
