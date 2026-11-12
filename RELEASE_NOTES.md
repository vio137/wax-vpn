# waX VPN v1.0.24

The Connect quick picker shows a quiet match count during local searches,
including singular and zero results. Clearing the search removes the count.
The filtered list stays shorter to keep the Connect action visible. No new
saved setting, file access, network request or connection action was added.
Search still matches every word across profile names and endpoints; a number
can match an endpoint as well as a profile name.

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

Short local checks: app tests passed (50 passed, one manual ignored), build
and PowerShell syntax passed. Windows count/clear/bounds checks and stable
publication are pending. No performance improvement is claimed.
