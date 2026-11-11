# waX VPN v1.0.8

The Connect-page profile picker has separate name and endpoint columns, shorter
single-line text with full details on hover, keyboard focus borders and an
accessible Switch profile action. Choosing a profile saves that choice without
starting a connection. The existing pin and recent-use order is reused.

About includes a collapsible keyboard shortcut reference. Profile search keeps
its normalized query until the typed text changes. No measured performance
gain is claimed. No tunnel, DNS or firewall behavior changed.

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

Preview build, runtime and fresh-installer gates passed:
https://github.com/vio137/wax-vpn/actions/runs/43117231739
Source: `d8881de92c5c9fcf4892bcf9ddf6d3f9667c0fac`.

Actual Windows evidence passed quick-picker selection and persistence without
a connection, plus keyboard help expanded through its visible click action.
Picker and expanded-help pixels were inspected. UIA ExpandCollapse support is
not verified. Existing native-file import and runtime checks passed as well.
Exact stable evidence and installer checksum will be recorded after its gate.
