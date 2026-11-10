# waX VPN v0.1.11

Unsigned Windows x64 native administrator-restart checkpoint. Windows may show an unknown-publisher warning. The official WireGuardNT driver is independently signed by its vendor. No kill switch: traffic can use the normal connection if the tunnel drops.

## Changes

- Replaced the interpolated PowerShell restart command with native ShellExecuteW using the runas verb. Windows still owns the UAC consent/credential prompt. There is no elevation bypass, service or network call.
- The executable path is a separate UTF-16 API argument, not shell script text. Apostrophes and spaces are not interpreted as PowerShell syntax.
- A reported failed or cancelled launch keeps the current window open and shows a failure message. An accepted launch request exits the old instance. The elevated child waits up to 10 seconds for that parent process to exit before acquiring the existing single-instance mutex. Wait failure or timeout aborts the child without killing another process.

## Verification

Exact release source ee886d0d5229a774a5602f264fe7edfea482cc2d / full manual gate 42828756989 passed Windows workspace tests, tunnel/GUI, fresh install/uninstall and publication. Prepared source f945aaf / full manual preview 42826605856 also passed. A fresh released installer download is 10,949,648 bytes with SHA256 `488b6bd619baddc38a491a6e6b2baabc9e6e4b69c3dc0f63df40faf02090270b`, matching the released SHA256SUMS. Raw test output confirms parser rejection cases, ShellExecute return-code boundary, UTF-16 roundtrip for apostrophe/space/Unicode/non-Unicode paths, and bounded process-wait cases passed. These tests do not invoke UAC or demonstrate the full restart handoff.

Retained exact stable results confirm tunnel-side handshake/ping/DNS/IPv4 MTU 1380, split exclusions, adapter-loss recovery after 5 seconds, pending-retry disable over 48 samples/12 seconds, manual reconnect while automatic recovery was off, close-to-tray/original HWND+PID restore/graceful quit and malformed-settings preservation/warnings. No console windows were observed in 440 samples at about 100 ms; shorter windows could be missed. Crash snapshot at 3 seconds before cleanup showed adapter gone, DNS none, zero old kill-switch rules. Actual Connected, Profiles, Features, WARP, About and installed pixels were inspected; all show v0.1.11. Profiles has scrollable lower controls outside the captured viewport.

Real UAC acceptance and cancellation, the resulting failure message pixels, a successful elevated restart and the complete parent-to-child handoff remain untested. ShellExecute success reports an accepted launch request, not proof that a usable elevated UI has opened.

## Scope and limits

The 8fc4241 push run 42805998294 lost its PowerShell GUI-harness host with exit 0xC0000409 after Connecting captures, before a GUI-connect result. This repeated an earlier unexplained harness termination. Later success does not diagnose either failure. An earlier separate installed quit timed out after 15 seconds in leftover cleanup; later passes do not diagnose that intermittent failure either. No timeout was widened or cleanup skipped. Full preview 42815620221 and exact stable 42817228387 passed with UI Automation steps on installed Windows PowerShell 5.1, without changing security settings. This host comparison does not establish the cause of the prior pwsh terminations.

Settings preservation is proven only for the tested failures, not concurrent edits by another process, storage hardware failure or power loss. Startup failure-warning pixels and a real Windows sign-in launch are not proven. Pending-retry cancellation is demonstrated, not deterministic in-flight worker-result cancellation or retry exhaustion. Peer handshake and packets to the demo server tunnel-side address do not prove Internet reachability. Adapter-loss recovery does not establish physical-network roaming. Crash evidence is a 3-second snapshot, not continuous leak observation. IPv6 was skipped without a proven runner baseline. Real notification-area tray menu remains manual-only.

WireGuardNT remains the only shipping backend. No BoringTun dependency, new product network service, Internet probe, signup, WARP registration, kill switch or IPv6 leak-protection claim. The Astryx-informed interface stays native Rust/egui, without React, JavaScript or WebView runtime. No Cloudflare implementation, assets, credentials or keys are copied.
