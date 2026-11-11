# waX VPN v1.0.4

Tunnel setup now uses native Windows APIs for administrator detection, MTU,
DNS settings and cleanup of old waX VPN KillSwitch rules. This removes the
PowerShell/NetTCPIP/NetSecurity and net session dependencies. No kill switch
is created. Normal traffic can resume when the tunnel drops.

DNS uses SetInterfaceDnsSettings when the OS provides it. netsh is retained
only on older Windows builds without that native DNS API. That API-absent
fallback has not been runtime-tested. Windows 10/11 x64 support is unchanged.
Installer/uninstaller taskkill use remains in this release; its replacement
is a separate later candidate. Windows system APIs, administrator consent,
networking, a valid profile and permission to load the bundled signed driver
are still required. This is not a zero-OS-dependency claim.

CPU software rendering through GDI remains the default. GPU rendering stays
opt-in with --accelerated-graphics; OpenGL Safe graphics still needs a
compatible driver. The app is unsigned, so Windows can show a publisher warning.

## Evidence and limits

Pre-release gate, exact native source 4b0e77c116e02e218dbe3c93759b686f606b9f27:
https://github.com/vio137/wax-vpn/actions/runs/43087900944
The stable workflow reruns every required gate before publication.

- Windows workspace tests passed. Isolated native API tests passed IPv4/IPv6
  MTU readback, per-family DNS set/readback/reset/readback, and removal of a
  disabled legacy rule while preserving an unrelated disabled rule.
- Demo peer handshake and tunnel-address ping, DNS apply/reset, IPv4 MTU1380,
  split routes, bounded reconnect, disabling pending recovery and manual
  reconnect passed. No claim of general Internet reachability is made.
- CPU launch/input/connect, close-to-tray, original HWND/PID restore, quit,
  warning states, NSIS install/upgrade/uninstall and packaged native/static-CRT
  direct/delay import audits passed.

The host was Windows Server2025 with Hyper-V Video and preinstalled runtimes,
not a clean Windows10/11 image. The reported consumer-PC crash has not been
reproduced or confirmed fixed. No flawless fresh-install claim is made.
Isolated IPv6 DNS/MTU checks do not establish IPv6 tunnel traffic or leak
behavior. No IPv6 baseline/leak test was established. Real tray-menu clicks
remain manual-only. OpenGL rendering/dialog dismissal were skipped on the
unsupported host. Console sampling can miss short-lived windows. No live
WARP signup was performed.
