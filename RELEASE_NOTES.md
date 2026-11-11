# waX VPN v0.1.29

Unsigned Windows x64 unknown-token error-wording checkpoint. Windows may show an unknown-publisher warning. The official WireGuardNT driver is independently signed by its vendor. No kill switch: traffic can use the normal connection if the tunnel drops.

## Changes

- Unknown section, unknown Interface key and unknown Peer key errors now give fixed expected-section/key guidance instead of repeating the unrecognized token. Line numbers remain present.
- Defensive wording only, not a confirmed leak fix. All acceptance/rejection rules and known unsupported wg-quick directive handling are unchanged. No serialization, profile-storage, crypto, tunnel, UI or network behavior change.

## Verification

Source 01c2212cf18d403d1e1ee3cd874d00635d014ff9 plus fixture follow-up f2d90ef822468c675a1e9e735737b7ca1c63eba5 / full preview 42946492976 passed all required jobs. The unknown_tokens_use_fixed_guidance_without_echoing_payloads and inline_hash_comments_parse_without_relaxing_validation fixtures passed with all 36 Windows core tests. The new fixture checks unknown sections and keys in both sections, long sentinel tokens, leading whitespace/BOM/comments, mixed-case known headers, retained line/category/guidance, no offending token/value in the error, all seven known unsupported directives and valid round-trips. Fixture proof only, not separate GUI screenshots of those imports.

Exact v0.1.29 run 42947907711 at 8ba657cd35b1b1a88c710146ea27b8d00aca0825 passed build, demo-tunnel/GUI verification, fresh-runner install/quit/uninstall and publish. Raw verify-results and installer diagnostics were read. Peer handshake and tunnel-side ping, DNS and IPv4 MTU 1380, split exclusions, owned adapter-loss recovery after 5 seconds, pending-retry disable across 48 samples/12 seconds, manual reconnect with recovery off, original main HWND/PID plus Connect rail restoration, graceful quit/adapter removal, profile/settings warnings and BOM-backed settings GUI startup passed. No console windows observed in 441 samples at about 100 ms; shorter windows could be missed. Crash evidence is a clean 3-second pre-cleanup snapshot. All 14 normal, warning, BOM and installed screenshots were inspected. Installed quit diagnostics show driver open/probe at 1 ms, absence confirmed 3 ms and legacy cleanup finished at 4548 ms. These timings describe this run, not a performance benchmark.

Freshly downloaded unsigned installer: 10,958,662 bytes. SHA256 70ab1589f878e87dfa4dee759d590d9b00910ec453628d10482e699efbafd534 matches the freshly downloaded SHA256SUMS-v0.1.29.txt.

## Limits and earlier failures

Internet reachability is not proven by the tunnel-side ping. IPv6 protection was skipped without a proven baseline and there is no kill switch. Actual notification-area menus, real UAC accept/cancel/failure/elevated handoff, profile-directory startup failure and stale-profile manual Connect pixels remain unverified.

The first unknown-token candidate push 42944794908 failed an older test assertion that expected the unknown token to be echoed. A fixture-only follow-up changed that expectation to fixed guidance and added an explicit no-token assertion. The new fixture already passed in the first run; subsequent full results are recorded above.

Earlier candidate push 42939812475 failed its CLI and GUI handshake deadlines and logged missing System.Drawing capture types. The unchanged full preview later passed. The cause remains unproven; no product or harness change was inferred from that failure.

Earlier installed quit run 42866864702 exceeded the unchanged 15-second gate; its stage log was missing. A workflow-only change ensures future diagnostic logs exist without changing timeouts or assertions. Later successful runs do not diagnose that timeout. Earlier PowerShell/CLR host failures, including 0xC0000409, are not fully diagnosed. Explicit Unicode P/Invoke declarations and stage markers were corrected, and subsequent full runs passed, but causality remains unproven.

Settings preservation covers tested failures only, not concurrent edits, hardware failure or power loss. Real Windows sign-in launch is unproven. Pending-retry disable is demonstrated, not deterministic in-flight cancellation or retry exhaustion. Adapter-loss recovery does not prove physical-network roaming. Genuine tray-construction failure and its taskbar fallback warning/restore remain untested. Mixed-case profile loading has filesystem proof rather than a separate GUI selection test. An earlier release publication HTTP 500 succeeded on failed-job retry without rebuilding; cause unknown.

WireGuardNT remains the only shipping backend. No BoringTun dependency, new product network service, Internet probe, signup, WARP registration, kill switch or IPv6 leak-protection claim. No Cloudflare implementation, assets, credentials or keys are copied.
