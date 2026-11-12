# waX VPN v1.0.17

Profiles search now shows a live match count when a query or the Pinned only
filter is active. Escape clears the focused search and keeps typing focus,
including when the field is already empty. The Clear button also returns
focus to search. Pins and the selected profile stay unchanged.
No tunnel, DNS, firewall, service endpoint or saved-setting behavior changed.

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

Preview: https://github.com/vio137/wax-vpn/actions/runs/43148541118
Source: `89973e77c54a4cfb74a20e437ea5172e833b108b`.

All three preview execution jobs passed. Actual Windows results passed focused
Profiles Escape/Clear reset, retained typing focus and pinned filter, empty
search Escape and subsequent Space without a connection. Match counts passed
for one pinned row, no search matches and unpin refresh. Focused-search,
pinned and empty-state pixels were inspected. Local app tests: 48 passed,
one manual networking test ignored. Build and PowerShell parser passed.

Existing native imports, config review/Clear, bounded profile list/picker,
keyboard help, GUI demo connection, tunnel-address ping, DNS apply/reset,
IPv4 MTU, split routes, bounded recovery, retry disable, original-window
restore and graceful quit passed. Installer lifecycle/native process-scope
checks and PE audits passed within the limits above.

Stable publication and downloaded-installer checksum verification are pending.
