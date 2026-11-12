# waX VPN v1.0.10

Keyboard help now uses explicit Show keyboard shortcuts and Hide keyboard
shortcuts buttons. Both have normal accessible invoke actions, replacing the
disclosure whose expanded state did not respond to UIA Expand. Help starts
hidden and keeps its open state while moving between pages during a session.
No saved setting, tunnel, DNS or firewall behavior changed.

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

Exact stable workflow: https://github.com/vio137/wax-vpn/actions/runs/43125943433
Source: `ff1801fbc75c4ebc8fc12bd5cf6bb956a365c6ca`.

All four stable jobs passed. Actual Windows results passed UIA Invoke for
Show and Hide, shortcut content appearing and disappearing, plus the existing
native file import, short-picker and 26-profile picker checks. Open, hidden
and installed-app pixels were inspected. Lower help rows require scrolling at
880x600; no complete screen-reader or clean Windows 10/11 claim is made.

GUI demo connection, tunnel-address ping, DNS apply/reset, IPv4 MTU, split
routes, bounded recovery, retry disable, original-window restore and graceful
quit checks passed. Fresh install, upgrade, uninstall, all three native process
scope checks and all three PE dependency audits passed, subject to the limits
above. Console sampling can miss shorter flashes.

The published installer was downloaded and matched its checksum file:

- File: `waX-VPN-Setup-v1.0.10-x64.exe`
- Size: 11,696,349 bytes
- SHA256: `7aa2447c7627ef710187cb21936cae80ef652d4226c79ac706b4abf377379271`
