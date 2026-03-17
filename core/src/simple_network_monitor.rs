use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::Networks;
use serde::{Deserialize, Serialize};

/// 简单的网络流量数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleTrafficDataPoint {
    pub timestamp: u64,
    pub total_upload: u64,
    pub total_download: u64,
}

/// 简单的网络流量监控器
pub struct SimpleNetworkMonitor {
    networks: Networks,
    traffic_history: Arc<Mutex<Vec<SimpleTrafficDataPoint>>>,
    max_history_points: usize,
    last_total_upload: u64,
    last_total_download: u64,
}

impl SimpleNetworkMonitor {
    pub fn new(max_history_points: usize) -> Self {
        let networks = Networks::new_with_refreshed_list();
        
        Self {
            networks,
            traffic_history: Arc::new(Mutex::new(Vec::with_capacity(max_history_points))),
            max_history_points,
            last_total_upload: 0,
            last_total_download: 0,
        }
    }

    /// 刷新网络信息
    pub fn refresh(&mut self) {
        self.networks.refresh();
    }

    /// 获取当前网络流量统计
    pub fn get_current_traffic(&mut self) -> (u64, u64) {
        self.refresh();
        
        let mut total_upload = 0;
        let mut total_download = 0;

        // 统计所有网络接口的流量
        for (_, network) in &self.networks {
            total_upload += network.total_transmitted();
            total_download += network.total_received();
        }

        (total_upload, total_download)
    }

    /// 记录流量数据点
    pub fn record_traffic_point(&mut self) {
        let (total_upload, total_download) = self.get_current_traffic();
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // 计算增量流量
        let delta_total_upload = total_upload.saturating_sub(self.last_total_upload);
        let delta_total_download = total_download.saturating_sub(self.last_total_download);

        // 更新最后记录的值
        self.last_total_upload = total_upload;
        self.last_total_download = total_download;

        let traffic_point = SimpleTrafficDataPoint {
            timestamp,
            total_upload: delta_total_upload,
            total_download: delta_total_download,
        };

        let mut history = self.traffic_history.lock().unwrap();
        history.push(traffic_point);
        
        // 保持历史数据不超过最大点数
        if history.len() > self.max_history_points {
            history.remove(0);
        }
    }

    /// 获取流量历史数据
    pub fn get_traffic_history(&self, max_points: usize) -> Vec<SimpleTrafficDataPoint> {
        let history = self.traffic_history.lock().unwrap();
        let start_index = if history.len() > max_points {
            history.len() - max_points
        } else {
            0
        };
        
        history[start_index..].to_vec()
    }

    /// 获取流量统计汇总
    #[allow(dead_code)]
    pub fn get_traffic_summary(&self) -> (u64, u64) {
        let history = self.traffic_history.lock().unwrap();
        
        let total_upload: u64 = history.iter().map(|p| p.total_upload).sum();
        let total_download: u64 = history.iter().map(|p| p.total_download).sum();
        
        (total_upload, total_download)
    }
}

impl Clone for SimpleNetworkMonitor {
    fn clone(&self) -> Self {
        Self {
            networks: Networks::new_with_refreshed_list(),
            traffic_history: Arc::clone(&self.traffic_history),
            max_history_points: self.max_history_points,
            last_total_upload: self.last_total_upload,
            last_total_download: self.last_total_download,
        }
    }
}