# waX VPN v0.1.21

Unsigned Windows x64 config-name compatibility checkpoint. Windows may show an unknown-publisher warning. The official WireGuardNT driver is independently signed by its vendor. No kill switch: traffic can use the normal connection if the tunnel drops.

## Changes

- Known section and field names accept ASCII case variations, such as [interface], privatekey and AllowedIPs. Values are not lowercased or otherwise changed: endpoint spelling and key material stay intact. The disabled keepalive value still requires lowercase off. Normalized saves keep canonical section and field names.
- Mixed-case repeated scalar fields still fail instead of replacing earlier values. Repeated Address, DNS and AllowedIPs lists remain additive. Unsupported directives and unknown fields/sections still fail. Existing list-gap, MTU and numeric validation remains in place.
- Parser-only change. No new network service, DNS resolution, endpoint validation or reconnect-policy change.

## Verification

Source 97d16a9790763a9dfcaa2ae4b30e50cd20bd68ab / full preview 42906265818 passed all required jobs. The new known-name fixture passed with all 27 Windows core tests. It covers mixed-case names, unchanged values and canonical roundtrip; all seven scalar duplicate fields across case variations; all seven unsupported directives; unknown names, invalid MTU, list gaps and uppercase OFF rejection; additive mixed-case lists. Fixture proof only, not a separate mixed-case GUI import or saved-profile selection test.

Full preview: peer handshake and tunnel-side ping, DNS and IPv4 MTU 1380, split exclusions, owned adapter-loss recovery after 5 seconds, pending-retry disable across 48 samples/12 seconds, manual reconnect with recovery off, original main HWND/PID plus Connect rail restoration, graceful quit/adapter removal, profile/settings warnings and BOM-backed settings GUI startup passed. No console windows observed in 402 samples at about 100 ms; shorter windows could be missed. Crash evidence is a clean 3-second pre-cleanup snapshot. All 14 normal, warning, BOM and installed screenshots were inspected. Fresh install/quit/uninstall passed; installed quit diagnostics show driver open 1 ms, absence confirmed 4 ms and legacy cleanup finished at 4755 ms. Exact v0.1.21 gate remains pending at preparation time.

## Limits and earlier failures

Internet reachability is not proven by the tunnel-side ping. IPv6 protection was skipped without a proven baseline and there is no kill switch. Actual notification-area menus, real UAC accept/cancel/failure/elevated handoff, profile-directory startup failure and stale-profile manual Connect pixels remain unverified.

Earlier installed quit run 42866864702 exceeded the unchanged 15-second gate; its stage log was missing. A workflow-only change ensures future diagnostic logs exist without changing timeouts or assertions. Later successful runs do not diagnose that timeout. Earlier PowerShell/CLR host failures, including 0xC0000409, are not fully diagnosed. Explicit Unicode P/Invoke declarations and stage markers were corrected, and subsequent full runs passed, but causality remains unproven.

Settings preservation covers tested failures only, not concurrent edits, hardware failure or power loss. Real Windows sign-in launch is unproven. Pending-retry disable is demonstrated, not deterministic in-flight cancellation or retry exhaustion. Adapter-loss recovery does not prove physical-network roaming. Genuine tray-construction failure and its taskbar fallback warning/restore remain untested. Mixed-case profile loading has filesystem proof rather than a separate GUI selection test. An earlier release publication HTTP 500 succeeded on failed-job retry without rebuilding; cause unknown.

WireGuardNT remains the only shipping backend. No BoringTun dependency, new product network service, Internet probe, signup, WARP registration, kill switch or IPv6 leak-protection claim. No Cloudflare implementation, assets, credentials or keys are copied.
