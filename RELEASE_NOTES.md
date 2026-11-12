# waX VPN v1.0.18

Profiles has a Duplicate selected action. It creates a local copy of the
selected loaded WireGuard config using the same keys, with a collision-safe
name such as Office - copy or Office - copy - 2. It does not replace the
original, change selection or start a connection. Pins, history and split
exclusions are not copied. The action is disabled while connecting/connected
or when the profile list could not be reloaded. A copy is not a new server,
account or credential.
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

Preview: https://github.com/vio137/wax-vpn/actions/runs/43151256402
Source: `483d49599c2636084da1dc1bd8bf9e34fc10fb34`.

All three preview execution jobs passed. Actual Windows results passed repeated
collision-safe copies, unchanged original/selection/settings and no adapter.
Copy notice and installed-GUI pixels were inspected. Local app tests: 48
passed, one manual networking test ignored. Core tests: 43 passed, including
complete config equality, original preservation, repeated naming and Unix
0600 permissions. Build and PowerShell parser passed.

Existing import/review/Clear, keyboard search/counts, pinned filter, bounded
lists/picker, GUI demo connection, DNS/IPv4 MTU/split routes, bounded recovery,
retry disable, original-window restore and quit passed. Installer lifecycle,
native process-scope checks and PE audits passed within the limits above.

Stable publication and downloaded-installer checksum verification are pending.
