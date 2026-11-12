# waX VPN v1.0.27

Copy summary in the expanded loaded-config card copies its allowlisted values
and configured-only limits to the system clipboard. It includes profile name,
addresses and DNS, but no keys or endpoints. Nothing is uploaded by the app;
clipboard sharing or sync follows the user's system settings. Copy happens
only when the button is pressed. No networking behavior or saved setting changed.

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

Local app tests passed (53 passed, one manual ignored), including clipboard
text allowlisting, key/endpoint exclusion and configured-only limits. Build and
PowerShell syntax passed. Revised preview:
https://github.com/vio137/wax-vpn/actions/runs/43177728480
Source: `e8a278adb998c6d06efbbf18e262887e43359b47`. All three execution jobs
passed. Actual Windows system clipboard contained the allowlisted summary and
limits without key/endpoint values. Prior summary, cache reload/restoration,
unchanged files/settings/selection/no adapter, profile/demo/recovery and installer
checks passed. Summary and installed-GUI pixels inspected. Stable publication
and checksum verification pending.

## Preview failure and correction

The first preview (43175710700, source 95e45143) failed the actual Windows
clipboard-content check: Copy summary did not update the system clipboard.
Inspection found the software backend's egui-winit OS clipboard feature was
disabled, using app-internal fallback text. Enabled its existing arboard feature
for OS text clipboard support; no third-party crate version changed. Revised
Windows actual clipboard-content check passed in the revised preview. This also enables OS clipboard integration for other
text controls on this backend. No network behavior changed.
