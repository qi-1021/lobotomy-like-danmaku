mod app;
mod panels;

use egui::FontDefinitions;
use std::path::Path;

fn load_cjk_font() -> Option<Vec<u8>> {
    // 1. 从 fonts_proper/ 找
    let font_dir = Path::new("fonts_proper");
    if font_dir.is_dir() {
        for entry in std::fs::read_dir(font_dir).ok()? {
            let entry = entry.ok()?;
            let path = entry.path();
            let name = path.file_name()?.to_string_lossy().to_lowercase();
            if name.contains("pingfang") || name.contains("noto") || name.contains("source")
                || name.contains("msyh") || name.contains("simhei") || name.contains("wqy")
            {
                if let Ok(data) = std::fs::read(&path) {
                    return Some(data);
                }
            }
        }
    }
    // 2. macOS 系统字体
    let candidates = [
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/Library/Fonts/Arial Unicode.ttf",
        "C:\\Windows\\Fonts\\msyh.ttc",
        "C:\\Windows\\Fonts\\simhei.ttf",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
    ];
    for p in &candidates {
        if let Ok(data) = std::fs::read(p) {
            return Some(data);
        }
    }
    None
}

fn main() {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 860.0])
            .with_title("脑叶公司 - 弹幕编辑器"),
        ..Default::default()
    };

    eframe::run_native(
        "Lobotomy Danmaku",
        options,
        Box::new(|cc| {
            // 注入 CJK 字体
            if let Some(font_data) = load_cjk_font() {
                let mut fonts = FontDefinitions::default();
                fonts.font_data.insert("cjk".to_owned(), egui::FontData::from_owned(font_data));
                // 把 CJK 字体加到所有 font family 的 fallback 列表末尾
                for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                    if let Some(list) = fonts.families.get_mut(&family) {
                        list.push("cjk".to_owned());
                    }
                }
                cc.egui_ctx.set_fonts(fonts);
            }
            Ok(Box::new(app::DanmakuApp::default()))
        }),
    ).unwrap();
}
