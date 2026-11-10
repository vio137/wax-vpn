# waX VPN v 0.1.6

Unsigned Windows x 64 interface checkpoint. Windows may show an unknown-publisher warning. The official WireGuardNT driver remains independently signed by its vendor. No kill switch: traffic can use the normal connection if the tunnel drops.

## Interface rebuilt from the Astryx reference

- Replace the particle background and icon rail with neutral surfaces, text navigation and a capped content region.
- Rebuild Connect around clear peer-evidence status, selected profile, one centered action, equal-width counters and stable status slots.
- Apply the same type and surface tokens to Profiles, Features, WARP and About. Keep the existing operations, local diagnostics and consent boundaries.
- Use short surface/color transitions, with immediate transitions when Reduced motion is enabled. Keep active toggle thumbs visible against their track.

Astryx 0.7.11 is an MIT-licensed React 19+/StyleX design system. A separate runnable design reference uses its actual components. The shipped app adapts the design in native Rust/egui; it does not embed React, JavaScript or a WebView. Its MIT notice is retained in the source and installer. See docs/ASTRYX-REDESIGN.md.

## Verification

Full manual preview gate42777283654 passed from source433f867. Actual Windows Disconnected, Connecting and Connected pixels were inspected at760x540,880x600and960x700, with matching counter-label anchors across states. Profiles, Features, WARP, About, generated diagnostics and installed-app pixels were inspected too. The exact-version stable gate still has to complete. Visual reference values are illustrative, not tunnel measurements.

## Scope and known limits

The previous stable gate verified peer handshake, packets to the demo server tunnel-side address, IPv 4 MTU 1380, DNS apply/cleanup, diagnostics generate/clear, external-tunnel observation, GUI connect, split routing, bounded adapter-loss recovery, close-to-tray, original-window/PID restore, graceful quit and fresh install/uninstall. Adapter-loss recovery does not establish physical-network roaming or Wi-Fi switching. Internet reachability is not tested. Full Windows retry-exhaustion/cancellation sequences remain unproven.

An earlier 8863 f 19 fresh-install job timed out after 15 seconds waiting for quit. Its same-source retry and subsequent gates passed. The first intermittent failure remains unresolved; later passes do not diagnose it. No timeout was widened or cleanup skipped.

Crash cleanup is a snapshot, not continuous leak observation. IPv 6 was SKIP without a proven runner baseline. Real notification-area tray menu remains manual-only in the hosted Windows environment.

No kill switch, IPv 6 leak-protection, speed improvement or WARP registration claim. No WARP signup, registration or connection is performed by release tests. No new product service or third-party network request was introduced. No Cloudflare code, assets, branding, credentials or private protocol implementation is copied.
