# waX VPN v1.0.7

Import a local WireGuard .conf file using the native Windows file picker.
Cancelling leaves profiles alone. Repeated imports get separate names and
keep the original profile and selected source file unchanged. Reads are bounded
to 1 MiB; UTF-8 files with or without a BOM are supported. The picker works off
the render thread and only one picker can be open at a time.

The first-run Connect page now offers Import profile and opens Profiles.
A connected externally owned session still offers Disconnect. Empty connection
details explain where to add a profile. No tunnel or firewall behavior changed.

## Compatibility and limits

CPU software rendering through Windows GDI remains the default. GPU rendering
is opt-in. No separate VC++/.NET/WebView2 installation is required, but Windows
system libraries, administrator consent, networking and a valid profile still
are. The bundled WireGuardNT driver is signed by its vendor. The app and
installer are unsigned and Windows may show an unknown-publisher warning.

DNS uses the native Windows API when available. netsh remains only for older
Windows builds without that API; the API-absent fallback is untested. IPv6
DNS/MTU settings checks do not prove IPv6 traffic routing or leak protection.
No kill switch is provided. Traffic can use the normal connection when the
tunnel is off. Live WARP registration is not verified.

The release gate runs on a hosted Windows Server 2025 runner with installed
runtimes and a virtual display, not a clean Windows 10/11 PC. It does not prove
the reported consumer-PC crash is fixed. A peer handshake and tunnel-address
ping do not prove Internet reachability. Tray-menu clicks need real Windows
review. Runner OpenGL rendering/dismissal remain unverified. Console-window
sampling can miss a shorter flash.

## Release verification

Exact stable workflow: https://github.com/vio137/wax-vpn/actions/runs/43113539426
Source: `4a42fbc67bfb7ac0486ac2146e553d8c202b01a8`.

All four stable jobs passed. Actual Windows evidence passed first-run Import
navigation, native picker cancellation, two chosen-file imports, duplicate
preservation, unchanged source-file hash and no tunnel adapter. Installed,
first-run and duplicate-result pixels were inspected.

Existing GUI demo connection, tunnel-address ping, DNS apply/reset, IPv4 MTU,
67 split routes, bounded recovery, pending-retry disable, manual reconnect,
close-to-tray, original-window restore, graceful quit, settings and startup
checks passed, subject to the limits above. Fresh install, upgrade, uninstall
and all three exact-path native process-scope checks passed. PE dependency
audits passed for all three binaries. Console sampling found no new visible
native console; shorter flashes can still be missed.

The published installer was downloaded and matched its checksum file:

- File: `waX-VPN-Setup-v1.0.7-x64.exe`
- Size: 11,692,119 bytes
- SHA256: `7572d2085c0544a9e65362e89befbe80152ce9fa9f81452d58ec739d20d4c9c0`

No speed improvement is claimed.
