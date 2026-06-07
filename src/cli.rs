use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "lobotomy-danmaku")]
pub struct CliArgs {
    #[arg(long)]
    pub headless: bool,
    #[arg(short, long)]
    pub input: Option<String>,
    #[arg(short, long)]
    pub output: Option<String>,
    #[arg(long)]
    pub video: Option<String>,
    #[arg(short, long, default_value = "12")]
    pub duration: f64,
    #[arg(long, default_value = "1280")]
    pub width: u32,
    #[arg(long, default_value = "720")]
    pub height: u32,
    #[arg(long, default_value = "30")]
    pub fps: u32,
    #[arg(long, default_value = "/tmp/lobotomy_extract/fonts_proper")]
    pub font_dir: String,
}

pub fn parse_args() -> CliArgs {
    CliArgs::parse()
}

pub fn run_headless(args: CliArgs) {
    let mut manager = crate::text_overlay::TextOverlayManager::new();
    if let Some(input) = &args.input {
        if let Ok(json) = std::fs::read_to_string(input) {
            manager.load_from_json(&json).ok();
        }
    }

    let video_path = args.video.unwrap_or_else(|| "test_input.mp4".to_string());
    let output_path = args.output.unwrap_or_else(|| "output.mp4".to_string());

    println!("Headless: {} overlays", manager.get_all().len());
    println!("Video: {} -> {}", video_path, output_path);

    // Probe video if exists
    if std::path::Path::new(&video_path).exists() {
        match crate::video::probe_video(&video_path) {
            Ok(info) => {
                println!("Video info: {}x{} @ {:.1}fps, {:.1}s", info.width, info.height, info.fps, info.duration);
                match crate::video::process_video(
                    &video_path, &output_path, manager.get_all(), &args.font_dir,
                    info.width, info.height, info.fps,
                ) {
                    Ok(()) => println!("Export complete!"),
                    Err(e) => eprintln!("Export error: {}", e),
                }
            }
            Err(e) => eprintln!("Probe error: {}", e),
        }
    } else {
        eprintln!("Video file not found: {}", video_path);
        // Generate with default settings
        match crate::video::process_video(
            &video_path, &output_path, manager.get_all(), &args.font_dir,
            args.width, args.height, args.fps as f64,
        ) {
            Ok(()) => println!("Export complete!"),
            Err(e) => eprintln!("Export error: {}", e),
        }
    }
}
