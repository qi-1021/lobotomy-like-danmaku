use rand::Rng;
use rand::SeedableRng;
use crate::overlay::{TextOverlay, OverlayConfig, estimate_text_box, intersects};

pub fn generate_overlays(
    texts: &[(String, [f32; 3])],
    duration: f64,
    width: u32,
    height: u32,
    config: &OverlayConfig,
) -> Vec<TextOverlay> {
    if texts.is_empty() || duration <= 0.0 {
        return vec![];
    }
    let mut rng: rand::rngs::StdRng = SeedableRng::seed_from_u64(config.seed);
    let mut overlays: Vec<TextOverlay> = Vec::new();
    let w = width as f32;
    let h = height as f32;

    let estimated_count = (texts.len().max(1) as f64).max(duration * config.density * 3.0) as usize;
    let step = (0.6_f64).max(1.6 - config.density);
    let mut t = 0.4_f64;

    while t < duration - 0.3 && overlays.len() < estimated_count * 2 {
        let active: Vec<_> = overlays.iter()
            .filter(|o| o.start_time <= t && t <= o.end_time)
            .collect();
        if active.len() >= config.max_active {
            t += 0.2;
            continue;
        }

        let idx = rng.gen_range(0..texts.len());
        let (text, color) = &texts[idx];
        let font_size = rng.gen_range(config.size_min..=config.size_max);
        let angle = rng.gen_range(config.angle_min..=config.angle_max);

        let (box_w, box_h) = estimate_text_box(text, font_size);
        let margin_x = (40.0_f32).max(box_w * 0.65);
        let margin_y = (36.0_f32).max(box_h * 1.2);

        let mut placed = false;
        for _ in 0..60 {
            let px = rng.gen_range(margin_x..w - margin_x);
            let py = rng.gen_range(margin_y..h - margin_y);
            let candidate = (px - box_w / 2.0, py - box_h / 2.0, px + box_w / 2.0, py + box_h / 2.0);

            let overlaps = active.iter().any(|o| {
                let (ow, oh) = estimate_text_box(&o.text, o.font_size);
                let rect = (o.x - ow / 2.0, o.y - oh / 2.0, o.x + ow / 2.0, o.y + oh / 2.0);
                intersects(candidate, rect, 18.0)
            });

            if !overlaps {
                let start = t;
                let type_duration = text.chars().count() as f64 * config.type_speed;
                let min_display = type_duration + config.post_hold + 0.5;
                let display = min_display.max(rng.gen_range(2.5..4.5));
                let end = (start + display).min(duration);
                let alpha = (config.alpha_max + rng.gen_range(-0.12..0.12)).clamp(0.15, 1.0);

                let mut overlay = TextOverlay::new(text, font_size, px, py);
                overlay.color = *color;
                overlay.angle = angle;
                overlay.start_time = start;
                overlay.end_time = end;
                overlay.alpha_max = alpha;
                overlay.type_speed = config.type_speed;
                overlay.post_hold = config.post_hold;
                overlays.push(overlay);
                placed = true;
                break;
            }
        }

        if !placed {
            t += 0.2;
            continue;
        }

        let current_active = overlays.iter()
            .filter(|o| o.start_time <= t && t <= o.end_time)
            .count();
        let gap = config.max_active - current_active;
        if gap > 1 {
            t += 0.05;
        } else if gap > 0 {
            t += rng.gen_range(0.1..0.2);
        } else {
            t += rng.gen_range(step * 0.75..step * 1.35);
        }
    }
    overlays
}
