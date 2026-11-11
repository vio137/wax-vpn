# waX VPN v0.1.28

Unsigned Windows x64 defensive Debug-redaction checkpoint. Windows may show an unknown-publisher warning. The official WireGuardNT driver is independently signed by its vendor. No kill switch: traffic can use the normal connection if the tunnel drops.

## Changes

- InterfaceConfig and PeerConfig Debug output now replaces private keys and present preshared keys with a fixed <redacted> marker. Nested WireGuardConfig and Profile Debug output inherits the protection. Absent preshared keys remain None; public keys and nonsecret fields stay visible.
- Defensive hardening only, with no observed production leak. No parser, serialization, profile storage, crypto, tunnel, UI or network behavior change. Serialized configs still contain the required keys as before and must be kept private.

## Verification

Source e4ae8cf85d0754ead85cb85dea50b0b5093fca3b / full preview 42941505112 passed all required jobs. The config_debug_redacts_secret_keys_including_nested_profiles fixture passed with all 35 Windows core tests. It checks compact and pretty Debug for InterfaceConfig, PeerConfig, WireGuardConfig and Profile against raw-array and base64 secret sentinels, visible nonsecret fields, Some/None preshared-key behavior and unchanged serialization/parse round-trips. Fixture proof only, not a claim that a production leak was observed.

Full preview: peer handshake and tunnel-side ping, DNS and IPv4 MTU 1380, split exclusions, owned adapter-loss recovery after 5 seconds, pending-retry disable across 48 samples/12 seconds, manual reconnect with recovery off, original main HWND/PID plus Connect rail restoration, graceful quit/adapter removal, profile/settings warnings and BOM-backed settings GUI startup passed. No console windows observed in 379 samples at about 100 ms; shorter windows could be missed. Crash evidence is a clean 3-second pre-cleanup snapshot. All 14 normal, warning, BOM and installed screenshots were inspected. Fresh install/quit/uninstall passed; installed quit diagnostics show driver open/probe at 11 ms, absence confirmed 15 ms and legacy cleanup finished at 4615 ms. Exact v0.1.28 gate remains pending at preparation time.

## Limits and earlier failures

Internet reachability is not proven by the tunnel-side ping. IPv6 protection was skipped without a proven baseline and there is no kill switch. Actual notification-area menus, real UAC accept/cancel/failure/elevated handoff, profile-directory startup failure and stale-profile manual Connect pixels remain unverified.

Candidate push 42939812475 failed its CLI and GUI handshake deadlines and logged missing System.Drawing capture types. The unchanged full preview later passed. The cause remains unproven; no product or harness change was inferred from that failure.

Earlier installed quit run 42866864702 exceeded the unchanged 15-second gate; its stage log was missing. A workflow-only change ensures future diagnostic logs exist without changing timeouts or assertions. Later successful runs do not diagnose that timeout. Earlier PowerShell/CLR host failures, including 0xC0000409, are not fully diagnosed. Explicit Unicode P/Invoke declarations and stage markers were corrected, and subsequent full runs passed, but causality remains unproven.

Settings preservation covers tested failures only, not concurrent edits, hardware failure or power loss. Real Windows sign-in launch is unproven. Pending-retry disable is demonstrated, not deterministic in-flight cancellation or retry exhaustion. Adapter-loss recovery does not prove physical-network roaming. Genuine tray-construction failure and its taskbar fallback warning/restore remain untested. Mixed-case profile loading has filesystem proof rather than a separate GUI selection test. An earlier release publication HTTP 500 succeeded on failed-job retry without rebuilding; cause unknown.

WireGuardNT remains the only shipping backend. No BoringTun dependency, new product network service, Internet probe, signup, WARP registration, kill switch or IPv6 leak-protection claim. No Cloudflare implementation, assets, credentials or keys are copied.
