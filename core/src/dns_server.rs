use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tracing::{debug, error, info};

use crate::scanner::Scanner;

pub struct DnsServer {
    scanner: Arc<Scanner>,
    upstream_dns: SocketAddr,
}

impl DnsServer {
    pub fn new(scanner: Arc<Scanner>, upstream_dns: String) -> anyhow::Result<Self> {
        let upstream_dns: SocketAddr = upstream_dns.parse()?;
        Ok(Self {
            scanner,
            upstream_dns,
        })
    }

    pub async fn start(&self, listen_addr: String) -> anyhow::Result<()> {
        let socket: SocketAddr = listen_addr.parse()?;
        let udp_socket = UdpSocket::bind(socket).await?;
        
        info!("DNS server listening on {}", socket);
        
        let mut buf = [0u8; 512];
        
        let udp_socket = Arc::new(udp_socket);
        
        loop {
            let socket_clone = Arc::clone(&udp_socket);
            match socket_clone.recv_from(&mut buf).await {
                Ok((size, src_addr)) => {
                    let scanner = Arc::clone(&self.scanner);
                    let upstream_dns = self.upstream_dns;
                    let data = buf[..size].to_vec();
                    
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_dns_query(&scanner, upstream_dns, &data, &socket_clone, src_addr).await {
                            error!("Failed to handle DNS query: {}", e);
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
        scanner: &Scanner,
        upstream_dns: SocketAddr,
        query_data: &[u8],
        socket: &UdpSocket,
        src_addr: SocketAddr,
    ) -> anyhow::Result<()> {
        // Parse DNS query (simplified - in production, use a proper DNS library)
        if query_data.len() < 12 {
            return Ok(());
        }
        
        let query_id = u16::from_be_bytes([query_data[0], query_data[1]]);
        let flags = u16::from_be_bytes([query_data[2], query_data[3]]);
        
        // Check if it's a standard query
        if (flags & 0x8000) != 0 {
            return Ok(()); // Not a query
        }
        
        // Extract domain name (simplified parsing)
        let mut domain = String::new();
        let mut pos = 12;
        
        while pos < query_data.len() && query_data[pos] != 0 {
            let label_len = query_data[pos] as usize;
            pos += 1;
            
            if pos + label_len > query_data.len() {
                break;
            }
            
            if !domain.is_empty() {
                domain.push('.');
            }
            
            domain.push_str(&String::from_utf8_lossy(&query_data[pos..pos + label_len]));
            pos += label_len;
        }
        
        debug!("Received DNS query for: {}", domain);
        
        // Check if this is a domain we should accelerate
        let accelerated_domains = vec![
            "github.com",
            "api.github.com", 
            "raw.githubusercontent.com",
            "assets-cdn.github.com"
        ];
        
        let should_accelerate = accelerated_domains.iter().any(|d| domain.ends_with(d));
        
        if should_accelerate {
            if let Some(best_ip) = scanner.get_best_ip(&domain).await {
                info!("Returning optimized IP {} for {}", best_ip, domain);
                
                // Create simplified DNS response with the optimized IP
                let mut response = Vec::new();
                
                // Transaction ID
                response.extend(&query_id.to_be_bytes());
                
                // Flags: Response, No error
                response.extend(&0x8180u16.to_be_bytes());
                
                // Questions: 1
                response.extend(&1u16.to_be_bytes());
                
                // Answer RRs: 1
                response.extend(&1u16.to_be_bytes());
                
                // Authority RRs: 0, Additional RRs: 0
                response.extend(&[0, 0, 0, 0]);
                
                // Copy the query section
                response.extend(&query_data[12..]);
                
                // Add answer: domain -> IP
                // Pointer to domain name in query
                response.push(0xC0);
                response.push(0x0C);
                
                // Type A, Class IN
                response.extend(&0x0001u16.to_be_bytes());
                response.extend(&0x0001u16.to_be_bytes());
                
                // TTL: 60 seconds
                response.extend(&60u32.to_be_bytes());
                
                // Data length: 4 bytes for IPv4
                response.extend(&4u16.to_be_bytes());
                
                // IP address
                response.extend(&best_ip.octets());
                
                socket.send_to(&response, src_addr).await?;
                return Ok(());
            }
        }
        
        // Forward to upstream DNS
        debug!("Forwarding query for {} to upstream DNS", domain);
        
        let upstream_socket = UdpSocket::bind("0.0.0.0:0").await?;
        upstream_socket.send_to(query_data, upstream_dns).await?;
        
        let mut response_buf = [0u8; 512];
        let size = upstream_socket.recv(&mut response_buf).await?;
        
        socket.send_to(&response_buf[..size], src_addr).await?;
        
        Ok(())
    }
}