#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use sysinfo::{Process, System};
use serde::{Deserialize, Serialize};

/// 目标进程配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessConfig {
    pub name_patterns: Vec<String>,
    pub executable_patterns: Vec<String>,
    pub description: String,
}

/// 进程流量统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessTraffic {
    pub pid: u32,
    pub name: String,
    pub upload_bytes: u64,
    pub download_bytes: u64,
    pub process_type: String,
}

/// 进程过滤器
pub struct ProcessFilter {
    system: System,
    target_processes: Vec<ProcessConfig>,
    process_cache: Arc<Mutex<HashMap<u32, ProcessTraffic>>>,
}

impl Clone for ProcessFilter {
    fn clone(&self) -> Self {
        Self {
            system: System::new(),
            target_processes: self.target_processes.clone(),
            process_cache: Arc::clone(&self.process_cache),
        }
    }
}

impl ProcessFilter {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        // 定义目标进程配置
        let target_processes = vec![
            ProcessConfig {
                name_patterns: vec![
                    "chrome".to_string(),
                    "firefox".to_string(),
                    "edge".to_string(),
                    "opera".to_string(),
                    "safari".to_string(),
                    "brave".to_string(),
                ],
                executable_patterns: vec![
                    "chrome.exe".to_string(),
                    "firefox.exe".to_string(),
                    "msedge.exe".to_string(),
                    "opera.exe".to_string(),
                    "safari.exe".to_string(),
                    "brave.exe".to_string(),
                ],
                description: "浏览器进程".to_string(),
            },
            ProcessConfig {
                name_patterns: vec![
                    "git".to_string(),
                    "git-bash".to_string(),
                    "git-cmd".to_string(),
                ],
                executable_patterns: vec![
                    "git.exe".to_string(),
                    "git-bash.exe".to_string(),
                    "git-cmd.exe".to_string(),
                ],
                description: "Git进程".to_string(),
            },
            ProcessConfig {
                name_patterns: vec![
                    "vscode".to_string(),
                    "code".to_string(),
                    "visual studio code".to_string(),
                ],
                executable_patterns: vec![
                    "code.exe".to_string(),
                    "vscode.exe".to_string(),
                ],
                description: "开发工具进程".to_string(),
            },
            ProcessConfig {
                name_patterns: vec![
                    "node".to_string(),
                    "npm".to_string(),
                    "yarn".to_string(),
                ],
                executable_patterns: vec![
                    "node.exe".to_string(),
                    "npm.exe".to_string(),
                    "yarn.exe".to_string(),
                ],
                description: "Node.js进程".to_string(),
            },
        ];
        
        Self {
            system,
            target_processes,
            process_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 刷新进程信息
    pub fn refresh(&mut self) {
        self.system.refresh_processes();
    }

    /// 检查进程是否为目标进程
    pub fn is_target_process(&self, process: &Process) -> Option<String> {
        let process_name = process.name().to_lowercase();
        let process_exe = process.exe().map(|p| p.to_string_lossy().to_lowercase()).unwrap_or_default();
        
        for config in &self.target_processes {
            // 检查进程名称匹配
            for pattern in &config.name_patterns {
                if process_name.contains(&pattern.to_lowercase()) {
                    return Some(config.description.clone());
                }
            }
            
            // 检查可执行文件路径匹配
            for pattern in &config.executable_patterns {
                if process_exe.contains(&pattern.to_lowercase()) {
                    return Some(config.description.clone());
                }
            }
        }
        
        None
    }

    /// 获取所有目标进程的流量统计
    pub fn get_target_processes_traffic(&mut self) -> Vec<ProcessTraffic> {
        self.refresh();
        
        let mut result = Vec::new();
        let mut cache = self.process_cache.lock().unwrap();
        
        for (pid, process) in self.system.processes() {
            if let Some(process_type) = self.is_target_process(process) {
                let process_name = process.name().to_string();
                
                // 获取进程的网络使用情况
                let (upload_bytes, download_bytes) = self.get_process_network_usage(process);
                
                let process_traffic = ProcessTraffic {
                    pid: pid.as_u32(),
                    name: process_name,
                    upload_bytes,
                    download_bytes,
                    process_type,
                };
                
                result.push(process_traffic.clone());
                
                // 更新缓存
                cache.insert(pid.as_u32(), process_traffic);
            }
        }
        
        result
    }

    /// 获取进程的网络使用情况（简化实现）
    fn get_process_network_usage(&self, process: &Process) -> (u64, u64) {
        // 在实际实现中，可以使用更精确的方法来获取进程的网络使用情况
        // 这里使用简化实现，返回进程的CPU使用率作为网络使用量的代理指标
        let cpu_usage = process.cpu_usage();
        
        // 将CPU使用率转换为网络使用量的估计值
        let upload_bytes = (cpu_usage * 1000.0) as u64; // 简化估算
        let download_bytes = (cpu_usage * 2000.0) as u64; // 简化估算
        
        (upload_bytes, download_bytes)
    }

    /// 获取目标进程的总流量
    pub fn get_total_target_traffic(&mut self) -> (u64, u64) {
        let processes = self.get_target_processes_traffic();
        
        let total_upload: u64 = processes.iter().map(|p| p.upload_bytes).sum();
        let total_download: u64 = processes.iter().map(|p| p.download_bytes).sum();
        
        (total_upload, total_download)
    }

    /// 获取按类型分组的流量统计
    pub fn get_traffic_by_type(&mut self) -> HashMap<String, (u64, u64)> {
        let processes = self.get_target_processes_traffic();
        let mut result = HashMap::new();
        
        for process in processes {
            let entry = result.entry(process.process_type.clone()).or_insert((0, 0));
            entry.0 += process.upload_bytes;
            entry.1 += process.download_bytes;
        }
        
        result
    }

    /// 添加自定义进程配置
    pub fn add_process_config(&mut self, config: ProcessConfig) {
        self.target_processes.push(config);
    }

    /// 获取所有进程配置
    pub fn get_process_configs(&self) -> &Vec<ProcessConfig> {
        &self.target_processes
    }
}

impl Default for ProcessFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// 创建进程过滤器实例
pub fn create_process_filter() -> ProcessFilter {
    ProcessFilter::new()
}