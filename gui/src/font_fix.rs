use eframe::egui;

/// 使用 egui 推荐的方法配置中文字体
pub fn setup_chinese_fonts_fixed(ctx: &egui::Context) -> bool {
    let mut fonts = egui::FontDefinitions::default();
    
    // 首先尝试加载微软雅黑字体
    if let Some(font_data) = load_font_file("C:\\Windows\\Fonts\\msyh.ttc") {
        // 使用 egui 推荐的字体配置方法
        fonts.font_data.insert("MicrosoftYaHei".to_owned(), std::sync::Arc::new(font_data));
        
        // 设置字体优先级：中文字体优先
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "MicrosoftYaHei".to_owned());
            
        fonts
            .families
            .get_mut(&egui::FontFamily::Monospace)
            .unwrap()
            .insert(0, "MicrosoftYaHei".to_owned());
        
        ctx.set_fonts(fonts);
        println!("✅ 成功配置微软雅黑字体");
        return true;
    }
    
    // 如果微软雅黑加载失败，尝试其他字体
    let fallback_fonts = vec![
        "C:\\Windows\\Fonts\\simhei.ttf",
        "C:\\Windows\\Fonts\\simsun.ttc",
        "C:\\Windows\\Fonts\\msyhbd.ttc",
    ];
    
    for font_path in fallback_fonts {
        if let Some(font_data) = load_font_file(font_path) {
            let font_name = match font_path {
                p if p.contains("simhei") => "SimHei",
                p if p.contains("simsun") => "SimSun", 
                p if p.contains("msyhbd") => "MicrosoftYaHeiBold",
                _ => "ChineseFont",
            };
            
            fonts.font_data.insert(font_name.to_owned(), std::sync::Arc::new(font_data));
            
            fonts
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, font_name.to_owned());
                
            ctx.set_fonts(fonts);
            println!("✅ 成功配置备用字体: {}", font_name);
            return true;
        }
    }
    
    println!("❌ 无法加载任何中文字体");
    false
}

/// 加载字体文件
fn load_font_file(path: &str) -> Option<egui::FontData> {
    if std::path::Path::new(path).exists() {
        match std::fs::read(path) {
            Ok(data) => {
                println!("📖 成功读取字体文件: {}", path);
                Some(egui::FontData::from_owned(data))
            }
            Err(e) => {
                println!("❌ 读取字体文件失败 {}: {}", path, e);
                None
            }
        }
    } else {
        println!("❌ 字体文件不存在: {}", path);
        None
    }
}

/// 设置默认字体回退（如果中文字体加载失败）
pub fn setup_fallback_fonts(ctx: &egui::Context) {
    let fonts = egui::FontDefinitions::default();
    
    // 使用 egui 的默认字体配置
    // 这里可以添加一些开源字体作为回退
    
    ctx.set_fonts(fonts);
    println!("⚠️ 使用默认字体配置");
}