use std::process::{Command, Stdio};
use std::io::{Write, Read};
use std::fs::File;

pub struct VideoInfo {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub duration: f64,
}

pub fn probe_video(path: &str) -> Result<VideoInfo, Box<dyn std::error::Error>> {
    let output = Command::new("ffprobe")
        .args(["-v", "quiet", "-print_format", "json", "-show_streams", "-show_format", path])
        .output()?;
    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let vs = json["streams"].as_array()
        .and_then(|s| s.iter().find(|s| s["codec_type"] == "video"))
        .ok_or("No video stream")?;
    let w = vs["width"].as_u64().unwrap_or(1280) as u32;
    let h = vs["height"].as_u64().unwrap_or(720) as u32;
    let rfr = vs["r_frame_rate"].as_str().unwrap_or("30/1");
    let parts: Vec<&str> = rfr.split('/').collect();
    let fps = if parts.len() == 2 {
        parts[0].parse::<f64>().unwrap_or(30.0) / parts[1].parse::<f64>().unwrap_or(1.0).max(1.0)
    } else { rfr.parse().unwrap_or(30.0) };
    let dur: f64 = json["format"]["duration"].as_str().and_then(|s| s.parse().ok()).unwrap_or(10.0);
    println!("Video: {}x{} @ {:.1}fps, {:.1}s", w, h, fps, dur);
    Ok(VideoInfo { width: w, height: h, fps, duration: dur })
}

pub fn process_video(
    input: &str, output: &str,
    overlays: &[crate::text_overlay::TextOverlay],
    font_dir: &str, width: u32, height: u32, fps: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut font_mgr = crate::fonts::FontManager::new();
    font_mgr.load_from_dir(font_dir);

    let max_t = overlays.iter().map(|o| o.end_time).fold(0.0f64, f64::max);
    let total_frames = ((max_t * fps) as u32).max(1);
    println!("Processing {} frames ({:.1}s)...", total_frames, max_t);

    // Write frames to temp file, then encode with ffmpeg
    let tmp_frames = "/tmp/lobotomy_extract/tmp_frames.raw";
    let mut frame_file = File::create(tmp_frames)?;

    // Read from video using ffmpeg
    let mut decoder = Command::new("ffmpeg")
        .args(["-i", input, "-f", "rawvideo", "-pix_fmt", "rgba",
               "-s", &format!("{}x{}", width, height), "-v", "quiet", "-"])
        .stdout(Stdio::piped())
        .spawn()?;

    let frame_size = (width * height * 4) as usize;
    let mut frame_buf = vec![0u8; frame_size];
    let mut frame_num: u32 = 0;
    let dec_out = decoder.stdout.as_mut().unwrap();

    loop {
        let mut total_read = 0;
        while total_read < frame_size {
            match dec_out.read(&mut frame_buf[total_read..]) {
                Ok(0) => break,
                Ok(n) => total_read += n,
                Err(_) => break,
            }
        }
        if total_read < frame_size { break; }

        let t = frame_num as f64 / fps;
        let mut img = image::RgbaImage::from_raw(width, height, frame_buf.clone()).unwrap();

        for o in overlays {
            let alpha = o.get_alpha(t);
            if alpha < 0.01 { continue; }
            font_mgr.render_text(&mut img, &o.text, o.font_size, o.x, o.y, o.angle, o.color, alpha);
        }

        frame_file.write_all(&img.into_raw())?;
        frame_num += 1;
        if frame_num % 30 == 0 {
            println!("  Frame {}/{}", frame_num, total_frames);
        }
        if frame_num >= total_frames { break; }
    }
    let _ = decoder.kill();
    drop(frame_file);

    println!("Frames written. Encoding with FFmpeg...");

    // Encode with ffmpeg
    let status = Command::new("ffmpeg")
        .args(["-y", "-f", "rawvideo", "-pix_fmt", "rgba",
               "-s", &format!("{}x{}", width, height),
               "-r", &fps.to_string(),
               "-i", tmp_frames,
               "-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", "18",
               "-v", "quiet",
               output])
        .status()?;

    std::fs::remove_file(tmp_frames).ok();

    if status.success() {
        println!("Done! Output: {}", output);
    } else {
        eprintln!("FFmpeg encoding failed!");
    }
    Ok(())
}
