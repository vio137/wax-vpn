# waX VPN v1.0.3 candidate

CPU software rendering is the startup default. It presents through Windows GDI
rather than initializing Direct3D, Vulkan or OpenGL. GPU rendering remains opt-in
with `--accelerated-graphics` or the Accelerated graphics Start-menu shortcut.
The previous OpenGL Safe graphics option remains available but still needs a
compatible driver. Windows 10/11 x64 is required; the installer rejects older OSes.

The installer includes WireGuardNT and its vendor-signed kernel driver. Rust
executables use a static C runtime. No separate VC++ runtime, .NET installer,
WebView2, external wgcf or first-run driver download is needed. The app still
needs Windows system components, administrator consent for tunnel changes, a
valid profile, network access and a policy that permits the signed driver.
Current tunnel setup uses Windows PowerShell/netsh and an admin probe through
`net session`; replacing those is a separate candidate. Windows PowerShell uses
Windows-provided .NET Framework. This is not a claim of zero OS dependencies.

## Evidence and limits

Final CPU-default candidate: 1.0.3-preview.439, source
9a417f139405528cc4583f1d4e5972774fa9f278:
https://github.com/vio137/wax-vpn/actions/runs/43082318105

- Passed native static-CRT direct/delay import audits for GUI, CLI and driver.
  No separate VC/managed runtime or external shader compiler imports found.
  Optional GPU modes retain OS graphics DLL imports. Dynamic OS/driver loads
  are not fully established by an import table.
- Passed hosted-runner NSIS install, installed CPU UI launch, upgrade and
  uninstall. User data is intentionally retained.
- Passed demo peer handshake/peer ping, DNS application/reset, IPv4 MTU,
  split routes, bounded recovery, disabling pending recovery and manual reconnect.
- Passed UI-driven connect, close-to-tray, original HWND restore, graceful quit,
  warning states, and 15-second/ten-frame CPU startup.
- Separate focused CPU gate passed forced no-tray fallback and same-window restore:
  https://github.com/vio137/wax-vpn/actions/runs/43080861499

The host was Windows Server 2025 with Hyper-V Video and preinstalled runtimes,
not a clean Windows 10/11 image. The crash on the reported consumer PC has not
been reproduced or confirmed fixed. No flawless fresh-install claim is made.
Peer ping does not prove Internet reachability. No IPv6 baseline or leak test was
established. Real notification-area menu clicks remain manual-only. OpenGL Safe
graphics rendering and dialog dismissal were skipped on the unsupported host.
Console sampling can miss short-lived windows. No kill switch is present; normal
traffic can resume when the tunnel drops. No live WARP signup was performed.

The app remains unsigned, so Windows can show an unknown-publisher warning.
