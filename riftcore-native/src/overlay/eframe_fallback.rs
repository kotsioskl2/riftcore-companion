//! Path A fallback surface host when Path B DirectComposition host initialization fails
//! or on non-Windows hosts.

use crate::overlay::host::{
    OverlayDiagnostics, OverlayHostBackend, OverlayRenderState, OverlaySurfaceHost,
};
use eframe::egui::{self, Pos2, Rect, Vec2};
use std::time::Instant;

/// Fallback overlay surface host preserving lifecycle state, passthrough state, and layout UI
/// calculations via a local `egui::Context` when `Win32OverlayHost::new` returns an error.
pub struct EframeChildViewportFallback {
    visible: bool,
    rect_px: [i32; 4],
    pixels_per_point: f32,
    passthrough: bool,
    frames_rendered: u64,
    resize_events: u64,
    egui_ctx: egui::Context,
    start_time: Instant,
    viewport_title: String,
}

pub type PathAEframeFallbackStub = EframeChildViewportFallback;

impl Default for EframeChildViewportFallback {
    fn default() -> Self {
        let egui_ctx = egui::Context::default();
        egui_ctx.set_zoom_factor(1.0);
        Self {
            visible: false,
            rect_px: [0, 0, 1, 1],
            pixels_per_point: 1.0,
            passthrough: true,
            frames_rendered: 0,
            resize_events: 0,
            egui_ctx,
            start_time: Instant::now(),
            viewport_title: "RiftCore In-Game HUD".to_string(),
        }
    }
}

impl EframeChildViewportFallback {
    pub fn with_viewport_title(mut self, title: impl Into<String>) -> Self {
        self.viewport_title = title.into();
        self
    }

    /// Re-applies Win32 extended styles onto a Path A `eframe` child viewport looked up by title.
    pub fn maintain_child_viewport(&self) {
        if self.visible && !self.viewport_title.is_empty() {
            crate::platform::overlay_window::maintain_native_overlay_by_title(&self.viewport_title);
        }
    }
}

impl OverlaySurfaceHost for EframeChildViewportFallback {
    fn backend_kind(&self) -> OverlayHostBackend {
        OverlayHostBackend::PathAEframeFallback
    }

    fn sync_window(
        &mut self,
        visible: bool,
        rect_px: [i32; 4],
        pixels_per_point: f32,
        passthrough: bool,
    ) {
        if self.rect_px != rect_px {
            let old_size = [
                (self.rect_px[2] - self.rect_px[0]).max(1),
                (self.rect_px[3] - self.rect_px[1]).max(1),
            ];
            let new_size = [
                (rect_px[2] - rect_px[0]).max(1),
                (rect_px[3] - rect_px[1]).max(1),
            ];
            if old_size != new_size {
                self.resize_events += 1;
            }
            self.rect_px = rect_px;
        }
        self.visible = visible;
        self.pixels_per_point = if pixels_per_point.is_finite() {
            pixels_per_point.max(1.0)
        } else {
            1.0
        };
        self.passthrough = passthrough;
    }

    fn render_if_needed(
        &mut self,
        force_redraw: bool,
        state: &mut OverlayRenderState,
        render_ui: &mut dyn FnMut(&mut egui::Ui, &mut OverlayRenderState, &mut Vec<Rect>),
    ) -> Vec<Rect> {
        if !self.visible || !force_redraw {
            return Vec::new();
        }
        let ppp = self.pixels_per_point;
        let w_px = (self.rect_px[2] - self.rect_px[0]).max(1) as f32;
        let h_px = (self.rect_px[3] - self.rect_px[1]).max(1) as f32;
        let screen_pts = Vec2::new(w_px / ppp, h_px / ppp);
        let mut raw_input = egui::RawInput {
            viewport_id: egui::ViewportId::ROOT,
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, screen_pts)),
            time: Some(self.start_time.elapsed().as_secs_f64()),
            focused: false,
            ..Default::default()
        };
        if let Some(vp) = raw_input.viewports.get_mut(&egui::ViewportId::ROOT) {
            vp.native_pixels_per_point = Some(ppp);
        }

        state.host_diag = self.diagnostics();
        let mut rects = Vec::new();
        let mut output = self.egui_ctx.run_ui(raw_input, |ui| {
            render_ui(ui, state, &mut rects);
        });
        // Path A fallback runs without a dedicated wgpu renderer; drain texture deltas cleanly
        // so epaint::TexturesDelta::drop never asserts in debug tests.
        output.textures_delta.clear();
        self.frames_rendered += 1;
        rects
    }

    fn diagnostics(&self) -> OverlayDiagnostics {
        OverlayDiagnostics {
            host_backend: OverlayHostBackend::PathAEframeFallback,
            frames_rendered: self.frames_rendered,
            resize_events: self.resize_events,
            passthrough_active: self.passthrough,
            rect_px: self.rect_px,
            last_surface_status: if self.visible {
                "path_a_fallback_visible".to_string()
            } else {
                "path_a_fallback_hidden".to_string()
            },
            ..Default::default()
        }
    }

    fn close(&mut self) {
        self.visible = false;
    }
}
