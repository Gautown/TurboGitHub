# TurboGitHub - GitHub加速工具

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Windows](https://img.shields.io/badge/Windows-10%2B-green.svg)](https://www.microsoft.com/windows)

TurboGitHub 是一个专为GitHub优化的DNS加速工具，提供图形化界面和智能IP优化功能，显著提升GitHub访问速度。

## ✨ 功能特性

### 🚀 核心功能
- **DNS服务器**：本地DNS解析，端口53535
- **智能IP优化**：自动选择最优GitHub服务器IP
- **流量监控**：实时显示网络流量和连接状态
- **图形化界面**：基于eframe/egui的现代化GUI

### 🎯 技术特性
- **无控制台窗口**：专业的Windows GUI应用程序
- **多格式图标**：支持SVG/PNG/ICO图标格式
- **体积优化**：Release构建仅21MB，极致压缩
- **异步架构**：基于Tokio的高性能异步运行时

## 📥 下载安装

### 最新版本下载
- **TurboGitHub v0.0.1**：[下载链接](https://github.com/Gautown/TurboGitHub/releases/latest)

### 系统要求
- **操作系统**：Windows 10/11 64位
- **内存**：至少100MB可用内存
- **网络**：正常互联网连接

## 🚀 快速开始

### 方法一：一键启动（推荐）
1. 下载并解压发布包
2. 双击 `启动TurboGitHub.bat` 或 `TurboGitHub.exe`
3. GUI窗口将直接打开，无控制台窗口

### 方法二：分步启动
1. **启动DNS服务**（可选）：运行 `启动守护进程.bat`
2. **启动GUI界面**：运行 `启动TurboGitHub.bat` 或双击 `TurboGitHub.exe`

### 方法三：静默启动
- 运行 `启动TurboGitHub.vbs` - 完全无窗口启动

## 🖥️ 界面说明

### 主界面布局
```
┌─────────────────────────────────────────┐
│ 流量 [● 运行中] 域名：4 | IP：5          │
├─────────────────────────────────────────┤
│             控制面板                     │
│  [启动DNS服务] [停止DNS服务]             │
├─────────────────────────────────────────┤
│               日志                       │
│ 连接成功：New IPC connection established │
└─────────────────────────────────────────┘
```

### 功能模块
- **流量标题栏**：显示服务状态和统计信息
- **控制面板**：DNS服务启动/停止控制
- **日志显示**：实时操作日志和错误信息

## ⚙️ 配置说明

### DNS服务器配置
配置文件：`config.toml`

```toml
[dns]
# DNS监听地址和端口
listen_addr = "127.0.0.1:53535"
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

### 构建命令
```bash
# 克隆项目
git clone https://github.com/Gautown/TurboGitHub.git
cd TurboGitHub

# 构建发布版本
cargo build --release

# 创建发布包
.\build-release.bat
```

### 构建选项
- **体积优化**：使用 `opt-level = "z"`
- **无控制台窗口**：配置 `console = false`
- **调试符号剥离**：启用 `strip = true`

## 🐛 故障排除

### 常见问题

#### 1. 连接失败：IO error 10061
**原因**：守护进程未运行
**解决**：先启动 `turbogithub-core.exe` 或运行 `启动守护进程.bat`

#### 2. 端口被占用
**原因**：53535端口被其他程序占用
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

### 资源使用
- **内存占用**：~50-100MB
- **CPU使用**：< 1%（空闲时）
- **启动时间**：< 3秒
- **网络延迟**：减少30-50%

### 优化效果
- **GitHub访问**：显著提升下载速度
- **DNS解析**：本地缓存减少延迟
- **IP选择**：自动选择最优服务器

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

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

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
