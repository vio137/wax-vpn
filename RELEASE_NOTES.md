# waX VPN v1.0.14

Clear pasted config sits beside Check config. It clears the editor and its
review/import feedback, then returns keyboard focus to the editor. Profile
name, saved profiles and system clipboard stay unchanged. Clear, Check and
Import actions are disabled when the editor is empty. This is an editor
convenience, not a secure memory-erasure feature. No tunnel, DNS, firewall or
saved-setting behavior changed.

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

Exact stable workflow: https://github.com/vio137/wax-vpn/actions/runs/43139362010
Source: `9741b584e5bcd3abe3e758d85415a4fe86b308b5`.

All four stable jobs passed. Actual Windows results passed clear-editor and
feedback, returned editor focus, preserved name/clipboard and disabled empty
actions without saving. Filled/cleared editor pixels were inspected. Existing
local syntax review, native imports, keyboard search, bounded 26-profile picker
and keyboard-help checks passed. Local app tests: 46 passed, one manual
networking test ignored. Build and PowerShell parser passed.

Existing GUI demo connection, tunnel-address ping, DNS apply/reset, IPv4 MTU,
split routes, bounded recovery, retry disable, original-window restore and
graceful quit passed. Fresh install, upgrade, uninstall, all three native
process-scope checks and three PE audits passed within the limits above.
331 console-window samples at about 100ms saw no new visible native console;
shorter flashes can be missed.

The published installer was downloaded and matched its checksum file:

- File: `waX-VPN-Setup-v1.0.14-x64.exe`
- Size: 11,703,149 bytes
- SHA256: `a96d202d77557fa957bd28b02053e7df88505aa0d884c24f925d392476c124ee`
