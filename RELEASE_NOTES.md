# waX VPN v0.1.10

Unsigned Windows x64 redraw-timing checkpoint. Windows may show an unknown-publisher warning. The official WireGuardNT driver is independently signed by its vendor. No kill switch: traffic can use the normal connection if the tunnel drops.

## Changes

- Removed an early 66 ms periodic redraw request that overrode the intended slower hidden-window and reduced-motion idle requests.
- One periodic policy now requests 1000 ms while hidden, 250 ms while visibly disconnected with Reduced motion on, and 33 ms otherwise. Input and egui animations may still request earlier redraws. These are polling intervals, not measured frame rates, CPU savings or battery-life claims.
- Hidden-window tray and cross-instance requests still use bounded polling. No new background service or network call.

## Verification

Exact release source eace04a98ff106601d889aa9c077cde61fdc5bc6 / full manual gate 42822679916 passed Windows workspace tests, tunnel/GUI, fresh install/uninstall and publication. Prepared source 5919dce / full manual gate 42820899117 also passed. The named repaint-policy unit test checks all eight hidden/motion/disconnected combinations. Exact retained results confirm close-to-tray with the adapter up, restored original HWND/PID and Connect rail, and graceful quit with adapter removed. Actual Connected and installed pixels were inspected. v0.1.10 is published. A fresh released installer download is 10,948,767 bytes with SHA256 `9b1420a793f9395deb8d3621c49f443e0a1d8eba110c48e794fd817a5ecac174`, matching the released SHA256SUMS.

Peer handshake, tunnel-side ping, DNS and IPv4 MTU 1380, split route exclusions, owned adapter-loss recovery after 4 seconds with the original process, pending-retry disable over 48 samples/12 seconds, explicit reconnect while automatic recovery was off, and malformed-settings preservation/load/save warnings passed. No console windows were observed in 367 samples at about 100 ms; shorter windows could be missed. Crash snapshot at 3 seconds before cleanup showed adapter gone, DNS none and zero old kill-switch rules. Fresh installer install/launch/uninstall passed.

## Scope and limits

The 8fc4241 push run 42805998294 lost its PowerShell GUI-harness host with exit 0xC0000409 after Connecting captures, before a GUI-connect result. This repeated an earlier unexplained harness termination. Later success does not diagnose either failure. An earlier separate installed quit timed out after 15 seconds in leftover cleanup; later passes do not diagnose that intermittent failure either. No timeout was widened or cleanup skipped. Full preview 42815620221 and exact stable 42817228387 passed with UI Automation steps on installed Windows PowerShell 5.1, without changing security settings. This host comparison does not establish the cause of the prior pwsh terminations.

Settings preservation is proven only for the tested failures, not concurrent edits by another process, storage hardware failure or power loss. Startup failure-warning pixels and a real Windows sign-in launch are not proven. Pending-retry cancellation is demonstrated, not deterministic in-flight worker-result cancellation or retry exhaustion. Peer handshake and packets to the demo server tunnel-side address do not prove Internet reachability. Adapter-loss recovery does not establish physical-network roaming. Crash evidence is a 3-second snapshot, not continuous leak observation. IPv6 was skipped without a proven runner baseline. Real notification-area tray menu remains manual-only.

WireGuardNT remains the only shipping backend. No BoringTun dependency, new product network service, Internet probe, signup, WARP registration, kill switch or IPv6 leak-protection claim. The Astryx-informed interface stays native Rust/egui, without React, JavaScript or WebView runtime. No Cloudflare implementation, assets, credentials or keys are copied.
