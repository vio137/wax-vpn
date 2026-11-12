# waX VPN v1.0.29

Show configured routes in the expanded loaded-config summary reveals up to 64
AllowedIPs entries, labeled by peer position, in a bounded 128px scroll area.
Extra entries are counted explicitly. This is the loaded config before split
exclusions, not current routes, IP membership, reachability or leak protection.
Keys and endpoints are excluded. The view opens only on request and can be
collapsed. Copy summary retains its existing allowlist and does not copy this
route preview. No files, settings or networking behavior changed.

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

Local app tests: 55 passed, one manual network test ignored. New coverage checks
ordered peer positions, IPv4/IPv6, empty routes, key/endpoint exclusion and the
64-entry bound with an explicit remainder count. Build and PowerShell parser
passed. Actual local screenshot inspected for readable limits and one route.
Preview https://github.com/vio137/wax-vpn/actions/runs/43185438033
source `c47e21be3676d7413c0988969a9b5fa5b734eb2a`: all three execution jobs
passed. Actual Windows proof checked explicit route expand/collapse, the loaded
CIDR and peer position, configured-only limits and unchanged local state without
an adapter. Prior search/clipboard/cache reload/profile/demo/recovery/installer
checks passed. Configured-route and installed-GUI pixels inspected. The 64-entry
bound and IPv6 routes are unit-tested, not Windows UI-fixture checked.
Stable https://github.com/vio137/wax-vpn/actions/runs/43187434822
source `76e27441f79b5052b7c4b1cf798e210a58ad699a`: all four jobs passed.
Actual stable results read: route expand/collapse and limits, unchanged local
state, prior search/clipboard/cache/profile checks, demo tunnel ping, recovery
in 4 seconds, disabled recovery in 48 samples over 12 seconds, original HWND/PID
restore and quit, and fresh install/upgrade/uninstall/process scope.
336 console samples saw no window; shorter flashes could be missed. Configured
route and installed-GUI pixels inspected.

Published installer downloaded and matched the published SHA-256:
`cf198d7dddffe626ff91ad7bc58a77d8494e857c0cd13fa201121d74fcaf0688`
(11,772,993 bytes).
https://github.com/vio137/wax-vpn/releases/tag/v1.0.29

Optional layout-state captures remain unverified because their separate harness
host lacks System.Drawing; the CI-only fix remains deferred for workflow
permissions. This candidate does not change the workflow.

## Design review

Keep the existing graphite card and text hierarchy. A small explicit action
reveals factual route content without decorative badges or motion; the route
list height is capped at 128px. Design reference: Unslop by Damir Sokolovsky
(@heydamir), CC BY 4.0, used as a review rubric.
