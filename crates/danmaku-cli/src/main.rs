use clap::Parser;
use anyhow::Result;
use danmaku_core::overlay::OverlayConfig;
use danmaku_core::text_specs;

#[derive(Parser)]
#[command(name = "danmaku", about = "Lobotomy Corporation Danmaku Video Generator")]
struct Cli {
    #[arg(short, long)]
    input: String,
    #[arg(short, long, default_value = "output.mp4")]
    output: String,
    #[arg(long)]
    texts: Option<String>,
    #[arg(long)]
    text: Vec<String>,
    #[arg(long, default_value = "#b43c3c")]
    color: String,
    #[arg(long, default_value = "0.45")]
    density: f64,
    #[arg(long, default_value = "4")]
    max_active: usize,
    #[arg(long, default_value = "18")]
    size_min: f32,
    #[arg(long, default_value = "32")]
    size_max: f32,
    #[arg(long, default_value = "-14")]
    angle_min: f32,
    #[arg(long, default_value = "14")]
    angle_max: f32,
    #[arg(long, default_value = "0.06")]
    type_speed: f64,
    #[arg(long, default_value = "1.5")]
    post_hold: f64,
    #[arg(long, default_value = "42")]
    seed: u64,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let info = danmaku_core::video::probe_video(&cli.input)?;
    eprintln!("Video: {}x{} @ {:.0}fps, {:.1}s", info.width, info.height, info.fps, info.duration);

    let mut font_cache = danmaku_core::font::FontCache::new();
    font_cache.load_from_dir(std::path::Path::new("fonts_proper"))?;
    eprintln!("Fonts: {}", font_cache.font_count());

    let text_specs = if let Some(path) = &cli.texts {
        text_specs::load_text_specs(path)?
    } else if !cli.text.is_empty() {
        let color = text_specs::parse_color(&cli.color);
        cli.text.iter().map(|t| (t.clone(), color)).collect()
    } else {
        eprintln!("Error: must specify --texts or --text");
        std::process::exit(1);
    };

    if text_specs.is_empty() {
        eprintln!("Error: no texts");
        std::process::exit(1);
    }

    let config = OverlayConfig {
        density: cli.density,
        max_active: cli.max_active,
        seed: cli.seed,
        size_min: cli.size_min,
        size_max: cli.size_max,
        angle_min: cli.angle_min,
        angle_max: cli.angle_max,
        type_speed: cli.type_speed,
        post_hold: cli.post_hold,
        alpha_max: 0.7,
    };

    let overlays = danmaku_core::generator::generate_overlays(
        &text_specs, info.duration, info.width, info.height, &config,
    );
    eprintln!("Overlays: {}", overlays.len());

    danmaku_core::video::process_video(
        &cli.input, &cli.output, &overlays, &font_cache,
        info.width, info.height, info.fps,
        Some(&|cur, total| eprint!("\rProcessing {}/{}", cur, total)),
    )?;
    eprintln!("\nDone: {}", cli.output);
    Ok(())
}
