use chrono::Local;
use eframe::egui;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 流量数据点
#[derive(Debug, Clone)]
pub struct TrafficDataPoint {
    pub upload_bytes: u64,      // 增量上传流量（字节）
    pub download_bytes: u64,    // 增量下载流量（字节）
    pub timestamp: u64,         // 真实时间戳（秒）
    pub upload_speed: f64,      // 上传速度（字节/秒）
    pub download_speed: f64,    // 下载速度（字节/秒）
}

/// 流量监控器
#[derive(Clone)]
pub struct TrafficMonitor {
    data_points: Arc<Mutex<VecDeque<TrafficDataPoint>>>,
    max_points: usize,
    last_update: Instant,
    last_upload_bytes: u64,     // 上次记录的累积上传流量
    last_download_bytes: u64,   // 上次记录的累积下载流量
}

impl TrafficMonitor {
    pub fn new(max_points: usize) -> Self {
        Self {
            data_points: Arc::new(Mutex::new(VecDeque::with_capacity(max_points))),
            max_points,
            last_update: Instant::now(),
            last_upload_bytes: 0,
            last_download_bytes: 0,
        }
    }
    
    /// 添加流量数据点（接收累积流量，自动计算增量）
    pub fn add_traffic(&mut self, total_upload_bytes: u64, total_download_bytes: u64) {
        let now = Instant::now();
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // 限制更新频率，避免数据点过于密集（每秒 1 个数据点）
        if now.duration_since(self.last_update) < Duration::from_secs(1) {
            return;
        }
        
        // 计算增量流量
        let delta_upload = total_upload_bytes.saturating_sub(self.last_upload_bytes);
        let delta_download = total_download_bytes.saturating_sub(self.last_download_bytes);
        
        // 计算时间间隔（秒）
        let time_interval = now.duration_since(self.last_update).as_secs_f64().max(1.0);
        
        // 计算速度（字节/秒）
        let upload_speed = delta_upload as f64 / time_interval;
        let download_speed = delta_download as f64 / time_interval;
        
        let data_point = TrafficDataPoint {
            upload_bytes: delta_upload,
            download_bytes: delta_download,
            timestamp: current_time,
            upload_speed,
            download_speed,
        };
        
        let mut points = self.data_points.lock().unwrap();
        
        // 保持数据点数量不超过最大值
        if points.len() >= self.max_points {
            points.pop_front();
        }
        
        points.push_back(data_point);
        
        // 更新上次的累积流量值
        self.last_upload_bytes = total_upload_bytes;
        self.last_download_bytes = total_download_bytes;
        self.last_update = now;
    }
    
    /// 直接添加增量数据点（用于从核心进程获取的增量数据）
    pub fn add_delta_traffic(&mut self, upload_bytes: u64, download_bytes: u64) {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // 计算速度（假设数据点是 1 秒间隔）
        let upload_speed = upload_bytes as f64;
        let download_speed = download_bytes as f64;
        
        let data_point = TrafficDataPoint {
            upload_bytes,
            download_bytes,
            timestamp: current_time,
            upload_speed,
            download_speed,
        };
        
        let mut points = self.data_points.lock().unwrap();
        
        // 保持数据点数量不超过最大值
        if points.len() >= self.max_points {
            points.pop_front();
        }
        
        points.push_back(data_point);
    }
    
    // 获取最近的数据点（暂时未使用）
    // pub fn get_recent_data(&self, duration: Duration) -> Vec<TrafficDataPoint> {
    //     let points = self.data_points.lock().unwrap();
    //     let cutoff_time = Instant::now() - duration;
    //     
    //     points
    //         .iter()
    //         .filter(|point| point.timestamp >= cutoff_time)
    //         .cloned()
    //         .collect()
    // }
    
    /// 获取所有数据点（用于图表显示）
    pub fn get_all_data(&self) -> Vec<TrafficDataPoint> {
        let points = self.data_points.lock().unwrap();
        points.iter().cloned().collect()
    }
    
    // 清空数据（暂时未使用）
    // pub fn clear(&mut self) {
    //     let mut points = self.data_points.lock().unwrap();
    //     points.clear();
    // }
    
    /// 获取当前总流量统计
    pub fn get_total_traffic(&self) -> (u64, u64) {
        let points = self.data_points.lock().unwrap();
        let total_upload: u64 = points.iter().map(|p| p.upload_bytes).sum();
        let total_download: u64 = points.iter().map(|p| p.download_bytes).sum();
        (total_upload, total_download)
    }
}

/// 优化的流量图表绘制函数，包含 Y 轴速度显示和 X 轴时间格式
pub fn draw_traffic_chart(
    ui: &mut egui::Ui,
    data_points: &[TrafficDataPoint],
    width: f32,
    height: f32,
    _app_start_time: std::time::SystemTime,
    _current_upload_speed: f32, // 当前上传速度 (MB/s)
    _current_download_speed: f32, // 当前下载速度 (MB/s)
) {
    // 创建绘图区域（为坐标轴标签留出足够空间）
    let y_axis_width = 80.0; // Y 轴标签宽度
    let x_axis_height = 40.0; // X 轴标签高度
    let chart_width = width - y_axis_width;
    let chart_height = height - x_axis_height;
    
    // 分配图表区域（从当前位置开始）
    let (rect, _response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    
    // 图表区域在分配的空间内
    let chart_rect = egui::Rect::from_min_size(
        egui::pos2(rect.left() + y_axis_width, rect.top()),
        egui::vec2(chart_width, chart_height)
    );
    
    // 绘制背景 - 白色背景
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, egui::Color32::WHITE);
    
    // 绘制 Y 轴速度标签（左侧，增加间距避免重叠）
    // 从 5 行增加到 8 行，使刻度更精细
    for i in 0..8 {
        let y = chart_rect.bottom() - (chart_height / 7.0) * i as f32;
        let speed_value = 0.0; // 没有数据时显示 0
        
        painter.text(
            egui::pos2(rect.left() + 10.0, y),
            egui::Align2::LEFT_CENTER,
            &format!("{:.1} KB/s", speed_value),
            egui::FontId::new(10.0, egui::FontFamily::Proportional),
            egui::Color32::from_gray(100),
        );
    }
    
    // 绘制 X 轴时间标签（底部，使用当前时间）
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    for i in 0..10 {
        let x = chart_rect.left() + (chart_width / 10.0) * i as f32;
        
        // 计算时间戳（从当前时间往前推）
        let time_offset = (9 - i) as u64 * 6; // 每格 6 秒
        let timestamp = current_time.saturating_sub(time_offset);
        
        // 将秒数转换为本地时间
        let utc_datetime = chrono::DateTime::from_timestamp(timestamp as i64, 0)
            .unwrap_or_else(|| chrono::Utc::now());
        let local_datetime = utc_datetime.with_timezone(&Local);
        
        // 格式化时间为 HH:MM:SS
        let time_str = local_datetime.format("%H:%M:%S").to_string();
        
        painter.text(
            egui::pos2(x, chart_rect.bottom() + 20.0),
            egui::Align2::CENTER_TOP,
            &time_str,
            egui::FontId::new(10.0, egui::FontFamily::Proportional),
            egui::Color32::from_gray(100),
        );
    }
    
    // 绘制网格线 - 浅灰色（与 Y 轴刻度线对齐，8 行）
    for i in 0..8 {
        let y = chart_rect.top() + (chart_height / 7.0) * i as f32;
        painter.line_segment(
            [egui::pos2(chart_rect.left(), y), egui::pos2(chart_rect.right(), y)],
            egui::Stroke::new(1.0, egui::Color32::from_gray(220)),
        );
    }
    
    // 绘制时间轴网格
    for i in 0..10 {
        let x = chart_rect.left() + (chart_width / 10.0) * i as f32;
        painter.line_segment(
            [egui::pos2(x, chart_rect.top()), egui::pos2(x, chart_rect.bottom())],
            egui::Stroke::new(1.0, egui::Color32::from_gray(220)),
        );
    }
    
    // 如果没有数据点，显示提示文字并返回
    if data_points.len() < 2 {
        painter.text(
            egui::pos2(chart_rect.center().x, chart_rect.center().y),
            egui::Align2::CENTER_CENTER,
            "等待流量数据...",
            egui::FontId::new(14.0, egui::FontFamily::Proportional),
            egui::Color32::from_gray(150),
        );
        return;
    }
    
    // 使用数据点中的速度值（字节/秒），转换为 KB/s 显示
    let max_upload_speed = data_points.iter().map(|p| p.upload_speed).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(1.0) / 1024.0;
    let max_download_speed = data_points.iter().map(|p| p.download_speed).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(1.0) / 1024.0;
    let max_value = max_upload_speed.max(max_download_speed).max(1.0) as f32; // 确保至少为 1KB/s，转换为 f32
    
    // 自定义颜色定义
    let upload_color = egui::Color32::from_rgb(0, 225, 160);   // RGB(0, 225, 160)
    let download_color = egui::Color32::from_rgb(25, 103, 210); // RGB(25, 103, 210)
    
    // 绘制上传流量曲线（自定义绿色）- 使用速度值
    if data_points.len() >= 2 {
        let points: Vec<egui::Pos2> = data_points
            .iter()
            .enumerate()
            .map(|(i, point)| {
                let upload_kbps = (point.upload_speed / 1024.0) as f32; // 转换为 KB/s 并转为 f32
                let x = chart_rect.left() + (chart_width * i as f32 / (data_points.len() - 1) as f32);
                let y = chart_rect.bottom() - (upload_kbps / max_value) * chart_height;
                egui::pos2(x, y)
            })
            .collect();
        
        painter.add(egui::Shape::line(
            points,
            egui::Stroke::new(2.0, upload_color),
        ));
    }
    
    // 绘制下载流量曲线（自定义蓝色）- 使用速度值
    if data_points.len() >= 2 {
        let points: Vec<egui::Pos2> = data_points
            .iter()
            .enumerate()
            .map(|(i, point)| {
                let download_kbps = (point.download_speed / 1024.0) as f32; // 转换为 KB/s 并转为 f32
                let x = chart_rect.left() + (chart_width * i as f32 / (data_points.len() - 1) as f32);
                let y = chart_rect.bottom() - (download_kbps / max_value) * chart_height;
                egui::pos2(x, y)
            })
            .collect();
        
        painter.add(egui::Shape::line(
            points,
            egui::Stroke::new(2.0, download_color),
        ));
    }
    
    // 绘制图例（调整位置避免重叠）
    let legend_rect = chart_rect.translate(egui::vec2(10.0, 10.0));
    
    // 检查图例是否会与Y轴标签重叠，如果会则调整位置
    let legend_x = if chart_width < 300.0 {
        chart_rect.right() - 130.0 // 如果图表较窄，将图例放在右侧
    } else {
        legend_rect.left() // 否则放在左侧
    };
    
    let legend_final_rect = egui::Rect::from_min_size(
        egui::pos2(legend_x, legend_rect.top()),
        egui::vec2(120.0, 50.0)
    );
    
    painter.rect_filled(
        legend_final_rect,
        5.0,
        egui::Color32::from_rgba_premultiplied(255, 255, 255, 0),
    );
    
    // 计算当前速度（使用最后一个数据点的速度值）
    let (current_upload_speed_kbps, current_download_speed_kbps) = if let Some(last_point) = data_points.last() {
        (last_point.upload_speed / 1024.0, last_point.download_speed / 1024.0)
    } else {
        (0.0, 0.0)
    };
    
    painter.text(
            egui::pos2(legend_final_rect.left() + 20.0, legend_final_rect.top() + 20.0),
            egui::Align2::LEFT_CENTER,
            &format!("上传 {:.2} KB/s", current_upload_speed_kbps),
            egui::FontId::default(),
            upload_color,
        );
        
        painter.text(
            egui::pos2(legend_final_rect.left() + 20.0, legend_final_rect.top() + 35.0),
            egui::Align2::LEFT_CENTER,
            &format!("下载 {:.2} KB/s", current_download_speed_kbps),
            egui::FontId::default(),
            download_color,
        );
    
    // 绘制图例颜色标记
    painter.line_segment(
        [
            egui::pos2(legend_final_rect.left() + 5.0, legend_final_rect.top() + 20.0),
            egui::pos2(legend_final_rect.left() + 15.0, legend_final_rect.top() + 20.0),
        ],
        egui::Stroke::new(2.0, upload_color),
    );
    
    painter.line_segment(
        [
            egui::pos2(legend_final_rect.left() + 5.0, legend_final_rect.top() + 35.0),
            egui::pos2(legend_final_rect.left() + 15.0, legend_final_rect.top() + 35.0),
        ],
        egui::Stroke::new(2.0, download_color),
    );
}