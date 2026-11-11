# waX VPN v1.0.7

Import a local WireGuard .conf file using the native Windows file picker.
Cancelling leaves profiles alone. Repeated imports get separate names and
keep the original profile and selected source file unchanged. Reads are bounded
to 1 MiB; UTF-8 files with or without a BOM are supported. The picker works off
the render thread and only one picker can be open at a time.

The first-run Connect page now offers Import profile and opens Profiles.
A connected externally owned session still offers Disconnect. Empty connection
details explain where to add a profile. No tunnel or firewall behavior changed.

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

The preview passed build, runtime and fresh-installer gates:
https://github.com/vio137/wax-vpn/actions/runs/43111832580
Source: `21d7e2382e679b245c5bea8e6371960daca6dd1e`.

Actual Windows evidence passed first-run Import navigation, native picker
cancellation, two chosen-file imports, duplicate preservation, unchanged
source-file hash and no tunnel adapter. Picker and duplicate-result pixels
were inspected. Existing GUI connection, recovery, split-route, settings,
startup and installer checks also passed, subject to the limits above.

Exact stable verification and installer checksum will be recorded after the
stable gate finishes. No speed improvement is claimed.
