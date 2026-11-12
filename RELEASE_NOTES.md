# waX VPN v1.0.20

Profiles and the quick picker match all whitespace-separated search words
across the profile name and peer endpoints. For example, Office 23 127.0.0.1
can find a profile without requiring an exact phrase. Search ignores case;
whitespace-only queries restore the list. Pinned only still applies to the
Profiles list, and searches do not select, save or connect a profile.
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

Local app tests: 49 passed, one manual networking test ignored. Build and
PowerShell parser passed. Tests cover Unicode, repeated words, whitespace,
name/endpoint and multiple peer terms, no cross-profile matches, pins and
cached repeat queries. Local pixels showed one matching Office 23 row from
name plus endpoint words in a 28-profile list.
Preview: https://github.com/vio137/wax-vpn/actions/runs/43155412781
Source: `d06638d43660d2142c5ee7f2a92eb5e122303bc2`.

All three execution jobs passed. Actual Windows proof passed multi-word name
plus endpoint queries in Profiles and quick picker, every-word requirement,
whitespace reset and unchanged files/settings/selection without an adapter.
Word-search and installed-GUI pixels were inspected. Prior local profile and
connection/installer checks also passed within the compatibility limits above.

Stable: https://github.com/vio137/wax-vpn/actions/runs/43156571792
Source: `fea8c74b1ddb05754629e557c266b4e490662239`.

All four stable jobs passed. Actual stable Windows proof passed the new name
plus endpoint/every-word/whitespace checks and unchanged files/settings/selection
without an adapter. Stable word-search and installed-GUI pixels were inspected.
Prior demo tunnel, DNS/IPv4 MTU/split, 4s recovery, retry disable (48 samples over
12s), manual reconnect, original HWND/PID restore and quit passed. No console
windows were observed in 337 samples at about 100ms; shorter flashes could be
missed. Installer lifecycle, native checks and PE audits passed.

Release: https://github.com/vio137/wax-vpn/releases/tag/v1.0.20
The downloaded 11,708,200-byte installer matches the published SHA256:
`cccb2268156b23316a603fe083aa918d93c29b9146b54538e924a33ab838f2f6`.
