# waX VPN v1.0.9

Long quick-picker lists now scroll inside a bounded area instead of growing
the whole Connect page. Only the visible rows and a small buffer are drawn.
Profile order, full-text hover, selection and saved choice stay the same.
No measured speed improvement is claimed. No networking behavior changed.

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

Exact stable workflow: https://github.com/vio137/wax-vpn/actions/runs/43122392479
Source: `eff79ce5ed407039f03c35245169dc2bc819139e`.

All four stable jobs passed. Actual Windows evidence passed the 26-profile
bounded picker, Connect remaining on-screen, scrolling to Office 24 and saved
selection without an adapter. Stable scrolled-list pixels show Office 24 fully
inside the list; selected-profile and installed-app pixels were inspected too.

Existing native-file import, cancellation, duplicate/source preservation,
quick selection, keyboard help, GUI demo connection, tunnel-address ping, DNS
apply/reset, IPv4 MTU, split routes, bounded recovery, retry disable, original
window restore and graceful quit checks passed. Fresh install, upgrade,
uninstall, all three native process-scope checks and all three PE dependency
audits passed, subject to the limits above. Console sampling can miss shorter
flashes. UIA ExpandCollapse support for keyboard help is not verified.

The published installer was downloaded and matched its checksum file:

- File: `waX-VPN-Setup-v1.0.9-x64.exe`
- Size: 11,695,271 bytes
- SHA256: `aec387ec6e610c14e0163d8add8fe748093c60a54b4c08dd411959e9ffce1017`
