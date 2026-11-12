# waX VPN v1.0.28

Local profile search now matches configured interface addresses, DNS addresses
and peer route CIDRs, alongside names and endpoints. All words must match one
profile. Matching is literal case-insensitive text, not CIDR membership, live
routing or a connection check. Both Profiles and the Connect quick picker use
the same cache. Key material is never indexed. The cache follows existing
successful-reload and ranking invalidation. No files, settings or network
behavior change just by searching.

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

Local app tests: 54 passed, one manual network test ignored. New coverage checks
IPv4/IPv6 addresses, DNS, route CIDRs, multi-field queries, literal misses,
key exclusion and reload invalidation. Build and PowerShell syntax passed.
Preview https://github.com/vio137/wax-vpn/actions/runs/43181431620
source `bc6194865a07945a282c0e0e3aaa851d6b96aa58`: all three execution jobs
passed. Actual Windows evidence checked address/route queries in Profiles and
the quick picker, unique matches, literal misses and unchanged local state with
no adapter. Prior profile/summary clipboard/cache reload, demo, recovery and
installer checks passed. Both search views and installed GUI pixels inspected.
Stable https://github.com/vio137/wax-vpn/actions/runs/43183513122
source `3cd93209b111931156e4446f35811c7c18aa156f`: all four jobs passed.
Actual stable evidence read: both configured-IP search fixtures and literal
misses, unchanged state, prior clipboard/cache/profile checks, demo tunnel ping,
4-second recovery, disabled recovery in 48 samples over 12 seconds, original
HWND/PID restore and quit, and fresh install/upgrade/uninstall/process scope.
334 console samples saw no window; shorter flashes could be missed. Both search
views and installed-GUI pixels inspected. DNS/IPv6 search cases are locally
tested, not separately checked by the Windows UI fixture.

Published installer downloaded and matched the published SHA-256:
`d7a285e75a4efd67e4c84742a8fa6cd6536aceb1f06db613dffd2e95c4b9c9b2`
(11,770,024 bytes).
https://github.com/vio137/wax-vpn/releases/tag/v1.0.28

Optional layout-state screenshots remain
unverified because their separate harness host lacks System.Drawing. A proposed
CI-only assembly-load fix could not be pushed with the current credential's
workflow permissions and is not included in this release candidate. Functional
profile screenshots use their own loaded assembly and remain part of the gate.

## Design review

Preserved the graphite surfaces, subdued controls and existing row hierarchy.
The search hint now names the expanded fields; hover text explains the literal,
configured-only limits. No new decoration or motion. Design reference:
Unslop by Damir Sokolovsky (@heydamir), CC BY 4.0, used as a review rubric.
