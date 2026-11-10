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
}
