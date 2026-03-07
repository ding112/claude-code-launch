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
```

## 项目架构

这是一个基于 Tauri 2 + React 19 + TypeScript 的桌面应用，用于 Claude Code 安装向导。

### 前端架构（React）

- **src/App.tsx** - 主流程控制，处理检测 → 安装 → 验证三个步骤
- **src/components/** - UI 组件
  - `Stepper.tsx` - 步骤指示器
  - `LogPanel.tsx` - 日志面板
- **src/hooks/** - 自定义 Hooks
  - `usePrereqs.ts` - 环境检测逻辑
  - `useInstall.ts` - 安装逻辑
  - `useVerify.ts` - 验证逻辑
- **src/types.ts** - 前后端共享类型定义（Rust serde 与 TypeScript）

### 后端架构（Rust / Tauri）

- **src-tauri/src/lib.rs** - Tauri 入口，注册命令
- **src-tauri/src/commands/mod.rs** - Tauri Command 入口
  - `check_prereqs` - 检测环境
  - `run_install` - 执行安装
  - `run_verify` - 执行验证
  - `append_log` - 附加日志
- **src-tauri/src/services/** - 业务逻辑
  - `prereq_service.rs` - 环境检测服务
  - `install_service.rs` - 安装服务
  - `node_install_service.rs` - Node.js 自动安装服务（Windows 专用）
  - `verify_service.rs` - 验证服务
- **src-tauri/src/dao/** - 数据访问层
  - `command_exists` - 检查命令是否存在
  - `run_command_with_streaming_logs_timeout` - 执行命令并流式输出日志
  - `download_file` - 下载文件
  - `refresh_path_from_registry` - Windows 专用：从注册表刷新 PATH
- **src-tauri/src/models/mod.rs** - 数据结构（与前端 types.ts 对应）

### 前后端通信

- **Tauri Command** - 使用 `invoke()` 调用后端命令
- **Tauri Events** - 使用 `emit()` / `listen()` 传递日志事件（`launch-log`）
- 日志流：后端通过 emit 发送 LogEvent，前端通过 listen 接收并显示

### 类型共享

前后端通过 TypeScript 类型定义保持一致：
- Rust 模型使用 `#[serde(rename_all = "camelCase")]` 序列化为 camelCase
- TypeScript 直接使用 camelCase 类型

### Windows 特殊处理

- **PATH 刷新** - `refresh_path_from_registry()` 从注册表读取最新的用户/系统 PATH，并探测 fnm/nvm 等版本管理器的 Node.js 路径
- **Node.js 自动安装** - Windows 下 npm 不可用时自动下载 Node.js LTS MSI 并静默安装
- **命令执行** - Windows 下通过 `cmd /C` 执行命令以支持 .cmd 文件

### Vite 配置

- 开发服务器固定端口：1420
- HMR 端口：1421
- 忽略 `src-tauri` 目录的 watch
