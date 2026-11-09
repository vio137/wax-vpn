//! System tray: closing the window hides it to the tray; the tray menu can
//! reopen the window, toggle the connection, or quit (with a clean disconnect).

#![cfg(windows)]

use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem};
use tray_icon::{Icon, TrayIcon};

pub struct Tray {
    _icon: TrayIcon,
    pub open_id: MenuId,
    pub toggle_id: MenuId,
    pub quit_id: MenuId,
    toggle: MenuItem,
}

impl Tray {
    pub fn new() -> Option<Self> {
        let img = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon-256.png")).ok()?;
        let icon = Icon::from_rgba(img.rgba, img.width, img.height).ok()?;
        let menu = Menu::new();
        let open = MenuItem::new("Open waX VPN", true, None);
        let toggle = MenuItem::new("Connect", true, None);
        let quit = MenuItem::new("Quit waX VPN", true, None);
        menu.append_items(&[&open, &toggle, &quit]).ok()?;
        let _icon = tray_icon::TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("waX VPN")
            .with_icon(icon)
            .build()
            .ok()?;
        Some(Self {
            _icon,
            open_id: open.id().clone(),
            toggle_id: toggle.id().clone(),
            quit_id: quit.id().clone(),
            toggle,
        })
    }

    pub fn set_connected(&self, connected: bool) {
        let _ = self.toggle.set_text(if connected { "Disconnect" } else { "Connect" });
    }
}

pub fn poll_menu() -> Option<MenuEvent> {
    MenuEvent::receiver().try_recv().ok()
}
