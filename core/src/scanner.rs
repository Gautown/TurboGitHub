use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::{Mutex, Semaphore};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone)]
pub struct IpInfo {
    pub ip: Ipv4Addr,
    pub rtt: Duration,
    pub reachable: bool,
    pub https_available: bool,
}

pub struct Scanner {
    config: Arc<crate::config::Config>,
    ip_pool: Arc<Mutex<HashMap<String, Vec<IpInfo>>>>,
}

impl Scanner {
    pub fn new(config: Arc<crate::config::Config>) -> Self {
        Self {
            config,
            ip_pool: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn start(&self) {
        let config = Arc::clone(&self.config);
        let ip_pool = Arc::clone(&self.ip_pool);
        
        tokio::spawn(async move {
            loop {
                match Self::scan_domains(&config, &ip_pool).await {
                    Ok(_) => info!("IP scan completed successfully"),
                    Err(e) => error!("IP scan failed: {}", e),
                }
                
                tokio::time::sleep(Duration::from_secs(config.scan_interval)).await;
            }
        });
    }

    async fn scan_domains(
        config: &crate::config::Config,
        ip_pool: &Arc<Mutex<HashMap<String, Vec<IpInfo>>>>,
    ) -> anyhow::Result<()> {
        info!("Starting IP scan for {} domains", config.domains.len());
        
        let semaphore = Arc::new(Semaphore::new(config.scan_concurrency));
        let mut tasks = Vec::new();
        
        for domain in &config.domains {
            let domain = domain.clone();
            let config = Arc::clone(&Arc::new(config.clone()));
            let semaphore = Arc::clone(&semaphore);
            let ip_pool = Arc::clone(ip_pool);
            
            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                
                let domain_clone = domain.clone();
                        match Self::scan_domain(&domain, &config).await {
                            Ok(ip_infos) => {
                                let mut pool = ip_pool.lock().await;
                                pool.insert(domain_clone.clone(), ip_infos);
                                debug!("Scanned domain: {}", domain_clone);
                            }
                    Err(e) => {
                        warn!("Failed to scan domain {}: {}", domain, e);
                    }
                }
            });
            
            tasks.push(task);
        }
        
        for task in tasks {
            task.await?;
        }
        
        Ok(())
    }

    async fn scan_domain(
        domain: &str,
        _config: &crate::config::Config,
    ) -> anyhow::Result<Vec<IpInfo>> {
        let resolver = trust_dns_resolver::TokioAsyncResolver::tokio_from_system_conf()?;
        let lookup = resolver.ipv4_lookup(domain).await?;
        
        let ips: Vec<Ipv4Addr> = lookup.iter().map(|ip| ip.0).collect();
        info!("Found {} IPs for domain {}", ips.len(), domain);
        
        let mut ip_infos = Vec::new();
        for ip in ips {
            match Self::test_ip(ip, domain).await {
                Ok(ip_info) => ip_infos.push(ip_info),
                Err(e) => warn!("Failed to test IP {} for {}: {}", ip, domain, e),
            }
        }
        
        ip_infos.sort_by(|a, b| a.rtt.cmp(&b.rtt));
        
        Ok(ip_infos)
    }

    async fn test_ip(ip: Ipv4Addr, _domain: &str) -> anyhow::Result<IpInfo> {
        let start = std::time::Instant::now();
        
        let reachable = match timeout(Duration::from_secs(3), TcpStream::connect((ip, 443))).await {
            Ok(Ok(_)) => true,
            _ => false,
        };
        
        let rtt = start.elapsed();
        
        let https_available = if reachable {
            match reqwest::Client::new()
                .get(&format!("https://{}/robots.txt", ip))
                .timeout(Duration::from_secs(5))
                .send()
                .await
            {
                Ok(response) => response.status().is_success(),
                Err(_) => false,
            }
        } else {
            false
        };
        
        Ok(IpInfo {
            ip,
            rtt,
            reachable,
            https_available,
        })
    }

    pub async fn get_best_ip(&self, domain: &str) -> Option<Ipv4Addr> {
        let pool = self.ip_pool.lock().await;
        
        if let Some(ips) = pool.get(domain) {
            // 过滤可用的IP地址
            let available_ips: Vec<&IpInfo> = ips.iter()
                .filter(|ip| ip.reachable && ip.https_available)
                .collect();
            
            if available_ips.is_empty() {
                return None;
            }
            
            // 选择延迟最低的IP地址
            let best_ip = available_ips.iter()
                .min_by(|a, b| a.rtt.cmp(&b.rtt))
                .map(|ip| ip.ip);
            
            if let Some(ip) = best_ip {
                debug!("Selected best IP {} for {} with RTT {:?}", ip, domain, ips.iter()
                    .find(|i| i.ip == ip)
                    .map(|i| i.rtt)
                    .unwrap_or_default());
            }
            
            best_ip
        } else {
            None
        }
    }

    pub async fn get_ip_pool(&self) -> HashMap<String, Vec<IpInfo>> {
        self.ip_pool.lock().await.clone()
    }
}