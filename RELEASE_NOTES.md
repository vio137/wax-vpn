# waX VPN v0.1.8

Unsigned Windows x64 reliability checkpoint. Windows may show an unknown-publisher warning. The official WireGuardNT driver is independently signed by its vendor. No kill switch: traffic can use the normal connection if the tunnel drops.

## Changes

- Start with Windows now checks registry write/delete errors and reads the exact command back before saving success. Failed verification restores the previous switch display and shows a fixed warning to check Windows Startup settings. This is not continuous monitoring of external changes.
- GUI and CLI profile imports now share additive, exclusive file creation. Existing names and sanitized-name collisions receive a numeric suffix instead of being overwritten. The actual stored name is returned for selection and settings. Explicit save/update paths are unchanged.
- Full-only Windows verification now exercises a real pending automatic retry, disables Auto-reconnect through the UI, checks saved false and disconnected state, samples adapter absence over 12 seconds, then manually reconnects while automation remains disabled.

## Verification

Source 6c123c5 / full manual gate 42798191756 passed Windows build/tests, full tunnel/GUI and fresh install/quit/uninstall. Named tests cover registry write/update/readback/idempotent removal in a non-startup sandbox key and additive profile import with sanitized-name collisions, preservation, invalid input and concurrent imports in a temporary directory. Actual Profiles and Connected pixels inspected. Exact v0.1.8 stable verification is pending.

## Scope and limits

The prior 6c123c5 push run 42796876302 lost its PowerShell GUI-harness host with exit 0xC0000409 after Connecting size captures, before a GUI-connect result. That termination remains unexplained. The same-source full manual pass is not a diagnosis. An earlier separate installed quit timed out after 15 seconds in leftover cleanup; later passes do not diagnose that intermittent failure either. No timeout was widened or cleanup skipped.

Startup failure-warning pixels and a real Windows sign-in launch are not proven. Pending-retry cancellation is demonstrated, not deterministic in-flight worker-result cancellation or retry exhaustion. Peer handshake and packets to the demo server tunnel-side address do not prove Internet reachability. Adapter-loss recovery does not establish physical-network roaming. Crash evidence is a 3-second snapshot, not continuous leak observation. IPv6 was skipped without a proven runner baseline. Real notification-area tray menu remains manual-only.

WireGuardNT remains the only shipping backend. No BoringTun dependency or experiment was added after that request was withdrawn. No new product network service, Internet probe, signup, WARP registration, kill switch or IPv6 leak-protection claim. The Astryx-informed interface stays native Rust/egui, without React, JavaScript or WebView runtime. No Cloudflare implementation, assets, credentials or keys are copied.
