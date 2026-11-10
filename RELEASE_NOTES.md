# waX VPN v0.1.22

Unsigned Windows x64 key-import error checkpoint. Windows may show an unknown-publisher warning. The official WireGuardNT driver is independently signed by its vendor. No kill switch: traffic can use the normal connection if the tunnel drops.

## Changes

- Invalid PrivateKey, PublicKey and PresharedKey values now report their line and canonical field name with a fixed expectation: a base64-encoded 32-byte key. Errors do not include the submitted key or decoder detail.
- Error wording only. The same decoder accepts and rejects the same inputs. No crypto, serializer, tunnel, DNS, network-service or reconnect-policy change.

## Verification

Source 72fc568737a12caa54512d116a40b2174c20dfce / full preview 42911904253 passed all required jobs. The new key-error fixture passed with all 28 Windows core tests. It covers all three key fields with mixed-case spelling, malformed base64, 31/33-byte decoded lengths and empty input; line and canonical field messages; no payload echo; valid three-key roundtrip. Fixture proof only, not a separate malformed-key GUI import screenshot.

Full preview: peer handshake and tunnel-side ping, DNS and IPv4 MTU 1380, split exclusions, owned adapter-loss recovery after 4 seconds, pending-retry disable across 48 samples/12 seconds, manual reconnect with recovery off, original main HWND/PID plus Connect rail restoration, graceful quit/adapter removal, profile/settings warnings and BOM-backed settings GUI startup passed. No console windows observed in 414 samples at about 100 ms; shorter windows could be missed. Crash evidence is a clean 3-second pre-cleanup snapshot. All 14 normal, warning, BOM and installed screenshots were inspected. Fresh install/quit/uninstall passed; installed quit diagnostics show driver open 1 ms, absence confirmed 2 ms and legacy cleanup finished at 5625 ms. Exact v0.1.22 gate remains pending at preparation time.

## Limits and earlier failures

Internet reachability is not proven by the tunnel-side ping. IPv6 protection was skipped without a proven baseline and there is no kill switch. Actual notification-area menus, real UAC accept/cancel/failure/elevated handoff, profile-directory startup failure and stale-profile manual Connect pixels remain unverified.

Earlier installed quit run 42866864702 exceeded the unchanged 15-second gate; its stage log was missing. A workflow-only change ensures future diagnostic logs exist without changing timeouts or assertions. Later successful runs do not diagnose that timeout. Earlier PowerShell/CLR host failures, including 0xC0000409, are not fully diagnosed. Explicit Unicode P/Invoke declarations and stage markers were corrected, and subsequent full runs passed, but causality remains unproven.

Settings preservation covers tested failures only, not concurrent edits, hardware failure or power loss. Real Windows sign-in launch is unproven. Pending-retry disable is demonstrated, not deterministic in-flight cancellation or retry exhaustion. Adapter-loss recovery does not prove physical-network roaming. Genuine tray-construction failure and its taskbar fallback warning/restore remain untested. Mixed-case profile loading has filesystem proof rather than a separate GUI selection test. An earlier release publication HTTP 500 succeeded on failed-job retry without rebuilding; cause unknown.

WireGuardNT remains the only shipping backend. No BoringTun dependency, new product network service, Internet probe, signup, WARP registration, kill switch or IPv6 leak-protection claim. No Cloudflare implementation, assets, credentials or keys are copied.
