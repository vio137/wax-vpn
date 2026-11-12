# waX VPN v1.0.25

The read-only loaded-config summary now lists explicit /0 default-route
families: IPv4 only, IPv6 only, both, or none. It checks all loaded peers,
without inferring coverage from split prefixes. This describes configured
AllowedIPs, not current routes, Internet reachability or leak protection.
Keys remain excluded. No saved setting or networking behavior changed.

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

Local app tests passed (51 passed, one manual ignored), including single,
dual-family, multiple-peer and split-prefix cases. Windows summary checks,
installer and stable publication are pending. No performance claim.
