# waX VPN v0.1.36

Unsigned Windows x64 local-diagnostics clarity checkpoint. Windows may show an unknown-publisher warning. The official WireGuardNT driver is independently signed by its vendor. No kill switch: traffic can use the normal connection if the tunnel drops.

## Changes

- Local diagnostics now shows the report age and how to refresh it. The age is UI-only elapsed time since report arrival, not an OS measurement timestamp. Existing snapshot and Internet-not-tested guidance wraps for readability. Report JSON, copied payload, OS queries, repaint schedule and connection policy are unchanged.
- Clear removes report and age; a new report arrival resets the age. Those age-reset transitions are code-reasoned, not separately asserted by the gate. Existing generate/clear behavior is checked. No background monitoring is added.

## Verification

Product source 5d81e51f61cb697931f65cd20c3d8a61c5cdd9ce passed push 42981469159 and full preview 42982601482. All 41 app, 40 core, 6 WARP, 4 diagnostics and 1 tunnel tests passed. Actual diagnostics pixels at 880x600 show readable report age, refresh guidance and snapshot warning with Generate/Copy/Clear unobscured. All 14 normal, warning, BOM and installed screenshots were inspected.

Raw full-preview verify-results and installer diagnostics were read. Handshake and tunnel-side ping, DNS and IPv4 MTU 1380, split exclusions, adapter-loss recovery after 4 seconds, pending-retry disable across 48 samples/12 seconds, manual reconnect, original main HWND/PID plus Connect rail restoration, graceful quit/adapter removal, profile/settings warnings and BOM-backed startup passed. No console windows observed in 366 samples at about 100 ms; shorter windows could be missed. Crash evidence is a clean 3-second pre-cleanup snapshot. Fresh-runner install/quit/uninstall passed. Installed quit stages were probe 1 ms, absence 4 ms and legacy removal 4412 ms, descriptive only.

Exact v0.1.36 gate and fresh installer checksum are pending. Preview evidence is not exact release verification.

Test-only cancellation ownership and one-shot Windows Notifications/Snapshot fixtures remain groundwork only. Their successful contract/bridge checks do not prove production observation, continuous monitoring, quit acknowledgement, OS cancellation-failure safety or actual physical-network roaming. No such feature is wired into the app.

## Limits and earlier failures

Internet reachability is not proven by the tunnel-side ping. IPv6 protection was skipped without a proven baseline and there is no kill switch. Actual notification-area menus, real UAC accept/cancel/failure/elevated handoff, profile-directory startup failure and stale-profile manual Connect pixels remain unverified.

The first unknown-token candidate push 42944794908 failed an older test assertion that expected the unknown token to be echoed. A fixture-only follow-up changed that expectation to fixed guidance and added an explicit no-token assertion. The new fixture already passed in the first run; subsequent full results are recorded above.

Earlier candidate push 42939812475 failed its CLI and GUI handshake deadlines and logged missing System.Drawing capture types. The unchanged full preview later passed. The cause remains unproven; no product or harness change was inferred from that failure.

Earlier installed quit run 42866864702 exceeded the unchanged 15-second gate; its stage log was missing. A workflow-only change ensures future diagnostic logs exist without changing timeouts or assertions. Later successful runs do not diagnose that timeout. Earlier PowerShell/CLR host failures, including 0xC0000409, are not fully diagnosed. Explicit Unicode P/Invoke declarations and stage markers were corrected, and subsequent full runs passed, but causality remains unproven.

Settings preservation covers tested failures only, not concurrent edits, hardware failure or power loss. Real Windows sign-in launch is unproven. Pending-retry disable is demonstrated, not deterministic in-flight cancellation or retry exhaustion. Adapter-loss recovery does not prove physical-network roaming. Genuine tray-construction failure and its taskbar fallback warning/restore remain untested. Mixed-case profile loading has filesystem proof rather than a separate GUI selection test. An earlier release publication HTTP 500 succeeded on failed-job retry without rebuilding; cause unknown.

WireGuardNT remains the only shipping backend. No BoringTun dependency, new product network service, Internet probe, signup, WARP registration, kill switch or IPv6 leak-protection claim. No Cloudflare implementation, assets, credentials or keys are copied.
