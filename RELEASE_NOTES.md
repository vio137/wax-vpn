# waX VPN v1.0.6

Ctrl+F opens Profiles and focuses its search field from any page, including
while another text field has focus. It never changes the tunnel. The Clear
button resets the search; Escape leaves the field using normal text-input
behavior.

Profile order and lowercase search terms are cached until profiles or settings
change. Unchanged frames reuse the filtered index list without cloning it.
Name and endpoint matches remain separate, so a query cannot match across
the boundary between two fields. No measured speed improvement is claimed.

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

Candidate only. Local 42 app tests passed, one existing test ignored. Build
and workflow lint passed; keyboard search and empty-results pixels inspected.
Windows runtime and exact stable installer gates are pending.
