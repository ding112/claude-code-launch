# Release Notes

## v1.0.0

**Released**: 2026-03-18

Claude Code Launch 首个正式版本！

### 🎉 主要功能

#### 配置发现与管理

自动发现 Claude Code 和 Cursor 的所有配置文件：

- **全局配置**: `~/.claude/` 和 `~/.cursor/` 下的设置文件
- **项目配置**: 每个项目目录下的 CLAUDE.md、.cursorrules 等
- **只读视图**: 查看配置内容，支持 JSON 格式化

![配置发现](https://via.placeholder.com/800x400?text=Config+Discovery)

#### 仪表盘

一目了然的开发活动概览：

- 今日会话数、总 Token 用量、活跃项目数
- 近 30 天活动趋势图表
- 最近会话列表
- 配置健康度状态

#### 会话历史

浏览所有 AI 对话记录：

- 支持 Claude Code 和 Cursor 两种来源
- 事件时间线查看
- Transcript 完整对话记录
- 会话归档功能

#### Token 用量统计

详细的 Token 使用分析：

- 按天聚合的用量图表
- 可切换柱状图/折线图
- 会话级别明细
- 支持时间范围和来源筛选

### 📦 安装

从 [Releases](https://github.com/dingjiyao/claude-code-launch/releases) 页面下载对应平台的安装包。

### 📝 完整变更日志

详见 [CHANGELOG.md](CHANGELOG.md)