#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::{Networks, System};
use tokio::time::{sleep, Duration};
use serde::{Deserialize, Serialize};
use crate::process_filter::{ProcessFilter, ProcessTraffic};

/// GitHub 核心域名列表
const GITHUB_DOMAINS: &[&str] = &[
    "github.com",
    "api.github.com", 
    "raw.githubusercontent.com",
    "gist.github.com",
    "github.io",
    "githubusercontent.com",
    "githubassets.com",
    "githubapp.com",
];

/// GitHub IP地址跟踪器
#[derive(Debug, Clone)]
pub struct GitHubIPTracker {
    github_ips: Arc<Mutex<HashMap<String, bool>>>,
}

impl GitHubIPTracker {
    pub fn new() -> Self {
        Self {
            github_ips: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 检查IP是否为GitHub相关IP
    pub fn is_github_ip(&self, ip: &str) -> bool {
        let github_ips = self.github_ips.lock().unwrap();
        github_ips.get(ip).copied().unwrap_or(false)
    }

    /// 添加GitHub IP地址
    pub fn add_github_ip(&self, ip: String) {
        let mut github_ips = self.github_ips.lock().unwrap();
        github_ips.insert(ip, true);
    }

    /// 获取所有GitHub IP地址
    pub fn get_github_ips(&self) -> Vec<String> {
        let github_ips = self.github_ips.lock().unwrap();
        github_ips.keys().cloned().collect()
    }
}

/// 网络流量数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficDataPoint {
    pub timestamp: u64,
    pub total_upload: u64,
    pub total_download: u64,
    pub github_upload: u64,
    pub github_download: u64,
    pub target_upload: u64,
    pub target_download: u64,
}

/// GitHub流量监控器
pub struct GitHubTrafficMonitor {
    system: System,
    networks: Networks,
    github_tracker: GitHubIPTracker,
    process_filter: ProcessFilter,
    traffic_history: Arc<Mutex<Vec<TrafficDataPoint>>>,
    process_traffic_history: Arc<Mutex<Vec<Vec<ProcessTraffic>>>>,
    max_history_points: usize,
    last_total_upload: u64,
    last_total_download: u64,
    last_github_upload: u64,
    last_github_download: u64,
    last_target_upload: u64,
    last_target_download: u64,
}

impl GitHubTrafficMonitor {
    pub fn new(max_history_points: usize) -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        let networks = Networks::new_with_refreshed_list();
        
        Self {
            system,
            networks,
            github_tracker: GitHubIPTracker::new(),
            process_filter: ProcessFilter::new(),
            traffic_history: Arc::new(Mutex::new(Vec::with_capacity(max_history_points))),
            process_traffic_history: Arc::new(Mutex::new(Vec::with_capacity(max_history_points))),
            max_history_points,
            last_total_upload: 0,
            last_total_download: 0,
            last_github_upload: 0,
            last_github_download: 0,
            last_target_upload: 0,
            last_target_download: 0,
        }
    }

    /// 刷新网络信息
    pub fn refresh(&mut self) {
        self.networks.refresh();
    }

    /// 获取当前网络流量统计
    pub fn get_current_traffic(&mut self) -> (u64, u64, u64, u64, u64, u64) {
        self.refresh();
        
        let mut total_upload = 0;
        let mut total_download = 0;
        let mut github_upload = 0;
        let mut github_download = 0;
        let mut target_upload = 0;
        let mut target_download = 0;

        // 统计所有网络接口的流量
        for (_interface_name, network) in &self.networks {
            let current_upload = network.total_transmitted();
            let current_download = network.total_received();
            
            total_upload += current_upload;
            total_download += current_download;

            // 所有流量都视为 GitHub 流量（因为我们通过 DNS 代理控制 GitHub 访问）
            github_upload += current_upload;
            github_download += current_download;
        }

        // 获取目标进程的流量统计
        let (process_upload, process_download) = self.process_filter.get_total_target_traffic();
        target_upload += process_upload;
        target_download += process_download;

        (total_upload, total_download, github_upload, github_download, target_upload, target_download)
    }

    /// 检查网络接口是否为 GitHub 相关（简化实现：总是返回 true）
    fn is_github_interface(&self, _interface_name: &str) -> bool {
        // 所有网络接口都用于 GitHub 访问
        true
    }

    /// 记录流量数据点
    pub fn record_traffic_point(&mut self) {
        let (total_upload, total_download, github_upload, github_download, target_upload, target_download) = self.get_current_traffic();
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // 计算增量流量
        let delta_total_upload = total_upload.saturating_sub(self.last_total_upload);
        let delta_total_download = total_download.saturating_sub(self.last_total_download);
        let delta_github_upload = github_upload.saturating_sub(self.last_github_upload);
        let delta_github_download = github_download.saturating_sub(self.last_github_download);
        let delta_target_upload = target_upload.saturating_sub(self.last_target_upload);
        let delta_target_download = target_download.saturating_sub(self.last_target_download);

        // 更新最后记录的值
        self.last_total_upload = total_upload;
        self.last_total_download = total_download;
        self.last_github_upload = github_upload;
        self.last_github_download = github_download;
        self.last_target_upload = target_upload;
        self.last_target_download = target_download;

        let traffic_point = TrafficDataPoint {
            timestamp,
            total_upload: delta_total_upload,
            total_download: delta_total_download,
            github_upload: delta_github_upload,
            github_download: delta_github_download,
            target_upload: delta_target_upload,
            target_download: delta_target_download,
        };

        let mut history = self.traffic_history.lock().unwrap();
        history.push(traffic_point);
        
        // 保持历史数据不超过最大点数
        if history.len() > self.max_history_points {
            history.remove(0);
        }

        // 记录进程流量数据
        let process_traffic = self.process_filter.get_target_processes_traffic();
        let mut process_history = self.process_traffic_history.lock().unwrap();
        process_history.push(process_traffic);
        
        // 保持进程历史数据不超过最大点数
        if process_history.len() > self.max_history_points {
            process_history.remove(0);
        }
    }

    /// 获取流量历史数据
    pub fn get_traffic_history(&self, max_points: usize) -> Vec<TrafficDataPoint> {
        let history = self.traffic_history.lock().unwrap();
        let start_index = if history.len() > max_points {
            history.len() - max_points
        } else {
            0
        };
        
        history[start_index..].to_vec()
    }

    /// 获取GitHub流量统计
    pub fn get_github_traffic_stats(&self) -> (u64, u64) {
        let history = self.traffic_history.lock().unwrap();
        
        let total_github_upload: u64 = history.iter().map(|p| p.github_upload).sum();
        let total_github_download: u64 = history.iter().map(|p| p.github_download).sum();
        
        (total_github_upload, total_github_download)
    }

    /// 获取目标进程流量统计
    pub fn get_target_traffic_stats(&self) -> (u64, u64) {
        let history = self.traffic_history.lock().unwrap();
        
        let total_target_upload: u64 = history.iter().map(|p| p.target_upload).sum();
        let total_target_download: u64 = history.iter().map(|p| p.target_download).sum();
        
        (total_target_upload, total_target_download)
    }

    /// 获取进程流量历史数据
    pub fn get_process_traffic_history(&self, max_points: usize) -> Vec<Vec<ProcessTraffic>> {
        let history = self.process_traffic_history.lock().unwrap();
        let start_index = if history.len() > max_points {
            history.len() - max_points
        } else {
            0
        };
        
        history[start_index..].to_vec()
    }

    /// 获取按类型分组的进程流量统计
    pub fn get_process_traffic_by_type(&self) -> HashMap<String, (u64, u64)> {
        let history = self.process_traffic_history.lock().unwrap();
        let mut result = HashMap::new();
        
        // 统计最近的数据点
        if let Some(latest_processes) = history.last() {
            for process in latest_processes {
                let entry = result.entry(process.process_type.clone()).or_insert((0, 0));
                entry.0 += process.upload_bytes;
                entry.1 += process.download_bytes;
            }
        }
        
        result
    }

    /// 获取所有活跃的目标进程
    pub fn get_active_target_processes(&self) -> Vec<ProcessTraffic> {
        let history = self.process_traffic_history.lock().unwrap();
        
        if let Some(latest_processes) = history.last() {
            latest_processes.clone()
        } else {
            Vec::new()
        }
    }

    /// 启动流量监控任务
    pub fn start_monitoring(&self, interval_seconds: u64) {
        let monitor = Arc::new(Mutex::new(self.clone()));
        
        tokio::spawn(async move {
            loop {
                {
                    let mut monitor_guard = monitor.lock().unwrap();
                    monitor_guard.record_traffic_point();
                }
                
                sleep(Duration::from_secs(interval_seconds)).await;
            }
        });
    }

    /// 获取GitHub IP跟踪器引用
    pub fn get_github_tracker(&self) -> &GitHubIPTracker {
        &self.github_tracker
    }
}

impl Clone for GitHubTrafficMonitor {
    fn clone(&self) -> Self {
        Self {
            system: System::new_all(),
            networks: Networks::new_with_refreshed_list(),
            github_tracker: self.github_tracker.clone(),
            process_filter: self.process_filter.clone(),
            traffic_history: Arc::clone(&self.traffic_history),
            process_traffic_history: Arc::clone(&self.process_traffic_history),
            max_history_points: self.max_history_points,
            last_total_upload: self.last_total_upload,
            last_total_download: self.last_total_download,
            last_github_upload: self.last_github_upload,
            last_github_download: self.last_github_download,
            last_target_upload: self.last_target_upload,
            last_target_download: self.last_target_download,
        }
    }
}

/// 创建并启动GitHub流量监控器
pub fn create_github_traffic_monitor(max_history_points: usize, interval_seconds: u64) -> GitHubTrafficMonitor {
    let monitor = GitHubTrafficMonitor::new(max_history_points);
    monitor.start_monitoring(interval_seconds);
    monitor
}