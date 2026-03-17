use crossbeam_channel::{Receiver, Sender};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

/// 托盘事件枚举
#[derive(Debug, Clone)]
pub enum TrayEvent {
    ShowWindow,
    Quit,
}

/// 系统托盘管理器
pub struct SystemTrayManager {
    event_tx: Sender<TrayEvent>,
    event_rx: Receiver<TrayEvent>,
    should_exit: Arc<AtomicBool>,
}

impl SystemTrayManager {
    /// 创建新的系统托盘管理器
    pub fn new() -> Self {
        let (event_tx, event_rx) = crossbeam_channel::unbounded();
        
        Self {
            event_tx,
            event_rx,
            should_exit: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 获取事件接收器
    pub fn event_receiver(&self) -> Receiver<TrayEvent> {
        self.event_rx.clone()
    }

    /// 获取退出标志
    pub fn get_exit_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.should_exit)
    }

    /// 初始化系统托盘图标
    pub fn init(&mut self) -> Result<(), String> {
        println!("🔔 初始化系统托盘...");

        // 在单独的线程中启动系统托盘
        let event_tx = self.event_tx.clone();
        let should_exit = Arc::clone(&self.should_exit);
        
        thread::spawn(move || {
            if let Err(e) = Self::run_tray_app(event_tx, should_exit) {
                println!("❌ 系统托盘线程错误：{}", e);
            }
        });

        println!("✅ 系统托盘初始化成功（异步模式）");
        Ok(())
    }

    /// 在单独线程中运行系统托盘应用程序
    fn run_tray_app(event_tx: Sender<TrayEvent>, should_exit: Arc<AtomicBool>) -> Result<(), String> {
        // 延迟启动，确保 GUI 完全初始化
        thread::sleep(std::time::Duration::from_millis(1000));
        
        println!("🔔 启动系统托盘线程...");

        // 使用 systray 库创建托盘应用程序
        let mut app = systray::Application::new()
            .map_err(|e| format!("创建应用程序失败：{}", e))?;

        // 设置托盘图标 - 使用绝对路径
        let icon_path = std::env::current_dir()
            .unwrap_or_default()
            .join("assets/icons/tray_icon.ico");
        
        println!("🔍 设置托盘图标：{}", icon_path.display());

        if icon_path.exists() {
            app.set_icon_from_file(icon_path.to_str().unwrap())
                .map_err(|e| format!("设置图标失败：{}", e))?;
            println!("✅ 托盘图标设置成功");
        } else {
            println!("⚠️ 图标文件不存在：{}，尝试使用备用图标", icon_path.display());
            
            // 尝试使用 logo.ico 作为备用
            let backup_icon_path = std::env::current_dir()
                .unwrap_or_default()
                .join("assets/icons/logo.ico");
            
            if backup_icon_path.exists() {
                println!("🔍 使用备用图标：{}", backup_icon_path.display());
                app.set_icon_from_file(backup_icon_path.to_str().unwrap())
                    .map_err(|e| format!("设置备用图标失败：{}", e))?;
                println!("✅ 备用图标设置成功");
            } else {
                println!("❌ 备用图标也不存在");
            }
        }

        // 设置工具提示
        app.set_tooltip("TurboGitHub - GitHub 加速工具")
            .map_err(|e| format!("设置工具提示失败：{}", e))?;

        // 添加菜单项
        let tx_show = event_tx.clone();
        app.add_menu_item("显示窗口", move |_| {
            println!("📺 用户选择显示窗口");
            let _ = tx_show.send(TrayEvent::ShowWindow);
            Ok::<_, systray::Error>(())
        }).map_err(|e| format!("添加显示窗口菜单项失败：{}", e))?;

        // 添加分隔符
        app.add_menu_separator()
            .map_err(|e| format!("添加分隔符失败：{}", e))?;

        // 添加退出菜单项
        let tx_quit = event_tx.clone();
        let exit_flag = Arc::clone(&should_exit);
        app.add_menu_item("退出", move |_| {
            println!("👋 用户选择退出");
            exit_flag.store(true, Ordering::SeqCst);
            let _ = tx_quit.send(TrayEvent::Quit);
            Ok::<_, systray::Error>(())
        }).map_err(|e| format!("添加退出菜单项失败：{}", e))?;

        println!("✅ 系统托盘菜单已配置，等待消息...");

        // 启动托盘应用程序（这会阻塞当前线程）
        app.wait_for_message()
            .map_err(|e| format!("启动托盘应用程序失败：{}", e))?;

        println!("🔚 系统托盘线程已退出");
        Ok(())
    }

    /// 启动事件监听器（返回事件接收器）
    #[allow(dead_code)]
    pub fn start_event_listener(&self) -> Receiver<TrayEvent> {
        self.event_rx.clone()
    }
}

impl Drop for SystemTrayManager {
    fn drop(&mut self) {
        println!("🔚 系统托盘已关闭");
    }
}
