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

Local app tests: 46 passed, one manual networking test ignored. Build passed.
Actual local pixels inspected after opening and typing, Ctrl+F, Escape and
Enter reopening. PowerShell parser and Windows gates remain pending.
