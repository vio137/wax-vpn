# waX VPN v0.1.3

Unsigned Windows x64 stable checkpoint. A certificate is not yet available; Windows may show an unknown-publisher warning. Official WireGuardNT driver payload remains independently signed by its vendor. No kill switch: traffic can use the normal connection if the tunnel drops.

## Changes

- Apply profile MTU on the dedicated tunnel adapter. Windows IPv4MTU1380 readback verified; IPv6 application code is not runtime-proven.
- Add `wax diagnostics`, an allowlisted local JSON report. No profiles, keys, tokens, account IDs, endpoints, hostnames, paths or logs. No network request or automatic upload.
- Distinguish adapter readiness, recent handshake and older handshake in the GUI. Peer handshake is not an Internet-reachability test.
- Monitor an existing CLI-owned tunnel when the GUI starts. Detect external stop without reconnecting a guessed profile.
- Report consistent stable/preview/dev labels in GUI, CLI version and diagnostics.

## Verification scope

Build/tests, peer handshake and packets to the official WireGuard demo server's tunnel-side address; IPv4MTU and DNS apply/cleanup; diagnostics schema; external-tunnel observer; GUI-driven connect; CIDR exclusion route check; close-to-tray; original-process/window restore; graceful quit; fresh-runner install, installed launch, quit and uninstall. Stable manual gate also runs focused crash cleanup and IPv6 baseline checks, reporting any unavailable baseline as SKIP, not protection.

Real notification-area tray menu remains manual-only in the hosted Windows test environment. Live WARP account registration has not been verified and is not performed by release testing. This release does not claim IPv6 leak protection, a kill switch or measured speed improvement.

## Official-client study

See `docs/WARP-STUDY-CHECKLIST.md`. REA launcher/native metadata, resolved MSI structure and publisher-verified CLI help informed independent improvements. Default deep CLI analysis timed out; reduced analysis remains in progress. No Cloudflare code, assets, branding, credentials or private protocol implementation is copied. Advertised CLI commands are not equivalent to tested waX features.
