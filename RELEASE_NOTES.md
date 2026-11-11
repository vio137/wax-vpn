# waX VPN v0.1.30

Unsigned Windows x64 WARP credential Debug-redaction checkpoint. Windows may show an unknown-publisher warning. The official WireGuardNT driver is independently signed by its vendor. No kill switch: traffic can use the normal connection if the tunnel drops.

## Changes

- Account Debug output now redacts a present license while preserving None. Registration Debug output redacts its token, including nested account license. WarpAccount Debug output redacts token, license and private key, including its nested registration.
- Defensive hardening only. No production Debug credential leak was observed. Serialization, stored JSON, registration requests, profile generation, crypto, tunnel, UI and network behavior are unchanged. No signup or registration was performed.

## Verification

Source commits 2363d25f61e5524ec8614161057d8e4ee12d75e1 and adb5d605a02c615ef54c329c0321d4a48b516f2a / full preview 42949842839 passed all required jobs. The account_and_registration_debug_redact_nested_credentials and stored_account_debug_redacts_credentials_without_changing_json fixtures passed with all 3 WARP tests. Synthetic offline fixtures cover compact and pretty Account, Registration, Envelope<Registration> and WarpAccount output, absent secret sentinels, redaction markers, None and visible non-secret fields, plus JSON value retention and round-trips. Fixture proof only, not a production leak reproduction.

Full preview raw verify-results and installer diagnostics were read. Peer handshake and tunnel-side ping, DNS and IPv4 MTU 1380, split exclusions, owned adapter-loss recovery after 5 seconds, pending-retry disable across 48 samples/12 seconds, manual reconnect with recovery off, original main HWND/PID plus Connect rail restoration, graceful quit/adapter removal, profile/settings warnings and BOM-backed settings GUI startup passed. No console windows observed in 433 samples at about 100 ms; shorter windows could be missed. Crash evidence is a clean 3-second pre-cleanup snapshot. All 14 normal, warning, BOM and installed screenshots were inspected. Installed quit diagnostics show driver open/probe at 1 ms, absence confirmed 2 ms and legacy cleanup finished at 4295 ms. These timings describe this run, not a performance benchmark.

Exact v0.1.30 gate and freshly downloaded installer checksum will be recorded after verification.

## Limits and earlier failures

Internet reachability is not proven by the tunnel-side ping. IPv6 protection was skipped without a proven baseline and there is no kill switch. Actual notification-area menus, real UAC accept/cancel/failure/elevated handoff, profile-directory startup failure and stale-profile manual Connect pixels remain unverified.

The first unknown-token candidate push 42944794908 failed an older test assertion that expected the unknown token to be echoed. A fixture-only follow-up changed that expectation to fixed guidance and added an explicit no-token assertion. The new fixture already passed in the first run; subsequent full results are recorded above.

Earlier candidate push 42939812475 failed its CLI and GUI handshake deadlines and logged missing System.Drawing capture types. The unchanged full preview later passed. The cause remains unproven; no product or harness change was inferred from that failure.

Earlier installed quit run 42866864702 exceeded the unchanged 15-second gate; its stage log was missing. A workflow-only change ensures future diagnostic logs exist without changing timeouts or assertions. Later successful runs do not diagnose that timeout. Earlier PowerShell/CLR host failures, including 0xC0000409, are not fully diagnosed. Explicit Unicode P/Invoke declarations and stage markers were corrected, and subsequent full runs passed, but causality remains unproven.

Settings preservation covers tested failures only, not concurrent edits, hardware failure or power loss. Real Windows sign-in launch is unproven. Pending-retry disable is demonstrated, not deterministic in-flight cancellation or retry exhaustion. Adapter-loss recovery does not prove physical-network roaming. Genuine tray-construction failure and its taskbar fallback warning/restore remain untested. Mixed-case profile loading has filesystem proof rather than a separate GUI selection test. An earlier release publication HTTP 500 succeeded on failed-job retry without rebuilding; cause unknown.

WireGuardNT remains the only shipping backend. No BoringTun dependency, new product network service, Internet probe, signup, WARP registration, kill switch or IPv6 leak-protection claim. No Cloudflare implementation, assets, credentials or keys are copied.
