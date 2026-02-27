use eframe::egui;
use std::collections::HashMap;

/// 字体管理器，负责加载和管理中文字体
pub struct FontManager {
    loaded_fonts: HashMap<String, egui::FontData>,
}

impl FontManager {
    pub fn new() -> Self {
        Self {
            loaded_fonts: HashMap::new(),
        }
    }
    
    /// 加载系统字体
    pub fn load_system_fonts(&mut self) -> bool {
        let system_fonts = Self::get_system_font_paths();
        
        for (name, path) in system_fonts {
            if let Ok(font_data) = std::fs::read(&path) {
                self.loaded_fonts.insert(name.clone(), egui::FontData::from_owned(font_data));
                println!("成功加载系统字体: {} -> {}", name, path);
                return true;
            }
        }
        
        false
    }
    
    /// 获取系统字体路径
    fn get_system_font_paths() -> Vec<(String, String)> {
        let mut paths = Vec::new();
        
        // 检测操作系统并返回相应的字体路径
        // 使用英文名称作为字体注册名称，避免中文名称问题
        if cfg!(target_os = "windows") {
            paths.extend(vec![
                ("MicrosoftYaHei".to_string(), "C:\\Windows\\Fonts\\msyh.ttc".to_string()),
                ("MicrosoftYaHeiBold".to_string(), "C:\\Windows\\Fonts\\msyhbd.ttc".to_string()),
                ("SimHei".to_string(), "C:\\Windows\\Fonts\\simhei.ttf".to_string()),
                ("SimSun".to_string(), "C:\\Windows\\Fonts\\simsun.ttc".to_string()),
                ("SimKai".to_string(), "C:\\Windows\\Fonts\\simkai.ttf".to_string()),
            ]);
        } else if cfg!(target_os = "macos") {
            paths.extend(vec![
                ("PingFang".to_string(), "/System/Library/Fonts/PingFang.ttc".to_string()),
                ("STHeiti".to_string(), "/System/Library/Fonts/STHeiti Light.ttc".to_string()),
            ]);
        } else {
            // Linux 和其他 Unix 系统
            paths.extend(vec![
                ("WenQuanYi".to_string(), "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc".to_string()),
                ("NotoSansCJK".to_string(), "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc".to_string()),
            ]);
        }
        
        paths
    }
    
    /// 应用字体到 egui 上下文
    pub fn apply_fonts(&self, ctx: &egui::Context) {
        let fonts = egui::FontDefinitions::default();
        
        // 添加已加载的字体
        for (name, font_data) in &self.loaded_fonts {
            fonts.font_data.insert(name.clone(), font_data.clone());
        }
        
        // 配置字体优先级 - 确保中文字体优先
        if let Some(fonts_for_family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            // 清空默认字体列表
            fonts_for_family.clear();
            
            // 优先使用中文字体
            for font_name in self.loaded_fonts.keys() {
                fonts_for_family.push(font_name.clone());
            }
            
            // 添加默认字体作为回退
            fonts_for_family.extend(vec![
                "Hack".to_owned(),
                "Ubuntu-Light".to_owned(),
            ]);
        }
        
        // 设置字体定义
        ctx.set_fonts(fonts);
    }
    
    /// 检查是否成功加载了字体
    pub fn has_loaded_fonts(&self) -> bool {
        !self.loaded_fonts.is_empty()
    }
}

/// 简化的字体设置函数
pub fn setup_chinese_fonts(ctx: &egui::Context) -> bool {
    let mut font_manager = FontManager::new();
    
    // 尝试加载系统字体
    if font_manager.load_system_fonts() {
        font_manager.apply_fonts(ctx);
        println!("成功应用系统中文字体");
        return true;
    }
    
    // 系统字体加载失败，尝试备用方案
    setup_fallback_fonts(ctx);
    false
}

/// 备用字体方案
fn setup_fallback_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // 这里可以添加内置字体数据或网络字体
    // 暂时使用默认配置，但可以改进为更好的回退方案
    
    ctx.set_fonts(fonts);
    eprintln!("警告：使用默认字体配置，中文显示可能不正常");
}