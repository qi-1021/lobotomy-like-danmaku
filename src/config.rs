use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub font_dir: PathBuf,
    pub output_dir: PathBuf,
    pub bg_color: [u8; 3],
    pub video_path: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            fps: 30,
            font_dir: PathBuf::from("/tmp/lobotomy_extract/fonts_proper"),
            output_dir: PathBuf::from("/tmp/lobotomy_extract/output"),
            bg_color: [12, 10, 18],
            video_path: None,
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = std::env::current_dir()?.join("config.json");
        if path.exists() {
            let data = std::fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&data)?)
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = std::env::current_dir()?.join("config.json");
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }
}
