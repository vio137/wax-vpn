# waX VPN v1.0.19

Profiles has a Show selected action. It clears this list's temporary search
and Pinned only filter, then scrolls to the selected row. Selection, stored
pins/history/settings, profile files and the connection stay unchanged.
It does not select a different profile or start a connection.
No tunnel, DNS, firewall or service behavior changed.

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

Local app tests: 48 passed, one manual networking test ignored. Build passed.
Local pixels showed the selected Office 24 row revealed from a hidden pinned
filter in a 28-profile list; settings file hash stayed unchanged.
PowerShell parser passed. Preview:
https://github.com/vio137/wax-vpn/actions/runs/43153197046
Source: `08773acd2ca20f7a4a295cd150b66a06b4f027e2`.

All three execution jobs passed. Actual Windows proof passed zero-match search
with Pinned only active, repeated reveal of the exact selected virtual row,
unchanged config/settings files and no adapter. Selected-row and installed-GUI
pixels were inspected. Prior import/duplicate/review/Clear, search/counts/pins,
virtualized lists/picker and connection/installer checks also passed within
the compatibility limits above.

Stable: https://github.com/vio137/wax-vpn/actions/runs/43154083692
Source: `470cad859f1e4a7164e2969d6dc6015813684dff`.

All four stable jobs passed. Actual stable Windows proof passed the new repeated
selected-row reveal and unchanged files/settings/selection/no-adapter checks.
Stable selected-row and installed-GUI pixels were inspected. Prior demo tunnel,
DNS/IPv4 MTU/split, bounded recovery, retry disable (48 samples over 12s), manual
reconnect, original HWND/PID restore and quit passed. No console windows were
observed in 324 samples at about 100ms; shorter flashes could be missed.

Release: https://github.com/vio137/wax-vpn/releases/tag/v1.0.19
The downloaded 11,707,077-byte installer matches the published SHA256:
`55440a973f659046fbcedcd878ae6470c65112cf2bdf5440a51bdba73e2d9c31`.
