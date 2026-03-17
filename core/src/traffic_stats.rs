use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

/// 域名流量统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainTraffic {
    pub domain: String,
    pub upload_bytes: u64,
    pub download_bytes: u64,
    pub query_count: u64,
    pub last_updated: u64, // 时间戳（秒）
}

/// 实时流量数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficDataPoint {
    pub timestamp: u64, // 时间戳（秒）
    pub upload_bytes: u64,
    pub download_bytes: u64,
    pub total_queries: u64,
}

/// 流量统计器
pub struct TrafficStats {
    domains: Arc<Mutex<HashMap<String, DomainTraffic>>>,
    realtime_data: Arc<Mutex<Vec<TrafficDataPoint>>>,
    max_data_points: usize,
    total_upload: Arc<Mutex<u64>>,
    total_download: Arc<Mutex<u64>>,
    total_queries: Arc<Mutex<u64>>,
    start_time: u64,
}

impl TrafficStats {
    pub fn new(max_data_points: usize) -> Self {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        Self {
            domains: Arc::new(Mutex::new(HashMap::new())),
            realtime_data: Arc::new(Mutex::new(Vec::with_capacity(max_data_points))),
            max_data_points,
            total_upload: Arc::new(Mutex::new(0)),
            total_download: Arc::new(Mutex::new(0)),
            total_queries: Arc::new(Mutex::new(0)),
            start_time,
        }
    }
    
    /// 记录DNS查询流量
    pub fn record_dns_query(&self, domain: &str, query_size: usize, response_size: usize) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        // 更新域名统计
        {
            let mut domains = self.domains.lock().unwrap();
            let domain_traffic = domains.entry(domain.to_string()).or_insert_with(|| DomainTraffic {
                domain: domain.to_string(),
                upload_bytes: 0,
                download_bytes: 0,
                query_count: 0,
                last_updated: timestamp,
            });
            
            domain_traffic.upload_bytes += query_size as u64;
            domain_traffic.download_bytes += response_size as u64;
            domain_traffic.query_count += 1;
            domain_traffic.last_updated = timestamp;
        }
        
        // 更新总流量
        {
            let mut total_upload = self.total_upload.lock().unwrap();
            let mut total_download = self.total_download.lock().unwrap();
            let mut total_queries = self.total_queries.lock().unwrap();
            
            *total_upload += query_size as u64;
            *total_download += response_size as u64;
            *total_queries += 1;
        }
        
        // 添加实时数据点
        self.add_realtime_data_point(query_size as u64, response_size as u64);
    }
    
    /// 添加实时数据点
    fn add_realtime_data_point(&self, upload_bytes: u64, download_bytes: u64) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        let total_queries = *self.total_queries.lock().unwrap();
        
        let data_point = TrafficDataPoint {
            timestamp,
            upload_bytes,
            download_bytes,
            total_queries,
        };
        
        let mut data = self.realtime_data.lock().unwrap();
        
        // 限制数据点数量
        if data.len() >= self.max_data_points {
            data.remove(0);
        }
        
        data.push(data_point);
    }
    
    /// 获取总流量统计
    pub fn get_total_traffic(&self) -> (u64, u64, u64) {
        let upload = *self.total_upload.lock().unwrap();
        let download = *self.total_download.lock().unwrap();
        let queries = *self.total_queries.lock().unwrap();
        (upload, download, queries)
    }
    
    /// 获取域名流量统计
    pub fn get_domain_traffic(&self) -> Vec<DomainTraffic> {
        let domains = self.domains.lock().unwrap();
        domains.values().cloned().collect()
    }
    
    /// 获取实时流量数据
    pub fn get_realtime_data(&self, max_points: usize) -> Vec<TrafficDataPoint> {
        let data = self.realtime_data.lock().unwrap();
        let start_idx = if data.len() > max_points {
            data.len() - max_points
        } else {
            0
        };
        data[start_idx..].to_vec()
    }
    
    /// 获取应用启动时间
    pub fn get_start_time(&self) -> u64 {
        self.start_time
    }
    
    // 重置功能在自动代理模式下不需要
}