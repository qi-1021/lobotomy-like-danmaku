mod config;
mod text_overlay;
mod fonts;
mod video;
mod cli;

use eframe::egui;
use config::AppConfig;
use text_overlay::{TextOverlayManager, TextOverlay};
use std::sync::{Arc, Mutex};

struct DanmakuApp {
    config: AppConfig,
    manager: TextOverlayManager,
    selected_id: Option<u64>,
    new_text: String,
    new_font_size: f32,
    new_color: [f32; 3],
    new_x: f32,
    new_y: f32,
    new_angle: f32,
    new_start: f64,
    new_end: f64,
    new_alpha: f32,
    status: String,
    preset_texts: Vec<(String, [f32; 3])>,
    show_preset_panel: bool,
    preview_time: f64,
    use_preview_time: bool,
    batch_count: usize,
    processing: Arc<Mutex<bool>>,
    video_path: String,
    font_dir: String,
}

impl Default for DanmakuApp {
    fn default() -> Self {
        let preset_texts = vec![
            ("WARNING".into(), [0.7, 0.2, 0.2]),
            ("CORE SUPPRESSION".into(), [0.7, 0.2, 0.2]),
            ("CONTAINMENT BREACH".into(), [0.6, 0.6, 0.2]),
            ("ABNORMALITY ESCAPED".into(), [0.6, 0.6, 0.2]),
            ("ALL DEPARTMENTS ALERT".into(), [0.7, 0.3, 0.1]),
            ("MALKUTH".into(), [0.3, 0.7, 0.85]),
            ("YESED".into(), [0.3, 0.7, 0.85]),
            ("NETZACH".into(), [0.3, 0.7, 0.4]),
            ("HOD".into(), [0.65, 0.3, 0.75]),
            ("TIPHERETH".into(), [0.8, 0.7, 0.2]),
            ("GEBURAH".into(), [0.7, 0.2, 0.2]),
            ("CHESED".into(), [0.3, 0.7, 0.85]),
            ("BINAH".into(), [0.65, 0.3, 0.75]),
            ("CHOKHMAH".into(), [0.8, 0.7, 0.2]),
            ("控制部".into(), [0.7, 0.2, 0.2]),
            ("情报部".into(), [0.3, 0.7, 0.85]),
            ("培训部".into(), [0.3, 0.7, 0.4]),
            ("安保部".into(), [0.65, 0.3, 0.75]),
            ("中央本部".into(), [0.8, 0.7, 0.2]),
            ("福利部".into(), [0.3, 0.7, 0.85]),
            ("惩戒部".into(), [0.7, 0.2, 0.2]),
            ("记录部".into(), [0.65, 0.3, 0.75]),
            ("研发部".into(), [0.3, 0.7, 0.4]),
            ("构建部".into(), [0.8, 0.7, 0.2]),
            ("핵심 억제".into(), [0.7, 0.2, 0.2]),
            ("SILENCE".into(), [0.5, 0.5, 0.55]),
            ("INITIATING LOCKDOWN".into(), [0.7, 0.2, 0.2]),
            ("SEPHIRAH CORE COLLAPSE".into(), [0.6, 0.6, 0.2]),
            ("逆卡巴拉能量实体化".into(), [0.7, 0.7, 0.75]),
            ("异想体已突破收容".into(), [0.7, 0.7, 0.75]),
            ("所有部门进入红色警戒".into(), [0.7, 0.2, 0.2]),
            ("已经太迟了".into(), [0.5, 0.5, 0.55]),
            ("我不想再看到死亡了".into(), [0.6, 0.6, 0.7]),
        ];

        Self {
            config: AppConfig::default(),
            manager: TextOverlayManager::new(),
            selected_id: None,
            new_text: String::new(),
            new_font_size: 22.0,
            new_color: [0.7, 0.7, 0.75],
            new_x: 640.0,
            new_y: 360.0,
            new_angle: 0.0,
            new_start: 0.0,
            new_end: 10.0,
            new_alpha: 0.8,
            status: String::new(),
            preset_texts,
            show_preset_panel: false,
            preview_time: 0.0,
            use_preview_time: false,
            batch_count: 40,
            processing: Arc::new(Mutex::new(false)),
            video_path: String::new(),
            font_dir: "/tmp/lobotomy_extract/fonts_proper".into(),
        }
    }
}

impl eframe::App for DanmakuApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = if self.use_preview_time { self.preview_time } else { ctx.input(|i| i.time) };
        self.manager.update(now);

        // Left panel - Video & Batch
        egui::SidePanel::left("left_panel").default_width(240.0).show(ctx, |ui| {
            ui.heading("Video");
            ui.separator();

            // Video import
            ui.horizontal(|ui| {
                if ui.button("Import Video").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Video", &["mp4", "mov", "avi", "mkv", "webm"])
                        .pick_file()
                    {
                        self.video_path = path.to_string_lossy().to_string();
                        match video::probe_video(&self.video_path) {
                            Ok(info) => {
                                self.config.width = info.width;
                                self.config.height = info.height;
                                self.config.fps = info.fps as u32;
                                self.new_end = info.duration;
                                self.status = format!("Loaded: {}x{} @ {:.0}fps, {:.1}s", info.width, info.height, info.fps, info.duration);
                            }
                            Err(e) => self.status = format!("Error: {}", e),
                        }
                    }
                }
            });
            if !self.video_path.is_empty() {
                ui.label(egui::RichText::new(&self.video_path).small().color(egui::Color32::GRAY));
            }

            ui.separator();

            // Auto-generate
            ui.heading("Auto Generate");
            ui.horizontal(|ui| { ui.label("Count:"); ui.add(egui::DragValue::new(&mut self.batch_count).range(1..=200)); });
            ui.horizontal(|ui| { ui.label("Duration:"); ui.add(egui::DragValue::new(&mut self.new_end).speed(0.5)); });

            if ui.button("Generate Random Overlays").clicked() {
                self.manager.clear();
                self.manager.generate_random(
                    &self.preset_texts, self.new_end,
                    self.config.width as f32, self.config.height as f32,
                );
                self.status = format!("Generated {} overlays", self.manager.get_all().len());
            }

            ui.separator();

            // Export
            ui.heading("Export");
            if ui.button("Export Video (FFmpeg)").clicked() {
                if self.video_path.is_empty() {
                    self.status = "Error: No video imported".into();
                } else if self.manager.get_all().is_empty() {
                    self.status = "Error: No overlays".into();
                } else {
                    let output = rfd::FileDialog::new()
                        .add_filter("Video", &["mp4"])
                        .save_file()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| "output.mp4".to_string());

                    self.status = "Processing...".into();
                    let overlays = self.manager.get_all().to_vec();
                    let font_dir = self.font_dir.clone();
                    let w = self.config.width;
                    let h = self.config.height;
                    let fps = self.config.fps as f64;
                    let input = self.video_path.clone();
                    let proc = self.processing.clone();
                    let status = self.processing.clone();

                    std::thread::spawn(move || {
                        *proc.lock().unwrap() = true;
                        match video::process_video(&input, &output, &overlays, &font_dir, w, h, fps) {
                            Ok(()) => println!("Export complete!"),
                            Err(e) => eprintln!("Export error: {}", e),
                        }
                        *proc.lock().unwrap() = false;
                    });
                }
            }

            ui.separator();

            ui.heading("Presets");
            if ui.button("Open Preset Panel").clicked() {
                self.show_preset_panel = !self.show_preset_panel;
            }

            ui.collapsing("Quick Add", |ui| {
                for (text, color) in &self.preset_texts {
                    let c = egui::Color32::from_rgb((color[0]*255.0) as u8, (color[1]*255.0) as u8, (color[2]*255.0) as u8);
                    if ui.add(egui::Button::new(egui::RichText::new(text).color(c)).small()).clicked() {
                        let mut o = TextOverlay::new(text, self.new_font_size, self.new_x, self.new_y);
                        o.color = *color;
                        o.angle = self.new_angle;
                        o.start_time = self.new_start;
                        o.end_time = self.new_end;
                        o.alpha_max = self.new_alpha;
                        self.manager.add(o);
                        self.status = format!("Added: {}", text);
                    }
                }
            });
        });

        // Right panel - editor
        egui::SidePanel::right("editor").default_width(300.0).show(ctx, |ui| {
            ui.heading("Editor");
            ui.separator();

            ui.collapsing("Add Text", |ui| {
                ui.horizontal(|ui| { ui.label("Text:"); ui.text_edit_singleline(&mut self.new_text); });
                ui.horizontal(|ui| { ui.label("Size:"); ui.add(egui::Slider::new(&mut self.new_font_size, 8.0..=60.0)); });
                ui.horizontal(|ui| { ui.label("Color:"); ui.color_edit_button_rgb(&mut self.new_color); });
                ui.horizontal(|ui| { ui.label("X:"); ui.add(egui::Slider::new(&mut self.new_x, 0.0..=1280.0)); });
                ui.horizontal(|ui| { ui.label("Y:"); ui.add(egui::Slider::new(&mut self.new_y, 0.0..=720.0)); });
                ui.horizontal(|ui| { ui.label("Angle:"); ui.add(egui::Slider::new(&mut self.new_angle, -45.0..=45.0)); });
                ui.horizontal(|ui| { ui.label("Start:"); ui.add(egui::DragValue::new(&mut self.new_start).speed(0.1)); });
                ui.horizontal(|ui| { ui.label("End:"); ui.add(egui::DragValue::new(&mut self.new_end).speed(0.1)); });
                ui.horizontal(|ui| { ui.label("Alpha:"); ui.add(egui::Slider::new(&mut self.new_alpha, 0.0..=1.0)); });
                if ui.button("Add").clicked() && !self.new_text.is_empty() {
                    let mut o = TextOverlay::new(&self.new_text, self.new_font_size, self.new_x, self.new_y);
                    o.color = self.new_color; o.angle = self.new_angle;
                    o.start_time = self.new_start; o.end_time = self.new_end; o.alpha_max = self.new_alpha;
                    self.manager.add(o);
                    self.status = format!("Added: {}", self.new_text);
                }
            });

            ui.separator();

            ui.label(format!("Overlays ({})", self.manager.get_all().len()));
            egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
                let ids: Vec<u64> = self.manager.get_all().iter().map(|o| o.id).collect();
                for id in ids {
                    let o = self.manager.get_all().iter().find(|o| o.id == id).unwrap();
                    let label = format!("[{}] {} ({:.0},{:.0})", id, o.text, o.x, o.y);
                    if ui.selectable_label(self.selected_id == Some(id), &label).clicked() {
                        self.selected_id = Some(id);
                    }
                }
            });

            ui.separator();

            if let Some(id) = self.selected_id {
                if let Some(o) = self.manager.get_all().iter().find(|o| o.id == id).cloned() {
                    ui.heading("Edit");
                    let mut text = o.text.clone();
                    ui.horizontal(|ui| { ui.label("Text:"); ui.text_edit_singleline(&mut text); });
                    let mut fs = o.font_size;
                    ui.horizontal(|ui| { ui.label("Size:"); ui.add(egui::Slider::new(&mut fs, 8.0..=60.0)); });
                    let mut c = o.color;
                    ui.horizontal(|ui| { ui.label("Color:"); ui.color_edit_button_rgb(&mut c); });
                    let mut x = o.x; let mut y = o.y;
                    ui.horizontal(|ui| { ui.label("X:"); ui.add(egui::Slider::new(&mut x, 0.0..=1280.0)); });
                    ui.horizontal(|ui| { ui.label("Y:"); ui.add(egui::Slider::new(&mut y, 0.0..=720.0)); });
                    let mut a = o.angle;
                    ui.horizontal(|ui| { ui.label("Angle:"); ui.add(egui::Slider::new(&mut a, -45.0..=45.0)); });
                    let mut st = o.start_time; let mut et = o.end_time;
                    ui.horizontal(|ui| { ui.label("Start:"); ui.add(egui::DragValue::new(&mut st).speed(0.1)); });
                    ui.horizontal(|ui| { ui.label("End:"); ui.add(egui::DragValue::new(&mut et).speed(0.1)); });
                    let mut al = o.alpha_max;
                    ui.horizontal(|ui| { ui.label("Alpha:"); ui.add(egui::Slider::new(&mut al, 0.0..=1.0)); });
                    ui.horizontal(|ui| {
                        if ui.button("Update").clicked() {
                            let mut u = o.clone();
                            u.text = text; u.font_size = fs; u.color = c; u.x = x; u.y = y; u.angle = a;
                            u.start_time = st; u.end_time = et; u.alpha_max = al;
                            self.manager.update_overlay(id, u);
                        }
                        if ui.button("Delete").clicked() {
                            self.manager.remove(id); self.selected_id = None;
                        }
                    });
                }
            }

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Export JSON").clicked() {
                    if let Ok(j) = self.manager.save_to_json() {
                        std::fs::write("overlays.json", j).ok();
                        self.status = "Exported".into();
                    }
                }
                if ui.button("Import JSON").clicked() {
                    if let Ok(j) = std::fs::read_to_string("overlays.json") {
                        self.manager.load_from_json(&j).ok();
                        self.status = "Imported".into();
                    }
                }
            });
            if ui.button("Clear All").clicked() {
                self.manager.clear(); self.selected_id = None; self.status = "Cleared".into();
            }

            ui.separator();
            ui.label(egui::RichText::new(&self.status).small().color(egui::Color32::GRAY));
        });

        // Central panel - preview + timeline
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.use_preview_time, "Preview");
                if self.use_preview_time {
                    ui.add(egui::Slider::new(&mut self.preview_time, 0.0..=self.new_end).text("Time"));
                }
                ui.label(format!("Visible: {}", self.manager.get_visible().len()));
            });

            let (resp, painter) = ui.allocate_painter(egui::vec2(960.0, 500.0), egui::Sense::hover());
            let r = resp.rect;
            let bg = egui::Color32::from_rgb(self.config.bg_color[0], self.config.bg_color[1], self.config.bg_color[2]);
            painter.rect_filled(r, 0.0, bg);

            let t = now;
            for o in self.manager.get_all() {
                let alpha = o.get_alpha(t);
                if alpha < 0.01 { continue; }
                let color = egui::Color32::from_rgba_premultiplied(
                    (o.color[0]*255.0) as u8, (o.color[1]*255.0) as u8, (o.color[2]*255.0) as u8, (alpha*255.0) as u8,
                );
                let pos = egui::pos2(r.min.x + o.x * (r.width() / 1280.0), r.min.y + o.y * (r.height() / 720.0));
                let galley = ui.painter().layout_no_wrap(o.text.clone(), egui::FontId::new(o.font_size, egui::FontFamily::Proportional), color);
                painter.galley(pos, galley, color);
            }

            // Timeline
            ui.separator();
            let (tl_resp, tl_painter) = ui.allocate_painter(egui::vec2(960.0, 60.0), egui::Sense::click_and_drag());
            let tl_rect = tl_resp.rect;
            tl_painter.rect_filled(tl_rect, 0.0, egui::Color32::from_rgb(20, 18, 28));
            let dur = self.new_end.max(1.0);
            for sec in 0..=dur as u32 {
                let x = tl_rect.min.x + (sec as f32 / dur as f32) * tl_rect.width();
                tl_painter.line_segment([egui::pos2(x, tl_rect.min.y), egui::pos2(x, tl_rect.max.y)],
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(40, 35, 55, 200)));
                if sec % 5 == 0 {
                    tl_painter.text(egui::pos2(x + 2.0, tl_rect.min.y + 2.0), egui::Align2::LEFT_TOP,
                        format!("{}s", sec), egui::FontId::new(10.0, egui::FontFamily::Proportional),
                        egui::Color32::from_rgba_premultiplied(80, 75, 100, 200));
                }
            }
            for o in self.manager.get_all() {
                let x0 = tl_rect.min.x + (o.start_time as f32 / dur as f32) * tl_rect.width();
                let x1 = tl_rect.min.x + (o.end_time as f32 / dur as f32) * tl_rect.width();
                let bar_y = tl_rect.min.y + 15.0;
                let c = egui::Color32::from_rgba_premultiplied(
                    (o.color[0]*255.0) as u8, (o.color[1]*255.0) as u8, (o.color[2]*255.0) as u8, 180,
                );
                tl_painter.rect_filled(egui::Rect::from_min_max(egui::pos2(x0, bar_y), egui::pos2(x1, bar_y + 10.0)), 2.0, c);
            }
            let cur_x = tl_rect.min.x + ((t as f32 / dur as f32) * tl_rect.width()).min(tl_rect.width());
            tl_painter.line_segment([egui::pos2(cur_x, tl_rect.min.y), egui::pos2(cur_x, tl_rect.max.y)],
                egui::Stroke::new(2.0, egui::Color32::RED));

            if tl_resp.clicked() {
                if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
                    let rel_x = (pos.x - tl_rect.min.x) / tl_rect.width();
                    self.preview_time = (rel_x as f64 * dur).max(0.0).min(dur);
                    self.use_preview_time = true;
                }
            }
        });

        // Preset window
        if self.show_preset_panel {
            egui::Window::new("Presets").open(&mut self.show_preset_panel).default_width(280.0).show(ctx, |ui| {
                for (text, color) in &self.preset_texts {
                    let c = egui::Color32::from_rgb((color[0]*255.0) as u8, (color[1]*255.0) as u8, (color[2]*255.0) as u8);
                    if ui.button(egui::RichText::new(text).color(c)).clicked() {
                        let mut o = TextOverlay::new(text, 24.0, 640.0, 360.0);
                        o.color = *color;
                        o.start_time = self.new_start;
                        o.end_time = self.new_end;
                        self.manager.add(o);
                    }
                }
            });
        }
    }
}

fn main() {
    env_logger::init();
    let cli_args = cli::parse_args();

    if cli_args.headless {
        cli::run_headless(cli_args);
        return;
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1520.0, 740.0])
            .with_title("Lobotomy Corporation - Danmaku Editor"),
        ..Default::default()
    };

    let mut app = DanmakuApp::default();
    app.config.width = cli_args.width;
    app.config.height = cli_args.height;
    app.config.fps = cli_args.fps;

    if let Some(input) = &cli_args.input {
        if let Ok(json) = std::fs::read_to_string(input) {
            app.manager.load_from_json(&json).ok();
        }
    }

    eframe::run_native("Lobotomy Danmaku", options, Box::new(|_cc| Ok(Box::new(app)))).unwrap();
}
