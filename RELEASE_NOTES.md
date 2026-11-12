# waX VPN v1.0.22

Profiles has a collapsed Selected config summary. It shows the loaded profile's
name, addresses, DNS, MTU, peer count and configured route-entry count. Keys are
not shown. The panel explicitly labels these as configured values, not a live
connection, routing or leak check. Opening and closing it does not change
selection, settings, files or the connection.
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

Local app tests: 50 passed, one manual networking test ignored. Build and
PowerShell parser passed. Summary tests cover IPv4/IPv6 values, missing values,
multiple peers, route-entry counts, unchanged config and excluded key fields.
Local collapsed and expanded pixels were inspected.
Windows execution proof is pending.
