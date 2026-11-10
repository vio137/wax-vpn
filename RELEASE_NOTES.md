# waX VPN v0.1.19

Unsigned Windows x64 host-prefix compatibility checkpoint. Windows may show an unknown-publisher warning. The official WireGuardNT driver is independently signed by its vendor. No kill switch: traffic can use the normal connection if the tunnel drops.

## Change

- Bare numeric IPs in Address and AllowedIPs now parse as single-host prefixes: /32 for IPv4 and /128 for IPv6. Saved normalized configs write those explicit prefixes. Existing CIDR prefixes retain their lengths. This does not turn a bare address into a default route or a wider network.
- Parser-only change. Invalid addresses, hostnames, malformed or out-of-range prefix lengths and bracketed IPv6 values remain rejected in these fields. IPv6 address MTU minimum checks remain active. DNS, endpoint resolution and reconnect policy are unchanged; no new network service. Loading does not rewrite existing profile files.

## Verification

Source 82f7d80bd7ce842de6abac331c70126573f9cdf8 / full preview 42891406081 passed all required jobs. Named bare_ip_values_normalize_to_host_prefixes_without_relaxing_validation passes with all 24 Windows core tests: mixed bare IPv4/IPv6 and explicit prefixes, repeated list entries, inline comments, normalized roundtrip, invalid numeric addresses/hostnames/prefix lengths/bracketed IPv6 and IPv6 MTU 1279 rejection. This is parser fixture proof only, not a separate bare-IP GUI import test.

Full preview: peer handshake and tunnel-side ping, DNS and IPv4 MTU 1380, split exclusions, owned adapter-loss recovery after 4 seconds, pending-retry disable across 48 samples/12 seconds, manual reconnect with recovery off, original main HWND/PID plus Connect rail restoration, graceful quit/adapter removal, profile/settings warnings and BOM-backed settings GUI startup passed. No console windows observed in 372 samples at about 100 ms; shorter windows could be missed. Crash evidence is a clean 3-second pre-cleanup snapshot. All 14 normal, warning, BOM and installed screenshots were inspected. Fresh install/quit/uninstall passed; installed quit diagnostics show driver open 1 ms, absence confirmed 2 ms and legacy cleanup finished at 6171 ms. Exact v0.1.19 gate remains pending at preparation time.

## Limits and earlier failures

Internet reachability is not proven by the tunnel-side ping. IPv6 protection was skipped without a proven baseline and there is no kill switch. Actual notification-area menus, real UAC accept/cancel/failure/elevated handoff, profile-directory startup failure and stale-profile manual Connect pixels remain unverified.

Earlier installed quit run 42866864702 exceeded the unchanged 15-second gate; its stage log was missing. A workflow-only change ensures future diagnostic logs exist without changing timeouts or assertions. Later successful runs do not diagnose that timeout. Earlier PowerShell/CLR host failures, including 0xC0000409, are not fully diagnosed. Explicit Unicode P/Invoke declarations and stage markers were corrected, and subsequent full runs passed, but causality remains unproven.

Settings preservation covers tested failures only, not concurrent edits, hardware failure or power loss. Real Windows sign-in launch is unproven. Pending-retry disable is demonstrated, not deterministic in-flight cancellation or retry exhaustion. Adapter-loss recovery does not prove physical-network roaming. Genuine tray-construction failure and its taskbar fallback warning/restore remain untested. Mixed-case profile loading has filesystem proof rather than a separate GUI selection test. An earlier release publication HTTP 500 succeeded on failed-job retry without rebuilding; cause unknown.

WireGuardNT remains the only shipping backend. No BoringTun dependency, new product network service, Internet probe, signup, WARP registration, kill switch or IPv6 leak-protection claim. No Cloudflare implementation, assets, credentials or keys are copied.
