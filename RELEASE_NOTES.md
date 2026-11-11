# waX VPN v0.1.26

Unsigned Windows x64 address-error checkpoint. Windows may show an unknown-publisher warning. The official WireGuardNT driver is independently signed by its vendor. No kill switch: traffic can use the normal connection if the tunnel drops.

## Changes

- Invalid Address and AllowedIPs entries now produce fixed IP-address/CIDR guidance instead of echoing the rejected entry. Existing line numbers are kept. The error names Address or AllowedIP as appropriate.
- Wording only: two error strings changed. The shared IP/prefix parser, acceptance and rejection rules, serialization and saved-profile loading are unchanged. Bare IPv4/IPv6 addresses and CIDR prefixes remain accepted. No crypto, tunnel, DNS, network-service or reconnect-policy change.

## Verification

Source 2488c0e4ae7b09c93cc2afd91d750ab98d3792d0 / full preview 42931781678 passed all required jobs. The new address_errors_explain_syntax_without_echoing_entries fixture passed with all 33 Windows core tests. It checks invalid hostnames, prefixes and private sentinels with line/field guidance and no rejected-entry echo; IPv4/IPv6 bare addresses, host prefixes and wider CIDR lists still parse with expected values and round-trip. Fixture proof only, not separate GUI screenshots of invalid address imports.

Full preview: peer handshake and tunnel-side ping, DNS and IPv4 MTU 1380, split exclusions, owned adapter-loss recovery after 5 seconds, pending-retry disable across 48 samples/12 seconds, manual reconnect with recovery off, original main HWND/PID plus Connect rail restoration, graceful quit/adapter removal, profile/settings warnings and BOM-backed settings GUI startup passed. No console windows observed in 442 samples at about 100 ms; shorter windows could be missed. Crash evidence is a clean 3-second pre-cleanup snapshot. All 14 normal, warning, BOM and installed screenshots were inspected. Fresh install/quit/uninstall passed; installed quit diagnostics show driver open 1 ms, absence confirmed 2 ms and legacy cleanup finished at 5104 ms. Exact v0.1.26 gate remains pending at preparation time.

## Limits and earlier failures

Internet reachability is not proven by the tunnel-side ping. IPv6 protection was skipped without a proven baseline and there is no kill switch. Actual notification-area menus, real UAC accept/cancel/failure/elevated handoff, profile-directory startup failure and stale-profile manual Connect pixels remain unverified.

Earlier installed quit run 42866864702 exceeded the unchanged 15-second gate; its stage log was missing. A workflow-only change ensures future diagnostic logs exist without changing timeouts or assertions. Later successful runs do not diagnose that timeout. Earlier PowerShell/CLR host failures, including 0xC0000409, are not fully diagnosed. Explicit Unicode P/Invoke declarations and stage markers were corrected, and subsequent full runs passed, but causality remains unproven.

Settings preservation covers tested failures only, not concurrent edits, hardware failure or power loss. Real Windows sign-in launch is unproven. Pending-retry disable is demonstrated, not deterministic in-flight cancellation or retry exhaustion. Adapter-loss recovery does not prove physical-network roaming. Genuine tray-construction failure and its taskbar fallback warning/restore remain untested. Mixed-case profile loading has filesystem proof rather than a separate GUI selection test. An earlier release publication HTTP 500 succeeded on failed-job retry without rebuilding; cause unknown.

WireGuardNT remains the only shipping backend. No BoringTun dependency, new product network service, Internet probe, signup, WARP registration, kill switch or IPv6 leak-protection claim. No Cloudflare implementation, assets, credentials or keys are copied.
