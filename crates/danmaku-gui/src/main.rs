mod app;
mod panels;

fn main() {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 860.0])
            .with_title("脑叶公司 - 弹幕编辑器"),
        ..Default::default()
    };
    eframe::run_native(
        "Lobotomy Danmaku",
        options,
        Box::new(|_cc| Ok(Box::new(app::DanmakuApp::default()))),
    ).unwrap();
}
