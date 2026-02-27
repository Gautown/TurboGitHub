mod app;
mod daemon_client;
mod font_fix;
mod traffic_monitor;

use app::TurboGitHubApp;
use eframe::egui;
use std::path::Path;

/// 图标类型枚举，用于指定不同的使用场景
#[derive(Clone, Copy)]
enum IconType {
    WindowIcon,    // 窗口图标（建议32x32或64x64）
    ButtonIcon,    // 按钮图标（建议16x16或24x24）
    StatusIcon,    // 状态图标（建议16x16）
}

/// 加载图标数据（根据类型自动调整尺寸）
fn load_icon_data(icon_type: IconType) -> Result<egui::IconData, Box<dyn std::error::Error>> {
    // 尝试加载不同格式的图标，优先使用SVG格式
    let icon_paths = vec![
        "assets/icons/logo.svg",
        "../assets/icons/logo.svg",
        "assets/icons/logo.ico",
        "../assets/icons/logo.ico",
        "assets/icons/logo.png",
        "../assets/icons/logo.png",
    ];
    
    for path in icon_paths {
        println!("尝试加载图标: {}", path);
        match load_icon_from_file(path, icon_type) {
            Ok(icon_data) => {
                println!("✅ 成功加载图标: {}", path);
                return Ok(icon_data);
            }
            Err(e) => {
                println!("❌ 加载图标失败 {}: {}", path, e);
            }
        }
    }
    
    // 如果所有图标文件都加载失败，创建一个默认图标
    println!("⚠️ 无法加载任何图标文件，使用默认图标");
    create_default_icon(icon_type)
}

/// 根据图标类型获取目标尺寸
fn get_target_size(icon_type: IconType) -> u32 {
    match icon_type {
        IconType::WindowIcon => 32,  // 窗口图标使用32x32像素
        IconType::ButtonIcon => 24,  // 按钮图标使用24x24像素
        IconType::StatusIcon => 16,  // 状态图标使用16x16像素
    }
}

/// 创建默认图标（根据类型调整尺寸）
fn create_default_icon(icon_type: IconType) -> Result<egui::IconData, Box<dyn std::error::Error>> {
    let target_size = get_target_size(icon_type);
    
    // 创建一个简单的默认图标（蓝色圆形）
    let mut rgba_data = Vec::new();
    for y in 0..target_size {
        for x in 0..target_size {
            let dx = x as f32 - target_size as f32 / 2.0;
            let dy = y as f32 - target_size as f32 / 2.0;
            let distance = (dx * dx + dy * dy).sqrt();
            
            if distance < target_size as f32 / 2.0 - 2.0 {
                // 蓝色圆形
                rgba_data.extend_from_slice(&[0, 0, 255, 255]);
            } else {
                // 透明背景
                rgba_data.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }
    
    Ok(egui::IconData {
        rgba: rgba_data,
        width: target_size,
        height: target_size,
    })
}

/// 从文件加载图标（根据类型调整尺寸）
fn load_icon_from_file<P: AsRef<Path>>(path: P, icon_type: IconType) -> Result<egui::IconData, Box<dyn std::error::Error>> {
    let path = path.as_ref();
    
    // 检查文件是否存在
    if !path.exists() {
        return Err(format!("图标文件不存在: {}", path.display()).into());
    }
    
    // 根据文件扩展名选择不同的加载方式
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("ico") => load_ico_icon(path, icon_type),
        Some("png") => load_png_icon(path, icon_type),
        Some("svg") => load_svg_icon(path, icon_type),
        _ => Err(format!("不支持的图标格式: {}", path.display()).into()),
    }
}

/// 加载ICO格式图标（根据类型调整尺寸）
fn load_ico_icon(path: &Path, icon_type: IconType) -> Result<egui::IconData, Box<dyn std::error::Error>> {
    let target_size = get_target_size(icon_type);
    
    // 读取ICO文件
    let file = std::fs::File::open(path)?;
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
        
        Ok(egui::IconData {
            rgba: rgba_data.to_vec(),
            width: image.width(),
            height: image.height(),
        })
    } else {
        Err("ICO文件中没有找到合适的图标".into())
    }
}

/// 加载PNG格式图标（根据类型调整尺寸）
fn load_png_icon(path: &Path, icon_type: IconType) -> Result<egui::IconData, Box<dyn std::error::Error>> {
    let target_size = get_target_size(icon_type);
    
    // 读取PNG文件
    let decoder = png::Decoder::new(std::fs::File::open(path)?);
    let mut reader = decoder.read_info()?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf)?;
    
    // 将图像数据转换为RGBA格式
    let rgba_data = match info.color_type {
        png::ColorType::Rgb => {
            buf.chunks(3)
                .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
                .collect()
        }
        png::ColorType::Rgba => buf,
        png::ColorType::Grayscale => {
            buf.iter()
                .flat_map(|gray| [*gray, *gray, *gray, 255])
                .collect()
        }
        png::ColorType::GrayscaleAlpha => {
            buf.chunks(2)
                .flat_map(|ga| [ga[0], ga[0], ga[0], ga[1]])
                .collect()
        }
        png::ColorType::Indexed => {
            // 对于索引颜色，需要调色板信息
            // 这里简化处理，假设为灰度
            buf.iter()
                .flat_map(|index| [*index, *index, *index, 255])
                .collect()
        }
    };
    
    Ok(egui::IconData {
        rgba: rgba_data,
        width: info.width,
        height: info.height,
    })
}

/// 加载SVG格式图标（根据类型调整尺寸）
fn load_svg_icon(path: &Path, icon_type: IconType) -> Result<egui::IconData, Box<dyn std::error::Error>> {
    println!("🔍 开始加载SVG图标: {}", path.display());
    
    // 检查文件是否存在
    if !path.exists() {
        return Err(format!("SVG文件不存在: {}", path.display()).into());
    }
    
    // 读取SVG文件内容
    let svg_content = std::fs::read_to_string(path)?;
    println!("✅ SVG文件内容长度: {} 字节", svg_content.len());
    
    // 根据图标类型获取目标尺寸
    let target_size = get_target_size(icon_type);
    println!("🎯 目标尺寸: {}x{} 像素", target_size, target_size);
    
    // 尝试使用resvg/usvg库渲染SVG
    match render_svg_with_resvg(&svg_content, target_size) {
        Ok(icon_data) => {
            println!("✅ resvg渲染成功");
            return Ok(icon_data);
        }
        Err(e) => {
            println!("❌ resvg渲染失败: {}", e);
            println!("⚠️ 使用备用方案：直接生成简单图标");
        }
    }
    
    // 备用方案：直接生成简单的GitHub风格图标
    create_simple_github_icon(target_size)
}

/// 使用resvg/usvg库渲染SVG
fn render_svg_with_resvg(svg_content: &str, target_size: u32) -> Result<egui::IconData, Box<dyn std::error::Error>> {
    // 预处理SVG内容，确保颜色正确设置
    let processed_svg = preprocess_svg_content(svg_content);
    
    // 解析SVG文件
    let opt = usvg::Options::default();
    let rtree = usvg::Tree::from_str(&processed_svg, &opt)?;
    println!("✅ SVG解析成功，原始尺寸: {}x{}", rtree.size.width(), rtree.size.height());
    
    // 创建渲染目标
    let mut pixmap = resvg::tiny_skia::Pixmap::new(target_size, target_size)
        .ok_or("无法创建Pixmap")?;
    
    // 填充白色背景，确保图标可见
    pixmap.fill(resvg::tiny_skia::Color::WHITE);
    
    // 计算缩放比例以适应目标尺寸，保持居中
    let size = rtree.size;
    let scale = (target_size as f32 / size.width()).min(target_size as f32 / size.height());
    let dx = (target_size as f32 - size.width() * scale) / 2.0;
    let dy = (target_size as f32 - size.height() * scale) / 2.0;
    let transform = resvg::tiny_skia::Transform::from_scale(scale, scale)
        .post_translate(dx, dy);
    
    println!("📐 缩放比例: {:.2}, 偏移量: ({:.1}, {:.1})", scale, dx, dy);
    
    // 渲染SVG到Pixmap
    resvg::render(&rtree, transform, &mut pixmap.as_mut());
    
    // 将Pixmap转换为RGBA数据（确保颜色通道顺序正确）
    let rgba_data = pixmap.data()
        .chunks(4)
        .flat_map(|chunk| [chunk[2], chunk[1], chunk[0], chunk[3]]) // BGRA转RGBA
        .collect::<Vec<u8>>();
    
    println!("🎨 RGBA数据长度: {} 字节", rgba_data.len());
    
    // 分析渲染结果
    let total_pixels = rgba_data.len() / 4;
    let transparent_pixels = rgba_data.chunks(4)
        .filter(|chunk| chunk[3] == 0)
        .count();
    let colored_pixels = total_pixels - transparent_pixels;
    
    println!("📊 像素统计: 总像素={}, 透明像素={}, 有色像素={}", 
        total_pixels, transparent_pixels, colored_pixels);
    
    if colored_pixels == 0 {
        return Err("SVG渲染结果全部为透明像素".into());
    }
    
    Ok(egui::IconData {
        rgba: rgba_data,
        width: target_size,
        height: target_size,
    })
}

/// 预处理SVG内容，确保颜色正确设置
fn preprocess_svg_content(svg_content: &str) -> String {
    // 移除可能的问题属性
    let mut processed = svg_content
        .replace("clip-rule=\"evenodd\"", "")
        .replace("xml:space=\"preserve\"", "")
        .replace("version=\"1.1\"", "");
    
    // 检查是否已经有fill属性
    let has_fill_black = processed.contains("fill=\"black\"");
    let has_fill_white = processed.contains("fill=\"white\"");
    let has_class_fil0 = processed.contains("class=\"fil0\"");
    let has_class_fil1 = processed.contains("class=\"fil1\"");
    
    println!("🔍 SVG预处理检查: fill-black={}, fill-white={}, class-fil0={}, class-fil1={}", 
        has_fill_black, has_fill_white, has_class_fil0, has_class_fil1);
    
    // 如果使用CSS类，确保样式定义存在
    if has_class_fil0 || has_class_fil1 {
        if !processed.contains("<style>") {
            if let Some(pos) = processed.find("<svg") {
                if let Some(end_pos) = processed[pos..].find('>') {
                    let style_insert = "<style type=\"text/css\"><![CDATA[ .fil0 {fill:#000000;fill-opacity:1.0} .fil1 {fill:#FFFFFF;fill-opacity:1.0} ]]></style>";
                    let insert_pos = pos + end_pos + 1;
                    processed.insert_str(insert_pos, style_insert);
                    println!("✅ 添加CSS样式定义");
                }
            }
        }
        // 不直接替换class属性，让CSS样式生效
    } else {
        // 如果没有使用CSS类，直接设置颜色
        if has_fill_black {
            processed = processed.replace("fill=\"black\"", "fill=\"#000000\" fill-opacity=\"1.0\"");
        }
        if has_fill_white {
            processed = processed.replace("fill=\"white\"", "fill=\"#FFFFFF\" fill-opacity=\"1.0\"");
        }
    }
    
    // 如果预处理后内容为空或有问题，返回原始内容
    if processed.trim().is_empty() {
        println!("⚠️ 预处理后内容为空，使用原始SVG内容");
        return svg_content.to_string();
    }
    
    println!("✅ SVG预处理完成，内容长度: {} 字节", processed.len());
    processed
}

/// 创建简单的GitHub风格图标（备用方案）
fn create_simple_github_icon(target_size: u32) -> Result<egui::IconData, Box<dyn std::error::Error>> {
    let mut rgba_data = Vec::new();
    
    for y in 0..target_size {
        for x in 0..target_size {
            // 计算像素到中心的距离
            let dx = x as f32 - target_size as f32 / 2.0;
            let dy = y as f32 - target_size as f32 / 2.0;
            let distance = (dx * dx + dy * dy).sqrt();
            
            if distance < target_size as f32 / 2.0 - 2.0 {
                // 内部区域：GitHub猫头鹰标志（白色）
                rgba_data.extend_from_slice(&[255, 255, 255, 255]);
            } else {
                // 外部区域：黑色背景
                rgba_data.extend_from_slice(&[0, 0, 0, 255]);
            }
        }
    }
    
    println!("🔄 备用方案：直接生成RGBA数据长度: {} 字节", rgba_data.len());
    println!("📊 像素统计: 总像素={}, 有色像素={}", 
        rgba_data.len() / 4, rgba_data.len() / 4);
    
    Ok(egui::IconData {
        rgba: rgba_data,
        width: target_size,
        height: target_size,
    })
}

fn main() -> Result<(), eframe::Error> {
    // 尝试加载应用程序图标
    let icon_data = match load_icon_data(IconType::WindowIcon) {
        Ok(data) => {
            println!("✅ 成功加载应用程序图标");
            Some(data)
        }
        Err(e) => {
            println!("⚠️ 无法加载图标: {}", e);
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