# waX VPN v0.1.42

Unsigned Windows x64 performance and installer checkpoint. Windows may show an unknown-publisher warning. The official WireGuardNT driver is independently signed by its vendor. Website: https://waxvpn.pages.dev/

## Changes

- Installer and uninstaller run taskkill and CLI cleanup without opening console windows, using NSIS's no-console execution plugin. Cleanup order is unchanged.
- Connect borrows selected-profile name and endpoint text instead of cloning it on every repaint. Tray menu text is updated on connection-state changes rather than every repaint. No system-level speedup or memory-saving claim is made.
- The app About link and installer branding use https://waxvpn.pages.dev/. The Windows installed-app entry includes the same website.

## Verification

Full non-publishing run 43018319942 at cabca929545e46c0814f69be24b7e5d7befa0df6 passed Windows build, GUI/demo tunnel verification and fresh-runner installation. Raw runtime and installer evidence were read and all 14 normal, warning, BOM and installed screenshots inspected.

Silent NSIS fresh install, same-version upgrade and uninstall had no new visible ConsoleWindowClass windows observed in 180, 395 and 329 samples respectively, at a requested 10ms interval. Shorter flashes and other console-host classes can be missed. Interactive installer pages and consumer Windows SmartScreen remain unverified. Installed GUI launch, graceful quit and uninstall passed.

Peer handshake, tunnel-side ping, DNS, imported IPv4 MTU 1380, split exclusions, adapter-loss recovery, pending-retry disable across 48 samples/12 seconds, explicit reconnect with automatic recovery off, original main HWND/PID plus visible Connect rail restoration, quit/adapter removal, settings/profile warnings and BOM-backed startup passed. No console windows were observed during GUI connection in 463 samples at about 100ms; shorter windows could be missed. Crash evidence is a 3-second pre-cleanup snapshot followed by explicit cleanup, not a general crash-cleanup guarantee.

Exact v0.1.42 run 43020463650 at d5969977a36f0bde9fada1b01f0a8c95dcf276de passed all four build, GUI/demo verification, installer and publication jobs. Raw build output confirms 43 app, 40 core, 6 WARP, 4 diagnostics and 1 tunnel tests passed. Exact raw runtime evidence passed the checks above, including recovery after 4 seconds, disable across 48 samples/12 seconds and manual reconnect. GUI console observation had no windows in 365 samples at about 100ms; shorter windows could be missed. All 14 exact-release screenshots were inspected, including the pages.dev About link and readable selected/empty profile states at 880x600.

Exact silent NSIS fresh install, same-version upgrade and uninstall had no new visible ConsoleWindowClass windows in 150, 364 and 329 samples respectively, at a requested 10ms interval, with the same sampling limits. Installed GUI launch, graceful quit and uninstall passed.

Fresh released installer: 10,963,400 bytes. SHA256 1feee1a6e93140a010c44b89885cc888df51d7dcbf20bca0454d38843426487d matches the released SHA256SUMS file and asset digest.

An earlier full run 43016948262 failed because checkout ran after artifact download and removed the installer input before installation. This was a test setup error; checkout was moved first. No product behavior or test assertion was weakened.

## Limits

No kill switch: traffic can use the normal connection when the tunnel drops. A tunnel-side ping does not prove Internet reachability. IPv6 protection was skipped without a proven baseline. No external leak-test claim, physical-network roaming proof, actual notification-area menu interaction, real UAC handoff proof, deterministic in-flight cancellation proof or continuous production network-observation claim.

WireGuardNT remains the shipping backend. No new product network service, signup, WARP registration, copied Cloudflare implementation/assets/credentials/keys or spending is part of this release.
