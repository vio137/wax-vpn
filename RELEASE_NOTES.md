# waX VPN v0.1.24

Unsigned Windows x64 endpoint-import checkpoint. Windows may show an unknown-publisher warning. The official WireGuardNT driver is independently signed by its vendor. No kill switch: traffic can use the normal connection if the tunnel drops.

## Changes

- New imports now reject structurally malformed Endpoint values before creating a profile file. This intentionally changes acceptance of malformed endpoints: a nonempty host and a port from 0 through 65535 are required; IPv6 addresses require brackets. Missing Endpoint remains allowed. This is a syntax check, not DNS resolution, hostname validation or proof that an endpoint can connect.
- Already-saved profiles with odd Endpoint values still load through the unchanged parsing/loading path. Their original files are not rewritten. Endpoint strings, including IPv6 zone suffixes, stay unchanged. The check runs only during import, before directory creation or file writes.
- Fixed peer-index errors do not include the endpoint payload. No crypto, tunnel, DNS, network-service or reconnect-policy change.

## Verification

Source 1a616fc000b4920959d859eebec19243183b6571 / full preview 42922361560 passed all required jobs. Both new fixtures passed with all 31 Windows core tests. Syntax fixtures cover IPv4, hostnames, bracketed IPv6, numeric/named IPv6 zones, ports 0/65535, missing Endpoint, peer index and preserved values; malformed ports/hosts/brackets/unbracketed IPv6 fail import validation while ordinary parsing stays permissive. Filesystem fixtures demonstrate an odd saved endpoint loads unchanged, bad imports create no directory/file and preserve existing files, and valid endpoint/zone imports roundtrip. Fixture proof only, not separate malformed-endpoint GUI import screenshots or real IPv6-zone connections.

Full preview: peer handshake and tunnel-side ping, DNS and IPv4 MTU 1380, split exclusions, owned adapter-loss recovery after 5 seconds, pending-retry disable across 48 samples/12 seconds, manual reconnect with recovery off, original main HWND/PID plus Connect rail restoration, graceful quit/adapter removal, profile/settings warnings and BOM-backed settings GUI startup passed. No console windows observed in 457 samples at about 100 ms; shorter windows could be missed. Crash evidence is a clean 3-second pre-cleanup snapshot. All 14 normal, warning, BOM and installed screenshots were inspected. Fresh install/quit/uninstall passed; installed quit diagnostics show driver open 1 ms, absence confirmed 2 ms and legacy cleanup finished at 4622 ms. Exact v0.1.24 source c76684ae2bba628a2955921e07fbdcb16b7c0a3e / run 42924289008 passed all four required jobs and published the unsigned stable release. Exact raw evidence confirms the same checks: recovery 5 seconds, recovery-disable 48 samples/12 seconds, manual reconnect with recovery off, original HWND/PID plus Connect rail restoration, graceful quit/adapter removal, warnings and BOM startup. No console windows observed in 435 samples at about 100 ms; shorter windows could be missed. All 14 exact-release screenshots were inspected. Exact fresh install/quit/uninstall passed; installed diagnostics show driver open 1 ms, absence confirmed 2 ms and legacy cleanup finished at 5721 ms. This successful run does not diagnose earlier failures.

Fresh release download: waX-VPN-Setup-v0.1.24-x64.exe, 10,958,882 bytes. SHA256 `2cefb5f13ebc5c0837debce6660dda805bcd43e707adceb43e2e7ee47ea30a3c` matches the separately downloaded SHA256SUMS-v0.1.24.txt.

## Limits and earlier failures

Internet reachability is not proven by the tunnel-side ping. IPv6 protection was skipped without a proven baseline and there is no kill switch. Actual notification-area menus, real UAC accept/cancel/failure/elevated handoff, profile-directory startup failure and stale-profile manual Connect pixels remain unverified.

Earlier installed quit run 42866864702 exceeded the unchanged 15-second gate; its stage log was missing. A workflow-only change ensures future diagnostic logs exist without changing timeouts or assertions. Later successful runs do not diagnose that timeout. Earlier PowerShell/CLR host failures, including 0xC0000409, are not fully diagnosed. Explicit Unicode P/Invoke declarations and stage markers were corrected, and subsequent full runs passed, but causality remains unproven.

Settings preservation covers tested failures only, not concurrent edits, hardware failure or power loss. Real Windows sign-in launch is unproven. Pending-retry disable is demonstrated, not deterministic in-flight cancellation or retry exhaustion. Adapter-loss recovery does not prove physical-network roaming. Genuine tray-construction failure and its taskbar fallback warning/restore remain untested. Mixed-case profile loading has filesystem proof rather than a separate GUI selection test. An earlier release publication HTTP 500 succeeded on failed-job retry without rebuilding; cause unknown.

WireGuardNT remains the only shipping backend. No BoringTun dependency, new product network service, Internet probe, signup, WARP registration, kill switch or IPv6 leak-protection claim. No Cloudflare implementation, assets, credentials or keys are copied.
