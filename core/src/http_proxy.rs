use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info, warn};

use crate::scanner::Scanner;
use crate::traffic_stats::TrafficStats;

/// HTTP/SOCKS 代理服务器
pub struct HttpProxy {
    scanner: Arc<Scanner>,
    traffic_stats: Arc<TrafficStats>,
}

impl HttpProxy {
    pub fn new(scanner: Arc<Scanner>, traffic_stats: Arc<TrafficStats>) -> Self {
        Self {
            scanner,
            traffic_stats,
        }
    }
    
    /// 启动 HTTP 代理服务器
    pub async fn start(&self, listen_addr: String) -> anyhow::Result<()> {
        let socket: SocketAddr = listen_addr.parse()?;
        let listener = TcpListener::bind(socket).await?;
        
        info!("🌐 HTTP proxy server listening on {}", socket);
        
        let mut buf = [0u8; 8192];
        
        loop {
            let (stream, src_addr) = listener.accept().await?;
            info!("🔌 New proxy connection from: {}", src_addr);
            
            let scanner_clone = Arc::clone(&self.scanner);
            let traffic_stats_clone = Arc::clone(&self.traffic_stats);
            
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(stream, scanner_clone, traffic_stats_clone, &mut buf).await {
                    error!("❌ Proxy connection error from {}: {}", src_addr, e);
                } else {
                    debug!("✅ Proxy connection from {} closed normally", src_addr);
                }
            });
        }
    }
    
    async fn handle_connection(
        mut stream: TcpStream,
        scanner: Arc<Scanner>,
        traffic_stats: Arc<TrafficStats>,
        buf: &mut [u8],
    ) -> anyhow::Result<()> {
        // 读取请求
        let n = stream.read(buf).await?;
        let request = String::from_utf8_lossy(&buf[..n]);
        
        debug!("Received proxy request:\n{}", request);
        
        // 解析请求行
        let request_line = request.lines().next().unwrap_or("");
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        
        if parts.len() < 3 {
            warn!("Invalid request: {}", request_line);
            return Ok(());
        }
        
        let method = parts[0];
        let url = parts[1];
        
        // 提取目标主机和端口
        let (host, port) = Self::parse_host_port(url)?;
        
        debug!("Proxy request: {} -> {}:{}", method, host, port);
        
        // 检查是否是 GitHub 域名
        if Self::is_github_domain(&host) {
            info!("🎯 GitHub domain detected: {}", host);
            
            // 尝试获取最佳 IP
            if let Some(best_ip) = scanner.get_best_ip(&host).await {
                info!("🚀 Using optimized IP {} for {}", best_ip, host);
                
                // 连接到优化后的 IP
                match TcpStream::connect(format!("{}:{}", best_ip, port)).await {
                    Ok(mut remote_stream) => {
                        info!("✅ Connected to optimized IP");
                        
                        // 转发请求
                        remote_stream.write_all(&buf[..n]).await?;
                        
                        // 双向转发数据
                        let (mut read_half, mut write_half) = stream.split();
                        let (mut remote_read, mut remote_write) = remote_stream.split();
                        
                        let client_to_remote = tokio::io::copy(&mut read_half, &mut remote_write);
                        let remote_to_client = tokio::io::copy(&mut remote_read, &mut write_half);
                        
                        tokio::select! {
                            result = client_to_remote => {
                                match result {
                                    Ok(bytes) => debug!("Client -> Remote: {} bytes", bytes),
                                    Err(e) => error!("Client -> Remote error: {}", e),
                                }
                            }
                            result = remote_to_client => {
                                match result {
                                    Ok(bytes) => debug!("Remote -> Client: {} bytes", bytes),
                                    Err(e) => error!("Remote -> Client error: {}", e),
                                }
                            }
                        }
                        
                        // 记录流量
                        traffic_stats.record_dns_query(&host, n, n);
                        
                        return Ok(());
                    }
                    Err(e) => {
                        error!("Failed to connect to optimized IP: {}", e);
                        // 回退到原始域名
                    }
                }
            }
        }
        
        // 直接连接到原始域名
        debug!("Connecting to original host: {}:{}", host, port);
        match TcpStream::connect(format!("{}:{}", host, port)).await {
            Ok(mut remote_stream) => {
                remote_stream.write_all(&buf[..n]).await?;
                
                let (mut read_half, mut write_half) = stream.split();
                let (mut remote_read, mut remote_write) = remote_stream.split();
                
                let client_to_remote = tokio::io::copy(&mut read_half, &mut remote_write);
                let remote_to_client = tokio::io::copy(&mut remote_read, &mut write_half);
                
                tokio::select! {
                    result = client_to_remote => {
                        if let Err(e) = result {
                            error!("Client -> Remote error: {}", e);
                        }
                    }
                    result = remote_to_client => {
                        if let Err(e) = result {
                            error!("Remote -> Client error: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to connect to {}: {}", host, e);
            }
        }
        
        Ok(())
    }
    
    fn parse_host_port(url: &str) -> anyhow::Result<(String, u16)> {
        let url = url.trim_start_matches("http://").trim_start_matches("https://");
        
        if let Some(colon_pos) = url.find(':') {
            let host = url[..colon_pos].to_string();
            let port = url[colon_pos + 1..].parse::<u16>()?;
            Ok((host, port))
        } else {
            // 默认端口 80
            Ok((url.to_string(), 80))
        }
    }
    
    fn is_github_domain(host: &str) -> bool {
        let github_domains = vec![
            "github.com",
            "www.github.com",
            "api.github.com",
            "raw.githubusercontent.com",
            "gist.github.com",
            "github.io",
            "githubusercontent.com",
            "githubassets.com",
            "githubapp.com",
            "assets-cdn.github.com",
            "avatars.githubusercontent.com",
            "camo.githubusercontent.com",
        ];
        
        github_domains.iter().any(|d| host.ends_with(d))
    }
}
