use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tracing::{debug, error, info};

use crate::scanner::Scanner;
use crate::traffic_stats::TrafficStats;
use crate::github_traffic_monitor::GitHubTrafficMonitor;

pub struct DnsServer {
    scanner: Arc<Scanner>,
    upstream_dns: SocketAddr,
    traffic_stats: Arc<TrafficStats>,
    github_traffic_monitor: Arc<std::sync::Mutex<GitHubTrafficMonitor>>,
}

impl DnsServer {
    pub fn new(
        scanner: Arc<Scanner>, 
        upstream_dns: String, 
        traffic_stats: Arc<TrafficStats>,
        github_traffic_monitor: Arc<std::sync::Mutex<GitHubTrafficMonitor>>,
    ) -> anyhow::Result<Self> {
        let upstream_dns: SocketAddr = upstream_dns.parse()?;
        Ok(Self {
            scanner,
            upstream_dns,
            traffic_stats,
            github_traffic_monitor,
        })
    }

    #[allow(dead_code)]
    pub async fn start(&self, listen_addr: String) -> anyhow::Result<u16> {
        let socket: SocketAddr = listen_addr.parse()?;
        let udp_socket = UdpSocket::bind(socket).await?;
        
        // 获取实际绑定的端口
        let local_addr = udp_socket.local_addr()?;
        let port = local_addr.port();
        
        info!("DNS server listening on {}", local_addr);
        info!("DNS server bound to: {}", local_addr);
        
        // 返回端口号
        Ok(port)
    }
    
    /// 启动 DNS 服务器并处理请求（内部方法）
    #[allow(dead_code)]
    pub async fn start_with_handler(&self, listen_addr: String) -> anyhow::Result<()> {
        let socket: SocketAddr = listen_addr.parse()?;
        let udp_socket = UdpSocket::bind(socket).await?;
        
        info!("DNS server listening on {}", udp_socket.local_addr()?);
        
        self.run_server(udp_socket).await
    }
    
    /// 从现有 socket 启动 DNS 服务器（用于动态端口）
    pub async fn start_with_handler_from_socket(&self, udp_socket: UdpSocket) -> anyhow::Result<()> {
        info!("DNS server listening on {}", udp_socket.local_addr()?);
        self.run_server(udp_socket).await
    }
    
    /// 运行 DNS 服务器主循环
    async fn run_server(&self, udp_socket: UdpSocket) -> anyhow::Result<()> {
        let mut buf = [0u8; 512];
        let udp_socket = Arc::new(udp_socket);
        
        info!("DNS server ready to receive queries...");
        
        loop {
            let socket_clone = Arc::clone(&udp_socket);
            match socket_clone.recv_from(&mut buf).await {
                Ok((size, src_addr)) => {
                    info!("Received DNS query from {} ({} bytes)", src_addr, size);
                    
                    let _scanner = Arc::clone(&self.scanner);
                    let _upstream_dns = self.upstream_dns;
                    let data = buf[..size].to_vec();
                    
                    let server_clone = Arc::new(DnsServer {
                            scanner: Arc::clone(&self.scanner),
                            upstream_dns: self.upstream_dns,
                            traffic_stats: Arc::clone(&self.traffic_stats),
                            github_traffic_monitor: Arc::clone(&self.github_traffic_monitor),
                        });
                        
                        tokio::spawn(async move {
                            info!("Processing DNS query in background task");
                            if let Err(e) = server_clone.handle_dns_query(&data, &socket_clone, src_addr).await {
                                error!("Failed to handle DNS query: {}", e);
                            } else {
                                info!("DNS query processed successfully");
                            }
                        });
                }
                Err(e) => {
                    error!("Error receiving DNS query: {}", e);
                }
            }
        }
    }
    
    async fn handle_dns_query(
        &self,
        query_data: &[u8],
        socket: &UdpSocket,
        src_addr: SocketAddr,
    ) -> anyhow::Result<()> {
        use trust_dns_proto::op::Message;
        use trust_dns_proto::serialize::binary::BinDecodable;
        
        // Parse DNS query using trust-dns library
        let message = match Message::from_bytes(query_data) {
            Ok(msg) => msg,
            Err(e) => {
                error!("Failed to parse DNS query: {}", e);
                return Ok(());
            }
        };
        
        // Check if it's a standard query
        if message.message_type() != trust_dns_proto::op::MessageType::Query {
            return Ok(()); // Not a query
        }
        
        // Extract domain name from the first query
        let domain = if let Some(query) = message.queries().first() {
            query.name().to_utf8()
        } else {
            error!("No queries found in DNS message");
            return Ok(());
        };
        
        debug!("Received DNS query for: {}", domain);
        
        // GitHub 核心域名列表
        let github_domains = vec![
            "github.com",
            "api.github.com", 
            "raw.githubusercontent.com",
            "gist.github.com",
            "github.io",
            "githubusercontent.com",
            "githubassets.com",
            "githubapp.com",
            "assets-cdn.github.com"
        ];
        
        // AO3 (Archive of Our Own) 域名列表
        let ao3_domains = vec![
            "archiveofourown.org",
            "archiveofourown.com",
        ];
        
        // Pixiv 域名列表
        let pixiv_domains = vec![
            "pixiv.net",
            "www.pixiv.net",
            "dic.pixiv.net",
            "fanbox.cc",
            "www.fanbox.cc",
        ];
        
        let is_github_domain = github_domains.iter().any(|d| domain.ends_with(d));
        let is_ao3_domain = ao3_domains.iter().any(|d| domain.ends_with(d));
        let is_pixiv_domain = pixiv_domains.iter().any(|d| domain.ends_with(d));
        
        // 如果是 GitHub 域名，记录日志
        if is_github_domain {
            info!("Detected GitHub domain: {}", domain);
            debug!("GitHub domain {} detected", domain);
        }
        
        // 如果是 AO3 域名，记录日志
        if is_ao3_domain {
            info!("Detected AO3 domain: {}", domain);
            debug!("AO3 domain {} detected", domain);
        }
        
        // 如果是 Pixiv 域名，记录日志
        if is_pixiv_domain {
            info!("Detected Pixiv domain: {}", domain);
            debug!("Pixiv domain {} detected", domain);
        }
        
        // Check if this is a domain we should accelerate
        let mut accelerated_domains = github_domains.clone();
        accelerated_domains.extend_from_slice(&ao3_domains);
        accelerated_domains.extend_from_slice(&pixiv_domains);
        
        let should_accelerate = accelerated_domains.iter().any(|d| domain.ends_with(d));
        
        if should_accelerate {
            if let Some(best_ip) = self.scanner.get_best_ip(&domain).await {
                info!("Returning optimized IP {} for {}", best_ip, domain);
                
                // Create proper DNS response using trust-dns library
                let mut response_msg = Message::new();
                response_msg.set_id(message.id());
                response_msg.set_message_type(trust_dns_proto::op::MessageType::Response);
                response_msg.set_op_code(message.op_code());
                response_msg.set_recursion_desired(message.recursion_desired());
                response_msg.set_recursion_available(true);
                response_msg.set_response_code(trust_dns_proto::op::ResponseCode::NoError);
                
                // Copy queries
                for query in message.queries() {
                    response_msg.add_query(query.clone());
                }
                
                // Add answer record
                if let Some(query) = message.queries().first() {
                    let name = query.name().clone();
                    let record = trust_dns_proto::rr::Record::from_rdata(
                        name,
                        60, // TTL
                        trust_dns_proto::rr::RData::A(trust_dns_proto::rr::rdata::A(best_ip)),
                    );
                    response_msg.add_answer(record);
                }
                
                // Serialize response
                let response_buf = match response_msg.to_vec() {
                    Ok(buf) => buf,
                    Err(e) => {
                        error!("Failed to serialize DNS response: {}", e);
                        return Ok(());
                    }
                };
                
                socket.send_to(&response_buf, src_addr).await?;
                
                // 记录流量数据：查询大小 + 响应大小
                self.traffic_stats.record_dns_query(&domain, query_data.len(), response_buf.len());
                
                return Ok(());
            }
        }
        
        // Forward to upstream DNS
        debug!("Forwarding query for {} to upstream DNS", domain);
        
        let upstream_socket = UdpSocket::bind("0.0.0.0:0").await?;
        upstream_socket.send_to(query_data, self.upstream_dns).await?;
        
        let mut response_buf = [0u8; 512];
        let size = upstream_socket.recv(&mut response_buf).await?;
        
        socket.send_to(&response_buf[..size], src_addr).await?;
        
        // 如果是GitHub域名，从DNS响应中提取IP地址并注册到跟踪器
        if is_github_domain {
            self.extract_and_register_github_ips(&response_buf[..size], &domain).await;
        }
        
        // 记录流量数据：查询大小 + 响应大小
        self.traffic_stats.record_dns_query(&domain, query_data.len(), size);
        
        Ok(())
    }

    /// 从DNS响应中提取IP地址并注册到GitHub IP跟踪器
    async fn extract_and_register_github_ips(&self, response_data: &[u8], domain: &str) {
        use trust_dns_proto::op::Message;
        
        // 解析 DNS 响应消息
        match Message::from_vec(response_data) {
            Ok(message) => {
                // 提取所有 A 记录（IPv4 地址）
                for answer in message.answers() {
                    if let Some(trust_dns_proto::rr::RData::A(ipv4)) = answer.data() {
                        let ip = ipv4.0.to_string();
                        
                        // 注册到 GitHub IP 跟踪器
                        if let Ok(monitor_guard) = self.github_traffic_monitor.lock() {
                            monitor_guard.get_github_tracker().add_github_ip(ip.clone());
                        }
                        debug!("Detected GitHub IP {} for domain {}", ip, domain);
                    }
                }
            }
            Err(e) => {
                debug!("Failed to parse DNS response for IP extraction: {}", e);
            }
        }
    }
}