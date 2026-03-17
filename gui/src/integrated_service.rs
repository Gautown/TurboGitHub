use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::ipc_client::IpcClient;

/// 集成化服务状态
#[derive(Debug, Clone)]
pub struct ServiceStatus {
    pub running: bool,
    pub current_ips: Vec<DomainIpInfo>,
    pub stats: ServiceStats,
}

/// 域名 IP 信息
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DomainIpInfo {
    pub domain: String,
    pub ip: String,
    pub rtt: u64,
}

/// 服务统计信息
#[derive(Debug, Clone)]
pub struct ServiceStats {
    pub domains_scanned: usize,
    pub total_ips: usize,
}

/// 集成化服务客户端（使用真实 IPC）
pub struct IntegratedService {
    ipc_client: Arc<Mutex<Option<IpcClient>>>,
    server_url: String,
}

impl IntegratedService {
    pub fn new(server_url: String) -> Self {
        // 如果指定地址为"dynamic"，则从文件读取动态端口
        let actual_addr = if server_url == "dynamic" {
            match Self::read_dynamic_ipc_port() {
                Ok(port) => {
                    format!("127.0.0.1:{}", port)
                }
                Err(e) => {
                    eprintln!("Failed to read dynamic IPC port: {}, using default 13626", e);
                    "127.0.0.1:13626".to_string()
                }
            }
        } else {
            server_url
        };
        
        Self {
            ipc_client: Arc::new(Mutex::new(None)),
            server_url: actual_addr,
        }
    }

    /// 读取动态 IPC 端口文件（FastGithub 风格：统一路径）
    fn read_dynamic_ipc_port() -> Result<u16, String> {
        Self::read_dynamic_port_from_file(".ipc_port", 13626)
    }

    /// 读取动态 DNS 端口文件
    #[allow(dead_code)]
    pub fn read_dynamic_dns_port() -> Result<u16, String> {
        Self::read_dynamic_port_from_file(".dns_port", 53535)
    }

    /// 读取动态 HTTP 代理端口文件
    #[allow(dead_code)]
    pub fn read_dynamic_http_port() -> Result<u16, String> {
        Self::read_dynamic_port_from_file(".http_port", 7890)
    }

    /// 通用方法：从文件读取动态端口
    fn read_dynamic_port_from_file(port_file_name: &str, default_port: u16) -> Result<u16, String> {
        // 尝试从当前目录读取
        let port_file = Path::new(port_file_name);
        if port_file.exists() {
            if let Ok(content) = fs::read_to_string(port_file) {
                if let Ok(port) = content.trim().parse::<u16>() {
                    eprintln!("✅ Found {} port file: {:?} -> port {}", 
                        if port_file_name == ".ipc_port" { "IPC" }
                        else if port_file_name == ".dns_port" { "DNS" }
                        else { "HTTP" },
                        port_file, port);
                    return Ok(port);
                }
            }
        }
        
        // 尝试从上级目录读取
        let port_file = Path::new("../").join(port_file_name);
        if port_file.exists() {
            if let Ok(content) = fs::read_to_string(&port_file) {
                if let Ok(port) = content.trim().parse::<u16>() {
                    eprintln!("✅ Found {} port file: {:?} -> port {}", 
                        if port_file_name == ".ipc_port" { "IPC" }
                        else if port_file_name == ".dns_port" { "DNS" }
                        else { "HTTP" },
                        port_file, port);
                    return Ok(port);
                }
            }
        }
        
        // 尝试从 core 目录读取
        let port_file = Path::new("core/").join(port_file_name);
        if port_file.exists() {
            if let Ok(content) = fs::read_to_string(&port_file) {
                if let Ok(port) = content.trim().parse::<u16>() {
                    eprintln!("✅ Found {} port file: {:?} -> port {}", 
                        if port_file_name == ".ipc_port" { "IPC" }
                        else if port_file_name == ".dns_port" { "DNS" }
                        else { "HTTP" },
                        port_file, port);
                    return Ok(port);
                }
            }
        }
        
        eprintln!("⚠️ {} port file not found, using default port {}", 
            if port_file_name == ".ipc_port" { "IPC" }
            else if port_file_name == ".dns_port" { "DNS" }
            else { "HTTP" },
            default_port);
        eprintln!("💡 Make sure TurboGitHub Core is running!");
        Ok(default_port)
    }

    /// 连接到 IPC 服务器
    async fn ensure_connected(&self) -> Result<(), String> {
        let mut client_guard = self.ipc_client.lock().await;
        
        if client_guard.is_none() {
            eprintln!("🔌 Attempting to connect to IPC server: {}", self.server_url);
            
            // 增加重试机制，最多重试 5 次
            let max_retries = 5;
            let mut last_error = None;
            
            for attempt in 1..=max_retries {
                let mut client = IpcClient::new(self.server_url.clone());
                match client.connect().await {
                    Ok(()) => {
                        eprintln!("✅ IPC connection established successfully (attempt {})", attempt);
                        *client_guard = Some(client);
                        return Ok(());
                    }
                    Err(e) => {
                        let error_str = e.to_string();
                        eprintln!("⚠️ Connection attempt {} failed: {}", attempt, error_str);
                        
                        if attempt < max_retries {
                            let delay = std::time::Duration::from_secs(2u64.pow((attempt - 1) as u32));
                            eprintln!("💡 Retrying in {} seconds... (attempt {}/{})", delay.as_secs(), attempt + 1, max_retries);
                            tokio::time::sleep(delay).await;
                        } else {
                            last_error = Some(error_str);
                        }
                    }
                }
            }
            
            eprintln!("❌ All connection attempts failed!");
            eprintln!("💡 Please make sure:");
            eprintln!("   1. TurboGitHub Core is running (cargo run -p turbogithub-core)");
            eprintln!("   2. Check if .ipc_port file exists in the project root directory");
            return Err(format!("连接失败：{}", last_error.unwrap_or_else(|| "未知错误".to_string())));
        }
        
        Ok(())
    }

    /// 获取服务状态
    pub async fn get_status(&self) -> Result<ServiceStatus, String> {
        self.ensure_connected().await?;
        
        let mut client_guard = self.ipc_client.lock().await;
        if let Some(ref mut client) = *client_guard {
            match client.get_status().await {
                Ok(status_data) => {
                    // 解析从 IPC 服务器返回的真实数据
                    let running = status_data["running"].as_bool().unwrap_or(false);
                    
                    let mut current_ips = Vec::new();
                    if let Some(ips_array) = status_data["current_ips"].as_array() {
                        for ip_data in ips_array {
                            let domain = ip_data["domain"].as_str().unwrap_or("unknown").to_string();
                            let ip = ip_data["ip"].as_str().unwrap_or("0.0.0.0").to_string();
                            let rtt = ip_data["rtt"].as_u64().unwrap_or(0);
                            
                            current_ips.push(DomainIpInfo {
                                domain,
                                ip,
                                rtt,
                            });
                        }
                    }
                    
                    let stats_data = &status_data["stats"];
                    let stats = ServiceStats {
                        domains_scanned: stats_data["domains_scanned"].as_u64().unwrap_or(0) as usize,
                        total_ips: stats_data["total_ips"].as_u64().unwrap_or(0) as usize,
                    };
                    
                    Ok(ServiceStatus {
                        running,
                        current_ips,
                        stats,
                    })
                }
                Err(e) => Err(format!("获取状态失败：{}", e)),
            }
        } else {
            Err("IPC 客户端未初始化".to_string())
        }
    }
    
    /// 启动服务
    pub async fn start_service(&self) -> Result<(), String> {
        self.ensure_connected().await?;
        
        let mut client_guard = self.ipc_client.lock().await;
        if let Some(ref mut client) = *client_guard {
            match client.start_service().await {
                Ok(success) => {
                    if success {
                        Ok(())
                    } else {
                        Err("启动服务失败".to_string())
                    }
                }
                Err(e) => Err(format!("启动服务调用失败：{}", e)),
            }
        } else {
            Err("IPC 客户端未初始化".to_string())
        }
    }
    
    /// 停止服务
    pub async fn stop_service(&self) -> Result<(), String> {
        self.ensure_connected().await?;
        
        let mut client_guard = self.ipc_client.lock().await;
        if let Some(ref mut client) = *client_guard {
            match client.stop_service().await {
                Ok(success) => {
                    if success {
                        Ok(())
                    } else {
                        Err("停止服务失败".to_string())
                    }
                }
                Err(e) => Err(format!("停止服务调用失败：{}", e)),
            }
        } else {
            Err("IPC 客户端未初始化".to_string())
        }
    }
    
    /// 模拟网络扫描（调用真实扫描）
    pub async fn scan_networks(&self) -> Result<(), String> {
        // 这里我们不需要做特殊处理，因为核心服务会自动进行扫描
        // 我们只需要确保连接正常，状态会自动更新
        self.ensure_connected().await?;
        Ok(())
    }
    
    /// 获取配置
    #[allow(dead_code)]
    pub async fn get_config(&self) -> Result<serde_json::Value, String> {
        self.ensure_connected().await?;
        
        let mut client_guard = self.ipc_client.lock().await;
        if let Some(ref mut client) = *client_guard {
            match client.get_config().await {
                Ok(config) => Ok(config),
                Err(e) => Err(format!("获取配置失败：{}", e)),
            }
        } else {
            Err("IPC 客户端未初始化".to_string())
        }
    }
    
    /// 设置配置
    #[allow(dead_code)]
    pub async fn set_config(&self, config: serde_json::Value) -> Result<(), String> {
        self.ensure_connected().await?;
        
        let mut client_guard = self.ipc_client.lock().await;
        if let Some(ref mut client) = *client_guard {
            match client.set_config(config).await {
                Ok(success) => {
                    if success {
                        Ok(())
                    } else {
                        Err("设置配置失败".to_string())
                    }
                }
                Err(e) => Err(format!("设置配置失败：{}", e)),
            }
        } else {
            Err("IPC 客户端未初始化".to_string())
        }
    }
    
    /// 获取日志
    #[allow(dead_code)]
    pub async fn get_logs(&self, lines: u64) -> Result<serde_json::Value, String> {
        self.ensure_connected().await?;
        
        let mut client_guard = self.ipc_client.lock().await;
        if let Some(ref mut client) = *client_guard {
            match client.get_logs(lines).await {
                Ok(logs) => Ok(logs),
                Err(e) => Err(format!("获取日志失败：{}", e)),
            }
        } else {
            Err("IPC 客户端未初始化".to_string())
        }
    }

    // 流量统计功能已整合到其他方法中

    /// 获取实时流量数据
    pub async fn get_realtime_traffic(&self, max_points: usize) -> Result<serde_json::Value, String> {
        self.ensure_connected().await?;
        
        let mut client_guard = self.ipc_client.lock().await;
        if let Some(ref mut client) = *client_guard {
            match client.get_realtime_traffic(max_points).await {
                Ok(traffic) => Ok(traffic),
                Err(e) => Err(format!("获取实时流量数据失败：{}", e)),
            }
        } else {
            Err("IPC 客户端未初始化".to_string())
        }
    }

    // 代理设置功能已改为自动模式，无需手动设置
}

impl Drop for IntegratedService {
    fn drop(&mut self) {
        // 使用同步方式处理断开连接，避免 Tokio 运行时依赖
        let _client = Arc::clone(&self.ipc_client);
        
        // 在 GUI 环境中，我们无法使用 tokio::spawn，因此使用同步方式
        // 在实际使用中，断开连接会在应用关闭时自动处理
        // 这里只记录日志，不进行实际的异步断开操作
        println!("IntegratedService 正在销毁，IPC 连接将在应用退出时自动关闭");
    }
}
