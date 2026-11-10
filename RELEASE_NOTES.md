# waX VPN v0.1.9

Unsigned Windows x64 settings reliability checkpoint. Windows may show an unknown-publisher warning. The official WireGuardNT driver is independently signed by its vendor. No kill switch: traffic can use the normal connection if the tunnel drops.

## Changes

- Settings load and save errors are now shown instead of silently discarded. An unreadable or malformed settings file is kept unchanged, with automatic recovery off for that session. Current session choices can still change, but the warning says they may not survive the next launch.
- Settings are written to a unique same-directory temporary file, flushed, then renamed over the destination. Failed writes or replacements remove the temporary file without deleting the original. This reduces the direct-truncation window, not a power-loss durability guarantee.
- Startup no longer rewrites settings just to load them. Existing disabled recovery and split exclusions survive the tested round trips.

## Verification

Prepared source 61f5dd4 / full manual gate 42807585480 passed Windows workspace tests, tunnel/GUI and fresh install/quit/uninstall. Named Windows settings tests passed for repeated round trips with disabled recovery and exclusions, malformed-original preservation, invalid parent errors, and a sharing-locked destination preserving previous bytes with temporary-file cleanup. Full-only UI verification preserved a deliberately malformed original, checked recovery off, showed load and save warnings, and quit. Actual 880x600 load-warning, save-warning, Connected and installed pixels were inspected. Exact v0.1.9 build and publication are pending.

Runtime evidence: peer handshake, tunnel-side ping, DNS and IPv4 MTU 1380, split route exclusions, owned adapter-loss recovery after 5 seconds with the original process, pending-retry disable over 48 samples/12 seconds, explicit reconnect while automatic recovery was off, close-to-tray, original main HWND/PID restore and graceful quit passed. The crash snapshot at 3 seconds before cleanup showed adapter gone, DNS none and zero old kill-switch rules. Fresh installer install/launch/uninstall passed.

## Scope and limits

The 8fc4241 push run 42805998294 lost its PowerShell GUI-harness host with exit 0xC0000409 after Connecting captures, before a GUI-connect result. This repeated an earlier unexplained harness termination. Later success does not diagnose either failure. An earlier separate installed quit timed out after 15 seconds in leftover cleanup; later passes do not diagnose that intermittent failure either. No timeout was widened or cleanup skipped.

Settings preservation is proven only for the tested failures, not concurrent edits by another process, storage hardware failure or power loss. Startup failure-warning pixels and a real Windows sign-in launch are not proven. Pending-retry cancellation is demonstrated, not deterministic in-flight worker-result cancellation or retry exhaustion. Peer handshake and packets to the demo server tunnel-side address do not prove Internet reachability. Adapter-loss recovery does not establish physical-network roaming. Crash evidence is a 3-second snapshot, not continuous leak observation. IPv6 was skipped without a proven runner baseline. Real notification-area tray menu remains manual-only.

WireGuardNT remains the only shipping backend. No BoringTun dependency, new product network service, Internet probe, signup, WARP registration, kill switch or IPv6 leak-protection claim. The Astryx-informed interface stays native Rust/egui, without React, JavaScript or WebView runtime. No Cloudflare implementation, assets, credentials or keys are copied.
