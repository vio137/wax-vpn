# waX VPN v0.1.18

Unsigned Windows x64 profile-comment compatibility checkpoint. Windows may show an unknown-publisher warning. The official WireGuardNT driver is independently signed by its vendor. No kill switch: traffic can use the normal connection if the tunnel drops.

## Change

- WireGuard profile headers and ordinary values can now have inline # comments. Text from the first # to the end of the line is treated as a comment, including after [Interface], [Peer], keys and numeric values. Comments are not kept in the saved normalized config. Existing whole-line semicolon comments still work; semicolons do not start inline comments.
- Parser-only change. Duplicate singleton keys, unsupported directives, unknown keys and invalid values before a comment remain errors. No scripts, new network service or reconnect policy change. Existing profile files are not rewritten by loading them.

## Verification

Source fd8fafe1969a67fe7289ab6c738cd6a604a3d473 / full preview 42883778050 passed all required jobs. Named inline_hash_comments_parse_without_relaxing_validation test passes with 23 Windows core tests: annotated BOM/CRLF config equals the plain config, normalized roundtrip drops comments, duplicate/unsupported/unknown keys still fail without value echo, invalid or empty keepalive and inline semicolons remain rejected. This is parser fixture proof, not a separately invoked annotated-config GUI import.

Full preview: peer handshake and tunnel-side ping, DNS and IPv4 MTU 1380, split exclusions, owned adapter-loss recovery after 6 seconds, pending-retry disable across 48 samples/12 seconds, manual reconnect with recovery off, original main HWND/PID plus Connect rail restoration, graceful quit/adapter removal, profile/settings warnings and BOM-backed settings GUI startup passed. No console windows observed in 473 samples at about 100 ms; shorter windows could be missed. Crash evidence is a clean 3-second pre-cleanup snapshot. All 14 normal, warning, BOM and installed screenshots were inspected. Fresh install/quit/uninstall passed; installed quit diagnostics show driver open 1 ms, absence confirmed 3 ms and legacy cleanup finished at 5367 ms. Exact v0.1.18 gate remains pending at preparation time.

## Limits and earlier failures

Internet reachability is not proven by the tunnel-side ping. IPv6 protection was skipped without a proven baseline and there is no kill switch. Actual notification-area menus, real UAC accept/cancel/failure/elevated handoff, profile-directory startup failure and stale-profile manual Connect pixels remain unverified.

Earlier installed quit run 42866864702 exceeded the unchanged 15-second gate; its stage log was missing. A workflow-only change ensures future diagnostic logs exist without changing timeouts or assertions. Later successful runs do not diagnose that timeout. Earlier PowerShell/CLR host failures, including 0xC0000409, are not fully diagnosed. Explicit Unicode P/Invoke declarations and stage markers were corrected, and subsequent full runs passed, but causality remains unproven.

Settings preservation covers tested failures only, not concurrent edits, hardware failure or power loss. Real Windows sign-in launch is unproven. Pending-retry disable is demonstrated, not deterministic in-flight cancellation or retry exhaustion. Adapter-loss recovery does not prove physical-network roaming. Genuine tray-construction failure and its taskbar fallback warning/restore remain untested. Mixed-case profile loading has filesystem proof rather than a separate GUI selection test. An earlier release publication HTTP 500 succeeded on failed-job retry without rebuilding; cause unknown.

WireGuardNT remains the only shipping backend. No BoringTun dependency, new product network service, Internet probe, signup, WARP registration, kill switch or IPv6 leak-protection claim. No Cloudflare implementation, assets, credentials or keys are copied.
