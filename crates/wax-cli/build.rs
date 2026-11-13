fn main() {
    println!("cargo:rerun-if-env-changed=GITHUB_RUN_NUMBER");
    let version = std::env::var("CARGO_PKG_VERSION").expect("Cargo provides the package version");
    println!("cargo:rerun-if-env-changed=WAX_RELEASE_VERSION");
    let label = if let Ok(explicit) = std::env::var("WAX_RELEASE_VERSION") {
        assert!(explicit == version || explicit.starts_with(&format!("{version}-preview.")), "Release version must match workspace version");
        explicit
    } else { match std::env::var("GITHUB_RUN_NUMBER") {
        Ok(number) if number.chars().all(|c| c.is_ascii_digit()) && !number.is_empty() => {
            format!("{version}-preview.{number}")
        }
        _ => format!("{version}-dev"),
    }};
    println!("cargo:rustc-env=WAX_BUILD_VERSION={label}");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        // Embed VERSIONINFO metadata: unsigned executables without version
        // information are a common antivirus ML false-positive trigger, and
        // v1.0.29 was flagged for exactly this profile.
        let version_rc = write_version_rc(&version, "waX VPN", "wax.exe", "waX VPN command line");
        if let Err(e) = embed_resource::compile(&version_rc, embed_resource::NONE).manifest_optional() {
            panic!("failed to compile the Windows version resource: {e:?}");
        }
    }
}

/// Generate a VERSIONINFO resource script that tracks the package version and
/// return its path inside OUT_DIR.
fn write_version_rc(version: &str, product: &str, original: &str, description: &str) -> std::path::PathBuf {
    let mut parts = version.split(|c| c == '.' || c == '-');
    let maj: u32 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let min: u32 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let patch: u32 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let rc = format!(r#"1 VERSIONINFO
FILEVERSION {maj},{min},{patch},0
PRODUCTVERSION {maj},{min},{patch},0
FILEFLAGSMASK 0x3fL
FILEFLAGS 0x0L
FILEOS 0x40004L
FILETYPE 0x1L
FILESUBTYPE 0x0L
BEGIN
    BLOCK "StringFileInfo"
    BEGIN
        BLOCK "040904b0"
        BEGIN
            VALUE "CompanyName", "vio137"
            VALUE "FileDescription", "{description}"
            VALUE "FileVersion", "{version}"
            VALUE "InternalName", "{product}"
            VALUE "LegalCopyright", "MIT License"
            VALUE "OriginalFilename", "{original}"
            VALUE "ProductName", "{product}"
            VALUE "ProductVersion", "{version}"
        END
    END
    BLOCK "VarFileInfo"
    BEGIN
        VALUE "Translation", 0x409, 1200
    END
END
"#);
    let path = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set for build scripts")).join("wax-version.rc");
    std::fs::write(&path, rc).expect("write version resource script");
    path
}
