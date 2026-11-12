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

Preview build, runtime and fresh-installer checks passed:
https://github.com/vio137/wax-vpn/actions/runs/43124621272
Source: `9b364c1757768eee7a43534b6a122683044db1c1`.

Actual Windows results passed accessible Show and Hide, content appearing and
disappearing, plus all existing native import, picker and runtime checks.
Open/hidden pixels were inspected. Exact stable verification and installer
checksum are pending.
