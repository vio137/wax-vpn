# waX VPN v1.0.8

The Connect-page profile picker has separate name and endpoint columns, shorter
single-line text with full details on hover, keyboard focus borders and an
accessible Switch profile action. Choosing a profile saves that choice without
starting a connection. The existing pin and recent-use order is reused.

About includes a collapsible keyboard shortcut reference. Profile search keeps
its normalized query until the typed text changes. No measured performance
gain is claimed. No tunnel, DNS or firewall behavior changed.

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

Exact stable workflow: https://github.com/vio137/wax-vpn/actions/runs/43118553949
Source: `20c97691dd86147a79755fc6a77f6f04b0327fe1`.

All four stable jobs passed. Actual Windows evidence passed accessible quick
picker open/select, saved selection without a connection, and keyboard help
expanded through a visible click. Stable picker, selected-profile, expanded-help
and installed-app pixels were inspected. UIA ExpandCollapse support is not
verified; the lower help rows need scrolling at the captured window size.

Existing native file selection, cancellation, duplicate/source preservation,
GUI demo connection, tunnel-address ping, DNS apply/reset, IPv4 MTU, 67 split
routes, bounded recovery, retry disable, original-window restore and graceful
quit checks passed. Fresh install, upgrade, uninstall, all three native process
scope checks and all three PE dependency audits passed. Console sampling found
no new visible native console; shorter flashes can still be missed. The limits
above still apply.

The published installer was downloaded and matched its checksum file:

- File: `waX-VPN-Setup-v1.0.8-x64.exe`
- Size: 11,693,719 bytes
- SHA256: `3662379994c2f7f6872b29154e2f0355e6274e8c03708e894e0ad5a56fab4cb3`
