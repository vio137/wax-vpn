# waX VPN v1.0.5

Pin profiles, find them by name or endpoint, and return to recent profiles.
Pins and the last eight successful tunnel starts stay on this device. Clear
recent removes that list without deleting profiles or pins. A recent entry
means the app started a tunnel, not that Internet access was verified.

Connection details show the active session's configured addresses, DNS and
peer route-entry count. For a tunnel owned by another app instance, the
configuration is marked unknown. These details are not a live routing or DNS
test. Private keys are not shown. Long content scrolls instead of clipping.

The installer no longer calls taskkill by executable name. It verifies the
installed executable's full path and retains process handles before stopping
anything. Current versions receive a PID-scoped graceful-quit signal first;
older versions without that signal use bounded, exact-path native termination.
An unrelated copy is not sent the session-global quit signal. If the process
cannot be verified or stopped, installation/uninstallation aborts.

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

Pending the exact stable workflow and downloaded installer checksum. Do not
interpret this preparation note as a released build.
