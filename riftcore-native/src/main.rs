#![windows_subsystem = "windows"]

mod app;
mod assets;
mod backend;
mod builds;
mod tft_data;

use riftcore_native::overlay::host::build_dx12_visual_wgpu_options;
use riftcore_native::ui::shell::{RiftCoreShell, UiRuntime};

fn main() -> eframe::Result {
    let runtime = UiRuntime::resolve(
        std::env::args(),
        std::env::var("RIFTCORE_UI").ok().as_deref(),
    );
    if runtime == UiRuntime::Modular {
        match run_modular() {
            Ok(()) => return Ok(()),
            Err(err) => {
                // Path B needs DX12 + DxgiFromVisual; any startup failure falls back to the
                // proven legacy runtime instead of leaving the user without an app.
                eprintln!(
                    "RiftCore: modular UI failed to start ({err}); falling back to legacy UI"
                );
            }
        }
    }
    run_legacy()
}

fn app_icon() -> std::sync::Arc<eframe::egui::IconData> {
    let image = image::load_from_memory(crate::assets::RIFTCORE_LOGO)
        .expect("embedded RiftCore logo must be a valid image")
        .resize(64, 64, image::imageops::FilterType::Lanczos3)
        .to_rgba8();
    std::sync::Arc::new(eframe::egui::IconData {
        width: image.width(),
        height: image.height(),
        rgba: image.into_raw(),
    })
}

fn base_viewport() -> eframe::egui::ViewportBuilder {
    eframe::egui::ViewportBuilder::default()
        .with_app_id("riftcore-companion")
        .with_title("RiftCore Companion")
        .with_icon(app_icon())
        .with_inner_size([1380.0, 880.0])
        .with_min_inner_size([1120.0, 740.0])
        .with_decorations(false)
}

/// Modular shell: opaque root window; the HUD lives in the separate Path B
/// DirectComposition window, so the root needs no transparency.
fn run_modular() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: base_viewport(),
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: build_dx12_visual_wgpu_options(),
        ..Default::default()
    };
    eframe::run_native(
        "RiftCore Companion",
        options,
        Box::new(|creation| {
            RiftCoreShell::new(creation)
                .map(|shell| Box::new(shell) as Box<dyn eframe::App>)
                .map_err(Into::into)
        }),
    )
}

fn run_legacy() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: base_viewport()
            // The HUD child viewport shares the Glow framebuffer config with the
            // root. Request alpha here so the child can composite over League;
            // the app's own panels paint an opaque surface over this clear color.
            .with_transparent(true),
        // Request an alpha-capable Glow surface for the per-pixel HUD layer.
        // One sample also avoids a black transparent surface on some Windows GPUs.
        multisampling: 1,
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "RiftCore Companion",
        options,
        Box::new(|creation| Ok(Box::new(app::RiftCoreApp::new(creation)))),
    )
}
