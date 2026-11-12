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
Windows gate and publication pending. The GUI interaction harness now explicitly
loads System.Drawing in its own PowerShell host before capturing frames, after
v1.0.27 gate logs exposed that missing assembly. This is CI-only; no app behavior
changed. Windows validation of that harness correction is pending.

## Design review

Preserved the graphite surfaces, subdued controls and existing row hierarchy.
The search hint now names the expanded fields; hover text explains the literal,
configured-only limits. No new decoration or motion. Design reference:
Unslop by Damir Sokolovsky (@heydamir), CC BY 4.0, used as a review rubric.
