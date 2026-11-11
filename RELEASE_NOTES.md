# waX VPN v1.0.6

Ctrl+F opens Profiles and focuses its search field from any page, including
while another text field has focus. It never changes the tunnel. The Clear
button resets the search; Escape leaves the field using normal text-input
behavior.

Profile order and lowercase search terms are cached until profiles or settings
change. Unchanged frames reuse the filtered index list without cloning it.
Name and endpoint matches remain separate, so a query cannot match across
the boundary between two fields. No measured speed improvement is claimed.

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

Stable workflow: https://github.com/vio137/wax-vpn/actions/runs/43103697052
Source: `32cad53a3b97cdc5794c2a01d1140c9a6be92eb3`.

The exact stable gate passed Ctrl+F opening Profiles with accessible keyboard
focus, pin/unpin persistence, name/endpoint search filtering and restoration,
GUI demo connection, tunnel-address ping, DNS apply/reset, IPv4 MTU, 67 split
routes, bounded recovery, pending-retry disable, manual reconnect, close-to-tray,
original-window restore and graceful quit. Actual connected, Profiles and
installed GUI pixels were inspected. The limits above still apply.

Fresh install, upgrade, uninstall, PID-scoped graceful running-GUI stops and
bounded exact-path native termination passed. Unrelated same-name processes
survived. Packaged PE import audits passed for all three binaries. No new
visible native console appeared in sampled installer/GUI checks; shorter
flashes can still be missed.

The published installer was downloaded and matched its checksum file:

- File: `waX-VPN-Setup-v1.0.6-x64.exe`
- Size: 11,677,072 bytes
- SHA256: `4620478437781489446a2def474a608bb36c894456f8f313602d5fa04cb7bd5b`
