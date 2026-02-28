use crate::integrated_service::{IntegratedService, ServiceStatus};
use crate::traffic_monitor::{TrafficMonitor, draw_traffic_chart};
use eframe::egui;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

pub struct TurboGitHubApp {
    rt: Runtime,
    service: Arc<IntegratedService>,
    status: Arc<Mutex<Option<ServiceStatus>>>,
    logs: Arc<Mutex<Vec<String>>>,
    traffic_monitor: TrafficMonitor,
    app_start_time: SystemTime,
    auto_refresh: bool,
    last_refresh: std::time::Instant,
    window_configured: bool,
}

impl TurboGitHubApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let service = Arc::new(IntegratedService::new());
        let status = Arc::new(Mutex::new(None));
        let logs = Arc::new(Mutex::new(Vec::new()));
        let traffic_monitor = TrafficMonitor::new(100); // 保存最近 100 个数据点
        let app_start_time = SystemTime::now(); // 记录应用启动时间
        
        // 启动后台任务
        let service_clone = Arc::clone(&service);
        let status_clone = Arc::clone(&status);
        let logs_clone = Arc::clone(&logs);
        
        rt.spawn(async move {
            Self::background_task(service_clone, status_clone, logs_clone).await;
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
        }
    }
    
    async fn background_task(
        service: Arc<IntegratedService>,
        status: Arc<Mutex<Option<ServiceStatus>>>,
        logs: Arc<Mutex<Vec<String>>>,
    ) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
        
        // 启动集成化服务
        if let Err(e) = service.start_service().await {
            let mut logs_guard = logs.lock().await;
            logs_guard.push(format!("启动服务失败：{}", e));
        } else {
            let mut logs_guard = logs.lock().await;
            logs_guard.push("集成化服务已启动".to_string());
        }
        
        loop {
            interval.tick().await;
            
            // 模拟网络扫描
            if let Err(e) = service.scan_networks().await {
                let mut logs_guard = logs.lock().await;
                logs_guard.push(format!("网络扫描失败：{}", e));
            }
            
            // 获取状态
            match service.get_status().await {
                Ok(new_status) => {
                    let mut status_guard = status.lock().await;
                    *status_guard = Some(new_status);
                }
                Err(e) => {
                    let mut logs_guard = logs.lock().await;
                    logs_guard.push(format!("获取状态失败：{}", e));
                }
            }
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
    
    fn start_service(&self) {
        let service = Arc::clone(&self.service);
        let logs = Arc::clone(&self.logs);
        self.add_log("启动服务...".to_string());
        
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
    
    fn stop_service(&self) {
        let service = Arc::clone(&self.service);
        let logs = Arc::clone(&self.logs);
        self.add_log("停止服务...".to_string());
        
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
}

impl eframe::App for TurboGitHubApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 设置窗口大小和属性（仅在第一次运行时设置）
        if !self.window_configured {
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(900.0, 650.0)));
            ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(egui::Vec2::new(900.0, 650.0)));
            ctx.send_viewport_cmd(egui::ViewportCommand::MaxInnerSize(egui::Vec2::new(900.0, 650.0)));
            ctx.send_viewport_cmd(egui::ViewportCommand::Resizable(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(false));
            self.window_configured = true;
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
                // 模拟流量数据（在实际应用中应从守护进程获取）
                let now = std::time::SystemTime::now();
                let elapsed = now.duration_since(self.app_start_time).unwrap_or_default().as_secs();
                
                // 基于应用运行时间生成变化的流量数据
                let upload_bytes = ((elapsed % 60) * 1024 * 1024) / 60;
                let download_bytes = ((elapsed % 45) * 2048 * 1024) / 45;
                
                self.traffic_monitor.add_traffic(upload_bytes, download_bytes);
                
                // 显示流量统计
                let (total_upload, total_download) = self.traffic_monitor.get_total_traffic();
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(0, 225, 160), "●");
                    ui.label(format!("上传：{:.2} MB", total_upload as f32 / 1024.0 / 1024.0));
                    ui.separator();
                    ui.colored_label(egui::Color32::from_rgb(25, 103, 210), "●");
                    ui.label(format!("下载：{:.2} MB", total_download as f32 / 1024.0 / 1024.0));
                });
                
                // 为图表分配固定高度的区域
                ui.add_space(10.0);
                
                // 绘制流量图表
                let data_points = self.traffic_monitor.get_all_data();
                
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
                // 启动服务按钮 - 绿色背景
                let start_button = egui::Button::new(egui::RichText::new("▶ 启动服务")
                    .size(14.0)
                    .color(egui::Color32::WHITE))
                    .fill(egui::Color32::from_rgb(0, 210, 127))
                    .min_size(egui::Vec2::new(90.0, 32.0));
                
                if ui.add(start_button).clicked() {
                    self.start_service();
                }
                
                // 停止服务按钮 - 红色背景
                let stop_button = egui::Button::new(egui::RichText::new("● 停止服务")
                    .size(14.0)
                    .color(egui::Color32::WHITE))
                    .fill(egui::Color32::from_rgb(250, 15, 70))
                    .min_size(egui::Vec2::new(90.0, 32.0));
                
                if ui.add(stop_button).clicked() {
                    self.stop_service();
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
        });
    }
}
