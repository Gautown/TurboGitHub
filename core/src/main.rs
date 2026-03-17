mod auto_proxy;
mod config;
mod dns_proxy_config;
mod dns_server;
mod http_proxy;
mod ipc_server;
mod pac_proxy;
mod scanner;
mod github_traffic_monitor;
mod traffic_stats;
mod process_filter;

use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::net::TcpStream;
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("Starting TurboGitHub Core Daemon");
    
    let config = Arc::new(load_config().await?);
    info!("Configuration loaded successfully: listen_addr={}, upstream_dns={}", 
          config.listen_addr, config.upstream_dns);
    
    // 创建自动代理配置管理器（FastGithub 风格）
    // 注意：不再在启动时自动设置 DNS，改为由 GUI 控制
    let auto_proxy_config = Arc::new(auto_proxy::AutoProxyConfig::new(61235));
    
    info!("💡 DNS and proxy settings will be controlled by GUI");
    
    let scanner = Arc::new(scanner::Scanner::new(Arc::clone(&config)));
    scanner.start().await;
    info!("IP scanner started");
    
    // 创建流量统计器
    let traffic_stats = Arc::new(traffic_stats::TrafficStats::new(100));
    info!("Traffic statistics module created");
    
    // 创建 GitHub 流量监控器（保留 100 个数据点，每 2 秒记录一次）
    let github_traffic_monitor = github_traffic_monitor::GitHubTrafficMonitor::new(100);
    github_traffic_monitor.start_monitoring(2);  // 每 2 秒记录一次
    let github_traffic_monitor_arc = Arc::new(std::sync::Mutex::new(github_traffic_monitor));
    info!("GitHub traffic monitor created and started monitoring");
    
    info!("Creating IPC server...");
    let ipc_server = ipc_server::IpcServer::new(
        Arc::clone(&scanner),
        Arc::clone(&config),
        Arc::clone(&traffic_stats),
        Arc::clone(&github_traffic_monitor_arc),
    );
    info!("IPC server created successfully");
    
    info!("Starting servers...");
    
    // FastGithub 风格：先启动 IPC 服务器，确保它完全建立监听
    info!("🚀 Starting IPC server first...");
    let ipc_result = ipc_server.start("127.0.0.1:0".to_string()).await;
    match ipc_result {
        Ok((addr, port)) => {
            info!("✅ IPC server started on {}:{} (dynamic port)", addr.ip(), port);
            
            // 保存端口信息到文件，供 GUI 读取
            if let Err(e) = save_ipc_port(port) {
                error!("❌ Failed to save IPC port: {}", e);
            } else {
                info!("✅ IPC port saved to .ipc_port: {}", port);
            }
            
            // 给 IPC 服务器足够的时间建立监听（增加到 5 秒）
            info!("💡 Waiting for IPC server to fully establish (5 seconds)...");
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            
            // 验证 IPC 服务器是否真正在监听
            info!("🔍 Verifying IPC server is actually listening...");
            match TcpStream::connect(format!("127.0.0.1:{}", port)).await {
                Ok(_) => info!("✅ IPC server verified: listening on port {}", port),
                Err(e) => error!("❌ IPC server verification failed: {}", e),
            }
            
            info!("✅ IPC server ready for connections");
        }
        Err(e) => {
            error!("❌ IPC server error: {}", e);
        }
    }
    
    // 确保 IPC 服务器在后台任务中完全启动后再继续
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    
    // 启动流量监控后台任务（每 2 秒记录一次流量数据）
    let github_traffic_monitor_clone = Arc::clone(&github_traffic_monitor_arc);
    let _traffic_monitor_task = tokio::spawn(async move {
        info!("📊 Starting GitHub traffic monitoring task...");
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(2));
        loop {
            interval.tick().await;
            if let Ok(mut monitor_guard) = github_traffic_monitor_clone.lock() {
                monitor_guard.record_traffic_point();
            }
        }
    });
    
    // 自动启动服务（DNS + HTTP 代理 + PAC，使用动态端口）
    info!("🚀 Auto-starting service with dynamic ports...");
    let auto_start_result = ipc_server.auto_start_service().await;
    match auto_start_result {
        Ok((dns_port, http_port)) => {
            info!("✅ Service auto-started successfully");
            if dns_port > 0 {
                info!("💡 DNS server listening on 127.0.0.1:{} (dynamic port)", dns_port);
            }
            if http_port > 0 {
                info!("💡 HTTP proxy listening on 127.0.0.1:{} (dynamic port)", http_port);
            }
            info!("💡 Dynamic ports prevent conflicts with other applications");
        }
        Err(e) => error!("❌ Service auto-start failed: {}", e),
    }
    
    info!("🎯 TurboGitHub Core Daemon is fully operational!");
    info!("💡 IPC server: ready for GUI connections");
    info!("💡 Service: running (auto-started)");
    info!("💡 GitHub acceleration: active");
    
    // FastGithub风格：持续运行，等待终止信号
    info!("🔄 Waiting for termination signal...");
    tokio::signal::ctrl_c().await?;
    info!("👋 Received termination signal, shutting down...");
    
    info!("TurboGitHub Core Daemon shutting down");
    
    // 自动恢复系统网络设置（FastGithub风格）
    info!("🔄 Restoring original system network settings...");
    match auto_proxy_config.restore_original_settings() {
        Ok(_) => info!("✅ System network settings restored successfully"),
        Err(e) => error!("❌ Failed to restore system network settings: {}", e),
    }
    
    Ok(())
}

fn save_ipc_port(port: u16) -> anyhow::Result<()> {
    let port_file_path = Path::new(".ipc_port");
    fs::write(port_file_path, port.to_string())?;
    info!("IPC port saved to: {:?}", port_file_path);
    Ok(())
}

async fn load_config() -> anyhow::Result<config::Config> {
    let config_path = std::env::current_dir()?
        .join("config.toml");
    
    if config_path.exists() {
        config::Config::load(config_path)
    } else {
        let default_config = config::Config::default();
        default_config.save(&config_path)?;
        info!("Created default configuration file: {:?}", config_path);
        Ok(default_config)
    }
}