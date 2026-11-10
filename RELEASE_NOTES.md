# waX VPN v0.1.5

Unsigned Windows x64 layout checkpoint. A certificate is not yet available; Windows may show an unknown-publisher warning. The official WireGuardNT driver payload remains independently signed by its vendor. No kill switch: traffic can use the normal connection if the tunnel drops.

## Changes

- Center each Session, Received and Sent label over its value, using three equal-width columns.
- Keep the Connect screen composition fixed across Disconnected, Connecting and Connected. Reserve the profile hint and connection-stage slots instead of moving the rest of the screen when their text appears.
- Center the connection stage beneath the main action. Show inactive counters as dashes, not stale values.

## Verification scope

Release preparation is not complete until its exact-version Windows gate and actual pixels have been inspected. The gate captures Disconnected, Connecting and Connected at 760x540, 880x600 and 960x700, with UI Automation label bounds as supporting evidence. Screenshot filenames alone do not prove which state was displayed.

The underlying v0.1.4 gate passed peer handshake, packets to the demo server tunnel-side address, IPv4 MTU 1380, DNS apply/cleanup, local diagnostics generate/clear, external-tunnel observation, GUI connect, split routing, bounded adapter-loss recovery, close-to-tray, original-window restore, graceful quit and fresh-runner install/quit/uninstall. The recovery proof covers adapter loss only, not physical-network roaming or Wi-Fi switching. Internet reachability is not tested. Full Windows retry-exhaustion/cancellation sequences are not runtime-proven.

## Known limits

An earlier 8863f19 fresh-install job timed out after 15 seconds waiting for quit. Its same-source retry and later full gates passed. The first failure remains an unresolved intermittent issue; subsequent passes do not diagnose it. No timeout was widened and no cleanup was skipped.

The crash-cleanup evidence is a snapshot after owner termination, not continuous leak observation. The IPv6 probe was SKIP because the runner had no IPv6 baseline. Real notification-area tray menu remains manual-only in the hosted Windows environment.

No kill switch, IPv6 leak-protection, speed improvement or WARP registration claim is made. Release tests do not sign up, register or connect to WARP. No Cloudflare implementation, assets, branding, credentials or private protocol implementation is copied.
