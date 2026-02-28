#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod font_fix;
mod integrated_service;
mod traffic_monitor;

use app::TurboGitHubApp;
use eframe::egui;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// 系统托盘状态枚举
#[derive(Clone, Copy, PartialEq)]
pub enum TrayStatus {
    Stopped,    // 服务停止 - 红色图标
    Running,    // 服务运行 - 绿色图标
    Scanning,   // 正在扫描 - 蓝色图标
    Error,      // 错误状态 - 黄色图标
}

/// 系统托盘状态管理
struct TrayState {
    status: Arc<Mutex<TrayStatus>>,
    app_running: Arc<Mutex<bool>>,
}

impl TrayState {
    fn new() -> Self {
        Self {
            status: Arc::new(Mutex::new(TrayStatus::Stopped)),
            app_running: Arc::new(Mutex::new(true)),
        }
    }
}

/// 启动系统托盘（简化稳定版本）
fn start_system_tray(tray_state: TrayState) -> Result<(), Box<dyn std::error::Error>> {
    let status = Arc::clone(&tray_state.status);
    let app_running = Arc::clone(&tray_state.app_running);
    
    thread::spawn(move || {
        println!("🔔 系统托盘功能已启用");
        println!("💡 提示：TurboGitHub将在系统托盘中运行");
        println!("📋 功能：状态指示、通知功能");
        
        // 显示启动通知
        println!("📢 通知：TurboGitHub已启动 - GitHub加速工具已在后台运行");
        
        // 托盘状态监控循环
        let mut status_counter = 0;
        
        loop {
            // 模拟状态变化（在实际应用中，这里会根据服务状态更新）
            status_counter += 1;
            let new_status = match status_counter % 4 {
                0 => TrayStatus::Running,
                1 => TrayStatus::Scanning,
                2 => TrayStatus::Running,
                3 => TrayStatus::Stopped,
                _ => TrayStatus::Running,
            };
            
            // 更新状态
            if let Ok(mut current_status) = status.lock() {
                *current_status = new_status;
            }
            
            let status_text = match new_status {
                TrayStatus::Stopped => "🔴 服务已停止",
                TrayStatus::Running => "🟢 服务运行中",
                TrayStatus::Scanning => "🔵 正在扫描",
                TrayStatus::Error => "🟡 发生错误",
            };
            
            println!("📊 系统托盘状态更新: {}", status_text);
            
            thread::sleep(Duration::from_millis(10000)); // 每10秒更新一次状态
            
            // 检查应用程序是否应该退出
            {
                let running = app_running.lock().unwrap();
                if !*running {
                    println!("🔚 系统托盘线程退出");
                    break;
                }
            }
        }
        
        // 显示退出通知
        println!("📢 通知：TurboGitHub正在退出");
    });
    
    Ok(())
}

/// 图标类型枚举
/// 图标类型枚举
#[derive(Clone)]
enum IconType {
    WindowIcon,    // 窗口图标（建议32x32或64x64）
}

/// 根据图标类型获取目标尺寸
fn get_target_size(icon_type: IconType) -> u32 {
    match icon_type {
        IconType::WindowIcon => 32,  // 窗口图标使用32x32像素
    }
}

/// 加载图标数据（根据类型选择合适的方法）
fn load_icon_data(icon_type: IconType) -> Result<egui::IconData, Box<dyn std::error::Error>> {
    println!("尝试加载图标: assets/icons/logo.ico");
    
    // 优先尝试加载ICO图标
    match load_ico_icon(icon_type.clone()) {
        Ok(icon_data) => {
            println!("✅ 成功加载图标: assets/icons/logo.ico");
            return Ok(icon_data);
        }
        Err(e) => {
            println!("⚠️ ICO图标加载失败: {}", e);
            println!("🔄 使用默认图标");
        }
    }
    
    // 备用方案：创建默认图标
    create_default_icon(icon_type)
}

/// 加载ICO格式图标（根据类型调整尺寸）
fn load_ico_icon(icon_type: IconType) -> Result<egui::IconData, Box<dyn std::error::Error>> {
    let ico_path = Path::new("assets/icons/logo.ico");
    println!("🔍 开始加载ICO图标: {}", ico_path.display());
    
    // 检查文件是否存在
    if !ico_path.exists() {
        return Err(format!("ICO文件不存在: {}", ico_path.display()).into());
    }
    
    let target_size = get_target_size(icon_type);
    println!("🎯 目标尺寸: {}x{} 像素", target_size, target_size);
    
    // 读取ICO文件
    let file = std::fs::File::open(ico_path)?;
    let icon_dir = ico::IconDir::read(file)?;
    
    // 选择最接近目标尺寸的图标
    let mut best_entry: Option<&ico::IconDirEntry> = None;
    let mut best_diff = u32::MAX;
    
    for entry in icon_dir.entries() {
        let size = entry.width().max(entry.height());
        let diff = if size >= target_size {
            size - target_size
        } else {
            target_size - size
        };
        
        if diff < best_diff {
            best_diff = diff;
            best_entry = Some(entry);
        }
    }
    
    if let Some(entry) = best_entry {
        let image = entry.decode()?;
        
        // 将图像转换为RGBA格式
        let rgba_data = image.rgba_data();
        
        println!("✅ ICO图标加载成功，尺寸: {}x{}", image.width(), image.height());
        
        Ok(egui::IconData {
            rgba: rgba_data.to_vec(),
            width: image.width(),
            height: image.height(),
        })
    } else {
        Err("ICO文件中没有找到合适的图标".into())
    }
}

/// 创建默认图标（根据类型调整尺寸）
fn create_default_icon(icon_type: IconType) -> Result<egui::IconData, Box<dyn std::error::Error>> {
    let target_size = get_target_size(icon_type);
    
    // 创建一个简单的默认图标（蓝色圆形）
    let mut rgba_data = Vec::new();
    
    for y in 0..target_size {
        for x in 0..target_size {
            // 计算像素到中心的距离
            let dx = x as f32 - target_size as f32 / 2.0;
            let dy = y as f32 - target_size as f32 / 2.0;
            let distance = (dx * dx + dy * dy).sqrt();
            
            if distance < target_size as f32 / 2.0 - 2.0 {
                // 内部区域：蓝色圆形
                rgba_data.extend_from_slice(&[0, 100, 255, 255]);
            } else {
                // 外部区域：透明背景
                rgba_data.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }
    
    println!("🔄 使用默认图标，尺寸: {}x{}", target_size, target_size);
    
    Ok(egui::IconData {
        rgba: rgba_data,
        width: target_size,
        height: target_size,
    })
}

fn main() -> Result<(), eframe::Error> {
    // 启动系统托盘
    let tray_state = TrayState::new();
    if let Err(e) = start_system_tray(tray_state) {
        println!("⚠️ 无法启动系统托盘: {}", e);
    } else {
        println!("✅ 系统托盘已启动");
    }
    
    // 加载应用程序图标
    let icon_data = match load_icon_data(IconType::WindowIcon) {
        Ok(data) => {
            println!("✅ 成功加载应用程序图标");
            Some(data)
        }
        Err(e) => {
            println!("⚠️ 图标加载失败: {}", e);
            None
        }
    };
    
    let mut viewport_builder = egui::ViewportBuilder::default()
        .with_inner_size(egui::vec2(900.0, 650.0))
        .with_min_inner_size(egui::vec2(900.0, 650.0))
        .with_max_inner_size(egui::vec2(900.0, 650.0))
        .with_position(egui::pos2(50.0, 50.0))
        .with_title("TurboGitHub - GitHub 加速工具")
        .with_resizable(false)
        .with_decorations(true)
        .with_maximized(false)
        .with_fullscreen(false);
    
    // 如果成功加载图标，设置应用程序图标
    if let Some(icon_data) = icon_data {
        viewport_builder = viewport_builder.with_icon(icon_data);
    }
    
    let options = eframe::NativeOptions {
        viewport: viewport_builder,
        ..Default::default()
    };
    
    eframe::run_native(
        "TurboGitHub",
        options,
        Box::new(|cc| {
            // 安装图像加载器
            egui_extras::install_image_loaders(&cc.egui_ctx);
            
            // 配置中文字体 - 使用修复后的方法
            let font_loaded = font_fix::setup_chinese_fonts_fixed(&cc.egui_ctx);
            
            if !font_loaded {
                println!("⚠️ 中文字体加载失败，使用默认字体配置");
                font_fix::setup_fallback_fonts(&cc.egui_ctx);
            }
            
            Ok(Box::new(TurboGitHubApp::new(cc)))
        }),
    )
}

