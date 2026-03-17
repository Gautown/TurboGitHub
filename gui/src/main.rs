#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::process::{Command, Stdio};
use std::path::Path;

mod app;
mod font_fix;
mod integrated_service;
mod ipc_client;
mod traffic_monitor;
mod tray_manager;

use app::TurboGitHubApp;
use tray_manager::{SystemTrayManager, TrayEvent};

/// 自动启动核心服务
fn auto_start_core_service() -> Option<std::process::Child> {
    println!("🔍 检查核心服务是否正在运行...");
    
    // 检查是否已有核心服务进程
    let output = Command::new("tasklist")
        .args(&["/FI", "IMAGENAME eq turbogithub-core.exe", "/NH"])
        .output()
        .ok()?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    if output_str.contains("turbogithub-core.exe") {
        println!("✅ 核心服务已在运行");
        return None;
    }
    
    println!("💡 核心服务未运行，尝试自动启动...");
    
    // 查找核心服务可执行文件
    let core_paths = [
        Path::new("../target/debug/turbogithub-core.exe"),
        Path::new("../../target/debug/turbogithub-core.exe"),
        Path::new("target/debug/turbogithub-core.exe"),
        Path::new("../turbogithub-core.exe"),
        Path::new("turbogithub-core.exe"),
    ];
    
    let mut core_path: Option<&Path> = None;
    for path in &core_paths {
        if path.exists() {
            core_path = Some(path);
            println!("🔍 找到核心服务：{}", path.display());
            break;
        }
    }
    
    match core_path {
        Some(path) => {
            println!("🚀 正在启动核心服务：{}", path.display());
            
            // 启动核心服务（隐藏窗口）
            match Command::new(path)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                Ok(child) => {
                    println!("✅ 核心服务已启动 (PID: {})", child.id());
                    Some(child)
                }
                Err(e) => {
                    eprintln!("❌ 启动核心服务失败：{}", e);
                    None
                }
            }
        }
        None => {
            eprintln!("⚠️ 未找到核心服务可执行文件");
            eprintln!("💡 请先运行：cargo build -p turbogithub-core");
            None
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    // 设置日志级别
    env_logger::init();
    
    println!("🚀 启动 TurboGitHub GUI...");
    
    // 自动启动核心服务
    let _core_process = auto_start_core_service();
    
    // 等待核心服务启动（最多等待 5 秒）
    println!("⏳ 等待核心服务启动...");
    std::thread::sleep(std::time::Duration::from_secs(3));
    
    // 创建集成服务实例
    let service = Arc::new(crate::integrated_service::IntegratedService::new("dynamic".to_string()));
    
    // 创建系统托盘管理器
    let mut tray_manager = SystemTrayManager::new();
    
    // 初始化系统托盘
    if let Err(e) = tray_manager.init() {
        eprintln!("❌ 系统托盘初始化失败: {}", e);
        println!("💡 继续运行，但没有系统托盘功能");
    } else {
        println!("✅ 系统托盘已启动");
    }
    
    // 获取事件接收器和退出标志
    let tray_event_rx = tray_manager.event_receiver();
    let should_exit = tray_manager.get_exit_flag();
    
    // 启动事件处理线程
    let ctx_clone = std::sync::Arc::new(std::sync::Mutex::new(None::<egui::Context>));
    let ctx_clone_for_thread = ctx_clone.clone();
    let should_exit_for_thread = Arc::clone(&should_exit);
    
    std::thread::spawn(move || {
        println!("🎧 托盘事件监听线程已启动");
        
        while !should_exit_for_thread.load(Ordering::SeqCst) {
            match tray_event_rx.recv_timeout(std::time::Duration::from_millis(100)) {
                Ok(event) => {
                    match event {
                        TrayEvent::ShowWindow => {
                            println!("📺 收到显示窗口请求");
                            // 更新全局窗口可见性标志
                            app::WINDOW_SHOULD_BE_VISIBLE.store(true, Ordering::SeqCst);
                            // 发送显示窗口命令
                            if let Some(ctx) = ctx_clone_for_thread.lock().unwrap().as_ref() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                                ctx.request_repaint();
                                println!("✅ 已发送显示窗口命令");
                            } else {
                                println!("❌ 上下文尚未初始化");
                            }
                        }
                        TrayEvent::Quit => {
                            println!("👋 收到退出请求");
                            should_exit_for_thread.store(true, Ordering::SeqCst);
                            std::process::exit(0);
                        }
                    }
                }
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                    // 超时，继续检查
                    continue;
                }
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                    println!("🔌 托盘事件通道已断开");
                    break;
                }
            }
        }
        println!("🔚 托盘事件监听线程已退出");
    });
    
    // 启动状态监控线程（更新托盘状态）
    let service_clone = Arc::clone(&service);
    let exit_flag_clone = Arc::clone(&should_exit);
    std::thread::spawn(move || {
        println!("🔔 系统托盘状态监控线程已启动");
        
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        
        loop {
            // 获取真实服务状态（用于日志记录）
            let _real_status = rt.block_on(async {
                service_clone.get_status().await
            });
            
            // 检查退出标志
            if exit_flag_clone.load(std::sync::atomic::Ordering::SeqCst) {
                println!("👋 收到退出信号，关闭应用程序");
                std::process::exit(0);
            }
            
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    });
    
    // 加载应用程序图标
    let icon_data = match load_app_icon() {
        Ok(data) => {
            println!("✅ 成功加载应用程序图标");
            Some(data)
        }
        Err(e) => {
            println!("⚠️ 图标加载失败：{}", e);
            None
        }
    };
    
    let viewport_builder = egui::ViewportBuilder::default()
        .with_inner_size(egui::vec2(900.0, 650.0))
        .with_min_inner_size(egui::vec2(900.0, 650.0))
        .with_max_inner_size(egui::vec2(900.0, 650.0))
        .with_position(egui::pos2(50.0, 50.0))
        .with_title("TurboGitHub - GitHub 加速工具")
        .with_resizable(false)
        .with_decorations(true)
        .with_maximized(false)
        .with_fullscreen(false)
        .with_close_button(true)
        .with_minimize_button(true)
        .with_maximize_button(false);
    
    // 如果成功加载图标，设置应用程序图标
    let viewport_builder = if let Some(icon_data) = icon_data {
        viewport_builder.with_icon(icon_data)
    } else {
        viewport_builder
    };
    
    let options = eframe::NativeOptions {
        viewport: viewport_builder,
        ..Default::default()
    };
    
    // 启动 eframe 应用
    eframe::run_native(
        "TurboGitHub",
        options,
        Box::new(move |cc| {
            println!("🎯 初始化 TurboGitHub 应用");
            
            // 存储上下文引用
            *ctx_clone.lock().unwrap() = Some(cc.egui_ctx.clone());
            
            // 安装图像加载器
            egui_extras::install_image_loaders(&cc.egui_ctx);
            
            // 配置中文字体 - 使用修复后的方法
            let font_loaded = font_fix::setup_chinese_fonts_fixed(&cc.egui_ctx);
            
            if !font_loaded {
                println!("⚠️ 中文字体加载失败，使用默认字体配置");
                font_fix::setup_fallback_fonts(&cc.egui_ctx);
            }
            
            // 创建应用实例
            Ok(Box::new(TurboGitHubApp::new_with_service(cc, Arc::clone(&service))))
        }),
    )
}

/// 加载应用程序图标
fn load_app_icon() -> Result<egui::IconData, Box<dyn std::error::Error>> {
    println!("🔍 尝试加载应用程序图标...");
    
    // 尝试加载 ICO 文件
    if let Ok(icon_data) = load_ico_icon("assets/icons/logo.ico") {
        println!("✅ 成功加载 ICO 图标");
        return Ok(icon_data);
    }
    
    // 尝试加载 PNG 文件
    if let Ok(icon_data) = load_png_icon("assets/icons/256x256.png") {
        println!("✅ 成功加载 PNG 图标");
        return Ok(icon_data);
    }
    
    // 尝试加载 32x32 PNG 文件
    if let Ok(icon_data) = load_png_icon("assets/icons/32x32.png") {
        println!("✅ 成功加载 32x32 PNG 图标");
        return Ok(icon_data);
    }
    
    println!("⚠️ 无法加载图标文件，使用默认图标");
    create_default_icon()
}

/// 加载 ICO 图标
fn load_ico_icon(path: &str) -> Result<egui::IconData, Box<dyn std::error::Error>> {
    println!("🔍 开始加载 ICO 图标: {}", path);
    
    let file = std::fs::File::open(path)?;
    let icon_dir = ico::IconDir::read(file)?;
    
    // 选择最佳尺寸的图标（256x256 优先）
    let best_entry = icon_dir
        .entries()
        .iter()
        .max_by_key(|entry| entry.width())
        .ok_or("没有找到有效的图标条目")?;
    
    let image = best_entry.decode()?;
    let rgba = image.rgba_data();
    
    println!("🎯 目标尺寸: {}x{} 像素", image.width(), image.height());
    
    Ok(egui::IconData {
        rgba: rgba.to_vec(),
        width: image.width(),
        height: image.height(),
    })
}

/// 加载 PNG 图标
fn load_png_icon(path: &str) -> Result<egui::IconData, Box<dyn std::error::Error>> {
    println!("🔍 开始加载 PNG 图标：{}", path);
    
    let img = image::open(path)?;
    let rgba = img.to_rgba8();
    
    println!("🎯 目标尺寸：{}x{} 像素", rgba.width(), rgba.height());
    
    Ok(egui::IconData {
        rgba: rgba.to_vec(),
        width: rgba.width(),
        height: rgba.height(),
    })
}

/// 创建默认图标
fn create_default_icon() -> Result<egui::IconData, Box<dyn std::error::Error>> {
    let size = 256;
    let mut rgba = vec![0u8; (size * size * 4) as usize];

    let center = size as i32 / 2;
    let radius = size as i32 / 2 - 1;

    for y in 0..size {
        for x in 0..size {
            let dx = x as i32 - center;
            let dy = y as i32 - center;
            let dist = (dx * dx + dy * dy) as i32;

            let idx = (y * size + x) as usize * 4;
            if dist <= radius * radius {
                // 蓝色圆形
                rgba[idx] = 30;      // R
                rgba[idx + 1] = 144; // G
                rgba[idx + 2] = 255; // B
                rgba[idx + 3] = 255; // A
            } else {
                // 透明
                rgba[idx + 3] = 0;
            }
        }
    }

    println!("🔄 使用默认图标，尺寸: {}x{}", size, size);
    
    Ok(egui::IconData {
        rgba,
        width: size,
        height: size,
    })
}