mod app;
mod panels;

use egui::FontDefinitions;

fn load_cjk_font() -> Option<Vec<u8>> {
    let mut db = fontdb::Database::new();
    
    // 加载 fonts_proper 目录
    let _ = db.load_fonts_dir("fonts_proper");
    
    // 加载系统字体目录
    let system_dirs: Vec<&str> = match std::env::consts::OS {
        "macos" => vec![
            "/System/Library/Fonts",
            "/System/Library/Fonts/Supplemental",
            "/Library/Fonts",
        ],
        "windows" => vec!["C:\\Windows\\Fonts"],
        _ => vec![
            "/usr/share/fonts",
            "/usr/local/share/fonts",
        ],
    };
    for dir in system_dirs {
        let _ = db.load_fonts_dir(dir);
    }

    // 找 CJK 字体
    let query = fontdb::Query {
        families: &[
            fontdb::Family::Name("PingFang SC"),
            fontdb::Family::Name("Heiti SC"),
            fontdb::Family::Name("STHeiti"),
            fontdb::Family::Name("Hiragino Sans GB"),
            fontdb::Family::Name("Microsoft YaHei"),
            fontdb::Family::Name("SimHei"),
            fontdb::Family::Name("Noto Sans CJK SC"),
            fontdb::Family::Name("Noto Sans SC"),
            fontdb::Family::Name("WenQuanYi Micro Hei"),
            fontdb::Family::SansSerif,
        ],
        ..Default::default()
    };

    if let Some(id) = db.query(&query) {
        let mut font_data = Vec::new();
        db.with_face_data(id, |data, _index| {
            font_data = data.to_vec();
        });
        if !font_data.is_empty() {
            return Some(font_data);
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
            if let Some(font_data) = load_cjk_font() {
                let mut fonts = FontDefinitions::default();
                fonts.font_data.insert("cjk".to_owned(), egui::FontData::from_owned(font_data));
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
