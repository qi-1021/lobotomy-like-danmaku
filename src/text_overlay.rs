use serde::{Deserialize, Serialize};
use rand::Rng;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextOverlay {
    pub id: u64,
    pub text: String,
    pub font_size: f32,
    pub color: [f32; 3],
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub start_time: f64,
    pub end_time: f64,
    pub alpha_max: f32,
    pub visible: bool,
}

pub struct TextOverlayManager {
    overlays: Vec<TextOverlay>,
    next_id: u64,
}

impl TextOverlayManager {
    pub fn new() -> Self {
        Self { overlays: Vec::new(), next_id: 1 }
    }

    pub fn update(&mut self, current_time: f64) {
        for overlay in &mut self.overlays {
            overlay.visible = current_time >= overlay.start_time && current_time < overlay.end_time;
        }
    }

    pub fn get_visible(&self) -> Vec<&TextOverlay> {
        self.overlays.iter().filter(|o| o.visible).collect()
    }

    pub fn get_all(&self) -> &[TextOverlay] {
        &self.overlays
    }

    pub fn add(&mut self, overlay: TextOverlay) {
        let mut o = overlay;
        o.id = self.next_id;
        self.next_id += 1;
        self.overlays.push(o);
    }

    pub fn remove(&mut self, id: u64) {
        self.overlays.retain(|o| o.id != id);
    }

    pub fn update_overlay(&mut self, id: u64, overlay: TextOverlay) {
        if let Some(o) = self.overlays.iter_mut().find(|o| o.id == id) {
            *o = overlay;
        }
    }

    pub fn clear(&mut self) {
        self.overlays.clear();
    }

    pub fn load_from_json(&mut self, json: &str) -> Result<(), Box<dyn std::error::Error>> {
        let overlays: Vec<TextOverlay> = serde_json::from_str(json)?;
        self.overlays = overlays;
        self.next_id = self.overlays.iter().map(|o| o.id).max().unwrap_or(0) + 1;
        Ok(())
    }

    pub fn save_to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string_pretty(&self.overlays)?)
    }

    /// Generate random overlays based on text list and video duration
    pub fn generate_random(&mut self, texts: &[(String, [f32; 3])], duration: f64, width: f32, height: f32) {
        let mut rng = rand::thread_rng();
        let mut t = 0.5;
        while t < duration - 2.0 {
            for (text, color) in texts {
                if t >= duration - 2.0 { break; }
                let font_size = rng.gen_range(14.0..40.0);
                let x = rng.gen_range(font_size * 2.0..(width - font_size * 6.0));
                let y = rng.gen_range(font_size * 2.0..(height - font_size * 2.0));
                let angle = rng.gen_range(-18.0..18.0);
                let display_time = rng.gen_range(2.0..5.0);
                let alpha = rng.gen_range(0.5..0.95);
                let mut overlay = TextOverlay::new(text, font_size, x, y);
                overlay.color = *color;
                overlay.angle = angle;
                overlay.start_time = t;
                overlay.end_time = t + display_time;
                overlay.alpha_max = alpha;
                self.add(overlay);
                t += rng.gen_range(0.3..1.5);
            }
        }
    }
}

impl TextOverlay {
    pub fn new(text: &str, font_size: f32, x: f32, y: f32) -> Self {
        Self {
            id: 0, text: text.to_string(), font_size, color: [0.7, 0.7, 0.75],
            x, y, angle: 0.0, start_time: 0.0, end_time: 10.0, alpha_max: 0.8, visible: true,
        }
    }

    pub fn ease_out(t: f32) -> f32 { 1.0 - (1.0 - t).powi(3) }
    pub fn ease_in(t: f32) -> f32 { t.powi(3) }

    pub fn get_alpha(&self, t: f64) -> f32 {
        if t < self.start_time || t >= self.end_time { return 0.0; }
        let fade_in = 0.5;
        let fade_out = 0.4;
        let local_start = t - self.start_time;
        let time_until_end = self.end_time - t;
        if local_start < fade_in {
            Self::ease_out((local_start / fade_in) as f32) * self.alpha_max
        } else if time_until_end < fade_out {
            Self::ease_in((time_until_end / fade_out) as f32) * self.alpha_max
        } else {
            self.alpha_max
        }
    }
}
