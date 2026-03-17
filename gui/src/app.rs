use crate::integrated_service::{IntegratedService, ServiceStatus};
use crate::traffic_monitor::{TrafficMonitor, draw_traffic_chart};
use eframe::egui;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};

// 全局共享的窗口可见性标志
pub static WINDOW_SHOULD_BE_VISIBLE: AtomicBool = AtomicBool::new(true);

pub struct TurboGitHubApp {
    rt: Runtime,
    service: Arc<IntegratedService>,
    status: Arc<Mutex<Option<ServiceStatus>>>,
    logs: Arc<Mutex<Vec<String>>>,
    traffic_monitor: Arc<Mutex<TrafficMonitor>>,
    app_start_time: SystemTime,
    auto_refresh: bool,
    last_refresh: std::time::Instant,
    window_configured: bool,
    service_running: bool,
    window_visible: bool,
}

impl TurboGitHubApp {
    #[allow(dead_code)]
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let service = Arc::new(IntegratedService::new("dynamic".to_string()));
        let status = Arc::new(Mutex::new(None));
        let logs = Arc::new(Mutex::new(Vec::new()));
        let traffic_monitor = Arc::new(Mutex::new(TrafficMonitor::new(100))); // 保存最近 100 个数据点
        let app_start_time = SystemTime::now(); // 记录应用启动时间
        
        // 启动后台任务
        let service_clone = Arc::clone(&service);
        let status_clone = Arc::clone(&status);
        let logs_clone = Arc::clone(&logs);
        let traffic_monitor_clone = Arc::clone(&traffic_monitor);
        
        rt.spawn(async move {
            Self::background_task(service_clone, status_clone, logs_clone, traffic_monitor_clone).await;
        });
        
        Self {
            rt,
            service,
            status,
            logs,
            traffic_monitor,
            app_start_time,
            auto_refresh: true,
            last_refresh: std::time::Instant::now(),
            window_configured: false,
            service_running: true,
            window_visible: true,
        }
    }
    
    /// 使用外部服务实例创建应用
    pub fn new_with_service(_cc: &eframe::CreationContext<'_>, service: Arc<IntegratedService>) -> Self {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let status = Arc::new(Mutex::new(None));
        let logs = Arc::new(Mutex::new(Vec::new()));
        let traffic_monitor = Arc::new(Mutex::new(TrafficMonitor::new(100))); // 保存最近 100 个数据点
        let app_start_time = SystemTime::now(); // 记录应用启动时间
        
        // 使用独立的线程运行后台任务
        let service_clone = Arc::clone(&service);
        let status_clone = Arc::clone(&status);
        let logs_clone = Arc::clone(&logs);
        let traffic_monitor_clone = Arc::clone(&traffic_monitor);
        
        thread::spawn(move || {
            let bg_rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            bg_rt.block_on(async move {
                Self::background_task(service_clone, status_clone, logs_clone, traffic_monitor_clone).await;
            });
        });
        
        Self {
            rt,
            service,
            status,
            logs,
            traffic_monitor,
            app_start_time,
            auto_refresh: true,
            last_refresh: std::time::Instant::now(),
            window_configured: false,
            service_running: true,
            window_visible: true,
        }
    }
    
    async fn background_task(
        service: Arc<IntegratedService>,
        status: Arc<Mutex<Option<ServiceStatus>>>,
        logs: Arc<Mutex<Vec<String>>>,
        traffic_monitor: Arc<Mutex<TrafficMonitor>>,
    ) {
        let mut status_interval = tokio::time::interval(std::time::Duration::from_secs(5));
        let mut traffic_interval = tokio::time::interval(std::time::Duration::from_secs(2));
        let mut traffic_count = 0usize;
        
        // 启动集成化服务
        if let Err(e) = service.start_service().await {
            let mut logs_guard = logs.lock().await;
            logs_guard.push(format!("启动服务失败：{}", e));
        } else {
            let mut logs_guard = logs.lock().await;
            logs_guard.push("集成化服务已启动".to_string());
        }
        
        // 立即获取一次流量数据
        match service.get_realtime_traffic(100).await {
            Ok(traffic_data) => {
                let mut logs_guard = logs.lock().await;
                logs_guard.push("✅ 初始流量数据获取成功".to_string());
                
                if let Some(network_traffic) = traffic_data["network_traffic"].as_object() {
                    if let Some(data_points) = network_traffic["data_points"].as_array() {
                        let mut monitor = traffic_monitor.lock().await;
                        for point in data_points {
                            if let (Some(upload_bytes), Some(download_bytes)) = (
                                point["total_upload"].as_u64(),
                                point["total_download"].as_u64()
                            ) {
                                monitor.add_traffic(upload_bytes, download_bytes);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                let mut logs_guard = logs.lock().await;
                logs_guard.push(format!("❌ 初始流量数据获取失败：{}", e));
            }
        }
        
        loop {
            // 获取状态
            match service.get_status().await {
                Ok(new_status) => {
                    let mut status_guard = status.lock().await;
                    *status_guard = Some(new_status);
                }
                Err(e) => {
                    let mut status_guard = status.lock().await;
                    *status_guard = None;
                    
                    let mut logs_guard = logs.lock().await;
                    logs_guard.push(format!("获取状态失败：{}", e));
                }
            }
            
            status_interval.tick().await;
            
            // 模拟网络扫描
            if let Err(e) = service.scan_networks().await {
                let mut logs_guard = logs.lock().await;
                logs_guard.push(format!("网络扫描失败：{}", e));
            }
            
            // 获取流量数据
            traffic_count += 1;
            match service.get_realtime_traffic(100).await {
                Ok(traffic_data) => {
                    let mut logs_guard = logs.lock().await;
                    logs_guard.push(format!("✅ 第 {} 次获取流量数据成功", traffic_count));
                    
                    // 优先使用网络流量数据（系统级真实流量）
                    if let Some(network_traffic) = traffic_data["network_traffic"].as_object() {
                        if let Some(data_points) = network_traffic["data_points"].as_array() {
                            let mut monitor = traffic_monitor.lock().await;
                            for point in data_points {
                                if let (Some(upload_bytes), Some(download_bytes)) = (
                                    point["total_upload"].as_u64(),
                                    point["total_download"].as_u64()
                                ) {
                                    // 使用增量数据方法（核心返回的是增量数据）
                                    monitor.add_delta_traffic(upload_bytes, download_bytes);
                                }
                            }
                        }
                    }
                    // 备用：使用 DNS 查询流量统计
                    else if let Some(dns_traffic) = traffic_data["dns_traffic"].as_object() {
                        if let Some(data_points) = dns_traffic["data_points"].as_array() {
                            let mut monitor = traffic_monitor.lock().await;
                            for point in data_points {
                                if let (Some(upload_bytes), Some(download_bytes)) = (
                                    point["upload_bytes"].as_u64(),
                                    point["download_bytes"].as_u64()
                                ) {
                                    // DNS 流量也是增量数据
                                    monitor.add_delta_traffic(upload_bytes, download_bytes);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    let mut logs_guard = logs.lock().await;
                    logs_guard.push(format!("❌ 第 {} 次获取流量数据失败：{}", traffic_count, e));
                }
            }
            
            traffic_interval.tick().await;
        }
    }
    
    fn add_log(&self, message: String) {
        let logs = Arc::clone(&self.logs);
        self.rt.spawn(async move {
            let mut logs_guard = logs.lock().await;
            logs_guard.push(message);
            if logs_guard.len() > 100 {
                logs_guard.remove(0);
            }
        });
    }
    
    fn start_service(&mut self) {
        let service = Arc::clone(&self.service);
        let logs = Arc::clone(&self.logs);
        self.add_log("启动服务...".to_string());
        
        self.service_running = true;
        
        self.rt.spawn(async move {
            match service.start_service().await {
                Ok(()) => {
                    let mut logs_guard = logs.lock().await;
                    logs_guard.push("✅ 服务启动成功".to_string());
                }
                Err(e) => {
                    let mut logs_guard = logs.lock().await;
                    logs_guard.push(format!("❌ 服务启动失败: {}", e));
                }
            }
        });
    }
    
    fn stop_service(&mut self) {
        let service = Arc::clone(&self.service);
        let logs = Arc::clone(&self.logs);
        self.add_log("停止服务...".to_string());
        
        self.service_running = false;
        
        self.rt.spawn(async move {
            match service.stop_service().await {
                Ok(()) => {
                    let mut logs_guard = logs.lock().await;
                    logs_guard.push("✅ 服务停止成功".to_string());
                }
                Err(e) => {
                    let mut logs_guard = logs.lock().await;
                    logs_guard.push(format!("❌ 服务停止失败: {}", e));
                }
            }
        });
    }
    
    fn scan_now(&self) {
        let service = Arc::clone(&self.service);
        let logs = Arc::clone(&self.logs);
        self.add_log("开始手动扫描...".to_string());
        
        self.rt.spawn(async move {
            match service.scan_networks().await {
                Ok(()) => {
                    let mut logs_guard = logs.lock().await;
                    logs_guard.push("✅ 网络扫描完成".to_string());
                }
                Err(e) => {
                    let mut logs_guard = logs.lock().await;
                    logs_guard.push(format!("❌ 网络扫描失败: {}", e));
                }
            }
        });
    }

    // DNS代理功能已改为自动模式，无需手动设置
}

impl eframe::App for TurboGitHubApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 检查窗口是否应该可见
        let should_be_visible = WINDOW_SHOULD_BE_VISIBLE.load(Ordering::SeqCst);
        
        // 如果窗口应该可见但当前不可见，显示它
        if should_be_visible && !self.window_visible {
            println!("🔵 显示窗口（从隐藏状态恢复）");
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            self.window_visible = true;
        }
        
        // 如果窗口应该不可见但当前可见，隐藏它
        if !should_be_visible && self.window_visible {
            println!("🔵 隐藏窗口到系统托盘");
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            self.window_visible = false;
        }
        
        // 设置窗口大小和属性（仅在第一次运行时设置）
        if !self.window_configured {
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(900.0, 650.0)));
            ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(egui::Vec2::new(900.0, 650.0)));
            ctx.send_viewport_cmd(egui::ViewportCommand::MaxInnerSize(egui::Vec2::new(900.0, 650.0)));
            ctx.send_viewport_cmd(egui::ViewportCommand::Resizable(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(false));
            self.window_configured = true;
        }
        
        // 处理关闭请求：隐藏窗口到系统托盘，程序继续在后台运行
        if ctx.input(|i| i.viewport().close_requested()) {
            println!("🔵 用户点击关闭按钮");
            // 设置窗口应该不可见
            WINDOW_SHOULD_BE_VISIBLE.store(false, Ordering::SeqCst);
            // 隐藏窗口
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            // 清除关闭请求，防止程序退出
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.window_visible = false;
        }
        
        // 自动刷新
        if self.auto_refresh && self.last_refresh.elapsed() > std::time::Duration::from_secs(1) {
            ctx.request_repaint();
            self.last_refresh = std::time::Instant::now();
        }
        
        // 底部状态栏
        egui::TopBottomPanel::bottom("bottom_panel")
            .min_height(25.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // 左侧：版本信息
                    ui.label(
                        egui::RichText::new("TurboGitHub v0.0.1")
                            .color(egui::Color32::from_gray(100))
                            .size(12.0)
                    );
                    
                    // 右侧：GitHub 链接
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.hyperlink_to(
                            egui::RichText::new("GitHub")
                                .color(egui::Color32::from_rgb(0, 100, 200))
                                .size(12.0),
                            "https://github.com/Gautown/TurboGitHub"
                        );
                    });
                });
            });
        
        // 日志显示 - 与窗体同宽
        egui::TopBottomPanel::bottom("logs_panel")
            .min_height(110.0)
            .show(ctx, |ui| {
                ui.heading("日志");
                
                let logs = self.rt.block_on(async {
                    self.logs.lock().await.clone()
                });
                
                egui::ScrollArea::vertical()
                    .max_height(100.0)
                    .stick_to_bottom(true)
                    .auto_shrink([false; 2]) // 确保滚动区域不会自动收缩
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible) // 始终显示滚动条
                    .show(ui, |ui| {
                        for log in logs.iter().rev().take(20) {
                            ui.monospace(log);
                        }
                    });
            });
        
        // 中央面板
        egui::CentralPanel::default().show(ctx, |ui| {
            // 确保面板有足够的高度
            ui.set_min_height(500.0);
            
            // 第一行：流量标题、服务状态和IP数量在同一行
            ui.horizontal(|ui| {
                ui.heading("流量");
                
                // 在流量标题右侧显示服务状态和IP信息
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // 获取服务状态数据
                    let status = self.rt.block_on(async {
                        self.status.lock().await.clone()
                    });
                    
                    if let Some(status) = status {
                        // 服务状态指示器
                        let (color, text) = if status.running {
                            (egui::Color32::GREEN, "运行中")
                        } else {
                            (egui::Color32::RED, "已停止")
                        };
                        
                        ui.colored_label(color, "●");
                        ui.label(text);
                        ui.separator();
                        ui.label(format!("域名：{} | IP：{}", 
                            status.stats.domains_scanned, status.stats.total_ips));
                        
                        // 显示优化IP数量
                        if !status.current_ips.is_empty() {
                            ui.separator();
                            ui.label(format!("优化IP：{}", status.current_ips.len()));
                        }
                    } else {
                        ui.label("正在连接守护进程...");
                    }
                });
            });
            
            ui.separator();
            
            // 流量图表区域
            ui.vertical(|ui| {
                // 显示流量统计（使用后台任务缓存的数据）
                let (total_upload, total_download) = self.rt.block_on(async {
                    let monitor = self.traffic_monitor.lock().await;
                    monitor.get_total_traffic()
                });
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(0, 225, 160), "●");
                    ui.label(format!("上传：{:.2} MB", total_upload as f32 / 1024.0 / 1024.0));
                    ui.separator();
                    ui.colored_label(egui::Color32::from_rgb(25, 103, 210), "●");
                    ui.label(format!("下载：{:.2} MB", total_download as f32 / 1024.0 / 1024.0));
                    ui.separator();
                    ui.colored_label(egui::Color32::from_rgb(255, 165, 0), "●");
                    ui.label("实时流量监控");
                });
                
                // 为图表分配固定高度的区域
                ui.add_space(10.0);
                
                // 绘制流量图表
                let data_points = self.rt.block_on(async {
                    let monitor = self.traffic_monitor.lock().await;
                    monitor.get_all_data()
                });
                
                // 计算当前上传和下载速度 (KB/s)
                let current_upload_speed = if data_points.len() >= 2 {
                    let last_point = &data_points[data_points.len() - 1];
                    let prev_point = &data_points[data_points.len() - 2];
                    let time_diff = last_point.timestamp.saturating_sub(prev_point.timestamp).max(1) as f32;
                    last_point.upload_bytes.saturating_sub(prev_point.upload_bytes) as f32 / time_diff / 1024.0
                } else {
                    0.0
                };
                
                let current_download_speed = if data_points.len() >= 2 {
                    let last_point = &data_points[data_points.len() - 1];
                    let prev_point = &data_points[data_points.len() - 2];
                    let time_diff = last_point.timestamp.saturating_sub(prev_point.timestamp).max(1) as f32;
                    last_point.download_bytes.saturating_sub(prev_point.download_bytes) as f32 / time_diff / 1024.0
                } else {
                    0.0
                };
                
                draw_traffic_chart(ui, &data_points, ui.available_width(), 350.0, self.app_start_time, 
                    current_upload_speed, current_download_speed);
            });
            
            // 控制按钮
            ui.separator();
            ui.heading("控制面板");
            
            ui.horizontal(|ui| {
                // 启动/停止服务按钮 - 根据状态动态切换
                let toggle_button = if self.service_running {
                    // 停止服务按钮 - 红色背景
                    egui::Button::new(egui::RichText::new("● 停止服务")
                        .size(14.0)
                        .color(egui::Color32::WHITE))
                        .fill(egui::Color32::from_rgb(250, 15, 70))
                        .min_size(egui::Vec2::new(90.0, 32.0))
                } else {
                    // 启动服务按钮 - 绿色背景
                    egui::Button::new(egui::RichText::new("▶ 启动服务")
                        .size(14.0)
                        .color(egui::Color32::WHITE))
                        .fill(egui::Color32::from_rgb(0, 210, 127))
                        .min_size(egui::Vec2::new(90.0, 32.0))
                };
                
                if ui.add(toggle_button).clicked() {
                    if self.service_running {
                        self.stop_service();
                    } else {
                        self.start_service();
                    }
                }
                
                // 立即扫描按钮 - 蓝色背景
                let scan_button = egui::Button::new(egui::RichText::new("🔎 立即扫描")
                    .size(14.0)
                    .color(egui::Color32::WHITE))
                    .fill(egui::Color32::from_rgb(25, 103, 210))
                    .min_size(egui::Vec2::new(90.0, 32.0));
                
                if ui.add(scan_button).clicked() {
                    self.scan_now();
                }
                
                ui.checkbox(&mut self.auto_refresh, "自动刷新");
            });
            
            // 简化界面，移除不必要的说明信息
        });
    }
}
