# waX VPN v0.1.4

Unsigned Windows x64 stable checkpoint. A certificate is not yet available; Windows may show an unknown-publisher warning. The official WireGuardNT driver payload remains independently signed by its vendor. No kill switch: traffic can use the normal connection if the tunnel drops.

## Changes

- Add About > Local diagnostics. Generate a local snapshot, review it, copy it explicitly or clear it. The GUI and CLI share one strict field allowlist. No profiles, keys, tokens, account IDs, endpoints, hostnames, paths or logs are collected, and nothing is uploaded.
- Bound owned-adapter-loss recovery to three attempts after2,4and8seconds. Keep the profile and split exclusions that started the connection, reject stale worker results, and allow cancellation while waiting. A60second up session refills the retry budget. Four deterministic policy tests cover bounds, flapping, cancellation generations and external-owner/disabled restrictions.
- Share peer-evidence health wording between CLI and GUI. An adapter without a handshake is Tunnel ready, a recent handshake is Connected, an older handshake is Tunnel up. Internet reachability is explicitly not tested. A future timestamp does not become a false zero-second handshake.
- Keep the full local-diagnostics privacy note visible before report generation.
- Add fixed quit-cleanup stage logging and failure evidence for the installer gate.

## Verification scope and known issue

The feature sourcee915163 passed full manual preview gate42772103670: actual peer handshake, packets to the demo server tunnel-side address, IPv4MTU1380, DNS apply/cleanup, allowlisted diagnostics and generate/clear, external-tunnel observer, GUI connect, split CIDR route checks, close-to-tray, original main-window/PID restore, graceful quit and fresh-runner install/quit/uninstall. Adapter-loss recovery restored a peer handshake after4seconds with the original GUI process alive. This demonstrates adapter-loss recovery only, not Wi-Fi switching, physical-network roaming or Internet routing. Full Windows retry-exhaustion/cancellation sequences are not runtime-proven.

An earlier8863f19 fresh-install job timed out after15seconds waiting for quit. Its same-source retry passed, and the full feature gate passed again. The first failure remains an unresolved intermittent issue; subsequent passes do not diagnose it. No timeout was widened and no cleanup was skipped. Fixed-stage logging is included to help capture recurrence.

The full stable gate reruns focused crash cleanup and IPv6 baseline checks. The feature gate's snapshot3seconds after owner termination showed adapter gone and DNS none before explicit cleanup; cleanup then passed. This is a snapshot, not continuous leak observation. IPv6 baseline was unavailable and the probe was SKIP, not protection. Real notification-area tray menu remains manual-only in the hosted Windows environment. Release preparation still needs its own exact-version gate.

No kill switch, IPv6 leak-protection, speed improvement or WARP registration claim is made. No WARP signup, registration or connection is performed by release tests.

## Official-client study

See docs/WARP-STUDY-CHECKLIST.md. REA and publisher-verified CLI help informed independent improvements. The reduced native CLI pass completed with31,503procedures and13,665defined strings, with Decompiler Switch Analysis disabled after the first analysis timed out. Complete switches/calls and full GUI/network behavior remain unknown. No Cloudflare implementation, assets, branding, credentials or private protocol implementation is copied.
