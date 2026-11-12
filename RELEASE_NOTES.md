# waX VPN v1.0.13

Profiles has a local Check config step before import. It uses the same syntax
and endpoint checks as import, then shows address, DNS, peer and route-entry
counts plus explicit IPv4/IPv6 default-route families. The review does not save,
connect or upload a profile. Editing the config clears the previous result;
invalid input gets a generic message without echoing keys or endpoint values.
This is a syntax review, not a reachability, routing or leak test.
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

Pending preview and stable Windows checks. Local core tests: 42 passed.
Local app tests: 46 passed, one manual networking test ignored. PowerShell
parser passed. The local valid-result layout was inspected at 880x600 using
fixed public fixture data. Release verification will be updated only after
reading the Windows results and inspecting their screenshots.
