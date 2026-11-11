# waX VPN v1.0.7

A first-run Import profile button opens Profiles instead of leaving a disabled
Connect button. Existing sessions retain Disconnect, even when this instance
has no selected profile. Empty Connection details explain how to add a profile.

Windows Profiles adds Choose .conf file, using the native Windows file picker.
It reads only the file you select, at most 1 MiB of UTF-8 WireGuard text. UTF-8
BOMs are accepted. Cancel leaves profiles unchanged. The existing import path
keeps duplicate names separate instead of overwriting another profile. The
source file is not changed. This action does not connect a tunnel. No shell,
new web runtime or third-party file-picker library is added.

The Ctrl+F control IDs use the current egui API. No networking or traffic
policy changes are included.

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

Candidate only. Local full workspace tests passed, including 44 app tests and
one existing ignored test. The native file-picker module cross-compiled for
Windows. First-run Import navigation pixels were inspected at 880x600 and
760x540. Windows file-picker pixels, cancellation and duplicate-import proof
are pending the exact candidate workflow. Nothing here is a release claim.
