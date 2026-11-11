# waX VPN v0.1.40

Unsigned Windows x64 split-field label checkpoint. Windows may show an unknown-publisher warning. The official WireGuardNT driver is independently signed by its vendor. No kill switch: traffic can use the normal connection if the tunnel drops.

## Changes

- The split input now has a visible "Excluded IP ranges for profile" label with the profile name. Existing wrapping, spacing and accessible input name are unchanged. Parsing, saving, routes and connection policy are unchanged.

## Verification

Product source f02ec9b124776bf665163e1127e1e23bec7a74dc passed push 43003802391 and full preview 43005252907. Raw build outputs confirm 43 app, 40 core, 6 WARP, 4 diagnostics and 1 tunnel tests passed. All 14 full-preview normal, warning, BOM and installed screenshots were inspected. Normal, partial-warning and failed-reload Profiles pixels show the field label, profile name, count, guidance and exclusion input readable without overlap at 880x600. Import stays within the existing scroll area. Unicode/long names remain formatter fixtures, not glyph/wrapping screenshot proof.

Raw full-preview verification and installer diagnostics were read. Handshake and tunnel-side ping, DNS and IPv4 MTU 1380, split exclusions, adapter-loss recovery after 5 seconds, pending-retry disable across 48 samples/12 seconds, manual reconnect, original main HWND/PID plus visible Connect rail restoration, graceful quit/adapter removal, profile/settings warnings and BOM-backed startup passed. No console windows observed in 447 samples at about 100 ms; shorter windows could be missed. Crash evidence is a clean 3-second pre-cleanup snapshot. Fresh-runner install/quit/uninstall passed. Installed quit stages were probe 1 ms, absence 3 ms and legacy removal 4273 ms, descriptive only.

Exact v0.1.40 stable gate and fresh installer checksum are pending. No exact-release result is claimed here.

Test-only cancellation ownership and one-shot Windows Notifications/Snapshot fixtures remain groundwork only. Their successful contract/bridge checks do not prove production observation, continuous monitoring, quit acknowledgement, OS cancellation-failure safety or actual physical-network roaming. No such feature is wired into the app. Unicode/long profile names remain formatter-fixture preservation proof, not actual glyph or wrapping screenshot proof.

## Limits and earlier failures

Internet reachability is not proven by the tunnel-side ping. IPv6 protection was skipped without a proven baseline and there is no kill switch. Actual notification-area menus, real UAC accept/cancel/failure/elevated handoff, profile-directory startup failure and stale-profile manual Connect pixels remain unverified.

The first unknown-token candidate push 42944794908 failed an older test assertion that expected the unknown token to be echoed. A fixture-only follow-up changed that expectation to fixed guidance and added an explicit no-token assertion. The new fixture already passed in the first run; subsequent full results are recorded above.

Earlier candidate push 42939812475 failed its CLI and GUI handshake deadlines and logged missing System.Drawing capture types. The unchanged full preview later passed. The cause remains unproven; no product or harness change was inferred from that failure.

Earlier installed quit run 42866864702 exceeded the unchanged 15-second gate; its stage log was missing. A workflow-only change ensures future diagnostic logs exist without changing timeouts or assertions. Later successful runs do not diagnose that timeout. Earlier PowerShell/CLR host failures, including 0xC0000409, are not fully diagnosed. Explicit Unicode P/Invoke declarations and stage markers were corrected, and subsequent full runs passed, but causality remains unproven.

Settings preservation covers tested failures only, not concurrent edits, hardware failure or power loss. Real Windows sign-in launch is unproven. Pending-retry disable is demonstrated, not deterministic in-flight cancellation or retry exhaustion. Adapter-loss recovery does not prove physical-network roaming. Genuine tray-construction failure and its taskbar fallback warning/restore remain untested. Mixed-case profile loading has filesystem proof rather than a separate GUI selection test. An earlier release publication HTTP 500 succeeded on failed-job retry without rebuilding; cause unknown.

WireGuardNT remains the only shipping backend. No BoringTun dependency, new product network service, Internet probe, signup, WARP registration, kill switch or IPv6 leak-protection claim. No Cloudflare implementation, assets, credentials or keys are copied.
