use std::path::Path;

/// Cached font data for rendering
pub struct FontCache {
    fonts: Vec<(String, Vec<u8>)>,
    cjk_idx: Option<usize>,
    latin_idx: Option<usize>,
}

impl FontCache {
    pub fn new() -> Self {
        Self { fonts: Vec::new(), cjk_idx: None, latin_idx: None }
    }

    pub fn load_from_dir(&mut self, dir: &Path) -> anyhow::Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !["ttf", "otf", "ttc"].contains(&ext) {
                continue;
            }
            let data = std::fs::read(&path)?;
            let name = path.file_stem().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
            let idx = self.fonts.len();
            
            if name.contains("pingfang") || name.contains("noto") || name.contains("source")
                || name.contains("msyh") || name.contains("simhei") || name.contains("wqy")
            {
                if self.cjk_idx.is_none() { self.cjk_idx = Some(idx); }
            } else if name.contains("norwester") || name.contains("arial")
                || name.contains("helvetica") || name.contains("dejavu")
            {
                if self.latin_idx.is_none() { self.latin_idx = Some(idx); }
            }
            
            if self.latin_idx.is_none() { self.latin_idx = Some(idx); }
            self.fonts.push((name, data));
        }
        Ok(())
    }

    pub fn get_font_data(&self, text: &str) -> Option<(&[u8], usize)> {
        let has_cjk = text.chars().any(|c| {
            ('\u{4e00}'..='\u{9fff}').contains(&c) || ('\u{ac00}'..='\u{d7af}').contains(&c)
        });
        let idx = if has_cjk {
            self.cjk_idx.or(self.latin_idx)
        } else {
            self.latin_idx.or(self.cjk_idx)
        };
        idx.and_then(|i| self.fonts.get(i).map(|(_, d)| (d.as_slice(), i)))
    }

    pub fn get_font_index(&self, text: &str) -> usize {
        let has_cjk = text.chars().any(|c| {
            ('\u{4e00}'..='\u{9fff}').contains(&c) || ('\u{ac00}'..='\u{d7af}').contains(&c)
        });
        if has_cjk {
            self.cjk_idx.or(self.latin_idx).unwrap_or(0)
        } else {
            self.latin_idx.or(self.cjk_idx).unwrap_or(0)
        }
    }

    pub fn is_empty(&self) -> bool { self.fonts.is_empty() }
    pub fn font_count(&self) -> usize { self.fonts.len() }
}
