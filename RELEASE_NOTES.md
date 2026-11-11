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

Preview build, runtime and fresh-installer checks passed:
https://github.com/vio137/wax-vpn/actions/runs/43120948811
Source: `2d37edb80d7d0943bab64e16fbfc6373a3616d80`.

Actual evidence passed the 26-profile bounded picker, Connect remaining
on-screen, scrolling and saving Office 24 without an adapter. Top-list and
selected-profile pixels were inspected. The preview's scrolled capture showed
a buffered row before it was fully visible; the stable gate scrolls another
step before capturing. Stable proof and checksum are pending.
