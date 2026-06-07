use ab_glyph::{FontVec, PxScale, Font, ScaleFont};
use std::collections::HashMap;
use image::{RgbaImage, Rgba};

pub struct FontManager {
    fonts: Vec<(String, FontVec)>,
}

impl FontManager {
    pub fn new() -> Self {
        Self { fonts: Vec::new() }
    }

    pub fn load_from_dir(&mut self, dir: &str) {
        let path = std::path::Path::new(dir);
        if !path.exists() { return; }
        for entry in std::fs::read_dir(path).unwrap().filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().to_string();
            let lower = name.to_lowercase();
            if lower.ends_with(".ttf") || lower.ends_with(".otf") {
                if let Ok(data) = std::fs::read(entry.path()) {
                    if let Ok(font) = FontVec::try_from_vec(data) {
                        println!("  Loaded font: {}", name);
                        self.fonts.push((name, font));
                    }
                }
            }
        }
        println!("Loaded {} fonts total", self.fonts.len());
    }

    pub fn pick_font(&self, text: &str) -> &FontVec {
        let has_cjk = text.chars().any(|c| {
            let cp = c as u32;
            (0x4E00..=0x9FFF).contains(&cp) || (0xAC00..=0xD7AF).contains(&cp) || (0x3040..=0x30FF).contains(&cp)
        });
        if has_cjk {
            self.fonts.iter().find(|(n, _)| {
                let l = n.to_lowercase();
                l.contains("nanum") || l.contains("bm")
            }).map(|(_, f)| f).unwrap_or_else(|| &self.fonts[0].1)
        } else {
            self.fonts.iter().find(|(n, _)| {
                n.to_lowercase().contains("norwester")
            }).map(|(_, f)| f).unwrap_or_else(|| &self.fonts[0].1)
        }
    }

    pub fn render_text(
        &self,
        canvas: &mut RgbaImage,
        text: &str,
        font_size: f32,
        x: f32,
        y: f32,
        _angle_deg: f32,
        color: [f32; 3],
        alpha: f32,
    ) {
        if self.fonts.is_empty() { return; }
        let font = self.pick_font(text);
        let scale = PxScale::from(font_size);
        let scaled_font = font.as_scaled(scale);

        let r = (color[0] * 255.0) as u8;
        let g = (color[1] * 255.0) as u8;
        let b = (color[2] * 255.0) as u8;
        let a = (alpha * 255.0) as u8;

        let mut cursor_x = 0.0f32;
        let baseline = scaled_font.ascent();

        // Calculate total width
        for ch in text.chars() {
            let glyph_id = font.glyph_id(ch);
            cursor_x += scaled_font.h_advance(glyph_id);
        }
        let text_w = cursor_x as u32 + 4;
        let text_h = (scaled_font.ascent() - scaled_font.descent()).abs() as u32 + 4;

        if text_w == 0 || text_h == 0 || text_w > 2000 || text_h > 400 { return; }

        let mut txt_img = RgbaImage::new(text_w, text_h);
        let mut cx = 2.0f32;

        for ch in text.chars() {
            let glyph_id = font.glyph_id(ch);
            let advance = scaled_font.h_advance(glyph_id);
            let scaled = glyph_id.with_scale_and_position(scale, ab_glyph::point(cx, baseline));

            if let Some(outlined) = font.outline_glyph(scaled) {
                let bb = outlined.px_bounds();
                outlined.draw(|px, py, coverage| {
                    let ix = (bb.min.x + px as f32) as i32;
                    let iy = (bb.min.y + py as f32) as i32;
                    if ix >= 0 && iy >= 0 && (ix as u32) < text_w && (iy as u32) < text_h {
                        let ca = ((coverage as f32) * alpha * 255.0) as u8;
                        if ca > 0 {
                            txt_img.put_pixel(ix as u32, iy as u32, Rgba([r, g, b, ca]));
                        }
                    }
                });
            }
            cx += advance;
        }

        // Paste onto canvas
        let px = (x - text_w as f32 / 2.0) as i64;
        let py = (y - text_h as f32 / 2.0) as i64;
        image::imageops::overlay(canvas, &txt_img, px, py);
    }
}
