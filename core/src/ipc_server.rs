use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

use crate::scanner::Scanner;
use crate::traffic_stats::TrafficStats;
use crate::dns_proxy_config::DnsProxyConfig;
use crate::github_traffic_monitor::GitHubTrafficMonitor;
use crate::dns_server::DnsServer;
use crate::http_proxy::HttpProxy;

/// 日志条目
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

pub struct IpcServer {
    scanner: Arc<Scanner>,
    config: Arc<crate::config::Config>,
    logs: Arc<std::sync::Mutex<Vec<LogEntry>>>,
    traffic_stats: Arc<TrafficStats>,
    dns_proxy_config: Arc<DnsProxyConfig>,
    github_traffic_monitor: Arc<std::sync::Mutex<GitHubTrafficMonitor>>,
    dns_server: Arc<Mutex<Option<JoinHandle<()>>>>,
    dns_running: Arc<AtomicBool>,
}

impl Clone for IpcServer {
    fn clone(&self) -> Self {
        Self {
            scanner: Arc::clone(&self.scanner),
            config: Arc::clone(&self.config),
            logs: Arc::clone(&self.logs),
            traffic_stats: Arc::clone(&self.traffic_stats),
            dns_proxy_config: Arc::clone(&self.dns_proxy_config),
            github_traffic_monitor: Arc::clone(&self.github_traffic_monitor),
            dns_server: Arc::clone(&self.dns_server),
            dns_running: Arc::clone(&self.dns_running),
        }
    }
}

impl IpcServer {
    pub fn new(
        scanner: Arc<Scanner>, 
        config: Arc<crate::config::Config>, 
        traffic_stats: Arc<TrafficStats>,
        github_traffic_monitor: Arc<std::sync::Mutex<GitHubTrafficMonitor>>,
    ) -> Self {
        let dns_proxy_config = Arc::new(DnsProxyConfig::new("127.0.0.1:61235".to_string()));
        
        Self {
            scanner,
            config,
            logs: Arc::new(std::sync::Mutex::new(Vec::new())),
            traffic_stats,
            dns_proxy_config,
            github_traffic_monitor,
            dns_server: Arc::new(Mutex::new(None)),
            dns_running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 自动启动服务（应用程序启动时调用）
    /// 返回 (DNS 端口，HTTP 代理端口)
    pub async fn auto_start_service(&self) -> anyhow::Result<(u16, u16)> {
        info!("🚀 Auto-starting service with dynamic ports...");
        
        // 检查是否已经在运行
        if self.dns_running.load(Ordering::SeqCst) {
            info!("Service is already running");
            // 服务已在运行时，返回 0 表示使用已分配的动态端口
            return Ok((0, 0));
        }
        
        // 调用启动逻辑
        match Self::handle_start(json!({}), self).await {
            Ok((result, dns_port, http_port)) => {
                info!("✅ Auto-start completed: {:?}", result);
                info!("💡 DNS server listening on 127.0.0.1:{} (dynamic port)", dns_port);
                info!("💡 HTTP proxy listening on 127.0.0.1:{} (dynamic port)", http_port);
                
                // 保存动态端口信息到文件，供 GUI 读取
                if let Err(e) = Self::save_dynamic_ports(dns_port, http_port) {
                    error!("❌ Failed to save dynamic ports: {}", e);
                } else {
                    info!("✅ Dynamic ports saved: DNS={}, HTTP={}", dns_port, http_port);
                }
                
                // 自动设置系统代理（使用 PAC 文件，无需用户手动配置）
                info!("🔧 Auto-configuring system proxy...");
                if let Err(e) = self.setup_system_proxy() {
                    error!("❌ Failed to setup system proxy: {}", e);
                    warn!("💡 You can manually configure PAC file: file:///G:/GitHub/TurboGitHub/core/turbogithub.pac");
                } else {
                    info!("✅ System proxy configured successfully!");
                    info!("💡 GitHub acceleration is now active automatically!");
                }
                
                Ok((dns_port, http_port))
            }
            Err(e) => {
                error!("❌ Auto-start failed: {}", e);
                Err(e)
            }
        }
    }

    /// 保存动态端口信息到文件
    fn save_dynamic_ports(dns_port: u16, http_port: u16) -> anyhow::Result<()> {
        use std::fs;
        
        // 保存 DNS 端口
        let dns_port_file = Path::new(".dns_port");
        fs::write(dns_port_file, dns_port.to_string())?;
        info!("💾 DNS port saved to: {:?}", dns_port_file);
        
        // 保存 HTTP 代理端口
        let http_port_file = Path::new(".http_port");
        fs::write(http_port_file, http_port.to_string())?;
        info!("💾 HTTP port saved to: {:?}", http_port_file);
        
        Ok(())
    }

    /// 设置系统代理（使用 PAC 文件）
    fn setup_system_proxy(&self) -> anyhow::Result<()> {
        use std::process::Command;
        
        let pac_url = "file:///G:/GitHub/TurboGitHub/core/turbogithub.pac";
        info!("📄 Setting up system PAC proxy: {}", pac_url);
        
        // 使用 PowerShell 设置 PAC 代理（最可靠的方法）
        let ps_command = format!(
            "Set-ItemProperty -Path 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings' -Name AutoConfigURL -Value '{}' -Force",
            pac_url
        );
        
        let output = Command::new("powershell")
            .args(&["-Command", &ps_command])
            .output()?;
        
        if output.status.success() {
            info!("✅ System PAC proxy configured");
            info!("💡 GitHub acceleration is now active automatically!");
            info!("💡 No manual configuration needed!");
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to configure PAC proxy: {}", error_msg);
            warn!("💡 You can manually configure PAC file: {}", pac_url);
        }
        
        Ok(())
    }

    /// 停止服务时恢复系统代理
    fn restore_system_proxy(&self) -> anyhow::Result<()> {
        use std::process::Command;
        
        info!("🔄 Restoring original proxy settings...");
        
        // 删除 PAC 代理设置
        let commands = vec![
            "reg delete \"HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings\" /v AutoConfigURL /f",
            "reg delete \"HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Internet Settings\" /v AutoConfigURL /f",
        ];
        
        for cmd in commands {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if parts.len() >= 3 {
                let _ = Command::new(parts[0])
                    .args(&parts[1..])
                    .output();
            }
        }
        
        info!("✅ Proxy settings restored");
        Ok(())
    }

    /// 启动 IPC 服务器，使用动态端口分配
    /// 如果指定端口为0，则让操作系统分配可用端口
    pub async fn start(&self, listen_addr: String) -> anyhow::Result<(SocketAddr, u16)> {
        let socket: SocketAddr = listen_addr.parse()?;
        let listener = TcpListener::bind(socket).await?;
        let local_addr = listener.local_addr()?;
        
        info!("✅ IPC server listening on {}", local_addr);
        
        // FastGithub风格：确保服务器持续运行，添加错误处理
        let scanner_clone = Arc::clone(&self.scanner);
        let config_clone = Arc::clone(&self.config);
        let ipc_server_clone = Arc::new(self.clone());
        
        // 创建服务器句柄，确保它不会被意外丢弃
        let server_handle = tokio::spawn(async move {
            info!("🚀 IPC server task started, waiting for connections...");
            
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        info!("🔌 New IPC connection from: {}", addr);
                        
                        let scanner = Arc::clone(&scanner_clone);
                        let config = Arc::clone(&config_clone);
                        let ipc_server = Arc::clone(&ipc_server_clone);
                        
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_connection(stream, scanner, config, ipc_server).await {
                                error!("❌ Connection error from {}: {}", addr, e);
                            } else {
                                info!("✅ Connection from {} closed normally", addr);
                            }
                        });
                    }
                    Err(e) => {
                        error!("❌ IPC server accept error: {}", e);
                        // 短暂等待后重试，避免快速循环
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                }
            }
        });
        
        // 保存服务器句柄，防止它被丢弃
        let _ = server_handle; // 忽略未使用警告，但保持句柄存活
        
        info!("🎯 IPC server fully operational on {}", local_addr);
        
        // 返回实际的监听地址和端口
        Ok((local_addr, local_addr.port()))
    }

    async fn handle_connection(
        stream: TcpStream,
        scanner: Arc<Scanner>,
        config: Arc<crate::config::Config>,
        ipc_server: Arc<IpcServer>,
    ) -> anyhow::Result<()> {
        let ws_stream = accept_async(stream).await?;
        let (mut write, mut read) = ws_stream.split();
        
        info!("New IPC connection established");
        
        while let Some(message) = read.next().await {
            let message = message?;
            
            if message.is_text() {
                let text = message.to_text()?;
                debug!("Received message: {}", text);
                
                match Self::handle_message(text, &scanner, &config, &ipc_server).await {
                    Ok(response) => {
                        write.send(Message::Text(response)).await?;
                    }
                    Err(e) => {
                        let error_response = json!({
                            "jsonrpc": "2.0",
                            "error": {
                                "code": -32603,
                                "message": e.to_string()
                            },
                            "id": null
                        });
                        write.send(Message::Text(error_response.to_string())).await?;
                    }
                }
            }
        }
        
        info!("IPC connection closed");
        Ok(())
    }

    async fn handle_message(
        message: &str,
        scanner: &Scanner,
        config: &crate::config::Config,
        ipc_server: &IpcServer,
    ) -> anyhow::Result<String> {
        let request: Value = serde_json::from_str(message)?;
        
        let id = request["id"].clone();
        let method = request["method"].as_str().unwrap_or("");
        let params = request["params"].clone();
        
        let result = match method {
            "start" => {
                let (value, _, _) = Self::handle_start(params, ipc_server).await?;
                value
            },
            "stop" => Self::handle_stop(params, ipc_server).await?,
            "get_status" => Self::handle_get_status(params, scanner, ipc_server).await?,
            "get_config" => Self::handle_get_config(params, config).await?,
            "set_config" => Self::handle_set_config(params, config).await?,
            "get_logs" => Self::handle_get_logs(params, ipc_server).await?,
            "get_traffic" => Self::handle_get_traffic(params, ipc_server).await?,
            "get_realtime_traffic" => Self::handle_get_realtime_traffic(params, ipc_server).await?,
            "get_process_traffic" => Self::handle_get_process_traffic(params, ipc_server).await?,
            "get_process_realtime_traffic" => Self::handle_get_process_realtime_traffic(params, ipc_server).await?,
            "ping" => Self::handle_ping(params).await?,
            _ => return Err(anyhow::anyhow!("Unknown method: {}", method)),
        };
        
        let response = json!({
            "jsonrpc": "2.0",
            "result": result,
            "id": id
        });
        
        Ok(response.to_string())
    }

    async fn handle_start(_params: Value, ipc_server: &IpcServer) -> anyhow::Result<(Value, u16, u16)> {
        info!("Received start command");
        
        // 检查服务是否已经在运行
        if ipc_server.dns_running.load(Ordering::SeqCst) {
            info!("Service is already running");
            return Ok((json!({ "success": true, "message": "Service is already running" }), 53535, 7890));
        }
        
        // 使用动态端口绑定 DNS 服务器
        let dns_listen_addr = "127.0.0.1:0".to_string();
        let dns_server = DnsServer::new(
            Arc::clone(&ipc_server.scanner),
            ipc_server.config.listen_addr.clone(),
            Arc::clone(&ipc_server.traffic_stats),
            Arc::clone(&ipc_server.github_traffic_monitor),
        )?;
        
        // 创建 HTTP 代理服务器实例（使用动态端口）
        let http_proxy = HttpProxy::new(
            Arc::clone(&ipc_server.scanner),
            Arc::clone(&ipc_server.traffic_stats),
        );
        
        // 克隆需要的数据
        let dns_server_arc = Arc::new(dns_server);
        let http_proxy_arc = Arc::new(http_proxy);
        let _dns_running = Arc::clone(&ipc_server.dns_running);
        
        // 启动 DNS 服务器（使用动态端口）
        let dns_server_for_binding = Arc::clone(&dns_server_arc);
        let dns_listen_addr_clone = dns_listen_addr.clone();
        
        // 先绑定 DNS 服务器获取端口
        let dns_socket = tokio::net::UdpSocket::bind(dns_listen_addr_clone).await?;
        let dns_local_addr = dns_socket.local_addr()?;
        let dns_port = dns_local_addr.port();
        info!("✅ DNS server bound to 127.0.0.1:{} (dynamic port)", dns_port);
        
        // 在后台运行 DNS 服务器
        let dns_handle = tokio::spawn(async move {
            info!("🚀 Starting DNS server background task...");
            if let Err(e) = dns_server_for_binding.start_with_handler_from_socket(dns_socket).await {
                error!("❌ DNS server error: {}", e);
            }
        });
        
        // 启动 HTTP 代理服务器（使用动态端口）
        let http_proxy_for_binding = Arc::clone(&http_proxy_arc);
        
        // 先绑定 HTTP 代理获取端口
        let http_socket = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let http_local_addr = http_socket.local_addr()?;
        let http_port = http_local_addr.port();
        info!("✅ HTTP proxy bound to 127.0.0.1:{} (dynamic port)", http_port);
        
        // 在后台运行 HTTP 代理
        let _proxy_handle = tokio::spawn(async move {
            info!("🚀 Starting HTTP proxy background task...");
            if let Err(e) = http_proxy_for_binding.start_from_socket(http_socket).await {
                error!("❌ HTTP proxy error: {}", e);
            }
        });
        
        // 保存 DNS 服务器句柄
        {
            let mut dns_server_guard = ipc_server.dns_server.lock().await;
            *dns_server_guard = Some(dns_handle);
        }
        
        // 设置运行状态
        ipc_server.dns_running.store(true, Ordering::SeqCst);
        
        // 创建 PAC 文件（使用动态 HTTP 端口）
        let pac_proxy = crate::pac_proxy::PacProxy::new(format!("127.0.0.1:{}", http_port));
        if let Err(e) = pac_proxy.create_pac_file() {
            error!("Failed to create PAC file: {}", e);
        } else {
            info!("✅ PAC file created: {:?}", pac_proxy.get_pac_path());
            info!("💡 PAC URL: {}", pac_proxy.get_pac_url());
            info!("💡 使用方法：在浏览器或系统代理设置中使用 PAC 文件：{}", pac_proxy.get_pac_url());
        }
        
        info!("✅ Service started successfully (DNS + HTTP Proxy + PAC)");
        info!("💡 DNS port: {}, HTTP port: {}", dns_port, http_port);
        Ok((json!({ "success": true, "message": "Service started" }), dns_port, http_port))
    }

    async fn handle_stop(_params: Value, ipc_server: &IpcServer) -> anyhow::Result<Value> {
        info!("Received stop command");
        
        // 检查 DNS 服务器是否在运行
        if !ipc_server.dns_running.load(Ordering::SeqCst) {
            info!("DNS server is not running");
            return Ok(json!({ "success": true, "message": "DNS server is not running" }));
        }
        
        // 停止 DNS 服务器
        {
            let mut dns_server_guard = ipc_server.dns_server.lock().await;
            if let Some(handle) = dns_server_guard.take() {
                handle.abort();
                info!("✅ DNS server stopped");
            }
        }
        
        // 删除 PAC 文件
        let pac_proxy = crate::pac_proxy::PacProxy::new("127.0.0.1:7890".to_string());
        if let Err(e) = pac_proxy.remove_pac_file() {
            error!("Failed to remove PAC file: {}", e);
        } else {
            info!("✅ PAC file removed");
        }
        
        // 恢复系统代理设置
        if let Err(e) = ipc_server.restore_system_proxy() {
            error!("Failed to restore proxy settings: {}", e);
        } else {
            info!("✅ System proxy settings restored");
        }
        
        // 设置运行状态
        ipc_server.dns_running.store(false, Ordering::SeqCst);
        
        info!("✅ DNS server stopped successfully");
        Ok(json!({ "success": true, "message": "DNS server stopped" }))
    }

    async fn handle_get_status(_params: Value, scanner: &Scanner, ipc_server: &IpcServer) -> anyhow::Result<Value> {
        let ip_pool = scanner.get_ip_pool().await;
        let mut current_ips = Vec::new();
        
        for (domain, ips) in &ip_pool {
            if let Some(best_ip) = ips.iter().find(|ip| ip.reachable && ip.https_available) {
                current_ips.push(json!({
                    "domain": domain,
                    "ip": best_ip.ip.to_string(),
                    "rtt": best_ip.rtt.as_millis()
                }));
            }
        }
        
        let dns_running = ipc_server.dns_running.load(Ordering::SeqCst);
        
        Ok(json!({
            "running": dns_running,
            "current_ips": current_ips,
            "stats": {
                "domains_scanned": ip_pool.len(),
                "total_ips": ip_pool.values().map(|v| v.len()).sum::<usize>()
            }
        }))
    }

    async fn handle_get_config(_params: Value, config: &crate::config::Config) -> anyhow::Result<Value> {
        Ok(json!({
            "domains": config.domains,
            "scan_interval": config.scan_interval,
            "scan_concurrency": config.scan_concurrency,
            "upstream_dns": config.upstream_dns,
            "listen_addr": config.listen_addr,
            "log_level": config.log_level
        }))
    }

    async fn handle_set_config(params: Value, config: &crate::config::Config) -> anyhow::Result<Value> {
        info!("Received set_config request");
        
        // 创建配置文件的备份
        let config_path = std::env::current_dir()?.join("config.toml");
        let backup_path = std::env::current_dir()?.join("config.toml.backup");
        
        if config_path.exists() {
            std::fs::copy(&config_path, &backup_path)?;
            info!("Created config backup: {:?}", backup_path);
        }
        
        // 更新配置
        let mut new_config = config.clone();
        
        if let Some(domains) = params["domains"].as_array() {
            new_config.domains = domains.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
        }
        
        if let Some(scan_interval) = params["scan_interval"].as_u64() {
            new_config.scan_interval = scan_interval;
        }
        
        if let Some(scan_concurrency) = params["scan_concurrency"].as_u64() {
            new_config.scan_concurrency = scan_concurrency as usize;
        }
        
        if let Some(upstream_dns) = params["upstream_dns"].as_str() {
            new_config.upstream_dns = upstream_dns.to_string();
        }
        
        if let Some(listen_addr) = params["listen_addr"].as_str() {
            new_config.listen_addr = listen_addr.to_string();
        }
        
        if let Some(log_level) = params["log_level"].as_str() {
            new_config.log_level = log_level.to_string();
        }
        
        // 保存新配置
        new_config.save(&config_path)?;
        info!("Configuration updated successfully: {:?}", config_path);
        
        Ok(json!({ "success": true, "message": "Configuration updated successfully" }))
    }

    async fn handle_get_logs(params: Value, ipc_server: &IpcServer) -> anyhow::Result<Value> {
        let lines = params["lines"].as_u64().unwrap_or(100) as usize;
        info!("Get logs request, requested {} lines", lines);
        
        let logs_guard = ipc_server.logs.lock().map_err(|e| anyhow::anyhow!("Failed to acquire logs lock: {}", e))?;
        let total_logs = logs_guard.len();
        let start_index = if total_logs > lines { total_logs - lines } else { 0 };
        
        let logs: Vec<Value> = logs_guard.iter()
            .skip(start_index)
            .map(|log| json!({
                "timestamp": log.timestamp,
                "level": log.level,
                "message": log.message
            }))
            .collect();
        
        Ok(json!({
            "logs": logs,
            "total_count": total_logs,
            "returned_count": logs.len()
        }))
    }
    
    async fn handle_ping(_params: Value) -> anyhow::Result<Value> {
        debug!("Received ping request");
        Ok(json!({ "pong": true, "timestamp": chrono::Utc::now().timestamp() }))
    }

    async fn handle_get_traffic(_params: Value, ipc_server: &IpcServer) -> anyhow::Result<Value> {
        info!("Received get_traffic request");
        
        // 使用 GitHub 流量监控器获取 GitHub 专属流量数据
        let (github_upload, github_download) = if let Ok(monitor_guard) = ipc_server.github_traffic_monitor.lock() {
            monitor_guard.get_github_traffic_stats()
        } else {
            (0, 0)
        };
        
        // 同时保留原有的 DNS 查询统计
        let (dns_upload, dns_download, total_queries) = ipc_server.traffic_stats.get_total_traffic();
        let domain_traffic = ipc_server.traffic_stats.get_domain_traffic();
        let start_time = ipc_server.traffic_stats.get_start_time();
        
        let domain_stats: Vec<Value> = domain_traffic.iter().map(|domain| {
            json!({
                "domain": domain.domain,
                "upload_bytes": domain.upload_bytes,
                "download_bytes": domain.download_bytes,
                "query_count": domain.query_count,
                "last_updated": domain.last_updated
            })
        }).collect();
        
        Ok(json!({
            "total_upload_bytes": github_upload,
            "total_download_bytes": github_download,
            "dns_upload_bytes": dns_upload,
            "dns_download_bytes": dns_download,
            "total_queries": total_queries,
            "start_time": start_time,
            "domains": domain_stats
        }))
    }

    async fn handle_get_realtime_traffic(params: Value, ipc_server: &IpcServer) -> anyhow::Result<Value> {
        let max_points = params["max_points"].as_u64().unwrap_or(100) as usize;
        info!("Received get_realtime_traffic request, max_points: {}", max_points);
        
        // 使用 GitHub 流量监控器获取 GitHub 专属流量数据
        let github_traffic_data = if let Ok(monitor_guard) = ipc_server.github_traffic_monitor.lock() {
            let data = monitor_guard.get_traffic_history(max_points);
            info!("Got GitHub traffic history: {} data points", data.len());
            data
        } else {
            error!("Failed to lock GitHub traffic monitor");
            Vec::new()
        };
        
        // 同时保留原有的 DNS 查询统计
        let dns_realtime_data = ipc_server.traffic_stats.get_realtime_data(max_points);
        info!("Got DNS traffic data: {} data points", dns_realtime_data.len());
        
        // 只返回 GitHub 专属流量（github_upload 和 github_download）
        let github_data_points: Vec<Value> = github_traffic_data.iter().map(|point| {
            json!({
                "timestamp": point.timestamp,
                "total_upload": point.github_upload,
                "total_download": point.github_download
            })
        }).collect();
        
        let dns_data_points: Vec<Value> = dns_realtime_data.iter().map(|point| {
            json!({
                "timestamp": point.timestamp,
                "upload_bytes": point.upload_bytes,
                "download_bytes": point.download_bytes,
                "total_queries": point.total_queries
            })
        }).collect();
        
        Ok(json!({
            "network_traffic": {
                "data_points": github_data_points,
                "count": github_data_points.len(),
                "type": "github_only"
            },
            "dns_traffic": {
                "data_points": dns_data_points,
                "count": dns_data_points.len()
            }
        }))
    }

    // 代理设置功能已改为自动模式，无需手动设置

    async fn handle_get_process_traffic(_params: Value, ipc_server: &IpcServer) -> anyhow::Result<Value> {
        info!("Received get_process_traffic request");
        
        // 使用 GitHub 流量监控器获取 GitHub 专属流量
        let (github_upload, github_download) = if let Ok(monitor_guard) = ipc_server.github_traffic_monitor.lock() {
            monitor_guard.get_github_traffic_stats()
        } else {
            (0, 0)
        };
        
        // 获取 DNS 查询统计
        let (dns_upload, dns_download, total_queries) = ipc_server.traffic_stats.get_total_traffic();
        
        Ok(json!({
            "total_upload_bytes": github_upload,
            "total_download_bytes": github_download,
            "dns_upload_bytes": dns_upload,
            "dns_download_bytes": dns_download,
            "total_queries": total_queries,
            "processes": [],
            "type_stats": []
        }))
    }

    async fn handle_get_process_realtime_traffic(params: Value, ipc_server: &IpcServer) -> anyhow::Result<Value> {
        let max_points = params["max_points"].as_u64().unwrap_or(50) as usize;
        info!("Received get_process_realtime_traffic request, max_points: {}", max_points);
        
        // 使用 GitHub 流量监控器获取 GitHub 专属流量历史数据
        let traffic_history = if let Ok(monitor_guard) = ipc_server.github_traffic_monitor.lock() {
            monitor_guard.get_traffic_history(max_points)
        } else {
            Vec::new()
        };
        
        // 转换历史数据格式（使用 GitHub 流量）
        let history_data: Vec<Value> = traffic_history.iter().map(|point| {
            json!({
                "timestamp": point.timestamp,
                "total_upload": point.github_upload,
                "total_download": point.github_download
            })
        }).collect();
        
        Ok(json!({
            "history_data": history_data
        }))
    }
}