use std::io::{Read, Write};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct VideoInfo {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub duration: f64,
}

pub fn probe_video(path: &str) -> Result<VideoInfo> {
    let output = std::process::Command::new("ffprobe")
        .args(["-v", "quiet", "-print_format", "json", "-show_streams", "-show_format", path])
        .output()?;
    let info: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let streams = info["streams"].as_array().ok_or_else(|| anyhow::anyhow!("No streams"))?;
    let vs = streams.iter()
        .find(|s| s["codec_type"].as_str() == Some("video"))
        .ok_or_else(|| anyhow::anyhow!("No video stream"))?;
    let width = vs["width"].as_u64().unwrap_or(1280) as u32;
    let height = vs["height"].as_u64().unwrap_or(720) as u32;
    let fps_str = vs["r_frame_rate"].as_str().unwrap_or("30/1");
    let fps = fps_str.split_once('/')
        .map(|(n, d)| n.parse::<f64>().unwrap_or(30.0) / d.parse::<f64>().unwrap_or(1.0).max(1.0))
        .unwrap_or(30.0);
    let duration = info["format"]["duration"].as_str()
        .and_then(|d| d.parse::<f64>().ok()).unwrap_or(10.0);
    Ok(VideoInfo { width, height, fps, duration })
}

pub fn process_video(
    input: &str, output: &str, overlays: &[crate::overlay::TextOverlay],
    font_cache: &crate::font::FontCache, width: u32, height: u32, fps: f64,
    progress_cb: Option<&dyn Fn(u64, u64)>,
) -> Result<()> {
    use image::RgbaImage;
    use crate::render::render_frame;

    let info = probe_video(input)?;
    let total_frames = (info.duration * fps).ceil() as u64;
    let frame_size = (width * height * 4) as usize;

    let mut decoder = std::process::Command::new("ffmpeg")
        .args(["-i", input, "-f", "rawvideo", "-pix_fmt", "rgba",
               "-s", &format!("{}x{}", width, height), "-an", "-"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    let tmp = format!("{}.tmp.mp4", output);
    let mut encoder = std::process::Command::new("ffmpeg")
        .args(["-y", "-f", "rawvideo", "-pix_fmt", "rgba",
               "-s", &format!("{}x{}", width, height), "-r", &fps.to_string(),
               "-i", "pipe:0",
               "-i", input, "-map", "0:v", "-map", "1:a?",
               "-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", "18",
               "-c:a", "aac", "-shortest", &tmp])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    let decoder_stdout = decoder.stdout.as_mut().ok_or_else(|| anyhow::anyhow!("No stdout"))?;
    let encoder_stdin = encoder.stdin.as_mut().ok_or_else(|| anyhow::anyhow!("No stdin"))?;

    let mut frame_buf = vec![0u8; frame_size];
    let mut frame_idx = 0u64;

    loop {
        let bytes_read = decoder_stdout.read(&mut frame_buf)?;
        if bytes_read < frame_size { break; }

        let t = frame_idx as f64 / fps;
        let mut img = RgbaImage::from_raw(width, height, frame_buf.clone())
            .ok_or_else(|| anyhow::anyhow!("Failed to create image"))?;
        render_frame(&mut img, overlays, t, font_cache);
        encoder_stdin.write_all(img.as_raw())?;

        frame_idx += 1;
        if let Some(cb) = progress_cb { cb(frame_idx, total_frames); }
    }

    drop(encoder_stdin);
    decoder.wait()?;
    encoder.wait()?;

    std::fs::rename(&tmp, output)?;
    Ok(())
}
