#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrayAction {
    ShowWindow,
    ToggleHud,
    ExitApp,
}

/// Manages the Windows system tray icon and context menu independently of the
/// root `eframe` window lifecycle so closing to tray keeps background monitoring alive.
pub struct TrayController {
    #[cfg(windows)]
    tray_icon: Option<tray_icon::TrayIcon>,
}

impl TrayController {
    /// Initializes a system tray icon with the RiftCore context menu (`Open RiftCore`,
    /// `Toggle in-game HUD`, `Exit RiftCore`).
    /// When `icon_rgba` is `None`, falls back to a minimal 1x1 transparent icon.
    pub fn new(tooltip: &str, icon_rgba: Option<(Vec<u8>, u32, u32)>) -> Result<Self, String> {
        #[cfg(windows)]
        {
            use tray_icon::{
                menu::{Menu, MenuItem},
                Icon, TrayIconBuilder,
            };

            let menu = Menu::new();
            let show = MenuItem::with_id("riftcore-show", "Open RiftCore", true, None);
            let hud = MenuItem::with_id("riftcore-hud", "Toggle in-game HUD", true, None);
            let exit = MenuItem::with_id("riftcore-exit", "Exit RiftCore", true, None);
            menu.append(&show)
                .map_err(|err| format!("Failed to add tray menu item: {err}"))?;
            menu.append(&hud)
                .map_err(|err| format!("Failed to add tray menu item: {err}"))?;
            menu.append(&exit)
                .map_err(|err| format!("Failed to add tray menu item: {err}"))?;

            let (rgba, width, height) = icon_rgba.unwrap_or_else(|| (vec![0, 0, 0, 0], 1, 1));
            let icon = Icon::from_rgba(rgba, width, height)
                .map_err(|err| format!("Invalid tray icon RGBA payload: {err}"))?;

            let tray_icon = TrayIconBuilder::new()
                .with_menu(Box::new(menu))
                .with_icon(icon)
                .with_tooltip(tooltip)
                .build()
                .map_err(|err| format!("Failed to build system tray icon: {err}"))?;

            Ok(Self {
                tray_icon: Some(tray_icon),
            })
        }

        #[cfg(not(windows))]
        {
            let _ = (tooltip, icon_rgba);
            Ok(Self {})
        }
    }

    pub fn unavailable() -> Self {
        #[cfg(windows)]
        {
            Self { tray_icon: None }
        }
        #[cfg(not(windows))]
        {
            Self {}
        }
    }

    pub fn is_available(&self) -> bool {
        #[cfg(windows)]
        {
            self.tray_icon.is_some()
        }
        #[cfg(not(windows))]
        {
            false
        }
    }

    pub fn set_tooltip(&self, tooltip: &str) {
        #[cfg(windows)]
        if let Some(tray) = &self.tray_icon {
            let _ = tray.set_tooltip(Some(tooltip));
        }
        #[cfg(not(windows))]
        let _ = tooltip;
    }

    /// Drains pending tray menu and tray click events without reacting to hover/move noise.
    pub fn poll_actions(&self) -> Vec<TrayAction> {
        let mut actions = Vec::new();
        #[cfg(windows)]
        {
            use tray_icon::{menu::MenuEvent, MouseButton, MouseButtonState, TrayIconEvent};

            while let Ok(event) = MenuEvent::receiver().try_recv() {
                match event.id.as_ref() {
                    "riftcore-show" => actions.push(TrayAction::ShowWindow),
                    "riftcore-hud" => actions.push(TrayAction::ToggleHud),
                    "riftcore-exit" => actions.push(TrayAction::ExitApp),
                    _ => {}
                }
            }
            while let Ok(event) = TrayIconEvent::receiver().try_recv() {
                match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    }
                    | TrayIconEvent::DoubleClick {
                        button: MouseButton::Left,
                        ..
                    } => actions.push(TrayAction::ShowWindow),
                    _ => {}
                }
            }
        }
        actions
    }
}
