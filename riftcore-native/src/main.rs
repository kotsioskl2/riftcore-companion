#![windows_subsystem = "windows"]

mod app;
mod assets;
mod backend;
mod builds;
mod tft_data;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_app_id("riftcore-companion")
            .with_title("RiftCore Companion")
            .with_inner_size([1380.0, 880.0])
            .with_min_inner_size([1120.0, 740.0])
            .with_decorations(false)
            .with_transparent(false),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    eframe::run_native(
        "RiftCore Companion",
        options,
        Box::new(|creation| Ok(Box::new(app::RiftCoreApp::new(creation)))),
    )
}
