# waX VPN v0.1.37

Unsigned Windows x64 profile-library clarity checkpoint. Windows may show an unknown-publisher warning. The official WireGuardNT driver is independently signed by its vendor. No kill switch: traffic can use the normal connection if the tunnel drops.

## Changes

- Profiles now shows the loaded-profile count beside Reload profiles. After a failed reload, the retained count is labeled Previous list. Loading, selection, import, storage and connection policy are unchanged.
- Formatter fixtures cover zero, one and many profiles for current and retained lists. Empty/plural wording is formatter-fixture proof only, not a separately captured UI state.

## Verification

Product source fa11466717dde45c92e4fd9f3a0ac4d9e8b9917a passed push 42986624357 and full preview 42987951097. All 42 app, 40 core, 6 WARP, 4 diagnostics and 1 tunnel tests passed. Actual normal and partial-warning Profiles pixels show 1 profile loaded. Failed reload shows Previous list: 1 profile, retained list and disabled Connect/Delete controls. Count, warning and buttons are readable without overlap at 880x600. All 14 normal, warning, BOM and installed screenshots were inspected.

Raw full-preview verify-results and installer diagnostics were read. Handshake and tunnel-side ping, DNS and IPv4 MTU 1380, split exclusions, adapter-loss recovery after 4 seconds, pending-retry disable across 48 samples/12 seconds, manual reconnect, original main HWND/PID plus Connect rail restoration, graceful quit/adapter removal, profile/settings warnings and BOM-backed startup passed. No console windows observed in 434 samples at about 100 ms; shorter windows could be missed. Crash evidence is a clean 3-second pre-cleanup snapshot. Fresh-runner install/quit/uninstall passed. Installed quit stages were probe 1 ms, absence 1 ms and legacy removal 4063 ms, descriptive only.

Exact v0.1.37 stable gate and fresh installer checksum are pending. No exact-release result is claimed here.

Test-only cancellation ownership and one-shot Windows Notifications/Snapshot fixtures remain groundwork only. Their successful contract/bridge checks do not prove production observation, continuous monitoring, quit acknowledgement, OS cancellation-failure safety or actual physical-network roaming. No such feature is wired into the app.

## Limits and earlier failures

Internet reachability is not proven by the tunnel-side ping. IPv6 protection was skipped without a proven baseline and there is no kill switch. Actual notification-area menus, real UAC accept/cancel/failure/elevated handoff, profile-directory startup failure and stale-profile manual Connect pixels remain unverified.

The first unknown-token candidate push 42944794908 failed an older test assertion that expected the unknown token to be echoed. A fixture-only follow-up changed that expectation to fixed guidance and added an explicit no-token assertion. The new fixture already passed in the first run; subsequent full results are recorded above.

Earlier candidate push 42939812475 failed its CLI and GUI handshake deadlines and logged missing System.Drawing capture types. The unchanged full preview later passed. The cause remains unproven; no product or harness change was inferred from that failure.

Earlier installed quit run 42866864702 exceeded the unchanged 15-second gate; its stage log was missing. A workflow-only change ensures future diagnostic logs exist without changing timeouts or assertions. Later successful runs do not diagnose that timeout. Earlier PowerShell/CLR host failures, including 0xC0000409, are not fully diagnosed. Explicit Unicode P/Invoke declarations and stage markers were corrected, and subsequent full runs passed, but causality remains unproven.

Settings preservation covers tested failures only, not concurrent edits, hardware failure or power loss. Real Windows sign-in launch is unproven. Pending-retry disable is demonstrated, not deterministic in-flight cancellation or retry exhaustion. Adapter-loss recovery does not prove physical-network roaming. Genuine tray-construction failure and its taskbar fallback warning/restore remain untested. Mixed-case profile loading has filesystem proof rather than a separate GUI selection test. An earlier release publication HTTP 500 succeeded on failed-job retry without rebuilding; cause unknown.

WireGuardNT remains the only shipping backend. No BoringTun dependency, new product network service, Internet probe, signup, WARP registration, kill switch or IPv6 leak-protection claim. No Cloudflare implementation, assets, credentials or keys are copied.
