use eframe::egui;
use crate::app::DanmakuApp;

const ACCENT: egui::Color32 = egui::Color32::from_rgb(180, 60, 60);
const SAVE_COLOR: egui::Color32 = egui::Color32::from_rgb(204, 180, 68);
const LOAD_COLOR: egui::Color32 = egui::Color32::from_rgb(78, 176, 216);
const TEXT_DIM: egui::Color32 = egui::Color32::from_rgb(136, 136, 136);

impl eframe::App for DanmakuApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 计算当前时间
        let t = if self.playing {
            let elapsed = self.play_start.map(|s| s.elapsed().as_secs_f64()).unwrap_or(0.0);
            let t = self.preview_time_offset + elapsed;
            let max_time = self.video_info.as_ref().map(|i| i.duration).unwrap_or(10.0);
            if t >= max_time {
                self.playing = false;
                self.play_start = None;
                max_time
            } else {
                self.preview_time = t;
                ctx.request_repaint();
                t
            }
        } else {
            self.preview_time
        };

        // 检查导出进度
        let mut remove_rx = false;
        if let Some(rx) = &self.export_rx {
            while let Ok(msg) = rx.try_recv() {
                if msg == "完成" {
                    self.processing = false;
                    self.export_progress = "导出完成".to_string();
                    remove_rx = true;
                } else if msg.starts_with("错误") {
                    self.processing = false;
                    self.export_progress = msg;
                    remove_rx = true;
                } else {
                    self.export_progress = msg;
                    ctx.request_repaint();
                }
            }
        }
        if remove_rx { self.export_rx = None; }

        // ═══ 颜色弹窗 ═══
        if self.show_color_picker {
            let mut open = self.show_color_picker;
            egui::Window::new("选择颜色")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    let r = self.color_picker_rgb[0];
                    let g = self.color_picker_rgb[1];
                    let b = self.color_picker_rgb[2];
                    ui.horizontal(|ui| {
                        ui.label("R:");
                        ui.add(egui::Slider::new(&mut self.color_picker_rgb[0], 0.0..=1.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("G:");
                        ui.add(egui::Slider::new(&mut self.color_picker_rgb[1], 0.0..=1.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("B:");
                        ui.add(egui::Slider::new(&mut self.color_picker_rgb[2], 0.0..=1.0));
                    });
                    // 预览色块
                    let c = egui::Color32::from_rgb(
                        (self.color_picker_rgb[0] * 255.0) as u8,
                        (self.color_picker_rgb[1] * 255.0) as u8,
                        (self.color_picker_rgb[2] * 255.0) as u8,
                    );
                    let (resp, painter) = ui.allocate_painter(egui::vec2(200.0, 30.0), egui::Sense::hover());
                    painter.rect_filled(resp.rect, 4.0, c);
                    // hex 输入
                    let mut hex_input = DanmakuApp::rgb01_to_hex(self.color_picker_rgb);
                    ui.horizontal(|ui| {
                        ui.label("HEX:");
                        if ui.text_edit_singleline(&mut hex_input).changed() {
                            if let Some(c) = parse_hex_color(&hex_input) {
                                self.color_picker_rgb = c;
                            }
                        }
                    });
                    // 常用颜色快捷
                    ui.label("快捷:");
                    ui.horizontal(|ui| {
                        let presets = [
                            [0.7, 0.2, 0.2], [0.3, 0.69, 0.85], [0.31, 0.74, 0.43],
                            [0.65, 0.29, 0.75], [0.8, 0.7, 0.2], [0.72, 0.7, 0.78],
                            [0.91, 0.89, 0.94], [0.31, 0.31, 0.38],
                        ];
                        for p in &presets {
                            let c = egui::Color32::from_rgb((p[0]*255.0) as u8, (p[1]*255.0) as u8, (p[2]*255.0) as u8);
                            let btn = egui::Button::new("  ").fill(c);
                            if ui.add(btn).clicked() {
                                self.color_picker_rgb = *p;
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui.button("确定").clicked() {
                            self.apply_color_picker();
                        }
                        if ui.button("取消").clicked() {
                            self.show_color_picker = false;
                        }
                    });
                });
            if !open { self.show_color_picker = false; }
        }

        // ═══ 左侧面板 ═══
        egui::SidePanel::left("left_panel")
            .default_width(460.0)
            .resizable(true)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // ── 视频 ──
                    ui.collapsing("视频", |ui| {
                        let btn_text = if self.video_path.is_empty() {
                            "  点击选择视频".to_string()
                        } else {
                            format!("  {}", std::path::Path::new(&self.video_path)
                                .file_name().unwrap_or_default().to_string_lossy())
                        };
                        if ui.button(&btn_text).clicked() {
                            self.import_video();
                        }
                    });

                    // ── 文字列表 ──
                    ui.collapsing("文字列表", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("文字:");
                            ui.text_edit_singleline(&mut self.new_text);
                        });
                        ui.horizontal(|ui| {
                            ui.label("颜色:");
                            let c = DanmakuApp::color_from_hex(&self.new_color_hex);
                            let btn = egui::Button::new(egui::RichText::new(&self.new_color_hex).color(c));
                            if ui.add(btn).clicked() {
                                self.open_color_picker("new");
                            }
                            if ui.button("添加").clicked() && !self.new_text.is_empty() {
                                self.text_list.push(crate::app::TextItem {
                                    text: self.new_text.clone(),
                                    color_hex: self.new_color_hex.clone(),
                                });
                                self.status = format!("已添加: {}", self.new_text);
                                self.new_text.clear();
                            }
                        });

                        let mut to_delete = None;
                        for (i, item) in self.text_list.iter().enumerate() {
                            ui.horizontal(|ui| {
                                let c = DanmakuApp::color_from_hex(&item.color_hex);
                                ui.label(egui::RichText::new(&item.text).color(c));
                                if ui.small_button("🗑").clicked() {
                                    to_delete = Some(i);
                                }
                            });
                        }
                        if let Some(i) = to_delete { self.text_list.remove(i); }
                        if ui.button("清空").clicked() { self.text_list.clear(); }
                    });

                    // ── 生成参数 ──
                    ui.collapsing("生成参数", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("密度:");
                            ui.add(egui::DragValue::new(&mut self.density).speed(0.05).range(0.1..=1.0));
                            ui.label("最多同时:");
                            ui.add(egui::DragValue::new(&mut self.max_active).range(1..=20));
                        });
                        ui.horizontal(|ui| {
                            ui.label("字号:");
                            ui.add(egui::DragValue::new(&mut self.size_min).range(8.0..=72.0));
                            ui.label("-");
                            ui.add(egui::DragValue::new(&mut self.size_max).range(8.0..=72.0));
                            ui.label("种子:");
                            ui.add(egui::DragValue::new(&mut self.seed).range(0..=9999));
                        });
                        ui.horizontal(|ui| {
                            ui.label("角度:");
                            ui.add(egui::DragValue::new(&mut self.angle_min).speed(1.0).range(-45.0..=45.0));
                            ui.label("~");
                            ui.add(egui::DragValue::new(&mut self.angle_max).speed(1.0).range(-45.0..=45.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("打字速度:");
                            ui.add(egui::DragValue::new(&mut self.type_speed).speed(0.01).range(0.01..=0.3));
                            ui.label("秒/字");
                            ui.label("留存:");
                            ui.add(egui::DragValue::new(&mut self.post_hold).speed(0.1).range(0.0..=10.0));
                            ui.label("秒");
                        });
                        ui.horizontal(|ui| {
                            ui.label("透明度:");
                            ui.add(egui::DragValue::new(&mut self.auto_alpha).speed(0.05).range(0.1..=1.0));
                        });
                        if ui.button("自动生成叠加").clicked() { self.auto_generate(); }
                    });

                    // ── 叠加列表 ──
                    ui.collapsing("叠加列表", |ui| {
                        let mut to_delete = None;
                        let mut selected = None;
                        for (i, o) in self.overlays.iter().enumerate() {
                            let label = format!("{} ({:.0},{:.0}) {:.1}-{:.1}s", o.text, o.x, o.y, o.start_time, o.end_time);
                            let is_sel = self.selected_overlay == Some(i);
                            let c = egui::Color32::from_rgb(
                                (o.color[0] * 255.0) as u8, (o.color[1] * 255.0) as u8, (o.color[2] * 255.0) as u8,
                            );
                            ui.horizontal(|ui| {
                                if ui.selectable_label(is_sel, egui::RichText::new(&label).color(c)).clicked() {
                                    selected = Some(i);
                                }
                                if ui.small_button("🗑").clicked() { to_delete = Some(i); }
                            });
                        }
                        if let Some(i) = selected { self.select_overlay(i); }
                        if let Some(i) = to_delete {
                            self.overlays.remove(i);
                            if self.selected_overlay == Some(i) { self.selected_overlay = None; }
                        }
                        if ui.button("清空").clicked() {
                            self.overlays.clear();
                            self.selected_overlay = None;
                        }
                    });

                    // ── 手动编辑 ──
                    ui.collapsing("手动编辑", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("文字:");
                            ui.text_edit_singleline(&mut self.ed_text);
                            ui.label("字号:");
                            ui.add(egui::DragValue::new(&mut self.ed_size).range(8.0..=72.0));
                            ui.label("角度:");
                            ui.add(egui::DragValue::new(&mut self.ed_angle).speed(0.5).range(-45.0..=45.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("颜色:");
                            let c = DanmakuApp::color_from_hex(&self.ed_color_hex);
                            let btn = egui::Button::new(egui::RichText::new(&self.ed_color_hex).color(c));
                            if ui.add(btn).clicked() {
                                self.open_color_picker("ed");
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("X:");
                            ui.add(egui::DragValue::new(&mut self.ed_x).speed(10.0).range(0.0..=1920.0));
                            ui.label("Y:");
                            ui.add(egui::DragValue::new(&mut self.ed_y).speed(10.0).range(0.0..=1080.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("开始:");
                            ui.add(egui::DragValue::new(&mut self.ed_start).speed(0.1).range(0.0..=300.0));
                            ui.label("结束:");
                            ui.add(egui::DragValue::new(&mut self.ed_end).speed(0.1).range(0.0..=300.0));
                            ui.label("留存:");
                            ui.add(egui::DragValue::new(&mut self.ed_post_hold).speed(0.1).range(0.0..=10.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("打字速度:");
                            ui.add(egui::DragValue::new(&mut self.ed_speed).speed(0.01).range(0.01..=0.3));
                            ui.label("透明度:");
                            ui.add(egui::DragValue::new(&mut self.ed_alpha).speed(0.05).range(0.1..=1.0));
                        });
                        ui.horizontal(|ui| {
                            if ui.button("添加新叠加").clicked() { self.manual_add(); }
                            if ui.button("更新选中").clicked() { self.manual_update(); }
                        });
                    });
                });

                // ── 导出 ──
                ui.separator();
                ui.label(egui::RichText::new(&self.status).small().color(TEXT_DIM));
                if !self.export_progress.is_empty() {
                    ui.label(egui::RichText::new(&self.export_progress).small().color(egui::Color32::YELLOW));
                }
                if ui.button("导出视频").clicked() { self.export_video(); }
                ui.horizontal(|ui| {
                    let save_btn = egui::Button::new("保存项目").fill(SAVE_COLOR);
                    if ui.add(save_btn).clicked() { self.export_json(); }
                    let load_btn = egui::Button::new("加载项目").fill(LOAD_COLOR);
                    if ui.add(load_btn).clicked() { self.import_json(); }
                });
            });

        // ═══ 右侧预览 ═══
        egui::CentralPanel::default().show(ctx, |ui| {
            // 播放控制
            ui.horizontal(|ui| {
                let play_text = if self.playing { "暂停" } else { "播放" };
                if ui.button(play_text).clicked() {
                    if self.playing {
                        self.playing = false;
                        self.play_start = None;
                    } else {
                        self.playing = true;
                        self.play_start = Some(std::time::Instant::now());
                        self.preview_time_offset = self.preview_time;
                        // 启动 pipe 到当前时间
                        self.seek_to_time(self.preview_time);
                    }
                }
                ui.checkbox(&mut self.show_overlay_text, "叠加文字");
                ui.label("时间:");
                let max_time = self.video_info.as_ref().map(|i| i.duration).unwrap_or(10.0);
                let slider_resp = ui.add(egui::Slider::new(&mut self.preview_time, 0.0..=max_time).show_value(false));
                if slider_resp.changed() {
                    self.playing = false;
                    self.play_start = None;
                    // seek pipe
                    self.seek_to_time(self.preview_time);
                    // 解码一帧
                    if let Some(raw) = self.decode_frame_from_pipe() {
                        self.update_frame_texture(ctx, raw);
                    }
                }
                ui.label(format!("{:.1}s", t));
            });

            // 预览画布
            let available = ui.available_size();
            let (resp, painter) = ui.allocate_painter(available, egui::Sense::hover());
            let r = resp.rect;
            painter.rect_filled(r, 0.0, egui::Color32::from_rgb(12, 10, 18));

            // 播放时从 pipe 读帧
            if self.playing {
                let info = self.video_info.as_ref();
                if let Some(info) = info {
                    let frame_duration = 1.0 / info.fps;
                    let target_frame = (t / frame_duration) as u64;
                    // 读到目标帧
                    while self.pipe_frame_idx <= target_frame {
                        if let Some(raw) = self.decode_frame_from_pipe() {
                            if self.pipe_frame_idx >= target_frame {
                                self.update_frame_texture(ctx, raw);
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
            }

            // 显示帧
            if let Some(tex) = &self.frame_texture {
                let vw = self.video_info.as_ref().map(|i| i.width as f32).unwrap_or(1280.0);
                let vh = self.video_info.as_ref().map(|i| i.height as f32).unwrap_or(720.0);
                let scale = (r.width() / vw).min(r.height() / vh);
                let draw_w = vw * scale;
                let draw_h = vh * scale;
                let ox = r.min.x + (r.width() - draw_w) / 2.0;
                let oy = r.min.y + (r.height() - draw_h) / 2.0;
                let img_rect = egui::Rect::from_min_size(egui::pos2(ox, oy), egui::vec2(draw_w, draw_h));
                painter.image(tex.id(), img_rect, egui::Rect::from_min_max(
                    egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0),
                ), egui::Color32::WHITE);
            }

            // 绘制叠加文字
            let vw = self.video_info.as_ref().map(|i| i.width as f32).unwrap_or(1280.0);
            let vh = self.video_info.as_ref().map(|i| i.height as f32).unwrap_or(720.0);
            let scale = (r.width() / vw).min(r.height() / vh);
            let ox = r.min.x + (r.width() - vw * scale) / 2.0;
            let oy = r.min.y + (r.height() - vh * scale) / 2.0;

            if self.show_overlay_text {
                for o in &self.overlays {
                    let alpha = o.get_alpha(t);
                    if alpha < 0.01 { continue; }
                    let visible = o.visible_text(t);
                    if visible.is_empty() { continue; }

                    let px = ox + o.x * scale;
                    let py = oy + o.y * scale;
                    let fs = (o.font_size * scale * 0.8).max(8.0);

                    let r_c = (o.color[0] * 255.0) as u8;
                    let g_c = (o.color[1] * 255.0) as u8;
                    let b_c = (o.color[2] * 255.0) as u8;
                    let mixed = egui::Color32::from_rgba_premultiplied(
                        ((1.0 - alpha) * 12.0 + alpha * r_c as f32) as u8,
                        ((1.0 - alpha) * 10.0 + alpha * g_c as f32) as u8,
                        ((1.0 - alpha) * 18.0 + alpha * b_c as f32) as u8,
                        255,
                    );

                    painter.text(
                        egui::pos2(px, py),
                        egui::Align2::CENTER_CENTER,
                        visible,
                        egui::FontId::proportional(fs),
                        mixed,
                    );
                }
            }
        });
    }
}

impl DanmakuApp {
    fn update_frame_texture(&mut self, ctx: &egui::Context, raw: Vec<u8>) {
        let info = match &self.video_info {
            Some(i) => i,
            None => return,
        };
        let img = image::RgbaImage::from_raw(info.width, info.height, raw);
        if let Some(img) = img {
            let dynamic = image::DynamicImage::ImageRgba8(img);
            let rgba = dynamic.to_rgba8();
            let pixels: Vec<u8> = rgba.into_raw();
            let tex = ctx.load_texture(
                "preview_frame",
                egui::ColorImage::from_rgba_unmultiplied(
                    [info.width as usize, info.height as usize],
                    &pixels,
                ),
                egui::TextureOptions::LINEAR,
            );
            self.frame_texture = Some(tex);
        }
    }
}

fn parse_hex_color(hex: &str) -> Option<[f32; 3]> {
    let hex = hex.trim().trim_start_matches('#');
    if hex.len() >= 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some([r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0])
    } else {
        None
    }
}
