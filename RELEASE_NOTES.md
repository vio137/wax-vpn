# waX VPN v0.1.6

Unsigned Windows x64 interface checkpoint. Windows may show an unknown-publisher warning. The official WireGuardNT driver remains independently signed by its vendor. No kill switch: traffic can use the normal connection if the tunnel drops.

## Interface rebuilt from the Astryx reference

- Replace the particle background and icon rail with neutral surfaces, text navigation and a capped content region.
- Rebuild Connect around clear peer-evidence status, selected profile, one centered action, equal-width counters and stable status slots.
- Apply the same type and surface tokens to Profiles, Features, WARP and About. Keep the existing operations, local diagnostics and consent boundaries.
- Use short surface/color transitions, with immediate transitions when Reduced motion is enabled. Keep active toggle thumbs visible against their track.

Astryx 0.7.11 is an MIT-licensed React 19+/StyleX design system. A separate runnable design reference uses its actual components. The shipped app adapts the design in native Rust/egui; it does not embed React, JavaScript or a WebView. Its MIT notice is retained in the source and installer. See docs/ASTRYX-REDESIGN.md.

## Verification

Exact-version stable gate 42778052338 passed from source cf976cf, including the fresh-runner installer. Real Windows Disconnected, Connecting and Connected pixels were inspected at 760x540, 880x600 and 960x700; counter-label anchors match across states. Profiles, Features, WARP, About, generated diagnostics and installed-app pixels were inspected too. The active-toggle contrast fix is visible in the stable capture. Visual reference values are illustrative, not tunnel measurements.

Released installer: 10,943,260 bytes. SHA256: e7d3ee08b346044805f128b989523ca434b883588ae0fda894b39850798f8a4e.


## Scope and known limits

This stable gate verified peer handshake, packets to the demo server tunnel-side address, IPv4 MTU 1380, DNS apply/cleanup, diagnostics generate/clear, external-tunnel observation, GUI connect, split routing, bounded adapter-loss recovery, close-to-tray, original-window/PID restore, graceful quit and fresh install/uninstall. Adapter-loss recovery does not establish physical-network roaming or Wi-Fi switching. Internet reachability is not tested. Full Windows retry-exhaustion/cancellation sequences remain unproven.

An earlier 8863f19 fresh-install job timed out after 15 seconds waiting for quit. Its same-source retry and subsequent gates passed. The first intermittent failure remains unresolved; later passes do not diagnose it. No timeout was widened or cleanup skipped.

Crash cleanup is a snapshot, not continuous leak observation. IPv6 was SKIP without a proven runner baseline. Real notification-area tray menu remains manual-only in the hosted Windows environment.

No kill switch, IPv6 leak-protection, speed improvement or WARP registration claim. No WARP signup, registration or connection is performed by release tests. No new product service or third-party network request was introduced. No Cloudflare code, assets, branding, credentials or private protocol implementation is copied.
