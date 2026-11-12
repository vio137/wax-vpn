# waX VPN v1.0.25

The read-only loaded-config summary now lists explicit /0 default-route
families: IPv4 only, IPv6 only, both, or none. It checks all loaded peers,
without inferring coverage from split prefixes. This describes configured
AllowedIPs, not current routes, Internet reachability or leak protection.
Keys remain excluded. No saved setting or networking behavior changed.

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

Local app tests passed (51 passed, one manual ignored), including single,
dual-family, multiple-peer and split-prefix cases. Build and PowerShell syntax passed.
Preview: https://github.com/vio137/wax-vpn/actions/runs/43170288176
Source: `fcf9c405ee1d130819d4ed9756889f0fe98d4d62`. All three execution jobs
passed. Actual Windows summary showed the explicit /0 None row, expanded and
collapsed, excluded keys and kept files/settings/selection unchanged without
creating an adapter. Prior profile/demo/recovery and installer checks passed.
Summary and installed-GUI pixels inspected.

Stable: https://github.com/vio137/wax-vpn/actions/runs/43171320662
Source: `7a4b38f71a2feb553eef90c623e3968c4ab0395d`. All four jobs passed.
Actual full results passed explicit-default summary and prior profile checks,
demo peer handshake/tunnel-address ping, DNS/MTU restore, split-route checks,
4-second recovery, recovery-disable with 48 samples over 12 seconds, manual
reconnect, original HWND/PID restore and graceful quit. No console observed in
336 samples at about 100 ms; shorter flashes can be missed. Stable summary and
installed-GUI pixels inspected. Fresh-runner installer checks passed.
Published installer (11,712,549 bytes) downloaded and independently hashed,
matching SHA256SUMS-v1.0.25.txt:
`6ff166544c82dec3ed34c1008100cb2b98ba778b72847350f65ae132ff0c3918`.
No performance claim.
