fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        // Embed the waX icon into the .exe so Explorer, the taskbar,
        // installer shortcuts and the UAC prompt show it.
        if let Err(e) = embed_resource::compile("assets/wax.rc", embed_resource::NONE).manifest_optional() {
            panic!("failed to compile the Windows icon resource: {e:?}");
        }
    }
}
