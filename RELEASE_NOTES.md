# waX VPN v1.0.16

The Profiles list now draws only visible rows plus a small buffer instead of
building every row. The list stays bounded at 250px, with the same search,
pin, select, connect and delete controls. Stable row IDs keep controls tied
to their profile while scrolling. No measured speedup is claimed.
No tunnel, DNS, firewall or saved-setting behavior changed.

## Compatibility and limits

CPU software rendering through Windows GDI remains the default. GPU rendering
is opt-in. No separate VC++/.NET/WebView2 installation is required, but Windows
system libraries, administrator consent, networking and a valid profile still
are. The bundled WireGuardNT driver is signed by its vendor. The app and
installer are unsigned and Windows may show an unknown-publisher warning.

DNS uses the native Windows API when available. netsh remains only for older
Windows builds without that API; the API-absent fallback is untested. IPv6
DNS/MTU settings checks do not prove IPv6 traffic routing or leak protection.
No kill switch is provided. Traffic can use the normal connection when the
tunnel is off. Live WARP registration is not verified.

The release gate runs on a hosted Windows Server 2025 runner with installed
runtimes and a virtual display, not a clean Windows 10/11 PC. It does not prove
the reported consumer-PC crash is fixed. A peer handshake and tunnel-address
ping do not prove Internet reachability. Tray-menu clicks need real Windows
review. Runner OpenGL rendering/dismissal remain unverified. Console-window
sampling can miss a shorter flash.

## Release verification

Exact stable workflow: https://github.com/vio137/wax-vpn/actions/runs/43147167278
Source: `fa59e9caa6d47a55042c9e81db16a1b8dd5f6a39`.

All four stable jobs passed. Actual Windows results passed bounded visible
Profiles rows, scroll to the final profile, correct Pin target and unchanged
selection/config without an adapter. First/final-row and installed-GUI pixels
were inspected. Existing pinned/search, local syntax review, editor Clear,
native imports, keyboard search and bounded quick-picker checks passed.
Local app tests: 47 passed, one manual networking test ignored. Build and
PowerShell parser passed. The first preview also passed; its new list image
names were corrected to match the evidence-upload glob for the second preview.

Existing GUI demo connection, tunnel-address ping, DNS apply/reset, IPv4 MTU,
split routes, bounded recovery, retry disable, original-window restore and
graceful quit passed. Fresh install, upgrade, uninstall, all three native
process-scope checks and three PE audits passed within the limits above.
330 console-window samples at about 100ms saw no new visible native console;
shorter flashes can be missed.

The published installer was downloaded and matched its checksum file:

- File: `waX-VPN-Setup-v1.0.16-x64.exe`
- Size: 11,704,255 bytes
- SHA256: `a7e6410d6c75b90654cd2fdfd1901899f11c26d8da6be4107649e06bb15cf55f`
