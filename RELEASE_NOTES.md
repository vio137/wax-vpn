## waX VPN Windows preview
Windows x64 installer with the pure-Rust egui interface, per-profile IP-range exclusions, single-instance window reopening, and opt-in Cloudflare WARP registration.

### Checked on the Windows CI runner
- GUI Connect established a demo WireGuard handshake
- Packets traversed the demo tunnel; DNS applied and restored on disconnect
- Scoped split-routing check: 8.8.8.8 excluded, 1.1.1.1 retained in tunnel routes
- Closing the window kept the demo adapter up
- A second launch restored the original main window with its Connect rail on-screen, then exited itself
- --quit after reopening exited the app and removed the adapter

These checks passed for preview.46. Later builds carry their own CI results and screenshots; this baseline is not a guarantee for every Windows environment.

### Limits
Unsigned preview: Windows SmartScreen may warn. Connecting requires administrator rights. Tray-menu interaction still needs a real Windows desktop check. Kill-switch blocking and live WARP registration are not traffic-verified. WARP registration requires explicit acceptance in the app. No third-party public-IP lookup is performed.
