use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TextSpecEntry {
    Object { text: String, color: ColorSpec },
    Array(String, ColorSpec),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ColorSpec {
    Hex(String),
    Rgb(Vec<f64>),
}

impl ColorSpec {
    pub fn to_rgb01(&self) -> [f32; 3] {
        match self {
            ColorSpec::Hex(s) => {
                let s = s.trim_start_matches('#');
                if s.len() >= 6 {
                    let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(180) as f32 / 255.0;
                    let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(50) as f32 / 255.0;
                    let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(50) as f32 / 255.0;
                    [r, g, b]
                } else {
                    [0.7, 0.2, 0.2]
                }
            }
            ColorSpec::Rgb(v) => {
                let r = v.get(0).copied().unwrap_or(180.0) as f32 / 255.0;
                let g = v.get(1).copied().unwrap_or(50.0) as f32 / 255.0;
                let b = v.get(2).copied().unwrap_or(50.0) as f32 / 255.0;
                [r, g, b]
            }
        }
    }
}

pub fn parse_color(s: &str) -> [f32; 3] {
    ColorSpec::Hex(s.to_string()).to_rgb01()
}

pub fn load_text_specs(path: &str) -> anyhow::Result<Vec<(String, [f32; 3])>> {
    let data = std::fs::read_to_string(path)?;
    let entries: Vec<TextSpecEntry> = serde_json::from_str(&data)?;
    let mut specs = Vec::new();
    for entry in entries {
        match entry {
            TextSpecEntry::Object { text, color } => {
                specs.push((text, color.to_rgb01()));
            }
            TextSpecEntry::Array(text, color) => {
                specs.push((text, color.to_rgb01()));
            }
        }
    }
    Ok(specs)
}
