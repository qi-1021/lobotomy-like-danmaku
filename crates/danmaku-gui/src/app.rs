use danmaku_core::overlay::{TextOverlay, OverlayConfig};
use danmaku_core::font::FontCache;
use danmaku_core::generator::generate_overlays;
use danmaku_core::video::VideoInfo;

pub struct TextItem {
    pub text: String,
    pub color_hex: String,
}

pub struct DanmakuApp {
    // 文字列表
    pub text_list: Vec<TextItem>,
    pub new_text: String,
    pub new_color_hex: String,

    // 生成参数
    pub density: f64,
    pub max_active: usize,
    pub size_min: f32,
    pub size_max: f32,
    pub seed: u64,
    pub angle_min: f32,
    pub angle_max: f32,
    pub type_speed: f64,
    pub post_hold: f64,
    pub auto_alpha: f32,

    // 叠加列表
    pub overlays: Vec<TextOverlay>,
    pub selected_overlay: Option<usize>,

    // 手动编辑
    pub ed_text: String,
    pub ed_size: f32,
    pub ed_angle: f32,
    pub ed_color_hex: String,
    pub ed_x: f32,
    pub ed_y: f32,
    pub ed_start: f64,
    pub ed_end: f64,
    pub ed_speed: f64,
    pub ed_alpha: f32,
    pub ed_post_hold: f64,

    // 视频
    pub video_path: String,
    pub video_info: Option<VideoInfo>,
    pub font_cache: FontCache,

    // 预览
    pub preview_time: f64,
    pub show_overlay_text: bool,
    pub playing: bool,
    pub play_start: Option<std::time::Instant>,

    // 状态
    pub status: String,
    pub processing: bool,
}

impl Default for DanmakuApp {
    fn default() -> Self {
        let mut font_cache = FontCache::new();
        let _ = font_cache.load_from_dir(std::path::Path::new("fonts_proper"));

        Self {
            text_list: Vec::new(),
            new_text: String::new(),
            new_color_hex: "#b43c3c".to_string(),

            density: 0.45,
            max_active: 4,
            size_min: 18.0,
            size_max: 32.0,
            seed: 42,
            angle_min: -14.0,
            angle_max: 14.0,
            type_speed: 0.06,
            post_hold: 1.5,
            auto_alpha: 0.7,

            overlays: Vec::new(),
            selected_overlay: None,

            ed_text: String::new(),
            ed_size: 24.0,
            ed_angle: 0.0,
            ed_color_hex: "#b43c3c".to_string(),
            ed_x: 640.0,
            ed_y: 360.0,
            ed_start: 0.0,
            ed_end: 5.0,
            ed_speed: 0.06,
            ed_alpha: 0.7,
            ed_post_hold: 1.5,

            video_path: String::new(),
            video_info: None,
            font_cache,

            preview_time: 0.0,
            show_overlay_text: true,
            playing: false,
            play_start: None,

            status: "就绪".to_string(),
            processing: false,
        }
    }
}

impl DanmakuApp {
    pub fn hex_to_rgb01(hex: &str) -> [f32; 3] {
        let hex = hex.trim_start_matches('#');
        if hex.len() >= 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(180) as f32 / 255.0;
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(50) as f32 / 255.0;
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(50) as f32 / 255.0;
            [r, g, b]
        } else {
            [0.7, 0.2, 0.2]
        }
    }

    pub fn rgb01_to_hex(c: [f32; 3]) -> String {
        let r = (c[0] * 255.0).clamp(0.0, 255.0) as u8;
        let g = (c[1] * 255.0).clamp(0.0, 255.0) as u8;
        let b = (c[2] * 255.0).clamp(0.0, 255.0) as u8;
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }

    pub fn is_dark(hex: &str) -> bool {
        let c = Self::hex_to_rgb01(hex);
        (c[0] * 0.299 + c[1] * 0.587 + c[2] * 0.114) < 0.5
    }

    pub fn color_from_hex(hex: &str) -> egui::Color32 {
        let c = Self::hex_to_rgb01(hex);
        egui::Color32::from_rgb((c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8)
    }

    pub fn import_video(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("视频", &["mp4", "mov", "avi", "mkv", "webm"])
            .pick_file()
        {
            let p = path.to_string_lossy().to_string();
            match danmaku_core::video::probe_video(&p) {
                Ok(info) => {
                    self.status = format!("已加载: {} ({}x{}, {:.0}fps, {:.1}s)",
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        info.width, info.height, info.fps, info.duration);
                    self.video_info = Some(info);
                    self.video_path = p;
                }
                Err(e) => self.status = format!("错误: {}", e),
            }
        }
    }

    pub fn auto_generate(&mut self) {
        if self.text_list.is_empty() {
            self.status = "请先添加文字".to_string();
            return;
        }
        let info = match &self.video_info {
            Some(i) => i.clone(),
            None => { self.status = "请先导入视频".to_string(); return; }
        };

        let text_specs: Vec<(String, [f32; 3])> = self.text_list.iter()
            .map(|item| (item.text.clone(), Self::hex_to_rgb01(&item.color_hex)))
            .collect();

        let config = OverlayConfig {
            density: self.density,
            max_active: self.max_active,
            seed: self.seed,
            size_min: self.size_min,
            size_max: self.size_max,
            angle_min: self.angle_min,
            angle_max: self.angle_max,
            type_speed: self.type_speed,
            post_hold: self.post_hold,
            alpha_max: self.auto_alpha,
        };

        self.overlays = generate_overlays(&text_specs, info.duration, info.width, info.height, &config);
        self.selected_overlay = None;
        self.status = format!("已生成 {} 条叠加", self.overlays.len());
    }

    pub fn manual_add(&mut self) {
        if self.ed_text.is_empty() {
            self.status = "请输入文字".to_string();
            return;
        }
        if self.ed_end <= self.ed_start {
            self.status = "结束时间必须大于开始时间".to_string();
            return;
        }
        let mut o = TextOverlay::new(&self.ed_text, self.ed_size, self.ed_x, self.ed_y);
        o.color = Self::hex_to_rgb01(&self.ed_color_hex);
        o.angle = self.ed_angle;
        o.start_time = self.ed_start;
        o.end_time = self.ed_end;
        o.alpha_max = self.ed_alpha;
        o.type_speed = self.ed_speed;
        o.post_hold = self.ed_post_hold;
        self.overlays.push(o);
        self.status = format!("已添加: {}", self.ed_text);
    }

    pub fn manual_update(&mut self) {
        let idx = match self.selected_overlay {
            Some(i) if i < self.overlays.len() => i,
            _ => { self.status = "请先选中一条叠加".to_string(); return; }
        };
        if self.ed_text.is_empty() { return; }
        let o = &mut self.overlays[idx];
        o.text = self.ed_text.clone();
        o.font_size = self.ed_size;
        o.color = Self::hex_to_rgb01(&self.ed_color_hex);
        o.x = self.ed_x;
        o.y = self.ed_y;
        o.angle = self.ed_angle;
        o.start_time = self.ed_start;
        o.end_time = self.ed_end;
        o.type_speed = self.ed_speed;
        o.alpha_max = self.ed_alpha;
        o.post_hold = self.ed_post_hold;
        self.status = format!("已更新: {}", self.ed_text);
    }

    pub fn select_overlay(&mut self, idx: usize) {
        if idx >= self.overlays.len() { return; }
        let o = &self.overlays[idx];
        self.ed_text = o.text.clone();
        self.ed_size = o.font_size;
        self.ed_angle = o.angle;
        self.ed_color_hex = Self::rgb01_to_hex(o.color);
        self.ed_x = o.x;
        self.ed_y = o.y;
        self.ed_start = o.start_time;
        self.ed_end = o.end_time;
        self.ed_speed = o.type_speed;
        self.ed_alpha = o.alpha_max;
        self.ed_post_hold = o.post_hold;
        self.selected_overlay = Some(idx);
    }

    pub fn export_json(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .save_file()
        {
            let data: Vec<serde_json::Value> = self.overlays.iter().map(|o| {
                serde_json::json!({
                    "text": o.text, "font_size": o.font_size, "color": o.color,
                    "x": o.x, "y": o.y, "angle": o.angle,
                    "start_time": o.start_time, "end_time": o.end_time,
                    "alpha_max": o.alpha_max, "type_speed": o.type_speed, "post_hold": o.post_hold,
                })
            }).collect();
            match std::fs::write(path.clone(), serde_json::to_string_pretty(&data).unwrap()) {
                Ok(()) => self.status = "项目已保存".to_string(),
                Err(e) => self.status = format!("保存失败: {}", e),
            }
        }
    }

    pub fn import_json(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .pick_file()
        {
            match std::fs::read_to_string(path) {
                Ok(data) => {
                    if let Ok(list) = serde_json::from_str::<Vec<serde_json::Value>>(&data) {
                        self.overlays.clear();
                        for v in list {
                            let mut o = TextOverlay::new(
                                v["text"].as_str().unwrap_or(""),
                                v["font_size"].as_f64().unwrap_or(24.0) as f32,
                                v["x"].as_f64().unwrap_or(640.0) as f32,
                                v["y"].as_f64().unwrap_or(360.0) as f32,
                            );
                            if let Some(c) = v["color"].as_array() {
                                o.color = [
                                    c.get(0).and_then(|v| v.as_f64()).unwrap_or(180.0) as f32 / 255.0,
                                    c.get(1).and_then(|v| v.as_f64()).unwrap_or(50.0) as f32 / 255.0,
                                    c.get(2).and_then(|v| v.as_f64()).unwrap_or(50.0) as f32 / 255.0,
                                ];
                            }
                            o.angle = v["angle"].as_f64().unwrap_or(0.0) as f32;
                            o.start_time = v["start_time"].as_f64().unwrap_or(0.0);
                            o.end_time = v["end_time"].as_f64().unwrap_or(5.0);
                            o.alpha_max = v["alpha_max"].as_f64().unwrap_or(0.7) as f32;
                            o.type_speed = v["type_speed"].as_f64().unwrap_or(0.06);
                            o.post_hold = v["post_hold"].as_f64().unwrap_or(1.5);
                            self.overlays.push(o);
                        }
                        self.status = format!("已加载 {} 条", self.overlays.len());
                    }
                }
                Err(e) => self.status = format!("加载失败: {}", e),
            }
        }
    }

    pub fn export_video(&mut self) {
        if self.video_path.is_empty() {
            self.status = "请先导入视频".to_string();
            return;
        }
        if self.overlays.is_empty() {
            self.status = "没有可导出的叠加".to_string();
            return;
        }
        let output = match rfd::FileDialog::new()
            .add_filter("MP4", &["mp4"])
            .save_file()
        {
            Some(p) => p.to_string_lossy().to_string(),
            None => return,
        };
        let input = self.video_path.clone();
        let overlays = self.overlays.clone();
        let font_dir = "fonts_proper".to_string();
        let info = self.video_info.as_ref().unwrap().clone();
        self.processing = true;
        self.status = "导出中...".to_string();

        std::thread::spawn(move || {
            let mut fc = FontCache::new();
            let _ = fc.load_from_dir(std::path::Path::new(&font_dir));
            let result = danmaku_core::video::process_video(
                &input, &output, &overlays, &fc,
                info.width, info.height, info.fps, None,
            );
            match result {
                Ok(()) => eprintln!("导出完成: {}", output),
                Err(e) => eprintln!("导出错误: {}", e),
            }
        });
    }
}
