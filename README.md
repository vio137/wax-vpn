# waX VPN

A WireGuard VPN client for Windows, written in Rust - with Cloudflare WARP
(wgcf-equivalent) support built in, strictly opt-in.

**Status: early preview.** The workspace compiles, core logic is unit-tested,
and the CI pipeline produces a real installer. Actual on-machine tunnel
behavior is being verified build by build - see "Verified vs. planned" below.
Nothing here is claimed to work until it has been tested.

## Download

Grab the latest installer from
[Releases](https://github.com/vio137/wax-vpn/releases) and check its SHA256
against the attached `SHA256SUMS` file.

## What it is

- **Real system VPN.** waX VPN drives the official WireGuardNT kernel driver
  (the same `wireguard.dll` that powers WireGuard for Windows, downloaded
  from [download.wireguard.com](https://download.wireguard.com/wireguard-nt/))
  through the MIT-licensed [`wireguard-nt`](https://crates.io/crates/wireguard-nt)
  Rust bindings. No browser proxy, no userspace tunnel pretending to be a VPN.
- **Rust first.** UI (egui), tunnel management, profile handling and the WARP
  client are all Rust. The only non-Rust components are the official signed
  WireGuardNT driver itself and standard OS facilities (netsh, Windows
  Firewall).
- **WARP built in, opt-in.** The functionality of
  [wgcf](https://github.com/ViRb3/wgcf) - register, account details, device
  management, profile generation - is reimplemented in Rust (`wax-warp`).
  Registration only ever happens when you explicitly accept Cloudflare's
  Terms of Service in the app or pass `--accept-tos` to the CLI. waX VPN is
  **not affiliated with Cloudflare**.

## Components

| Crate | Purpose |
| --- | --- |
| `wax-core` | Keys (audited `x25519-dalek`), wg-quick `.conf` parsing/serialization, profile storage |
| `wax-warp` | Cloudflare WARP API client (wgcf equivalent), account persistence, profile generation |
| `wax-tunnel` | Adapter lifecycle on WireGuardNT, DNS (netsh), firewall-based kill switch (PowerShell) |
| `wax-cli` (`wax.exe`) | Full CLI: profiles, import, connect/disconnect/status, WARP commands |
| `wax-app` (`waX VPN.exe`) | egui GUI: animated connect orb, profile picker, stats, settings, WARP panel, drag-and-drop import |

## Verified vs. planned

Verified (automated):
- Unit tests: key generation round-trips, `.conf` parse/serialize round-trip,
  split-tunnel CIDR subtraction.
- CI: full workspace builds on Windows; installer produced; artifact SHA256 published.

Verified in CI on Windows runners (see `verify-results.txt` in each build's
verification-evidence artifact):
- Profile import, tunnel connect with a live handshake, ping through the tunnel.
- GUI-driven connect (UI Automation clicks the app's Connect button), and
  close-to-tray with the process and tunnel adapter still up.
- Tunnel DNS applied on connect and restored on disconnect; no firewall
  leftovers after disconnect.
- Crash recovery: adapter and DNS are reclaimed by Windows when the owner
  process dies; kill-switch firewall rules persist across an owner crash
  (persistence observed - effective traffic blocking is not traffic-tested).

Not yet verified (honest list):
- Effective traffic blocking by the kill switch (rule presence only).
- Tray-menu Open/Quit interaction (the runner session has no notification
  area; the graceful-quit path itself is verified via `wax-vpn.exe --quit`).
- WARP registration against the live Cloudflare API (requires a person to
  accept the ToS first - by design).
- Leak testing (IPv6, DNS) against an external observer.
- Installer launch on a clean machine.

Implemented and unit-tested, not yet exercised on Windows:
- Split tunneling: per-profile CIDR exclusions subtracted from AllowedIPs at
  connect time (Profiles tab -> Split tunneling).

Planned: per-profile MTU/DNS overrides, speed test, installer polish (icon,
branding), localization.

## Building

```
cargo build --release --workspace
```

The app expects the official `wireguard.dll` next to the executable (the
installer handles this).

## License

waX VPN's own code is MIT (see `LICENSE`). The bundled `wireguard.dll` is the
official prebuilt WireGuardNT binary, distributed by the WireGuard project
under the permissive license included in the installer
(`LICENSE-wireguard-nt.txt`). WireGuard is a registered trademark of Jason A.
Donenfeld; waX VPN is not affiliated with the WireGuard project.

## Design process

Interface refinements follow the unslop design-review rubric by Damir Sokolovsky
(https://github.com/heydamir/unslop), used under CC BY 4.0. The connect
control is a Gargantua-inspired procedural black hole - a lensed starfield,
an accretion disk with doppler brightening, a photon ring - painted in plain
egui with no textures or shaders, ported from the same scene on the project
website. Animation speed and brightness are fixed per connection state and
never derived from measured traffic.

The interface typeface is Inter Display by Rasmus Andersson
(https://rsms.me/inter/), used under the SIL Open Font License 1.1.
