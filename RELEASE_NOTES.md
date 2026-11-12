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
passed. Preview https://github.com/vio137/wax-vpn/actions/runs/43189387412
source `eea8fb3a4ef0ee836116e5782fd5dce030db0406`: all three execution jobs
passed. Actual Windows proof checked the selected route changing from
10.55.0.1/32 to 10.55.0.7/32 at the same profile index on reload, then the
restored original route after original file bytes were restored and reloaded.
Settings remained unchanged; no adapter was created. Prior profile/clipboard/
cache/search/demo/recovery/installer checks passed. Changed/restored route and
installed-GUI pixels inspected. Stable publication/checksum verification pending.

Optional layout-state captures remain unverified because their separate harness
host lacks System.Drawing; the CI-only fix remains deferred for workflow
permissions. This candidate does not change the workflow.

## Design review

Preserved the existing graphite card, route labels, explicit limits and 128px
scroll bound. No visual decoration, spacing or motion changes. Design reference:
Unslop by Damir Sokolovsky (@heydamir), CC BY 4.0, used as a review rubric.

## First stable-gate attempt

Run 43191637387 attempt 1 stopped publication when the local import fixture
could not hash the just-imported Chosen profile.conf because another process
held the file. The log does not identify that process. Route-cache checks had
not run in this attempt. Demo tunnel ping, GUI/recovery/restore/quit, startup and
fresh installer checks passed. Retried the failed gate once on identical source
without weakening assertions. Attempt 2 passed import but stopped at the
Show selected unchanged-settings hash assertion, after a preceding Unpin.
The log lacks snapshots to distinguish delayed preceding persistence from a new
change. Route-cache checks had not run. The revised fixture now waits for the
expected persisted Unpin state before taking its baseline and prints before/
after settings if the unchanged-state assertion fails. A bounded sharing-
violation retry wraps the initial imported-file hash read, preserving the content
assertions. No app or networking code changed for these harness corrections.
Revised Windows proof pending.
