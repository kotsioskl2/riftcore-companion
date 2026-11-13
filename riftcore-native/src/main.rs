#![windows_subsystem = "windows"]

mod app;
mod assets;
mod backend;
mod builds;
mod tft_data;

fn main() -> eframe::Result {
    let image = image::load_from_memory(crate::assets::RIFTCORE_LOGO)
        .expect("embedded RiftCore logo must be a valid image")
        .resize(64, 64, image::imageops::FilterType::Lanczos3)
        .to_rgba8();
    let app_icon = std::sync::Arc::new(eframe::egui::IconData {
        width: image.width(),
        height: image.height(),
        rgba: image.into_raw(),
    });
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_app_id("riftcore-companion")
            .with_title("RiftCore Companion")
            .with_icon(app_icon)
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
