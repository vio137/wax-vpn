# waX VPN v1.0.15

Profiles has a Pinned only filter that combines with name/endpoint search.
Unpinning refreshes the filtered list; disabling the filter restores all
matching profiles. No pinned profiles and no search matches have separate
empty states. The filter lasts for this app session only and does not change
the selected profile, profile configuration or the Connect-page picker.
No tunnel, DNS or firewall behavior changed.

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

Pending focused Windows and stable-release checks. Local app tests: 47 passed,
one manual networking test ignored. Build and PowerShell parser passed.
Pinned and search-empty layouts inspected at 880x600.
