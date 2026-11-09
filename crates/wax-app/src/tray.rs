//! System tray: closing the window hides it to the tray; the tray menu can
//! reopen the window, toggle the connection, or quit (with a clean disconnect).

#![cfg(windows)]

use std::sync::mpsc::{channel, Receiver};

use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem};
use tray_icon::{Icon, TrayIcon};

pub struct Tray {
    _icon: TrayIcon,
    events: Receiver<MenuEvent>,
    pub open_id: MenuId,
    pub toggle_id: MenuId,
    pub quit_id: MenuId,
    toggle: MenuItem,
}

impl Tray {
    /// `ctx` is woken the moment a tray menu event arrives, so menu clicks
    /// stay responsive even while the hidden window repaints slowly.
    pub fn new(ctx: &egui::Context) -> Option<Self> {
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
        let (tx, events) = channel();
        let wake = ctx.clone();
        std::thread::spawn(move || {
            while let Ok(ev) = MenuEvent::receiver().recv() {
                if tx.send(ev).is_err() {
                    break;
                }
                wake.request_repaint();
            }
        });
        Some(Self {
            _icon,
            events,
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

    pub fn poll(&self) -> Option<MenuEvent> {
        self.events.try_recv().ok()
    }
