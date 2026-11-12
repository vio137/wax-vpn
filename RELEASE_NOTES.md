# waX VPN v1.0.30

The configured-route preview now reuses its bounded display strings while the
selected loaded profile stays unchanged, instead of formatting them each frame.
The cache is lazy, created only when routes are expanded, and retains at most
64 route labels for one selected profile plus a remainder count. It retains no
config object or keys. Selection change and successful profile reload invalidate
it. Failed reload keeps the existing loaded snapshot under the existing warning.
No measured speedup is claimed. No UI layout, saved settings, clipboard export
or networking behavior changed.

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

Local app tests: 56 passed, one manual network test ignored. New coverage checks
lazy creation, storage reuse across unchanged frames, selection refresh, reload
invalidation, empty routes and the 64-entry bound. Build and PowerShell syntax
passed. Windows same-index route-change/restoration proof, publication and
checksum verification pending.

Optional layout-state captures remain unverified because their separate harness
host lacks System.Drawing; the CI-only fix remains deferred for workflow
permissions. This candidate does not change the workflow.

## Design review

Preserved the existing graphite card, route labels, explicit limits and 128px
scroll bound. No visual decoration, spacing or motion changes. Design reference:
Unslop by Damir Sokolovsky (@heydamir), CC BY 4.0, used as a review rubric.
