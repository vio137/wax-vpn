# waX VPN v0.1.33

Unsigned Windows x64 split-exclusion error-wording checkpoint. Windows may show an unknown-publisher warning. The official WireGuardNT driver is independently signed by its vendor. No kill switch: traffic can use the normal connection if the tunnel drops.

## Changes

- Malformed split-exclusion entries now give fixed expected-IPv4/IPv6-CIDR guidance instead of repeating the malformed token.
- Defensive wording only. No production credential leak was observed. Entry splitting, empty entries, CIDR acceptance, input order and route subtraction are unchanged. No UI or network behavior change.

## Verification

Source 2059ac25069ac8ede3fb86115542efe853c37d19 / full preview 42962992616 passed all required jobs. The exclusion_errors_use_fixed_guidance_without_echoing_tokens fixture passed with all 37 core tests. Synthetic offline fixtures cover invalid-token sentinels, current valid mixed-family/separator/empty entries, duplicates, unaligned values and ordering. Fixture proof only, not a production leak reproduction.

Exact v0.1.33 run 42964757326 at 00b9db7c9245e4bd28cae8dd4925630b1ea348a4 passed build, demo-tunnel/GUI verification, fresh-runner install/quit/uninstall and publish. Raw verify-results and installer diagnostics were read. Peer handshake and tunnel-side ping, DNS and IPv4 MTU 1380, split exclusions, owned adapter-loss recovery after 4 seconds, pending-retry disable across 48 samples/12 seconds, manual reconnect with recovery off, original main HWND/PID plus Connect rail restoration, graceful quit/adapter removal, profile/settings warnings and BOM-backed settings GUI startup passed. No console windows observed in 356 samples at about 100 ms; shorter windows could be missed. Crash evidence is a clean 3-second pre-cleanup snapshot. All 14 normal, warning, BOM and installed screenshots were inspected. Installed quit diagnostics show driver open/probe at 0 ms, absence confirmed 2 ms and legacy cleanup finished at 4039 ms. These timings describe this run, not a performance benchmark.

Freshly downloaded unsigned installer: 10,958,144 bytes. SHA256 79c2912d5a6a4347743fb69f700316cf809530297229f4d5a91107a6f217fa4a matches the freshly downloaded SHA256SUMS-v0.1.33.txt.

## Limits and earlier failures

Internet reachability is not proven by the tunnel-side ping. IPv6 protection was skipped without a proven baseline and there is no kill switch. Actual notification-area menus, real UAC accept/cancel/failure/elevated handoff, profile-directory startup failure and stale-profile manual Connect pixels remain unverified.

The first unknown-token candidate push 42944794908 failed an older test assertion that expected the unknown token to be echoed. A fixture-only follow-up changed that expectation to fixed guidance and added an explicit no-token assertion. The new fixture already passed in the first run; subsequent full results are recorded above.

Earlier candidate push 42939812475 failed its CLI and GUI handshake deadlines and logged missing System.Drawing capture types. The unchanged full preview later passed. The cause remains unproven; no product or harness change was inferred from that failure.

Earlier installed quit run 42866864702 exceeded the unchanged 15-second gate; its stage log was missing. A workflow-only change ensures future diagnostic logs exist without changing timeouts or assertions. Later successful runs do not diagnose that timeout. Earlier PowerShell/CLR host failures, including 0xC0000409, are not fully diagnosed. Explicit Unicode P/Invoke declarations and stage markers were corrected, and subsequent full runs passed, but causality remains unproven.

Settings preservation covers tested failures only, not concurrent edits, hardware failure or power loss. Real Windows sign-in launch is unproven. Pending-retry disable is demonstrated, not deterministic in-flight cancellation or retry exhaustion. Adapter-loss recovery does not prove physical-network roaming. Genuine tray-construction failure and its taskbar fallback warning/restore remain untested. Mixed-case profile loading has filesystem proof rather than a separate GUI selection test. An earlier release publication HTTP 500 succeeded on failed-job retry without rebuilding; cause unknown.

WireGuardNT remains the only shipping backend. No BoringTun dependency, new product network service, Internet probe, signup, WARP registration, kill switch or IPv6 leak-protection claim. No Cloudflare implementation, assets, credentials or keys are copied.
