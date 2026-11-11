# waX VPN v0.1.27

Unsigned Windows x64 Interface-error checkpoint. Windows may show an unknown-publisher warning. The official WireGuardNT driver is independently signed by its vendor. No kill switch: traffic can use the normal connection if the tunnel drops.

## Changes

- A config with an [Interface] section but no PrivateKey now reports that section's first header line: line N: [Interface] missing PrivateKey. Mixed-case section names are recognized as before. The existing interface missing PrivateKey fallback remains when no Interface section exists.
- Wording only: first-header line tracking is used for this error. Acceptance and rejection rules, serialization and saved-profile loading are unchanged, including repeated Interface handling. No crypto, tunnel, DNS, network-service or reconnect-policy change.

## Verification

Source 420657a870154d44d7c6bc4730e802fab38a8e7d / full preview 42936408572 passed all required jobs. The missing_private_key_identifies_first_interface_header_without_payloads fixture passed with all 34 Windows core tests. It checks normal, mixed-case, BOM and leading-comment headers, first-header selection with repeated Interface sections, the no-interface fallback, absence of config payloads in the error and valid round-trips. Fixture proof only, not separate GUI screenshots of missing-key imports.

Exact v0.1.27 run 42938264349 at f734afd262de5f9411e8181b7c7500f04da0c49c passed build, demo-tunnel/GUI verification, fresh-runner install/quit/uninstall and publish. Raw verify-results and installer diagnostics were read. Peer handshake and tunnel-side ping, DNS and IPv4 MTU 1380, split exclusions, owned adapter-loss recovery after 6 seconds, pending-retry disable across 48 samples/12 seconds, manual reconnect with recovery off, original main HWND/PID plus Connect rail restoration, graceful quit/adapter removal, profile/settings warnings and BOM-backed settings GUI startup passed. No console windows observed in 416 samples at about 100 ms; shorter windows could be missed. Crash evidence is a clean 3-second pre-cleanup snapshot. All 14 normal, warning, BOM and installed screenshots were inspected. Installed quit diagnostics show driver open completed at 1 ms, absence confirmed at 3 ms and legacy cleanup finished at 4873 ms. These timings describe this run, not a performance benchmark.

Freshly downloaded unsigned installer: 10,958,737 bytes. SHA256 d88d3b17389d12c0fdab5667e00e7c2fa1efd3711866f080e9ccc49bcc780130 matches the freshly downloaded SHA256SUMS-v0.1.27.txt.

## Limits and earlier failures

Internet reachability is not proven by the tunnel-side ping. IPv6 protection was skipped without a proven baseline and there is no kill switch. Actual notification-area menus, real UAC accept/cancel/failure/elevated handoff, profile-directory startup failure and stale-profile manual Connect pixels remain unverified.

Earlier installed quit run 42866864702 exceeded the unchanged 15-second gate; its stage log was missing. A workflow-only change ensures future diagnostic logs exist without changing timeouts or assertions. Later successful runs do not diagnose that timeout. Earlier PowerShell/CLR host failures, including 0xC0000409, are not fully diagnosed. Explicit Unicode P/Invoke declarations and stage markers were corrected, and subsequent full runs passed, but causality remains unproven.

Settings preservation covers tested failures only, not concurrent edits, hardware failure or power loss. Real Windows sign-in launch is unproven. Pending-retry disable is demonstrated, not deterministic in-flight cancellation or retry exhaustion. Adapter-loss recovery does not prove physical-network roaming. Genuine tray-construction failure and its taskbar fallback warning/restore remain untested. Mixed-case profile loading has filesystem proof rather than a separate GUI selection test. An earlier release publication HTTP 500 succeeded on failed-job retry without rebuilding; cause unknown.

WireGuardNT remains the only shipping backend. No BoringTun dependency, new product network service, Internet probe, signup, WARP registration, kill switch or IPv6 leak-protection claim. No Cloudflare implementation, assets, credentials or keys are copied.
