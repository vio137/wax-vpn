# waX VPN v0.1.20

Unsigned Windows x64 list-validation checkpoint. Windows may show an unknown-publisher warning. The official WireGuardNT driver is independently signed by its vendor. No kill switch: traffic can use the normal connection if the tunnel drops.

## Changes and compatibility

- Address, DNS and AllowedIPs reject leading/trailing commas and empty entries between commas, instead of silently dropping them. Entirely empty Address and DNS values also fail. Errors name the field and line without echoing values. Previously accepted malformed lists may now fail to import or load; fix their missing entries before use. Existing files are not rewritten or deleted by loading them.
- Compatibility exception: an entirely empty AllowedIPs value still means no routes. Older waX versions wrote this line for empty route lists, so those saved files continue to load. New normalized saves omit the empty AllowedIPs line. A missing AllowedIPs line also means no routes. This does not add a default route. Repeated nonempty list entries remain additive.
- Parser/serializer-only change. No network service, DNS resolution, endpoint or reconnect-policy change.

## Verification

Source 4c320015ab6d228bdc6d89f4fb22af3962d899b3 / full preview 42898953721 passed all required jobs. Both new named tests passed with all 26 Windows core tests. Parser fixtures cover list gaps/whitespace/comments, empty Address/DNS rejection, legacy empty AllowedIPs acceptance, repeated lists and empty-route normalization/roundtrip. Filesystem fixture loads an older saved empty AllowedIPs profile with one profile and zero skipped, preserving its bytes; malformed imports create no file and preserve existing bytes. Fixture proof only, not a separate malformed-list GUI import or saved-profile GUI selection test.

Full preview: peer handshake and tunnel-side ping, DNS and IPv4 MTU 1380, split exclusions, owned adapter-loss recovery after 5 seconds, pending-retry disable across 48 samples/12 seconds, manual reconnect with recovery off, original main HWND/PID plus Connect rail restoration, graceful quit/adapter removal, profile/settings warnings and BOM-backed settings GUI startup passed. No console windows observed in 458 samples at about 100 ms; shorter windows could be missed. Crash evidence is a clean 3-second pre-cleanup snapshot. All 14 normal, warning, BOM and installed screenshots were inspected. Fresh install/quit/uninstall passed; installed quit diagnostics show driver open 1 ms, absence confirmed 2 ms and legacy cleanup finished at 6371 ms. Exact v0.1.20 source 7975de817cb009188b1d46622b20fdd15d19e88a / full run 42902285364 passed all four required jobs. Exact recovery was 5 seconds; pending-retry disable passed 48 samples/12 seconds; no console windows were observed in 471 samples at about 100 ms. Exact installed quit confirmed absence at 2 ms and completed legacy cleanup at 4969 ms. All 14 exact normal, warning, BOM and installed screenshots were inspected. Fresh release installer is 10,955,592 bytes; SHA256 9950688e4fee439c73411b384cd3ae68fa27e49285a84b83e2399a080aba70cb matches the published sums.

Earlier v0.1.19 exact attempt 1 failed NSIS setup because Chocolatey returned HTTP 504 for nsis.install 3.13.0, after tests/release compilation passed. One unchanged failed-jobs retry passed all gates. That retry does not diagnose the service error.

## Limits and earlier failures

Internet reachability is not proven by the tunnel-side ping. IPv6 protection was skipped without a proven baseline and there is no kill switch. Actual notification-area menus, real UAC accept/cancel/failure/elevated handoff, profile-directory startup failure and stale-profile manual Connect pixels remain unverified.

Earlier installed quit run 42866864702 exceeded the unchanged 15-second gate; its stage log was missing. A workflow-only change ensures future diagnostic logs exist without changing timeouts or assertions. Later successful runs do not diagnose that timeout. Earlier PowerShell/CLR host failures, including 0xC0000409, are not fully diagnosed. Explicit Unicode P/Invoke declarations and stage markers were corrected, and subsequent full runs passed, but causality remains unproven.

Settings preservation covers tested failures only, not concurrent edits, hardware failure or power loss. Real Windows sign-in launch is unproven. Pending-retry disable is demonstrated, not deterministic in-flight cancellation or retry exhaustion. Adapter-loss recovery does not prove physical-network roaming. Genuine tray-construction failure and its taskbar fallback warning/restore remain untested. Mixed-case profile loading has filesystem proof rather than a separate GUI selection test. An earlier release publication HTTP 500 succeeded on failed-job retry without rebuilding; cause unknown.

WireGuardNT remains the only shipping backend. No BoringTun dependency, new product network service, Internet probe, signup, WARP registration, kill switch or IPv6 leak-protection claim. No Cloudflare implementation, assets, credentials or keys are copied.
