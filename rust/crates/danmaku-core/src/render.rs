use image::RgbaImage;
use ab_glyph::{FontRef, PxScale, Font, GlyphId, ScaleFont};
use crate::overlay::TextOverlay;
use crate::font::FontCache;

fn rotate_point(x: f32, y: f32, cx: f32, cy: f32, angle_rad: f32) -> (f32, f32) {
    let dx = x - cx;
    let dy = y - cy;
    let cos = angle_rad.cos();
    let sin = angle_rad.sin();
    (cx + dx * cos - dy * sin, cy + dx * sin + dy * cos)
}

pub fn render_overlay(img: &mut RgbaImage, overlay: &TextOverlay, t: f64, font_cache: &FontCache) {
    let alpha = overlay.get_alpha(t);
    if alpha < 0.01 { return; }
    let visible = overlay.visible_text(t);
    if visible.is_empty() { return; }

    let (font_data, font_idx) = match font_cache.get_font_data(visible) {
        Some(f) => f,
        None => return,
    };
    let font = match FontRef::try_from_slice_and_index(font_data, font_idx) {
        Ok(f) => f,
        Err(_) => return,
    };

    let scale = PxScale::from(overlay.font_size);
    let scaled_font = ab_glyph::Font::as_scaled(&font, scale);
    let color = [
        (overlay.color[0] * 255.0) as u8,
        (overlay.color[1] * 255.0) as u8,
        (overlay.color[2] * 255.0) as u8,
    ];
    let angle_rad = overlay.angle * std::f32::consts::PI / 180.0;
    let has_rotation = angle_rad.abs() > 0.005;

    // 用 ab_glyph layout 获取每个字形在整行中的正确位置
    struct GlyphInfo {
        glyph_id: GlyphId,
        x: f32,
    }
    let mut glyph_infos: Vec<GlyphInfo> = Vec::new();
    let mut cursor_x = 0.0f32;
    let mut prev_glyph: Option<GlyphId> = None;

    for ch in visible.chars() {
        let glyph_id = font.glyph_id(ch);
        if let Some(prev) = prev_glyph {
            cursor_x += scaled_font.kern(prev, glyph_id);
        }
        let advance = scaled_font.h_advance(glyph_id);
        glyph_infos.push(GlyphInfo { glyph_id, x: cursor_x });
        cursor_x += advance;
        prev_glyph = Some(glyph_id);
    }

    let total_width = cursor_x;
    let cx = overlay.x;
    let cy = overlay.y;
    let ascent = scaled_font.ascent();
    let descent = scaled_font.descent();
    let line_height = ascent - descent;

    // 渲染每个字形
    for gi in &glyph_infos {
        // 计算字形在图像中的目标位置
        let glyph_x = gi.x - total_width / 2.0 + cx;
        let glyph_y = cy - line_height / 2.0 + ascent;

        let positioned = gi.glyph_id.with_scale_and_position(
            scale,
            ab_glyph::point(glyph_x, glyph_y),
        );
        if let Some(outlined) = font.outline_glyph(positioned) {
            let bounds = outlined.px_bounds();
            outlined.draw(|x, y, c| {
                // x, y 是相对于 bounds.min 的像素偏移
                let raw_px = bounds.min.x + x as f32;
                let raw_py = bounds.min.y + y as f32;

                let (px, py) = if has_rotation {
                    rotate_point(raw_px, raw_py, cx, cy, angle_rad)
                } else {
                    (raw_px, raw_py)
                };

                let px_i = px.round() as i32;
                let py_i = py.round() as i32;
                if px_i >= 0 && py_i >= 0 && (px_i as u32) < img.width() && (py_i as u32) < img.height() {
                    let existing = img.get_pixel(px_i as u32, py_i as u32);
                    let a = c * alpha;
                    img.put_pixel(px_i as u32, py_i as u32, image::Rgba([
                        ((1.0 - a) * existing[0] as f32 + a * color[0] as f32) as u8,
                        ((1.0 - a) * existing[1] as f32 + a * color[1] as f32) as u8,
                        ((1.0 - a) * existing[2] as f32 + a * color[2] as f32) as u8,
                        255,
                    ]));
                }
            });
        }
    }
}

pub fn render_frame(img: &mut RgbaImage, overlays: &[TextOverlay], t: f64, font_cache: &FontCache) {
    for overlay in overlays {
        render_overlay(img, overlay, t, font_cache);
    }
}
