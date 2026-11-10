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
| `wax-tunnel` | Adapter lifecycle on WireGuardNT and DNS (netsh) |
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
- Crash recovery: adapter and DNS are reclaimed by Windows when the owner dies.

No kill switch is included. If the tunnel drops, traffic can use the normal connection. IPv6 leak protection is not claimed.

Not yet verified (honest list):
- Tray-menu Open/Quit interaction (the runner session has no notification
  area; the graceful-quit path itself is verified via `wax-vpn.exe --quit`).
- WARP registration against the live Cloudflare API (requires a person to
  accept the ToS first - by design).
- Leak testing (IPv6, DNS) against an external observer.
- Consumer Windows/SmartScreen behavior (unsigned installer).
- General IPv6 leak protection (runner has no working baseline).

Scoped Windows split-routing check passed: 8.8.8.8 excluded, 1.1.1.1 retained in tunnel routes. This is not proof for every configuration.

v0.1.0 also passed a separate fresh Windows runner silent NSIS install, installed GUI main-window launch, graceful quit and uninstall. User data is retained. It remains unsigned; Windows publisher/SmartScreen warnings are expected.

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

### Configuration compatibility

waX VPN imports standard WireGuard keys, addresses, IP DNS servers, peer endpoints and AllowedIPs. DNS search domains and hostnames are rejected explicitly; only literal IPv4/IPv6 DNS server addresses are supported. It does not execute wg-quick scripts or implement wg-quick routing-table settings. Profiles containing PreUp, PostUp, PreDown, PostDown, Table, SaveConfig or FwMark are rejected explicitly rather than silently changing their behavior. Remove a directive only if you understand what it did; an imported profile without it can behave differently.

### Local support report

`wax diagnostics` prints a small JSON report: app version, OS/architecture, administrator state, adapter status, aggregate byte counters and handshake age, or a fixed status-query error code. It does not test Internet reachability and does not read profiles, logs, account files, keys or tokens. It makes no network requests and does not send the report anywhere. Review it before sharing. Save it yourself with shell redirection if needed.
