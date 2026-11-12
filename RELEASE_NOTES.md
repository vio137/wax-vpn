# waX VPN v1.0.26

The loaded-config summary reuses its allowlisted display strings while the
selection and loaded profiles are unchanged, instead of rebuilding its values
every frame. Switching profiles or successfully reloading profiles refreshes
the cache. Only display text is cached, never a config or key material.
The visible summary and networking behavior are unchanged. No measured speedup
is claimed. The cache is bounded to one selected profile.

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

Local app tests passed (52 passed, one manual ignored), including reuse,
selection changes and explicit invalidation after same-index replacement.
Build and PowerShell syntax passed.
Preview: https://github.com/vio137/wax-vpn/actions/runs/43173001910
Source: `9888c33085f67f1edfd55c7a7d9feb36551bfc86`. All three execution jobs
passed. Actual Windows results passed selected-config address change at the
same index, reload refresh, restored original bytes and second reload refresh,
unchanged selected settings and no adapter. Prior summary/profile/demo/recovery
and installer checks passed. Reloaded-summary and installed-GUI pixels inspected.
Stable: https://github.com/vio137/wax-vpn/actions/runs/43174042593
Source: `4a2488390a3c7e589f2afb05218749daaeba4210`. All four jobs passed.
Actual full results passed same-index reload/restoration cache refresh and prior
summary/profile checks, demo handshake/tunnel-address ping, DNS/MTU restore,
split-route checks, 4-second recovery, recovery-disable with 48 samples over
12 seconds, manual reconnect, original HWND/PID restore and graceful quit.
No console observed in 341 samples at about 100 ms; shorter flashes can be missed.
Stable reloaded-summary and installed-GUI pixels inspected. Installer passed.
Published installer (11,712,294 bytes) downloaded and independently hashed,
matching SHA256SUMS-v1.0.26.txt:
`89187ae6e6e3d9c3b074f79750dea2637ebdd9ab46eccb1193af6cbedc2015cf`.
An initial test fixture omitted the required Peer section; corrected before
passing the tests. No app runtime failure was found by that fixture error.
