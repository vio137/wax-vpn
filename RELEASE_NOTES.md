# waX VPN v0.1.35

Unsigned Windows x64 profile-import guidance checkpoint. Windows may show an unknown-publisher warning. The official WireGuardNT driver is independently signed by its vendor. No kill switch: traffic can use the normal connection if the tunnel drops.

## Changes

- Profiles now explains that imports keep existing profiles and give duplicate names a number. This is clearer guidance for existing behavior, not a change to parsing, storage, selection, connections or network policy.
- Folded in test-only cancellation ownership fixtures and a one-shot Windows Notifications/Snapshot bridge. These prove the supplied stop/result/ack contract and one successful register/baseline/read/shutdown/ack sequence only. No production observer, continuous monitoring, quit-ack integration or physical-network roaming is implemented.

## Verification

Product source dced2505d8a9bda67531a7f5903ae4ecf9884fb8 passed push 42976780122 and full preview 42977715778. All 41 app, 40 core, 6 WARP, 4 diagnostics and 1 tunnel tests passed. The Profiles helper fits fully on one line at the 880x600 runner window; all 14 normal, warning, BOM and installed screenshots were inspected.

Raw full-preview verify-results and installer diagnostics were read. Handshake and tunnel-side ping, DNS and IPv4 MTU 1380, split exclusions, owned adapter-loss recovery after 5 seconds, pending-retry disable across 48 samples/12 seconds, manual reconnect with recovery off, original main HWND/PID plus Connect rail restoration, graceful quit/adapter removal, profile/settings warnings and BOM-backed startup passed. No console windows observed in 458 samples at about 100 ms; shorter windows could be missed. Crash evidence is a clean 3-second pre-cleanup snapshot. Fresh-runner install/quit/uninstall passed. Installed quit stages were probe 1 ms, absence 2 ms and legacy removal 6142 ms, descriptive only.

Exact v0.1.35 run 42979370349 at c4e39ff0dfc0691a8cb93d8ea0043db299f9cac9 passed build, demo-tunnel/GUI verification, fresh-runner install/quit/uninstall and publish. Raw verify-results and installer diagnostics were read. The same scoped checks passed, including adapter-loss recovery after 5 seconds, pending-retry disable across 48 samples/12 seconds, original HWND/PID plus Connect rail restoration, graceful quit/adapter removal and warning/BOM cases. No console windows observed in 461 samples at about 100 ms; shorter windows could be missed. All 14 exact-release screenshots were inspected, including the readable Profiles helper at 880x600. Installed quit stages were probe 1 ms, absence 3 ms and legacy cleanup 4372 ms, descriptive only.

Freshly downloaded unsigned installer: 10,958,379 bytes. SHA256 3fe132b7b0286d8da8a0082138f815d31e1b46d65b9fa99cf44f1ca8997f042a matches the freshly downloaded SHA256SUMS-v0.1.35.txt.

## Limits and earlier failures

Internet reachability is not proven by the tunnel-side ping. IPv6 protection was skipped without a proven baseline and there is no kill switch. Actual notification-area menus, real UAC accept/cancel/failure/elevated handoff, profile-directory startup failure and stale-profile manual Connect pixels remain unverified.

The first unknown-token candidate push 42944794908 failed an older test assertion that expected the unknown token to be echoed. A fixture-only follow-up changed that expectation to fixed guidance and added an explicit no-token assertion. The new fixture already passed in the first run; subsequent full results are recorded above.

Earlier candidate push 42939812475 failed its CLI and GUI handshake deadlines and logged missing System.Drawing capture types. The unchanged full preview later passed. The cause remains unproven; no product or harness change was inferred from that failure.

Earlier installed quit run 42866864702 exceeded the unchanged 15-second gate; its stage log was missing. A workflow-only change ensures future diagnostic logs exist without changing timeouts or assertions. Later successful runs do not diagnose that timeout. Earlier PowerShell/CLR host failures, including 0xC0000409, are not fully diagnosed. Explicit Unicode P/Invoke declarations and stage markers were corrected, and subsequent full runs passed, but causality remains unproven.

Settings preservation covers tested failures only, not concurrent edits, hardware failure or power loss. Real Windows sign-in launch is unproven. Pending-retry disable is demonstrated, not deterministic in-flight cancellation or retry exhaustion. Adapter-loss recovery does not prove physical-network roaming. Genuine tray-construction failure and its taskbar fallback warning/restore remain untested. Mixed-case profile loading has filesystem proof rather than a separate GUI selection test. An earlier release publication HTTP 500 succeeded on failed-job retry without rebuilding; cause unknown.

WireGuardNT remains the only shipping backend. No BoringTun dependency, new product network service, Internet probe, signup, WARP registration, kill switch or IPv6 leak-protection claim. No Cloudflare implementation, assets, credentials or keys are copied.
