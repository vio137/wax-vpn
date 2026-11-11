mod startup {
    use std::io::Write;
    pub fn record(message: &str) {
        let dir = wax_core::profile::config_dir();
        if std::fs::create_dir_all(&dir).is_err() { return; }
        let path = dir.join("startup.log");
        let truncate = std::fs::metadata(&path).is_ok_and(|m| m.len() > 65536);
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).write(true)
            .append(!truncate).truncate(truncate).open(path) {
            let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
            let _ = writeln!(file, "{timestamp} pid={} {}", std::process::id(), message);
            let _ = file.flush();
        }
    }
    pub fn alert(message: &str) {
        #[cfg(windows)]
        unsafe {
            use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
            let body = format!("{message}\n\nLocal details: %APPDATA%\\wax-vpn\\startup.log\nTry the Safe graphics shortcut if normal startup fails. No data was uploaded.");
            let body: Vec<u16> = body.encode_utf16().chain(Some(0)).collect();
            let title: Vec<u16> = "waX VPN startup problem".encode_utf16().chain(Some(0)).collect();
            MessageBoxW(0, body.as_ptr(), title.as_ptr(), MB_OK | MB_ICONERROR);
        }
        #[cfg(not(windows))]
        eprintln!("{message}");
    }
    pub fn install() {
        record(concat!("launch version=", env!("WAX_BUILD_VERSION")));
        std::panic::set_hook(Box::new(|info| {
            if let Some(location) = info.location() {
                record(&format!("rust panic at {}:{} (payload omitted)", location.file(), location.line()));
            } else { record("rust panic (payload omitted)"); }
            alert("The app encountered a Rust error and must close. Your saved profiles were not reset.");
        }));
    }
}

fn main() -> eframe::Result<()> {
    startup::install();
    if std::env::args_os().any(|a| a == "--quit") {
        #[cfg(windows)]
        if !single::request_quit() {
            eprintln!("waX VPN is not running");
        }
        return Ok(());
    }
    #[cfg(windows)]
    if elevation::finish_restart().is_err() {
        eprintln!("Administrator restart could not complete its bounded parent handoff");
        return Ok(());
    }
    #[cfg(windows)]
    let instance = match single::acquire() {
        single::Acquire::Primary(inst) => Some(inst),
        single::Acquire::AlreadyRunning => return Ok(()),
    };
    #[cfg(not(windows))]
    let instance: Option<single::Instance> = None;
    startup::record("single-instance acquired; decoding icon");
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon-256.png"))
        .expect("embedded icon must decode");
    if std::env::args_os().any(|arg| arg == "--software-graphics") {
        startup::record("renderer=CPU software (GDI on Windows)");
        let mut instance = instance;
        let config = egui_software_backend::SoftwareBackendAppConfiguration::new()
            .viewport_builder(egui::ViewportBuilder::default()
                .with_title("waX VPN").with_app_id("wax-vpn").with_icon(icon)
                .with_inner_size([880.0, 600.0]).with_min_inner_size([760.0, 540.0])
                .with_resizable(true).with_decorations(false));
        let result = egui_software_backend::run_app_with_software_backend(config, |ctx| {
            startup::record("software surface ready; app creation starting");
            let app = WaxApp::from_context(&ctx, instance.take());
            startup::record("app creation complete");
            app
        });
        if let Err(error) = &result {
            startup::record(&format!("software window error: {error}"));
            startup::alert("The software-rendered window could not start.");
        } else { startup::record("software event loop returned normally"); }
        return Ok(());
    }
    let safe_graphics = std::env::args_os().any(|arg| arg == "--safe-graphics");
    startup::record(if safe_graphics { "renderer=OpenGL compatibility" } else { "renderer=wgpu" });
    let options = eframe::NativeOptions {
        renderer: if safe_graphics { eframe::Renderer::Glow } else { eframe::Renderer::Wgpu },
        viewport: egui::ViewportBuilder::default()
            .with_title("waX VPN")
            .with_app_id("wax-vpn")
            .with_icon(icon)
            .with_inner_size([880.0, 600.0])
            .with_min_inner_size([760.0, 540.0])
            .with_resizable(true)
            .with_decorations(false),
        ..Default::default()
    };
    let result = eframe::run_native(
        "waX VPN",
        options,
        Box::new(|cc| {
            startup::record("graphics context ready; app creation starting");
            let app = WaxApp::new(cc, instance);
            startup::record("app creation complete");
            Ok(Box::new(app))
        }),
    );
    if let Err(error) = &result {
        // eframe errors here describe window/graphics setup, not profile contents.
        startup::record(&format!("window/graphics error: {}", error.to_string().chars().take(1600).collect::<String>()));
        startup::alert("The window or graphics system could not start. Try the Safe graphics shortcut.");
    } else { startup::record("event loop returned normally"); }
    result
              }
