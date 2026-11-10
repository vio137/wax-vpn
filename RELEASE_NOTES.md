## waX VPN v0.1.0 for Windows x64

Unsigned release, as approved by the maintainer while SignPath Foundation review is pending. No code-signing certificate is present. Windows SmartScreen and publisher warnings are expected. Administrator rights are required to connect.

Pure-Rust egui interface, official WireGuardNT driver, importable WireGuard profiles, per-profile CIDR exclusions, single-instance window reopening, and optional ToS-gated WARP setup.

### Protection limit

There is no kill switch in v0.1.0. It has been removed from the interface, settings, CLI and tunnel backend. Old preview settings are ignored and rewritten without those fields. Only cleanup of stale preview firewall rules remains. If the tunnel drops, traffic may use the normal connection. This is NOT fail-closed protection. IPv6 leak protection is not claimed. Do not use this build where either is required.

### Release verification gates

Publication requires workspace tests/build, Windows demo tunnel and GUI smoke checks, and isolated fresh-runner silent NSIS install, installed-window launch, graceful quit and uninstall. Read the associated workflow's evidence for the exact outcome. Existing scoped checks cover a demo handshake, tunnel-side packets, DNS apply/restore, 8.8.8.8 excluded from tunnel routes while 1.1.1.1 stays routed, original main window reopening, and --quit adapter removal. These do not prove every network/Windows environment.

### Still unverified

Real-desktop tray menu interaction, consumer SmartScreen behavior, live WARP registration, and general IPv6 leak behavior. WARP registration remains opt-in and requires explicit ToS acceptance. No third-party public-IP lookup is performed. The official vendor driver DLL is not modified. User profiles/settings are retained on uninstall.
