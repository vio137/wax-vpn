# waX VPN v0.1.34

Unsigned Windows x64 Auto-reconnect guidance checkpoint. Windows may show an unknown-publisher warning. The official WireGuardNT driver is independently signed by its vendor. No kill switch: traffic can use the normal connection if the tunnel drops.

## Changes

- Features now explains that Auto-reconnect retries owned adapter loss and does not yet support physical-network roaming. Toggle behavior, retry delays and budget policy are unchanged. This is clearer guidance, not a new recovery capability.
- Added offline split-subtraction boundary regression coverage and a pure test-only coordinator for caller-supplied underlay snapshots. These do not change production routing or wire network notifications into recovery. Regression coverage, not a bug fix or leak proof. Physical-network roaming remains unimplemented.

## Verification

Source c28e451ae3b0cce0d736cdcfacfac115650053cd / full preview 42970156634 passed all required jobs. All 35 app, 40 core, 6 WARP, 4 diagnostics and 1 tunnel tests passed, including the four coordinator and three split-boundary fixtures. Features pixels at the 880x600 runner window show readable two-line guidance and an unobscured Auto-reconnect toggle. No layout or policy change was needed.

Full preview raw verify-results and installer diagnostics were read. Peer handshake and tunnel-side ping, DNS and IPv4 MTU 1380, split exclusions, owned adapter-loss recovery after 4 seconds, pending-retry disable across 48 samples/12 seconds, manual reconnect with recovery off, original main HWND/PID plus Connect rail restoration, graceful quit/adapter removal, profile/settings warnings and BOM-backed settings GUI startup passed. No console windows observed in 459 samples at about 100 ms; shorter windows could be missed. Crash evidence is a clean 3-second pre-cleanup snapshot. All 14 normal, warning, BOM and installed screenshots were inspected. Installed quit diagnostics show driver open/probe at 1 ms, absence confirmed 3 ms and legacy cleanup finished at 4521 ms. These timings describe this run, not a performance benchmark.

Exact v0.1.34 gate and freshly downloaded installer/hash checks are pending. This checkpoint is not promotion-ready until those checks and final release notes are complete.

## Limits and earlier failures

Internet reachability is not proven by the tunnel-side ping. IPv6 protection was skipped without a proven baseline and there is no kill switch. Actual notification-area menus, real UAC accept/cancel/failure/elevated handoff, profile-directory startup failure and stale-profile manual Connect pixels remain unverified.

The first unknown-token candidate push 42944794908 failed an older test assertion that expected the unknown token to be echoed. A fixture-only follow-up changed that expectation to fixed guidance and added an explicit no-token assertion. The new fixture already passed in the first run; subsequent full results are recorded above.

Earlier candidate push 42939812475 failed its CLI and GUI handshake deadlines and logged missing System.Drawing capture types. The unchanged full preview later passed. The cause remains unproven; no product or harness change was inferred from that failure.

Earlier installed quit run 42866864702 exceeded the unchanged 15-second gate; its stage log was missing. A workflow-only change ensures future diagnostic logs exist without changing timeouts or assertions. Later successful runs do not diagnose that timeout. Earlier PowerShell/CLR host failures, including 0xC0000409, are not fully diagnosed. Explicit Unicode P/Invoke declarations and stage markers were corrected, and subsequent full runs passed, but causality remains unproven.

Settings preservation covers tested failures only, not concurrent edits, hardware failure or power loss. Real Windows sign-in launch is unproven. Pending-retry disable is demonstrated, not deterministic in-flight cancellation or retry exhaustion. Adapter-loss recovery does not prove physical-network roaming. Genuine tray-construction failure and its taskbar fallback warning/restore remain untested. Mixed-case profile loading has filesystem proof rather than a separate GUI selection test. An earlier release publication HTTP 500 succeeded on failed-job retry without rebuilding; cause unknown.

WireGuardNT remains the only shipping backend. No BoringTun dependency, new product network service, Internet probe, signup, WARP registration, kill switch or IPv6 leak-protection claim. No Cloudflare implementation, assets, credentials or keys are copied.
