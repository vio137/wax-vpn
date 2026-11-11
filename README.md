<p align="center">
  <a href="https://waxvpn.pages.dev/">
    <img src="https://waxvpn.pages.dev/readme-banner.svg" alt="waX VPN - a WireGuard VPN client for Windows, written in Rust" width="100%">
  </a>
</p>

<p align="center">
  <a href="https://github.com/vio137/wax-vpn/releases"><img alt="Latest release" src="https://img.shields.io/github/v/release/vio137/wax-vpn"></a>
  <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-2f81f7"></a>
  <a href="https://waxvpn.pages.dev/"><img alt="Website: waxvpn.pages.dev" src="https://img.shields.io/badge/site-waxvpn.pages.dev-2f81f7"></a>
</p>

<p align="center">
  <strong><a href="https://waxvpn.pages.dev/">Website</a></strong> ·
  <strong><a href="https://github.com/vio137/wax-vpn/releases">Download</a></strong> ·
  <strong><a href="#how-it-works">How it works</a></strong> ·
  <strong><a href="#verified-vs-not-yet-verified">Test status</a></strong>
</p>

---

waX VPN puts a real WireGuard tunnel behind a native Rust interface. It drives
the official WireGuardNT kernel driver, the same signed `wireguard.dll` that
powers WireGuard for Windows, through the MIT-licensed
[`wireguard-nt`](https://crates.io/crates/wireguard-nt) bindings. No browser
proxy, no userspace tunnel pretending to be a VPN.

The functionality of [wgcf](https://github.com/ViRb3/wgcf) is built in as an
opt-in Cloudflare WARP client, reimplemented in Rust. Registration only happens
when you explicitly accept Cloudflare's Terms of Service in the app or pass
`--accept-tos` to the CLI. waX VPN is not affiliated with Cloudflare.

The released v1.0.3 has no update checker or telemetry uploader. Tunnel traffic
and DNS lookups can happen in the background while connected; WARP API calls
happen only when you request WARP actions. Local diagnostics are not uploaded.
Optional public-IP, HTTPS latency and manual quick-speed measurements are off
by default. Those checks contact the configured service (Cloudflare by default)
and disclose your public IP to it. The local adapter-counter graph is on by
default and makes no extra network requests. These are current-route
observations, not leak tests; results stay in memory and are not uploaded as
analytics or included in diagnostics.

## Download

Grab the latest `x64.exe` installer from
[Releases](https://github.com/vio137/wax-vpn/releases) and check its SHA256
against the attached `SHA256SUMS` file. The installer needs administrator
rights. It is unsigned for now, so Windows will show an unknown-publisher
warning; the bundled WireGuardNT driver is independently signed by its vendor.

## Startup compatibility

The Rust executables avoid a VC++ redistributable dependency by linking
the C runtime statically and checking packaged PE imports. It is a native app,
not a .NET app. Windows system libraries are still required; no bare-image compatibility claim
is made. Released v1.0.3 defaults to CPU software rendering (Windows GDI),
with GPU rendering available through `--accelerated-graphics`. Windows 10/11 x64
is required. No separate VC++/.NET/WebView2 installation or first-run driver
download is required. The bundled WireGuardNT DLL installs its signed kernel
driver when connecting, with administrator consent. Windows PowerShell, netsh
and Windows networking services are still used for tunnel setup; removing
those shell dependencies is a separate follow-up, not part of v1.0.3.

The CPU path passed launch, accessible tab input, close-to-tray, no-tray fallback,
same-window restore and quit on a Windows Server 2025 hosted runner with Hyper-V
Video. That runner has runtimes installed and is not a clean Windows 10/11 image.
The reported consumer PC crash remains unverified. Stable v1.0.3 evidence:
https://github.com/vio137/wax-vpn/actions/runs/43083908145

If startup fails, check `%APPDATA%\wax-vpn\startup.log`. Fixed startup markers
are local and bounded; no profile/account/settings values are written. A Rust
panic or returned graphics setup error shows a dialog. Native driver faults
can bypass it. The Start-menu Safe graphics shortcut uses OpenGL instead of
wgpu, but also needs a compatible driver. Quit any existing instance first.
The reported v1.0.0 crash has not been reproduced on the user's other PC, so
v1.0.3 is a compatibility mitigation rather than a confirmed fix there.

## Screenshots

Real captures from the CI verification gate, not mockups. Each shows the build
it was taken from.

<table>
  <tr>
    <td><img src="https://waxvpn.pages.dev/connected.svg" alt="Connected, tunnel up with session counters"></td>
    <td><img src="https://waxvpn.pages.dev/disconnected.svg" alt="Disconnected home screen"></td>
  </tr>
  <tr>
    <td align="center"><sub>Connected, with session counters (v0.1.15)</sub></td>
    <td align="center"><sub>First screen, ready to connect (v0.1.10)</sub></td>
  </tr>
  <tr>
    <td><img src="https://waxvpn.pages.dev/profiles.svg" alt="Profiles page with split tunneling and import"></td>
    <td><img src="https://waxvpn.pages.dev/features.svg" alt="Features page with startup and reconnect options"></td>
  </tr>
  <tr>
    <td align="center"><sub>Profiles, split tunneling and import (v0.1.15)</sub></td>
    <td align="center"><sub>Startup and reconnect options (v0.1.15)</sub></td>
  </tr>
  <tr>
    <td><img src="https://waxvpn.pages.dev/warp.svg" alt="Cloudflare WARP registration page"></td>
    <td><img src="https://waxvpn.pages.dev/about.svg" alt="About page with local diagnostics"></td>
  </tr>
  <tr>
    <td align="center"><sub>Opt-in WARP registration (v0.1.15)</sub></td>
    <td align="center"><sub>About and local diagnostics (v0.1.15)</sub></td>
  </tr>
</table>

## Features

- **Real system VPN.** The adapter runs on the official WireGuardNT driver,
  downloaded from
  [download.wireguard.com](https://download.wireguard.com/wireguard-nt/).
  Closing the window sends the app to the tray with the tunnel still up.
- **Rust everywhere.** UI (egui), tunnel management, profile handling and the
  WARP client are all Rust. The only non-Rust parts are the signed WireGuardNT
  driver itself and standard OS facilities (netsh, Windows Firewall). No
  embedded browser or WebView anywhere.
- **Profiles without ceremony.** Drop a `.conf` anywhere in the window or
  paste it in. Split tunneling per profile through CIDR exclusions. Import is
  strict on purpose: configs with wg-quick scripts (`PreUp`, `PostUp`,
  `Table`, `SaveConfig`, `FwMark`) are rejected with an explanation instead of
  being run.
- **Cloudflare WARP, opt-in.** The full wgcf flow (register, account details,
  device management, profile generation) reimplemented in Rust. A WARP profile
  connects like any other profile afterwards.
- **Reconnect and recovery.** Auto-reconnect retries the same profile after 2,
  4 and 8 seconds. If the adapter disappears underneath the tunnel, the app
  recovers it. A CI crash check recorded adapter/DNS state three seconds after killing
  the tunnel owner, then ran explicit cleanup. This is a snapshot, not a
  continuous or general crash-cleanup guarantee.
- **Local diagnostics.** About &gt; Generate local report builds an
  allowlisted snapshot only when you click it; nothing is uploaded.
  `wax diagnostics` prints the same allowlisted JSON to your shell.
- **Opt-in network observations.** Network shows original/public route IP,
  local received/sent throughput and HTTPS latency graphs, plus a manual
  8 MB download / 2 MB upload quick test. Features &gt; Network measurements
  has separate toggles, editable HTTPS service settings and a latency interval.
  Automatic latency checks run only while Network is visible and connected.
  Quick-test results are approximate, not line capacity; split routing can
  send another destination over a different route. See the release notes for limits.
- **Full CLI.** `wax.exe` covers profiles, import, connect, status and WARP
  for scripting or SSH sessions.

## How it works

| Crate | Purpose |
| --- | --- |
| `wax-core` | Keys (audited `x25519-dalek`), wg-quick `.conf` parsing, profile storage |
| `wax-warp` | Cloudflare WARP API client (wgcf equivalent), account persistence, profile generation |
| `wax-tunnel` | Adapter lifecycle on WireGuardNT and DNS (netsh) |
| `wax-cli` (`wax.exe`) | CLI: profiles, import, connect/disconnect/status, WARP commands |
| `wax-app` (`wax-vpn.exe`; installed as `waX VPN.exe`) | Native egui GUI: navigation, profile picker, stats, settings, WARP panel, drag-and-drop import |
| `wax-diagnostics` | Allowlisted local status reports |

The NSIS installer places the app and the official `wireguard.dll` together,
registers the icon and uninstaller, and runs adapter/DNS cleanup on
uninstall. The silent CI install/upgrade/uninstall checks passed. Connect applies DNS for the tunnel; disconnect restores
what was there before.

The interface is a native Rust/egui adaptation of Meta's
[Astryx](https://github.com/meta/astryx) design system (MIT): neutral
surfaces, one primary action per region, stable slots for status and counters
so nothing jumps when state changes, and a reduced-motion option. No React or
JavaScript ships in the app; the design was rebuilt natively. See
`docs/ASTRYX-REDESIGN.md` for the mapping.

## Verified vs. not yet verified

Claims here are tested build by build on clean Windows runners. Raw evidence is retained in the linked GitHub Actions run artifacts;
the release notes describe the checks and their limits.

Verified release checks (exact run and sample counts are in the release notes):

- WireGuard handshake and ping through the tunnel, DNS applied on connect and
  restored on disconnect, MTU from imported configs.
- Scoped split routing: excluded CIDRs bypass the tunnel, the rest stays in.
- Adapter-loss recovery, manual reconnect, close-to-tray with the tunnel up,
  graceful quit with adapter removal.
- Silent NSIS fresh install, same-version upgrade and uninstall: no new
  visible `ConsoleWindowClass` windows in 221, 502 and 344 samples at a
  requested 10ms interval in v1.0.2. Shorter flashes or other console-host
  classes may be missed; interactive installation remains unverified.
  Installed-GUI launch and profile/settings warning paths passed.

Not yet verified (honest list):

- Tray-menu Open/Quit interaction (the runner session has no notification
  area; the graceful-quit path itself is verified via `wax-vpn.exe --quit`).
- Public-IP before/after checks through a production Internet tunnel on Windows;
  development-host checks are not proof of Windows Internet routing.
- WARP registration against the live Cloudflare API (requires a person to
  accept the ToS first, by design).
- Leak testing (IPv6, DNS) against an external observer; IPv6 leak protection
  is not claimed.
- Consumer Windows SmartScreen behavior (unsigned installer).
- Bare-image Windows runtime compatibility and the reported other-PC crash.
- Safe graphics rendering and dialog dismissal on the hosted Hyper-V GPU:
  unsupported OpenGL produced the expected visible error and local log.

**No kill switch.** If the tunnel drops, traffic can use the normal
connection.

## Configuration compatibility

waX VPN imports standard WireGuard keys, addresses, IP DNS servers, peer
endpoints and AllowedIPs. DNS search domains and hostnames are rejected
explicitly; only literal IPv4/IPv6 DNS server addresses are supported. It does
not execute wg-quick scripts or implement wg-quick routing-table settings.
Profiles containing `PreUp`, `PostUp`, `PreDown`, `PostDown`, `Table`,
`SaveConfig` or `FwMark` are rejected explicitly rather than silently changing
their behavior. Remove a directive only if you understand what it did; an
imported profile without it can behave differently.

## Building

```
# Windows PowerShell, matching the release static-CRT setting:
$env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS = '-C target-feature=+crt-static'
cargo build --release --workspace
```

The app expects the official `wireguard.dll` next to the executable (the
installer handles this).

## License and credits

waX VPN's own code is MIT (see `LICENSE`). The bundled `wireguard.dll` is the
official prebuilt WireGuardNT binary, distributed by the WireGuard project
under the permissive license included in the installer
(`LICENSE-wireguard-nt.txt`). WireGuard is a registered trademark of Jason A.
Donenfeld; waX VPN is not affiliated with the WireGuard project.

Design tokens and layout adapted from Meta's
[Astryx](https://github.com/meta/astryx) design system, MIT (notice retained
in `docs/vendor-licenses/`). Interface typeface is Inter Display by Rasmus
Andersson, used under the SIL Open Font License 1.1.
