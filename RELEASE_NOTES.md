# waX VPN v1.0.21

Clear in the Connect-page quick picker returns focus to its search editor.
You can clear and immediately type another query, including spaces, without
an extra click. Selection, profile files and connection stay unchanged.
No tunnel, DNS, firewall or service behavior changed.

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

Local app tests: 49 passed, one manual networking test ignored. Build and
PowerShell parser passed. Local pixels showed Clear followed by a typed
Office 23 name/endpoint query, with Office 24 still selected and disconnected.
Preview: https://github.com/vio137/wax-vpn/actions/runs/43157848403
Source: `f34877982f06c88f1754616ef538843425c06eb4`.

All three execution jobs passed. Actual Windows proof passed Clear returning
focus, typed name/endpoint words with spaces without connecting and repeated
Clear focus. Quick-picker focus and installed-GUI pixels were inspected.
Prior local-profile and connection/installer checks passed within the
compatibility limits above.

Stable: https://github.com/vio137/wax-vpn/actions/runs/43159028745
Source: `0cc74a402ad8b263cf34f4a2b2e4333b257f2061`.

All four stable jobs passed. Actual stable Windows proof passed Clear returning
search focus, spaced typing without connection and repeated Clear focus.
Stable focus and installed-GUI pixels were inspected. Prior demo tunnel,
DNS/IPv4 MTU/split, 3s recovery, retry disable (48 samples over 12s), manual
reconnect, original HWND/PID restore and quit passed. No console windows were
observed in 321 samples at about 100ms; shorter flashes could be missed.
Installer lifecycle, native checks and PE audits passed.

Release: https://github.com/vio137/wax-vpn/releases/tag/v1.0.21
The downloaded 11,708,272-byte installer matches the published SHA256:
`578565fe0d6a92550f907bf692e74343eb2ffe13a37572cf6f4d8267c631c408`.
