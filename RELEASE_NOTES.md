# waX VPN v0.1.25

Unsigned Windows x64 numeric-error checkpoint. Windows may show an unknown-publisher warning. The official WireGuardNT driver is independently signed by its vendor. No kill switch: traffic can use the normal connection if the tunnel drops.

## Changes

- Numeric parse errors now explain the expected range for MTU, ListenPort and PersistentKeepalive while retaining the existing line number. Errors do not include the rejected value.
- MTU wording names 576 through 65535, with a minimum of 1280 when an IPv6 interface address is present. ListenPort accepts 0 through 65535. PersistentKeepalive accepts off or 0 through 65535 seconds. These are existing parser rules, not new validation or a guarantee that a device/network supports every value.
- Three fixed error strings changed. Numeric parsing, accepted configurations, duplicate handling and the separate MTU address-family minimum check are unchanged. No profile-loading, crypto, tunnel, DNS, network-service or reconnect-policy change.

## Verification

Source 94950da669e8575dcb26041b9a62acb7a8df814d / full preview 42927144125 passed all required jobs. The new numeric_errors_explain_ranges_without_echoing_values fixture passed with all 32 Windows core tests. It checks negative, nonnumeric, overflow, fractional and empty errors with line/field guidance and no value echo; ListenPort/keepalive 0 and 65535, keepalive off, MTU 65535 and 576/1280 boundaries; and unchanged separate MTU minimum errors for 0/575/1279. Fixture proof only, not separate GUI screenshots of invalid numeric imports.

Full preview: peer handshake and tunnel-side ping, DNS and IPv4 MTU 1380, split exclusions, owned adapter-loss recovery after 6 seconds, pending-retry disable across 48 samples/12 seconds, manual reconnect with recovery off, original main HWND/PID plus Connect rail restoration, graceful quit/adapter removal, profile/settings warnings and BOM-backed settings GUI startup passed. No console windows observed in 465 samples at about 100 ms; shorter windows could be missed. Crash evidence is a clean 3-second pre-cleanup snapshot. All 14 normal, warning, BOM and installed screenshots were inspected. Fresh install/quit/uninstall passed; installed quit diagnostics show driver open 1 ms, absence confirmed 3 ms and legacy cleanup finished at 4347 ms. Exact v0.1.25 source 17a9ae54eee84fbe76321f869732775d0bb3eb30 / run 42928838728 passed all four required jobs and published the unsigned stable release. Exact raw evidence confirms the same checks: recovery 5 seconds, recovery-disable 48 samples/12 seconds, manual reconnect with recovery off, original HWND/PID plus Connect rail restoration, graceful quit/adapter removal, warnings and BOM startup. No console windows observed in 440 samples at about 100 ms; shorter windows could be missed. All 14 exact-release screenshots were inspected. Exact fresh install/quit/uninstall passed; installed diagnostics show driver open 1 ms, absence confirmed 2 ms and legacy cleanup finished at 4534 ms. This successful run does not diagnose earlier failures.

Fresh release download: waX-VPN-Setup-v0.1.25-x64.exe, 10,958,801 bytes. SHA256 `72a69f6ab62749d6159e25a8a003198b67b1c73309f1609071faa431169b89f1` matches the separately downloaded SHA256SUMS-v0.1.25.txt.

## Limits and earlier failures

Internet reachability is not proven by the tunnel-side ping. IPv6 protection was skipped without a proven baseline and there is no kill switch. Actual notification-area menus, real UAC accept/cancel/failure/elevated handoff, profile-directory startup failure and stale-profile manual Connect pixels remain unverified.

Earlier installed quit run 42866864702 exceeded the unchanged 15-second gate; its stage log was missing. A workflow-only change ensures future diagnostic logs exist without changing timeouts or assertions. Later successful runs do not diagnose that timeout. Earlier PowerShell/CLR host failures, including 0xC0000409, are not fully diagnosed. Explicit Unicode P/Invoke declarations and stage markers were corrected, and subsequent full runs passed, but causality remains unproven.

Settings preservation covers tested failures only, not concurrent edits, hardware failure or power loss. Real Windows sign-in launch is unproven. Pending-retry disable is demonstrated, not deterministic in-flight cancellation or retry exhaustion. Adapter-loss recovery does not prove physical-network roaming. Genuine tray-construction failure and its taskbar fallback warning/restore remain untested. Mixed-case profile loading has filesystem proof rather than a separate GUI selection test. An earlier release publication HTTP 500 succeeded on failed-job retry without rebuilding; cause unknown.

WireGuardNT remains the only shipping backend. No BoringTun dependency, new product network service, Internet probe, signup, WARP registration, kill switch or IPv6 leak-protection claim. No Cloudflare implementation, assets, credentials or keys are copied.
