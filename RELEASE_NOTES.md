# waX VPN v0.1.13

Unsigned Windows x64 tray-initialization fallback checkpoint. Windows may show an unknown-publisher warning. The official WireGuardNT driver is independently signed by its vendor. No kill switch: traffic can use the normal connection if the tunnel drops.

## Changes

- Both close paths share one background policy. If the tray initializes, closing still hides the window. If tray initialization fails, closing requests minimize instead of hiding an unreachable window.
- A generic tray-unavailable notice explains taskbar restoration and Ctrl+Q to quit. The close tooltip also reflects the fallback. No service, network call or security setting was added.

## Verification

Exact stable source 3e455d268468b403cd4c0d083b0312b3e8c4e0c0 / run 42842372496 passed Windows workspace tests, tunnel/GUI and fresh install/uninstall. Freshly downloaded installer: 10,950,419 bytes, SHA256 4fcdaa872f2eb0bc16a796e685aded24640da7fc44db6d9989b3aac5d2e72d1d, matched published checksums. The named background-policy unit test confirms hide only with a tray, minimize without a tray, and warning presence/absence. This is command-selection proof, not Windows UI proof.

A genuine tray initialization failure was not reproduced. Fallback warning pixels and actual taskbar restoration without a tray remain untested. No production environment switch was added to fake tray failure. Successful tray construction does not prove that Windows displayed its notification-area icon. The real tray menu remains manual-only on a Windows desktop.

Retained exact stable results confirm normal close-to-tray with the tunnel adapter still up, restored original HWND/PID and Connect rail, and graceful quit with adapter removed. Actual v0.1.13 Connected, Profiles, Features, WARP, About, profile-warning and installed pixels were inspected. Peer handshake, tunnel-side ping, DNS and IPv4 MTU 1380, split route exclusions, owned adapter-loss recovery after 4 seconds, pending-retry disable over 48 samples/12 seconds, manual reconnect while recovery was off, profile reload warnings and malformed-settings preservation/warnings passed. No console windows were observed in 361 samples at about 100 ms; shorter windows could be missed. Crash snapshot at 3 seconds before cleanup showed adapter gone, DNS none and zero old kill-switch rules. Fresh install/launch/uninstall passed.

Profile startup directory-error pixels and an invoked stale manual Connect failure remain untested. The earlier profile-load unit fixture at 412c26c / run 42831837117 failed because a supposed-valid config had no peer; d5941d4 corrected the fixture, not the parser or assertions. Real UAC acceptance/cancellation, native restart failure pixels and the full elevated parent-to-child handoff remain untested; native policy/path/wait unit tests pass and runas retains the Windows prompt.

## Scope and limits

The 8fc4241 push run 42805998294 lost its PowerShell GUI-harness host with exit 0xC0000409 after Connecting captures, before a GUI-connect result. This repeated an earlier unexplained harness termination. Later success does not diagnose either failure. An earlier separate installed quit timed out after 15 seconds in leftover cleanup; later passes do not diagnose that intermittent failure either. No timeout was widened or cleanup skipped. Full preview 42815620221 and exact stable 42817228387 passed with UI Automation steps on installed Windows PowerShell 5.1, without changing security settings. This host comparison does not establish the cause of the prior pwsh terminations.

Settings preservation is proven only for the tested failures, not concurrent edits by another process, storage hardware failure or power loss. Startup failure-warning pixels and a real Windows sign-in launch are not proven. Pending-retry cancellation is demonstrated, not deterministic in-flight worker-result cancellation or retry exhaustion. Peer handshake and packets to the demo server tunnel-side address do not prove Internet reachability. Adapter-loss recovery does not establish physical-network roaming. Crash evidence is a 3-second snapshot, not continuous leak observation. IPv6 was skipped without a proven runner baseline. Real notification-area tray menu remains manual-only.

WireGuardNT remains the only shipping backend. No BoringTun dependency, new product network service, Internet probe, signup, WARP registration, kill switch or IPv6 leak-protection claim. The Astryx-informed interface stays native Rust/egui, without React, JavaScript or WebView runtime. No Cloudflare implementation, assets, credentials or keys are copied.
