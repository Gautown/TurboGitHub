# TurboGitHub - GitHub 加速工具

# **特别提醒**
1. TurboGitHub 不具备"翻墙"功能，只为方便访问 GitHub 学习而已。
2. TurboGitHub 不支持 Windows10 之前的已被发行方停止支持的操作系统，并且不会主动提供支持。
3. TurboGitHub 不能为您的游戏加速，也不会提供相关的功能。

## 项目概述

**TurboGitHub** 是一款使用 Rust 编写的 GitHub 访问加速工具，旨在通过智能 DNS 解析和最优 IP 优选，显著提升国内开发者访问 GitHub 相关域名的速度与稳定性。工具采用客户端 - 守护进程架构，提供现代化的图形用户界面（GUI），支持 Windows 操作系统。

[![Rust](https://img.shields.io/badge/Rust-1.93+-orange.svg)](https://www.rust-lang.org/) [![License](https://img.shields.io/badge/License-GPL%202.0-blue.svg)](LICENSE) [![Windows](https://img.shields.io/badge/Windows-10%2B-green.svg)](https://www.microsoft.com/windows)

TurboGitHub 是一个专为 GitHub 优化的 DNS 加速工具，提供图形化界面和智能 IP 优化功能，显著提升 GitHub 访问速度。

## ✨ 功能特性

### 核心加速功能
1. **自动优选 IP**：持续探测 GitHub 域名的 IPv4 地址，筛选出连通性好、延迟低的 IP
2. **智能 DNS 解析**：作为本地 DNS 服务，拦截对加速域名的查询请求，动态返回当前最优 IP
3. **HTTP/HTTPS 代理**：支持 HTTP 和 SOCKS5 代理转发，智能路由 GitHub 流量
4. **PAC 自动代理规则**：自动生成 PAC 文件，只加速 GitHub，其他网站直连
5. **可视化控制**：通过 GUI 界面轻松启动/停止加速服务、查看实时状态、编辑配置、监控日志
6. **系统托盘集成**：后台常驻，无需主窗口即可快速操作

### 🚀 核心功能模块

| 模块 | 说明 | 状态 |
|------|------|------|
| DNS 服务器 | 本地 DNS 解析（端口 61235），自动选择最优 GitHub 服务器 IP | ✅ 已实现 |
| IP 扫描器 | 持续扫描 GitHub 域名 IP，测试连通性和延迟 | ✅ 已实现 |
| HTTP 代理 | 支持 HTTP/SOCKS5 代理，智能路由 GitHub 流量 | ✅ 已实现 |
| PAC 代理 | 自动生成 PAC 文件，智能代理规则 | ✅ 已实现 |
| 流量监控 | 实时显示 DNS 和 HTTP 流量统计 | ✅ 已实现 |
| 图形界面 | 基于 eframe/egui 的现代化 GUI | ✅ 已实现 |

### 🎯 技术架构

| 组件 | 技术栈 | 说明 |
|------|--------|------|
| 核心守护进程 | Rust + Tokio | 高性能异步运行时，负责 IP 扫描、DNS 服务、IPC 服务器 |
| GUI 前端 | eframe/egui 0.33 | 轻量级、即时模式的纯 Rust 跨平台 GUI 库 |
| IPC 通信 | WebSocket (JSON-RPC) | 守护进程与 GUI 之间的双向通信协议 |
| DNS 服务 | Hickory DNS 0.23 | 提供完整的 DNS 服务器和解析器实现 |
| HTTP 客户端 | reqwest 0.11 | 高性能 HTTP 客户端，用于 IP 测试 |
| 配置管理 | Serde + TOML | 配置文件解析与序列化 |
| 日志系统 | Tracing 0.1 | 结构化日志，支持输出到控制台和文件 |

## 📥 下载安装

### 最新版本下载
- **TurboGitHub v0.0.1**：[下载链接](https://github.com/Gautown/TurboGitHub/releases/latest)

### 系统要求
- **操作系统**：Windows 10/11 64 位
- **内存**：至少 100MB 可用内存
- **网络**：正常互联网连接
- **Rust 工具链**：1.93+（开发需要）

## 🚀 快速开始

### 方法一：一键启动（推荐）
1. 下载并解压发布包
2. 双击 `启动 TurboGitHub.bat`
3. GUI 窗口将直接打开，无控制台窗口

### 方法二：使用 Cargo 运行（开发者）
```bash
# 克隆项目
git clone https://github.com/Gautown/TurboGitHub.git
cd TurboGitHub

# 运行发布版本
cargo run --release

# 或者运行调试版本
cargo run
```

### 方法三：分步启动
- **分步启动**：先运行核心守护进程，再启动 GUI 界面
- **静默启动**：运行 VBS 脚本 - 完全无窗口启动

## 🖥️ 界面说明

### 主界面布局
```
┌─────────────────────────────────────────┐
│ 流量 [● 运行中] 域名：4 | IP：5           │
├─────────────────────────────────────────┤
│             控制面板                     │
│  [启动 DNS 服务] [停止 DNS 服务]          │
├─────────────────────────────────────────┤
│               日志                       │
│ 连接成功：New IPC connection established │
└─────────────────────────────────────────┘
```

**功能模块说明**：
- **流量标题栏**：显示服务状态和统计信息
- **控制面板**：DNS 服务启动/停止控制
- **日志显示**：实时操作日志和错误信息
- **系统托盘**：最小化到系统托盘，后台运行

## ⚙️ 配置说明

### DNS 服务器配置
配置文件：`core/config.toml`

```toml
# GitHub 域名列表
domains = [
    "github.com",
    "api.github.com", 
    "raw.githubusercontent.com",
    "assets-cdn.github.com"
]

# IP 扫描间隔（秒）
scan_interval = 1800

# 最大并发扫描数
scan_concurrency = 50

# 上游 DNS 服务器
upstream_dns = "223.5.5.5:53"

# DNS 服务器监听地址
listen_addr = "127.0.0.1:61235"

# 日志级别
log_level = "info"
```

### 自定义配置
1. 编辑 `core/config.toml` 文件
2. 修改 DNS 端口或上游服务器
3. 添加自定义 GitHub 域名
4. 重启应用程序生效

## 🔧 构建说明

### 开发环境要求
- **Rust 工具链**：1.93+
- **Cargo**：Rust 包管理器
- **Windows SDK**：Windows 开发工具包

### 构建命令
```bash
# 克隆项目
git clone https://github.com/Gautown/TurboGitHub.git
cd TurboGitHub

# 构建发布版本（体积优化，无控制台窗口）
cargo build --release

# 构建调试版本
cargo build
```

**构建特性**：
- 体积优化（opt-level = "z"）
- 链接时优化（LTO = true）
- 剥离调试符号（strip = true）
- 无控制台窗口

## 🧪 测试验证

### 功能验证脚本
运行 PowerShell 脚本验证所有功能是否已实现：

```powershell
# 验证代码实现
.\verify_implementation.ps1

# 测试加速功能
.\test_acceleration.ps1
```

### 验证结果
根据自动化测试验证：
- ✅ **DNS 服务器** - 已实现
- ✅ **IP 扫描器** - 已实现
- ✅ **HTTP 代理** - 已实现
- ✅ **PAC 代理规则** - 已实现
- ✅ **自动代理配置** - 已实现
- ✅ **流量监控** - 已实现
- ✅ **GitHub 域名检测** - 已实现

**实现率**: 7/7 = **100%** ✅

详细测试报告请查看 [TEST_REPORT.md](TEST_REPORT.md)

## 🐛 故障排除

### 常见问题

#### 1. 连接失败：IO error 10061
**原因**：守护进程未运行  
**解决**：先启动核心守护进程或运行 `启动 TurboGitHub.bat`

#### 2. 端口被占用
**原因**：61235 端口被其他程序占用  
**解决**：修改 `core/config.toml` 中的端口号

#### 3. Cargo 文件锁错误
**原因**：多个 cargo 进程同时运行  
**解决**：
```powershell
# 终止所有 cargo 进程
taskkill /F /IM cargo.exe

# 等待 2 秒后重新运行
Start-Sleep -Seconds 2
cargo run --release
```

#### 4. 图标显示异常
**原因**：SVG 渲染问题  
**解决**：应用程序会自动使用备用图标方案

#### 5. 内存占用过高
**原因**：长时间运行积累  
**解决**：重启应用程序释放内存

### 日志查看
- 查看 GUI 界面中的日志面板
- 守护进程日志输出到控制台
- 详细错误信息显示在日志中

## 📊 性能指标

### 资源使用与优化效果
- **内存占用**：~50-100MB，CPU 使用 < 1%（空闲时）
- **启动时间**：< 3 秒
- **网络延迟**：减少 30-50%
- **GitHub 访问**：显著提升下载速度，自动选择最优服务器
- **DNS 解析**：本地缓存减少延迟

### 加速原理
```
1. 用户访问 github.com
   ↓
2. 本地 DNS 服务器 (127.0.0.1:61235) 拦截查询
   ↓
3. 从 IP 池中选择最优 IP（延迟最低、可达）
   ↓
4. 返回最优 IP 给用户
   ↓
5. 用户直接连接到最优 IP
   ↓
6. 加速访问成功！
```

## 🤝 贡献指南

### 开发贡献
1. Fork 项目仓库
2. 创建功能分支
3. 提交代码更改
4. 发起 Pull Request

### 问题报告
- 使用 GitHub Issues 报告问题
- 提供详细的错误描述
- 包含系统环境和复现步骤

### 功能建议
- 在 Discussions 中提出建议
- 描述使用场景和预期效果

## 📄 许可证

本项目采用 **GNU General Public License v2.0** 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🙏 致谢

### 使用的开源库
- [eframe/egui](https://github.com/emilk/egui) - 现代化 GUI 框架
- [Tokio](https://tokio.rs/) - 异步运行时
- [Hickory DNS](https://github.com/hickory-dns/hickory-dns) - DNS 服务器实现
- [reqwest](https://github.com/seanmonstar/reqwest) - HTTP 客户端
- [Tracing](https://github.com/tokio-rs/tracing) - 日志框架

### 特别感谢
- GitHub 社区提供的技术支持和反馈
- Rust 语言生态系统的优秀工具链

## 📞 联系方式

- **GitHub**：[@Gautown](https://github.com/Gautown)
- **项目地址**：https://github.com/Gautown/TurboGitHub
- **问题反馈**：GitHub Issues
- **测试报告**：[TEST_REPORT.md](TEST_REPORT.md)

---

## 📝 更新日志

### v0.0.1 (最新)
- ✅ 实现完整的 GitHub 加速功能
- ✅ 实现本地 DNS 服务器拦截 GitHub 域名查询
- ✅ 实现 IP 扫描器持续扫描 GitHub 域名的 IP 地址
- ✅ 实现 HTTP 代理支持 HTTP/SOCKS 代理转发
- ✅ 实现 PAC 代理规则自动生成
- ✅ 实现自动代理配置管理
- ✅ 实现流量监控功能
- ✅ 实现 GitHub 域名智能检测
- ✅ 添加功能验证脚本和测试报告
- ✅ 优化 DNS 服务器和 IP 扫描器逻辑

---

**⭐ 如果这个项目对您有帮助，请给个 Star 支持一下！**
