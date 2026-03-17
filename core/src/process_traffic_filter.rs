use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::{Networks, ProcessExt, System, SystemExt};
use serde::{Deserialize, Serialize};

/// 进程流量数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessTrafficPoint {
    pub timestamp: u64,
    pub browser_upload: u64,
    pub browser_download: u64,
    pub git_upload: u64,
    pub git_download: u64,
    pub dev_tools_upload: u64,
    pub dev_tools_download: u64,
    pub other_upload: u64,
    pub other_download: u64,
}

/// 进程流量过滤器
pub struct ProcessTrafficFilter {
    system: System,
    networks: Networks,
    traffic_history: Arc<Mutex<Vec<ProcessTrafficPoint>>>,
    max_history_points: usize,
    last_total_upload: u64,
    last_total_download: u64,
    last_browser_upload: u64,
    last_browser_download: u64,
    last_git_upload: u64,
    last_git_download: u64,
    last_dev_tools_upload: u64,
    last_dev_tools_download: u64,
}

impl ProcessTrafficFilter {
    pub fn new(max_history_points: usize) -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        let networks = Networks::new_with_refreshed_list();
        
        Self {
            system,
            networks,
            traffic_history: Arc::new(Mutex::new(Vec::with_capacity(max_history_points))),
            max_history_points,
            last_total_upload: 0,
            last_total_download: 0,
            last_browser_upload: 0,
            last_browser_download: 0,
            last_git_upload: 0,
            last_git_download: 0,
            last_dev_tools_upload: 0,
            last_dev_tools_download: 0,
        }
    }

    /// 刷新系统信息
    pub fn refresh(&mut self) {
        self.system.refresh_all();
        self.networks.refresh();
    }

    /// 识别进程类型
    fn classify_process(&self, process_name: &str) -> &str {
        let name_lower = process_name.to_lowercase();
        
        // 浏览器进程
        if name_lower.contains("chrome") || 
           name_lower.contains("firefox") || 
           name_lower.contains("edge") || 
           name_lower.contains("opera") || 
           name_lower.contains("safari") || 
           name_lower.contains("brave") {
            return "browser";
        }
        
        // Git进程
        if name_lower.contains("git") || 
           name_lower.contains("git-bash") || 
           name_lower.contains("git-cmd") {
            return "git";
        }
        
        // 开发工具进程
        if name_lower.contains("vscode") || 
           name_lower.contains("code") || 
           name_lower.contains("visual studio") || 
           name_lower.contains("intellij") || 
           name_lower.contains("pycharm") || 
           name_lower.contains("webstorm") {
            return "dev_tools";
        }
        
        // Node.js进程
        if name_lower.contains("node") || 
           name_lower.contains("npm") || 
           name_lower.contains("yarn") {
            return "dev_tools";
        }
        
        "other"
    }

    /// 获取进程流量统计（简化实现）
    pub fn get_process_traffic(&mut self) -> (u64, u64, u64, u64, u64, u64, u64, u64) {
        self.refresh();
        
        let mut browser_upload = 0;
        let mut browser_download = 0;
        let mut git_upload = 0;
        let mut git_download = 0;
        let mut dev_tools_upload = 0;
        let mut dev_tools_download = 0;
        let mut other_upload = 0;
        let mut other_download = 0;

        // 统计所有网络接口的总流量
        let mut total_upload = 0;
        let mut total_download = 0;
        
        for (_, network) in &self.networks {
            total_upload += network.total_transmitted();
            total_download += network.total_received();
        }

        // 基于进程CPU使用率估算流量（简化实现）
        for (_, process) in self.system.processes() {
            let process_name = process.name();
            let process_type = self.classify_process(process_name);
            
            // 使用CPU使用率作为流量估算的代理指标
            let cpu_usage = process.cpu_usage();
            let estimated_upload = (cpu_usage * 500.0) as u64; // 简化估算
            let estimated_download = (cpu_usage * 1000.0) as u64; // 简化估算
            
            match process_type {
                "browser" => {
                    browser_upload += estimated_upload;
                    browser_download += estimated_download;
                }
                "git" => {
                    git_upload += estimated_upload;
                    git_download += estimated_download;
                }
                "dev_tools" => {
                    dev_tools_upload += estimated_upload;
                    dev_tools_download += estimated_download;
                }
                _ => {
                    other_upload += estimated_upload;
                    other_download += estimated_download;
                }
            }
        }

        (browser_upload, browser_download, git_upload, git_download, 
         dev_tools_upload, dev_tools_download, other_upload, other_download)
    }

    /// 记录进程流量数据点
    pub fn record_traffic_point(&mut self) {
        let (browser_upload, browser_download, git_upload, git_download, 
             dev_tools_upload, dev_tools_download, other_upload, other_download) = self.get_process_traffic();
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // 计算增量流量
        let delta_browser_upload = browser_upload.saturating_sub(self.last_browser_upload);
        let delta_browser_download = browser_download.saturating_sub(self.last_browser_download);
        let delta_git_upload = git_upload.saturating_sub(self.last_git_upload);
        let delta_git_download = git_download.saturating_sub(self.last_git_download);
        let delta_dev_tools_upload = dev_tools_upload.saturating_sub(self.last_dev_tools_upload);
        let delta_dev_tools_download = dev_tools_download.saturating_sub(self.last_dev_tools_download);
        let delta_other_upload = other_upload.saturating_sub(self.last_other_upload);
        let delta_other_download = other_download.saturating_sub(self.last_other_download);

        // 更新最后记录的值
        self.last_browser_upload = browser_upload;
        self.last_browser_download = browser_download;
        self.last_git_upload = git_upload;
        self.last_git_download = git_download;
        self.last_dev_tools_upload = dev_tools_upload;
        self.last_dev_tools_download = dev_tools_download;

        let traffic_point = ProcessTrafficPoint {
            timestamp,
            browser_upload: delta_browser_upload,
            browser_download: delta_browser_download,
            git_upload: delta_git_upload,
            git_download: delta_git_download,
            dev_tools_upload: delta_dev_tools_upload,
            dev_tools_download: delta_dev_tools_download,
            other_upload: delta_other_upload,
            other_download: delta_other_download,
        };

        let mut history = self.traffic_history.lock().unwrap();
        history.push(traffic_point);
        
        // 保持历史数据不超过最大点数
        if history.len() > self.max_history_points {
            history.remove(0);
        }
    }

    /// 获取进程流量历史数据
    pub fn get_traffic_history(&self, max_points: usize) -> Vec<ProcessTrafficPoint> {
        let history = self.traffic_history.lock().unwrap();
        let start_index = if history.len() > max_points {
            history.len() - max_points
        } else {
            0
        };
        
        history[start_index..].to_vec()
    }

    /// 获取进程流量统计汇总
    pub fn get_traffic_summary(&self) -> HashMap<String, (u64, u64)> {
        let history = self.traffic_history.lock().unwrap();
        let mut summary = HashMap::new();
        
        let mut browser_upload = 0;
        let mut browser_download = 0;
        let mut git_upload = 0;
        let mut git_download = 0;
        let mut dev_tools_upload = 0;
        let mut dev_tools_download = 0;
        let mut other_upload = 0;
        let mut other_download = 0;
        
        for point in history.iter() {
            browser_upload += point.browser_upload;
            browser_download += point.browser_download;
            git_upload += point.git_upload;
            git_download += point.git_download;
            dev_tools_upload += point.dev_tools_upload;
            dev_tools_download += point.dev_tools_download;
            other_upload += point.other_upload;
            other_download += point.other_download;
        }
        
        summary.insert("browser".to_string(), (browser_upload, browser_download));
        summary.insert("git".to_string(), (git_upload, git_download));
        summary.insert("dev_tools".to_string(), (dev_tools_upload, dev_tools_download));
        summary.insert("other".to_string(), (other_upload, other_download));
        
        summary
    }

    /// 启动进程流量监控
    pub fn start_monitoring(&self, interval_seconds: u64) {
        let filter = Arc::new(Mutex::new(self.clone()));
        
        tokio::spawn(async move {
            loop {
                {
                    let mut filter_guard = filter.lock().unwrap();
                    filter_guard.record_traffic_point();
                }
                
                tokio::time::sleep(tokio::time::Duration::from_secs(interval_seconds)).await;
            }
        });
    }
}

impl Clone for ProcessTrafficFilter {
    fn clone(&self) -> Self {
        Self {
            system: System::new_all(),
            networks: Networks::new_with_refreshed_list(),
            traffic_history: Arc::clone(&self.traffic_history),
            max_history_points: self.max_history_points,
            last_total_upload: self.last_total_upload,
            last_total_download: self.last_total_download,
            last_browser_upload: self.last_browser_upload,
            last_browser_download: self.last_browser_download,
            last_git_upload: self.last_git_upload,
            last_git_download: self.last_git_download,
            last_dev_tools_upload: self.last_dev_tools_upload,
            last_dev_tools_download: self.last_dev_tools_download,
        }
    }
}

/// 创建进程流量过滤器
pub fn create_process_traffic_filter(max_history_points: usize, interval_seconds: u64) -> ProcessTrafficFilter {
    let filter = ProcessTrafficFilter::new(max_history_points);
    filter.start_monitoring(interval_seconds);
    filter
}