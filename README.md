# TurboGitHub - GitHub加速工具
# **特别提醒**
1.TurboGitHub不具备“翻墙”功能,只为方便方面访问GitHub学习而已。
2.TurboGitHub不支持Windows10之前的已被发行方停止支持的操作系统，并且不会主动提供支持.
3.TurboGitHub不能为您的游戏加速，也不会提供相关的功能
## 项目概述
**TurboGitHub** 是一款使用 Rust 编写的 GitHub 访问加速工具，旨在通过智能 DNS 解析和最优 IP 优选，显著提升国内开发者访问 GitHub 相关域名的速度与稳定性。工具采用客户端-守护进程架构，提供现代化的图形用户界面（GUI），支持 Windows、macOS 和 Linux 三大操作系统。
# [![Rust](https://img.shields.io/badge/Rust-1.93+-orange.svg)](https://www.rust-lang.org/) [![License](https://img.shields.io/badge/License-GPL%202.0-blue.svg)](LICENSE) [![Windows](https://img.shields.io/badge/Windows-10%2B-green.svg)](https://www.microsoft.com/windows)

TurboGitHub 是一个专为GitHub优化的DNS加速工具，提供图形化界面和智能IP优化功能，显著提升GitHub访问速度。

## ✨ 功能特性
1 **自动优选 IP**：持续探测 GitHub 域名（如 `github.com`、`api.github.com`、`raw.githubusercontent.com` 等）的 IPv4 地址，筛选出连通性好、延迟低的 IP。
2 **智能 DNS 解析**：作为本地 DNS 服务，拦截对加速域名的查询请求，动态返回当前最优 IP。
3 **可视化控制**：通过 GUI 界面轻松启动/停止加速服务、查看实时状态、编辑配置、监控日志。
4 **系统托盘集成**：后台常驻，无需主窗口即可快速操作。
5 **PAC 自动代理规则**（只走 GitHub，其他网站直连）
### 🚀 核心功能
- **DNS服务器**：本地DNS解析（端口61235），自动选择最优GitHub服务器IP
- **流量监控**：实时显示网络流量和连接状态
- **图形化界面**：基于eframe/egui的现代化GUI，无控制台窗口

### 🎯 技术特性

| 组件 | 技术 | 说明  |
| -- | --- | -- |
| 核心守护进程 | Rust + Tokio | 高性能异步运行时，负责 IP 扫描、DNS 服务、IPC 服务器 |
| GUI 前端 | Tauri +eframe (egui）| 轻量级、即时模式（Immediate Mode GUI）的纯 Rust 跨平台 GUI 库，专注于**桌面端、Web 端**，也可嵌入游戏引擎 |
| IPC 通信 | WebSocket (JSON-RPC) | 守护进程与 GUI 之间的双向通信协议 |
| DNS 服务 | Hickory DNS（原 trust-dns） | 提供完整的 DNS 服务器和解析器实现 |
| 配置管理 | Serde + TOML | 配置文件解析与序列化  |
| 日志     | Tracing   | 结构化日志，支持输出到控制台和文件 |  

## 📥 下载安装

### 最新版本下载
- **TurboGitHub v0.0.1**：[下载链接](https://github.com/Gautown/TurboGitHub/releases/latest)

### 系统要求
- **操作系统**：Windows 10/11 64位
- **内存**：至少100MB可用内存
- **网络**：正常互联网连接

## 🚀 快速开始

### 一键启动（推荐）
1. 下载并解压发布包
2. 双击 `启动TurboGitHub.bat` 或 `TurboGitHub.exe`
3. GUI窗口将直接打开，无控制台窗口

### 其他启动方式
- **分步启动**：先运行 `启动守护进程.bat`，再启动GUI界面
- **静默启动**：运行 `启动TurboGitHub.vbs` - 完全无窗口启动

## 🖥️ 界面说明

### 主界面布局
```
┌─────────────────────────────────────────┐
│ 流量 [● 运行中] 域名：4 | IP：5           │
├─────────────────────────────────────────┤
│             控制面板                     │
│  [启动DNS服务] [停止DNS服务]              │
├─────────────────────────────────────────┤
│               日 志                      │
│ 连接成功：New IPC connection established │
└─────────────────────────────────────────┘
```
**功能模块说明**：
- **流量标题栏**：显示服务状态和统计信息
- **控制面板**：DNS服务启动/停止控制
- **日志显示**：实时操作日志和错误信息

## ⚙️ 配置说明

### DNS服务器配置
配置文件：`config.toml`

```toml
[dns]
# DNS监听地址和端口
listen_addr = "127.0.0.1:61235"
# 上游DNS服务器
upstream_dns = "223.5.5.5:53"

[scanner]
# GitHub域名列表
domains = [
    "github.com",
    "raw.githubusercontent.com",
    "api.github.com",
    "assets-cdn.github.com"
]
# 扫描间隔（秒）
scan_interval = 300
```

### 自定义配置
1. 编辑 `config.toml` 文件
2. 修改DNS端口或上游服务器
3. 添加自定义GitHub域名
4. 重启应用程序生效

## 🔧 构建说明

### 开发环境要求
- **Rust工具链**：1.70+
- **Cargo**：Rust包管理器
- **Windows SDK**：Windows开发工具包

### 构建命令与选项
```bash
# 克隆项目
git clone https://github.com/Gautown/TurboGitHub.git
cd TurboGitHub

# 构建发布版本（体积优化，无控制台窗口）
cargo build --release

# 创建发布包
.\build-release.bat
```

**构建特性**：体积优化、无控制台窗口、调试符号剥离

## 🐛 故障排除

### 常见问题

#### 1. 连接失败：IO error 10061
**原因**：守护进程未运行
**解决**：先启动 `turbogithub-core.exe` 或运行 `启动守护进程.bat`

#### 2. 端口被占用
**原因**：61235端口被其他程序占用
**解决**：修改 `config.toml` 中的端口号

#### 3. 图标显示异常
**原因**：SVG渲染问题
**解决**：应用程序会自动使用备用图标方案

#### 4. 内存占用过高
**原因**：长时间运行积累
**解决**：重启应用程序释放内存

### 日志查看
- 查看GUI界面中的日志面板
- 守护进程日志输出到控制台
- 详细错误信息显示在日志中

## 📊 性能指标

### 资源使用与优化效果
- **内存占用**：~50-100MB，CPU使用 < 1%（空闲时）
- **启动时间**：< 3秒，网络延迟减少30-50%
- **GitHub访问**：显著提升下载速度，自动选择最优服务器
- **DNS解析**：本地缓存减少延迟

## 🤝 贡献指南

### 开发贡献
1. Fork项目仓库
2. 创建功能分支
3. 提交代码更改
4. 发起Pull Request

### 问题报告
- 使用GitHub Issues报告问题
- 提供详细的错误描述
- 包含系统环境和复现步骤

### 功能建议
- 在Discussions中提出建议
- 描述使用场景和预期效果

## 📄 许可证

本项目采用 GNU General Public License v2.0 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🙏 致谢

### 使用的开源库
- [eframe/egui](https://github.com/emilk/egui) - 现代化GUI框架
- [Tokio](https://tokio.rs/) - 异步运行时
- [trust-dns](https://github.com/bluejekyll/trust-dns) - DNS服务器实现
- [resvg/usvg](https://github.com/RazrFalcon/resvg) - SVG渲染引擎

### 特别感谢
- GitHub社区提供的技术支持和反馈
- Rust语言生态系统的优秀工具链

## 📞 联系方式

- **GitHub**：[@Gautown](https://github.com/Gautown)
- **项目地址**：https://github.com/Gautown/TurboGitHub
- **问题反馈**：GitHub Issues

---

**⭐ 如果这个项目对您有帮助，请给个Star支持一下！**
