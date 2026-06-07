use danmaku_core::overlay::{TextOverlay, OverlayConfig};
use danmaku_core::font::FontCache;
use danmaku_core::generator::generate_overlays;
use danmaku_core::video::VideoInfo;
use std::io::Read;
use std::sync::mpsc;

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
    pub preview_time_offset: f64,
    pub show_overlay_text: bool,
    pub playing: bool,
    pub play_start: Option<std::time::Instant>,
    pub frame_texture: Option<egui::TextureHandle>,
    pub last_decoded_time: f64,
    // ffmpeg pipe
    pub decoder_pipe: Option<std::process::ChildStdout>,
    pub decoder_process: Option<std::process::Child>,
    pub pipe_frame_idx: u64,

    // 颜色弹窗
    pub show_color_picker: bool,
    pub color_picker_target: String, // "new", "ed", "list"
    pub color_picker_rgb: [f32; 3],
    pub color_picker_list_idx: Option<usize>,

    // 导出进度
    pub export_rx: Option<mpsc::Receiver<String>>,
    pub export_progress: String,

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
            preview_time_offset: 0.0,
            show_overlay_text: true,
            playing: false,
            play_start: None,
            frame_texture: None,
            last_decoded_time: -1.0,
            decoder_pipe: None,
            decoder_process: None,
            pipe_frame_idx: 0,

            show_color_picker: false,
            color_picker_target: String::new(),
            color_picker_rgb: [0.7, 0.2, 0.2],
            color_picker_list_idx: None,

            export_rx: None,
            export_progress: String::new(),

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

    pub fn open_color_picker(&mut self, target: &str) {
        self.color_picker_target = target.to_string();
        let hex = if target == "new" { &self.new_color_hex } else { &self.ed_color_hex };
        self.color_picker_rgb = Self::hex_to_rgb01(hex);
        self.show_color_picker = true;
    }

    pub fn apply_color_picker(&mut self) {
        let hex = Self::rgb01_to_hex(self.color_picker_rgb);
        match self.color_picker_target.as_str() {
            "new" => self.new_color_hex = hex,
            "ed" => self.ed_color_hex = hex,
            "list" => {
                if let Some(idx) = self.color_picker_list_idx {
                    if idx < self.text_list.len() {
                        self.text_list[idx].color_hex = hex;
                    }
                }
                self.color_picker_list_idx = None;
            }
            _ => {}
        }
        self.show_color_picker = false;
    }

    fn start_decoder_pipe(&mut self) {
        if self.video_path.is_empty() { return; }
        let info = match &self.video_info {
            Some(i) => i.clone(),
            None => return,
        };
        // Kill old decoder
        if let Some(mut proc) = self.decoder_process.take() {
            let _ = proc.kill();
        }
        let child = std::process::Command::new("ffmpeg")
            .args(["-i", &self.video_path,
                   "-f", "rawvideo", "-pix_fmt", "rgba",
                   "-s", &format!("{}x{}", info.width, info.height),
                   "-an", "-"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn().ok();
        if let Some(mut c) = child {
            self.decoder_pipe = c.stdout.take();
            self.decoder_process = Some(c);
            self.pipe_frame_idx = 0;
        }
    }

    pub fn decode_frame_from_pipe(&mut self) -> Option<Vec<u8>> {
        let info = self.video_info.as_ref()?;
        let frame_size = (info.width * info.height * 4) as usize;

        // 如果没有 pipe，启动
        if self.decoder_pipe.is_none() {
            self.start_decoder_pipe();
        }

        let pipe = self.decoder_pipe.as_mut()?;
        let mut buf = vec![0u8; frame_size];
        let mut total = 0;
        while total < frame_size {
            match pipe.read(&mut buf[total..]) {
                Ok(0) => return None, // EOF
                Ok(n) => total += n,
                Err(_) => return None,
            }
        }
        self.pipe_frame_idx += 1;
        Some(buf)
    }

    pub fn seek_to_time(&mut self, t: f64) {
        // 重启 decoder pipe 到指定时间
        if let Some(mut proc) = self.decoder_process.take() {
            let _ = proc.kill();
        }
        self.decoder_pipe = None;

        if self.video_path.is_empty() { return; }
        let info = match &self.video_info {
            Some(i) => i.clone(),
            None => return,
        };

        let child = std::process::Command::new("ffmpeg")
            .args(["-ss", &format!("{:.3}", t), "-i", &self.video_path,
                   "-f", "rawvideo", "-pix_fmt", "rgba",
                   "-s", &format!("{}x{}", info.width, info.height),
                   "-an", "-"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn().ok();
        if let Some(mut c) = child {
            self.decoder_pipe = c.stdout.take();
            self.decoder_process = Some(c);
            self.pipe_frame_idx = (t * info.fps) as u64;
        }
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
                    self.frame_texture = None;
                    self.last_decoded_time = -1.0;
                    self.decoder_pipe = None;
                    if let Some(mut proc) = self.decoder_process.take() {
                        let _ = proc.kill();
                    }
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

    pub fn export_preset(&mut self) {
        if self.text_list.is_empty() {
            self.status = "没有文字列表可导出".to_string();
            return;
        }
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .save_file()
        {
            let preset = serde_json::json!({
                "text_list": self.text_list.iter().map(|item| {
                    serde_json::json!({"text": item.text, "color": item.color_hex})
                }).collect::<Vec<_>>(),
                "params": {
                    "density": self.density,
                    "max_active": self.max_active,
                    "size_min": self.size_min,
                    "size_max": self.size_max,
                    "angle_min": self.angle_min,
                    "angle_max": self.angle_max,
                    "type_speed": self.type_speed,
                    "post_hold": self.post_hold,
                    "seed": self.seed,
                }
            });
            match std::fs::write(path.clone(), serde_json::to_string_pretty(&preset).unwrap()) {
                Ok(()) => self.status = "预设已保存".to_string(),
                Err(e) => self.status = format!("保存失败: {}", e),
            }
        }
    }

    pub fn import_preset(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .pick_file()
        {
            match std::fs::read_to_string(path) {
                Ok(data) => {
                    if let Ok(preset) = serde_json::from_str::<serde_json::Value>(&data) {
                        // 恢复文字列表
                        if let Some(list) = preset["text_list"].as_array() {
                            self.text_list.clear();
                            for item in list {
                                let text = item["text"].as_str().unwrap_or("").to_string();
                                let color = item["color"].as_str().unwrap_or("#b43c3c").to_string();
                                self.text_list.push(TextItem { text, color_hex: color });
                            }
                        }
                        // 恢复参数
                        if let Some(p) = preset.get("params") {
                            self.density = p["density"].as_f64().unwrap_or(0.45);
                            self.max_active = p["max_active"].as_u64().unwrap_or(4) as usize;
                            self.size_min = p["size_min"].as_f64().unwrap_or(18.0) as f32;
                            self.size_max = p["size_max"].as_f64().unwrap_or(32.0) as f32;
                            self.angle_min = p["angle_min"].as_f64().unwrap_or(-14.0) as f32;
                            self.angle_max = p["angle_max"].as_f64().unwrap_or(14.0) as f32;
                            self.type_speed = p["type_speed"].as_f64().unwrap_or(0.06);
                            self.post_hold = p["post_hold"].as_f64().unwrap_or(1.5);
                            self.seed = p["seed"].as_u64().unwrap_or(42);
                        }
                        self.status = "预设已加载".to_string();
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
        self.export_progress = "导出中...".to_string();

        let (tx, rx) = mpsc::channel();
        self.export_rx = Some(rx);

        std::thread::spawn(move || {
            let mut fc = FontCache::new();
            let _ = fc.load_from_dir(std::path::Path::new(&font_dir));
            let tx_cb = tx.clone();
            let total = overlays.len();
            let result = danmaku_core::video::process_video(
                &input, &output, &overlays, &fc,
                info.width, info.height, info.fps,
                Some(&move |cur, total_frames| {
                    let pct = if total_frames > 0 { cur * 100 / total_frames } else { 0 };
                    let bar_len = 20;
                    let filled = (pct as usize * bar_len) / 100;
                    let bar: String = "█".repeat(filled) + &"░".repeat(bar_len - filled);
                    let _ = tx_cb.send(format!("[{}] {}% ({}/{})", bar, pct, cur, total_frames));
                }),
            );
            match result {
                Ok(()) => { let _ = tx.send("完成".to_string()); }
                Err(e) => { let _ = tx.send(format!("错误: {}", e)); }
            }
        });
    }
}
