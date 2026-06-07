use std::path::Path;

pub struct FontCache {
    fonts: Vec<(String, Vec<u8>, u32)>,  // (name, data, ttc_index)
    cjk_idx: Option<usize>,
    latin_idx: Option<usize>,
}

impl FontCache {
    pub fn new() -> Self {
        Self { fonts: Vec::new(), cjk_idx: None, latin_idx: None }
    }

    fn is_cjk_name(name: &str) -> bool {
        name.contains("pingfang") || name.contains("noto") || name.contains("source")
            || name.contains("msyh") || name.contains("simhei") || name.contains("wqy")
            || name.contains("heiti") || name.contains("songti")
    }

    pub fn load_from_dir(&mut self, dir: &Path) -> anyhow::Result<()> {
        if !dir.is_dir() { return Ok(()); }
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !["ttf", "otf", "ttc"].contains(&ext) { continue; }
            let data = std::fs::read(&path)?;
            let name = path.file_stem().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
            let is_cjk = Self::is_cjk_name(&name);

            if ext == "ttc" {
                // TTC: 尝试每个 index，找到能解析的
                for index in 0..20u32 {
                    match ab_glyph::FontRef::try_from_slice_and_index(&data, index) {
                        Ok(_) => {
                            let idx = self.fonts.len();
                            if is_cjk && self.cjk_idx.is_none() { self.cjk_idx = Some(idx); }
                            if !is_cjk && self.latin_idx.is_none() { self.latin_idx = Some(idx); }
                            self.fonts.push((format!("{}_{}", name, index), data.clone(), index));
                        }
                        Err(_) => {} // 无效 index 跳过
                    }
                }
            } else {
                let idx = self.fonts.len();
                if is_cjk && self.cjk_idx.is_none() { self.cjk_idx = Some(idx); }
                if !is_cjk && self.latin_idx.is_none() { self.latin_idx = Some(idx); }
                if self.latin_idx.is_none() { self.latin_idx = Some(idx); }
                self.fonts.push((name, data, 0));
            }
        }
        Ok(())
    }

    pub fn get_font_data(&self, text: &str) -> Option<(&[u8], u32)> {
        let has_cjk = text.chars().any(|c| {
            ('\u{4e00}'..='\u{9fff}').contains(&c) || ('\u{ac00}'..='\u{d7af}').contains(&c)
        });
        let idx = if has_cjk { self.cjk_idx.or(self.latin_idx) }
                  else { self.latin_idx.or(self.cjk_idx) };
        idx.and_then(|i| self.fonts.get(i).map(|(_, d, index)| (d.as_slice(), *index)))
    }

    pub fn is_empty(&self) -> bool { self.fonts.is_empty() }
    pub fn font_count(&self) -> usize { self.fonts.len() }
}
