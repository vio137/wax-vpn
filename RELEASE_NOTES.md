# waX VPN v1.0.12

Quick-picker search receives keyboard focus on opening. Ctrl+F searches the
open picker instead of switching pages; outside it, Ctrl+F still opens Profiles.
Escape closes the picker even while typing and returns focus to Switch profile.
Enter can then reopen it with a fresh query. Text-field spaces remain ordinary
text and do not connect. The About keyboard reference explains this behavior.
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

Exact stable workflow: https://github.com/vio137/wax-vpn/actions/runs/43132837961
Source: `2693a57a407659bdfc6095b78c4219900b5773e0`.

All four stable jobs passed. Actual Windows results passed opening focus,
contextual Ctrl+F, Escape while typing, returned Switch focus and Enter
reopening with a cleared query. Existing name/endpoint search, empty result,
Clear, selection without connection, native imports, bounded 26-profile picker
and keyboard-help checks passed. Focused search, dismissed picker and installed
app pixels were inspected. Local app tests: 46 passed, one manual networking
test ignored. Build and PowerShell parser passed.

Existing GUI demo connection, tunnel-address ping, DNS apply/reset, IPv4 MTU,
split routes, bounded recovery, retry disable, original-window restore and
graceful quit passed. Fresh install, upgrade, uninstall, all three native
process-scope checks and three PE audits passed within the limits above.
337 console-window samples at about 100ms saw no new visible native console;
shorter flashes can be missed.

The published installer was downloaded and matched its checksum file:

- File: `waX-VPN-Setup-v1.0.12-x64.exe`
- Size: 11,700,370 bytes
- SHA256: `8d6e1820a1171b70163481d6a9ad9eeeecf37d230a12d73edc23b6604221dd7c`
