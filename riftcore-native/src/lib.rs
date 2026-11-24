// RiftCore Native — crate root (library)
// All public modules are declared here; binaries import via `riftcore_native::*`.
// Wave 0: skeleton stubs only. Monolith files (app.rs, backend.rs, builds.rs, tft_data.rs)
// are NOT yet deleted — they remain compiled via main.rs until Wave 3 integration.

pub mod assets;
pub mod config;
pub mod domain;
pub mod overlay;
pub mod platform;
pub mod services;
pub mod ui;
