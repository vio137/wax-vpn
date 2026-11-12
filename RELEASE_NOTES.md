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
checks passed. Summary and installed-GUI pixels inspected.

Stable run https://github.com/vio137/wax-vpn/actions/runs/43178994579
attempt 2 passed all four jobs on source
`dcde1825248f5230c6862297db9a11a9b6c4202c`. Actual retry evidence checked:
tunnel-address ping, clipboard content, summary/cache reload, local-state
preservation, profile and recovery assertions, startup and installer proof.
Recovery took 4 seconds; disabled recovery remained off in 48 samples over 12
seconds; 344 console samples saw no window, with shorter flashes still possible.
Retry summary and installed-GUI pixels inspected. Optional layout-state captures
remain unverified because the separate harness host did not load System.Drawing.

Published installer downloaded and SHA-256 matched its published checksum:
`5e1f7c11d164e31715df4628cb5869896bd2203f259c8d336ecc633b49d57099`
(11,768,661 bytes).
https://github.com/vio137/wax-vpn/releases/tag/v1.0.27


## Preview failure and correction

The first preview (43175710700, source 95e45143) failed the actual Windows
clipboard-content check: Copy summary did not update the system clipboard.
Inspection found the software backend's egui-winit OS clipboard feature was
disabled, using app-internal fallback text. Enabled its existing arboard feature
for OS text clipboard support; no third-party crate version changed. Revised
Windows actual clipboard-content check passed in the revised preview. This also enables OS clipboard integration for other
text controls on this backend. No network behavior changed.

## First stable-gate attempt

Run 43178994579 attempt 1, source dcde182, stopped publication. The demo
peer handshake passed, but its tunnel-address ping to 192.168.4.1 failed.
The accumulated failure also made the later GUI step fail; its functional
connect/recovery/disable/restore/quit assertions passed. Some optional connected
frame captures failed because System.Drawing types were not loaded in that
step; those frames are unverified.
The local profile/clipboard and startup-soak steps did not run in this attempt.
Build and fresh-runner installer jobs passed. Retried the failed gate once on
the exact same source without weakening any assertions. Attempt 2 passed the
tunnel ping and full gate. The original failure remains recorded here.
