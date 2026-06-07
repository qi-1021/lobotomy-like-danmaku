use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextOverlay {
    pub text: String,
    pub font_size: f32,
    pub color: [f32; 3],
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub start_time: f64,
    pub end_time: f64,
    pub alpha_max: f32,
    pub type_speed: f64,
    pub post_hold: f64,
}

impl TextOverlay {
    pub fn new(text: &str, font_size: f32, x: f32, y: f32) -> Self {
        Self {
            text: text.to_string(),
            font_size,
            color: [0.7, 0.7, 0.75],
            x,
            y,
            angle: 0.0,
            start_time: 0.0,
            end_time: 5.0,
            alpha_max: 0.7,
            type_speed: 0.06,
            post_hold: 1.5,
        }
    }

    pub fn get_alpha(&self, t: f64) -> f32 {
        if t < self.start_time || t > self.end_time {
            return 0.0;
        }
        let a = self.alpha_max;
        if t < self.start_time + 0.4 {
            ((t - self.start_time) / 0.4).min(1.0) as f32 * a
        } else if t > self.end_time - 0.3 {
            ((self.end_time - t) / 0.3).min(1.0) as f32 * a
        } else {
            a
        }
    }

    pub fn chars_visible(&self, t: f64) -> usize {
        let elapsed = t - self.start_time;
        if elapsed <= 0.0 {
            return 0;
        }
        let count = (elapsed / self.type_speed).floor() as usize;
        count.min(self.text.chars().count())
    }

    pub fn pop_scale(&self, t: f64) -> f32 {
        let elapsed = t - self.start_time;
        let chars = self.chars_visible(t);
        let total = self.text.chars().count();
        if chars >= total || chars == 0 {
            return 1.0;
        }
        let last_char_progress = (elapsed / self.type_speed) - chars as f64 + 1.0;
        if last_char_progress < 1.0 {
            1.0 + 0.3 * (last_char_progress * std::f64::consts::PI).sin() as f32
        } else {
            1.0
        }
    }

    pub fn visible_text(&self, t: f64) -> &str {
        let chars = self.chars_visible(t);
        let byte_idx = self.text.char_indices().nth(chars).map(|(i, _)| i).unwrap_or(self.text.len());
        &self.text[..byte_idx]
    }

    /// Rect for overlap checking
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        let (w, h) = estimate_text_box(&self.text, self.font_size);
        (self.x - w / 2.0, self.y - h / 2.0, self.x + w / 2.0, self.y + h / 2.0)
    }
}

pub fn estimate_text_box(text: &str, font_size: f32) -> (f32, f32) {
    let cjk_count = text.chars().filter(|c| {
        ('\u{4e00}'..='\u{9fff}').contains(c) || ('\u{ac00}'..='\u{d7af}').contains(c)
    }).count();
    let latin_count = text.chars().count() - cjk_count;
    let width = cjk_count as f32 * font_size * 1.1 + latin_count as f32 * font_size * 0.55;
    let height = font_size * 1.4;
    (width, height)
}

pub fn intersects(a: (f32, f32, f32, f32), b: (f32, f32, f32, f32), pad: f32) -> bool {
    !(a.2 + pad < b.0 || b.2 + pad < a.0 || a.3 + pad < b.1 || b.3 + pad < a.1)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayConfig {
    pub density: f64,
    pub max_active: usize,
    pub seed: u64,
    pub size_min: f32,
    pub size_max: f32,
    pub angle_min: f32,
    pub angle_max: f32,
    pub type_speed: f64,
    pub post_hold: f64,
    pub alpha_max: f32,
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            density: 0.45,
            max_active: 4,
            seed: 42,
            size_min: 18.0,
            size_max: 32.0,
            angle_min: -14.0,
            angle_max: 14.0,
            type_speed: 0.06,
            post_hold: 1.5,
            alpha_max: 0.7,
        }
    }
}
