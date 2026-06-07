use image::RgbaImage;
use ab_glyph::{FontRef, PxScale, Font};
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

    let (font_data, _idx) = match font_cache.get_font_data(visible) {
        Some(f) => f,
        None => return,
    };
    let font = match FontRef::try_from_slice(font_data) {
        Ok(f) => f,
        Err(_) => return,
    };

    let scale = PxScale::from(overlay.font_size);
    let color = [
        (overlay.color[0] * 255.0) as u8,
        (overlay.color[1] * 255.0) as u8,
        (overlay.color[2] * 255.0) as u8,
    ];
    let angle_rad = overlay.angle * std::f32::consts::PI / 180.0;
    let has_rotation = angle_rad.abs() > 0.005;

    // Measure total width
    let mut total_width = 0.0f32;
    let mut char_widths = Vec::new();
    for ch in visible.chars() {
        let glyph_id = font.glyph_id(ch);
        let glyph = glyph_id.with_scale(scale);
        let w = font.outline_glyph(glyph).map(|o| o.px_bounds().width()).unwrap_or(0.0);
        char_widths.push(w);
        total_width += w;
    }

    let cx = overlay.x;
    let cy = overlay.y;
    let start_x = cx - total_width / 2.0;
    let baseline_y = cy + overlay.font_size * 0.35;
    let mut cursor_x = start_x;

    for (i, ch) in visible.chars().enumerate() {
        let glyph_id = font.glyph_id(ch);
        let glyph = glyph_id.with_scale(scale);
        if let Some(outlined) = font.outline_glyph(glyph) {
            outlined.draw(|x, y, c| {
                let raw_px = cursor_x + x as f32;
                let raw_py = baseline_y + y as f32;

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
        cursor_x += char_widths[i];
    }
}

pub fn render_frame(img: &mut RgbaImage, overlays: &[TextOverlay], t: f64, font_cache: &FontCache) {
    for overlay in overlays {
        render_overlay(img, overlay, t, font_cache);
    }
}
