# waX VPN v0.1.0-preview

First automated preview build from CI.

## What's in it
- `waX VPN.exe` - egui GUI with animated connect orb, profile picker,
  settings (kill switch, LAN allow, autostart, reduced motion), opt-in
  Cloudflare WARP panel, drag-and-drop .conf import.
- `wax.exe` - CLI: keygen, profiles, import/show/remove, connect/disconnect,
  status, WARP register/account/generate/unregister.
- Official WireGuardNT `wireguard.dll` (kernel driver) bundled from
  download.wireguard.com.

## Verified
- Unit tests (keys, config round-trip) and a clean Windows CI build.

## NOT yet verified on a real machine
- Live tunnel traffic, kill switch, DNS restore, WARP registration, GUI on a
  real display. Treat this build as a work-in-progress preview.

## Known limits
- Connecting needs administrator rights (the app offers a restart-as-admin
  button when it is not elevated).
- The kill switch is firewall-rule based (not WFP yet).
- Unsigned binaries: SmartScreen may warn.
