use tray_icon::menu::accelerator::{Accelerator, Code, Modifiers};
use tray_icon::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

pub const ID_CONFIG: &str = "config";
pub const ID_SHOW_OVERLAY: &str = "show_overlay";
pub const ID_QUIT: &str = "quit";

pub struct Tray {
    _icon: TrayIcon,
    config_item: MenuItem,
    show_overlay: CheckMenuItem,
}

fn load_icon() -> Option<Icon> {
    let img = image::load_from_memory(include_bytes!("../icons/32x32.png"))
        .ok()?
        .to_rgba8();
    let (w, h) = img.dimensions();
    Icon::from_rgba(img.into_raw(), w, h).ok()
}

impl Tray {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let info = MenuItem::with_id(
            "info",
            format!("Transcendia - v{}", env!("CARGO_PKG_VERSION")),
            false,
            None,
        );
        let config_item = MenuItem::with_id(ID_CONFIG, "Configuration", true, None);
        let show_overlay = CheckMenuItem::with_id(
            ID_SHOW_OVERLAY,
            "Show overlay",
            true,
            true,
            Some(Accelerator::new(
                Modifiers::META | Modifiers::SHIFT,
                Code::KeyT,
            )),
        );
        let quit = MenuItem::with_id(ID_QUIT, "Quit Transcendia", true, None);

        let menu = Menu::new();
        menu.append_items(&[
            &info,
            &PredefinedMenuItem::separator(),
            &config_item,
            &show_overlay,
            &PredefinedMenuItem::separator(),
            &quit,
        ])?;

        let mut builder = TrayIconBuilder::new()
            .with_id("main")
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(true)
            .with_tooltip("Transcendia");
        if let Some(icon) = load_icon() {
            builder = builder.with_icon(icon);
        } else {
            log::warn!("Cannot decode tray icon");
        }
        let icon = builder.build()?;

        Ok(Self {
            _icon: icon,
            config_item,
            show_overlay,
        })
    }

    pub fn set_config_enabled(&self, enabled: bool) {
        self.config_item.set_enabled(enabled);
    }

    pub fn set_overlay_checked(&self, checked: bool) {
        self.show_overlay.set_checked(checked);
    }
}
