# waX VPN v0.1.12

Unsigned Windows x64 profile-read checkpoint. Windows may show an unknown-publisher warning. The official WireGuardNT driver is independently signed by its vendor. No kill switch: traffic can use the normal connection if the tunnel drops.

## Changes

- Profile loading reports a count of unreadable or malformed .conf files instead of silently hiding them. The warning contains no file names, keys or config contents. Skipped files are not changed.
- A directory-level reload failure keeps the previous list visible with an explicit warning. Manual connection from that stale snapshot is guarded until a successful reload. Existing tunnel ownership/recovery is unchanged.
- Profiles has a Reload profiles action. A successful reload clears the read warning; this does not repair unreadable files.

## Verification

Prepared source d5941d4 / full manual preview 42834021598 passed Windows workspace tests, tunnel/GUI and fresh install/uninstall. Exact v0.1.12 build and publication remain pending.

Named unit tests cover mixed valid/malformed/unreadable entries, unchanged original bytes, missing folder versus directory failure, sorted names and count-only warning text. The first mixed-load fixture at 412c26c / run 42831837117 failed because its intended-valid config had no peer. The existing parser correctly rejected it. The fixture was corrected to a peer-bearing config and explicitly checked for parse validity; parser rules and assertions were not relaxed.

Retained GUI results and actual warning pixels demonstrate a skipped-file count, the previous demo profile still visible after a directory reload failure, and warnings clearing after the directory was restored and the malformed fixture removed. Connect/Delete are disabled in the failed-reload Profiles capture. Startup directory-error pixels and an invoked manual Connect failure from that stale snapshot remain untested. Core directory-error behavior and the manual guard are tested or source-checked respectively, not full UI proof for those cases. Lower Import controls remain scrollable beyond the captured viewport.

Peer handshake, tunnel-side ping, DNS and IPv4 MTU 1380, split route exclusions, owned adapter-loss recovery after 6 seconds, pending-retry disable over 48 samples/12 seconds, manual reconnect while recovery was off, close-to-tray/original HWND+PID restore/graceful quit and malformed-settings preservation/warnings passed. No console windows were observed in 438 samples at about 100 ms; shorter windows could be missed. Crash snapshot at 3 seconds before cleanup showed adapter gone, DNS none and zero old kill-switch rules. Actual Profiles/warning and installed pixels were inspected. Fresh install/launch/uninstall passed.

Real UAC acceptance/cancellation, native restart failure pixels and the full elevated parent-to-child handoff from v0.1.11 remain untested. Native restart policy/path/wait unit tests pass; runas retains the Windows prompt.

## Scope and limits

The 8fc4241 push run 42805998294 lost its PowerShell GUI-harness host with exit 0xC0000409 after Connecting captures, before a GUI-connect result. This repeated an earlier unexplained harness termination. Later success does not diagnose either failure. An earlier separate installed quit timed out after 15 seconds in leftover cleanup; later passes do not diagnose that intermittent failure either. No timeout was widened or cleanup skipped. Full preview 42815620221 and exact stable 42817228387 passed with UI Automation steps on installed Windows PowerShell 5.1, without changing security settings. This host comparison does not establish the cause of the prior pwsh terminations.

Settings preservation is proven only for the tested failures, not concurrent edits by another process, storage hardware failure or power loss. Startup failure-warning pixels and a real Windows sign-in launch are not proven. Pending-retry cancellation is demonstrated, not deterministic in-flight worker-result cancellation or retry exhaustion. Peer handshake and packets to the demo server tunnel-side address do not prove Internet reachability. Adapter-loss recovery does not establish physical-network roaming. Crash evidence is a 3-second snapshot, not continuous leak observation. IPv6 was skipped without a proven runner baseline. Real notification-area tray menu remains manual-only.

WireGuardNT remains the only shipping backend. No BoringTun dependency, new product network service, Internet probe, signup, WARP registration, kill switch or IPv6 leak-protection claim. The Astryx-informed interface stays native Rust/egui, without React, JavaScript or WebView runtime. No Cloudflare implementation, assets, credentials or keys are copied.
