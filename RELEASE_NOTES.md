# waX VPN v1.0.0

Stable Windows x64 release after a whole-codebase review and focused fixes. Unsigned: Windows may show an unknown-publisher warning. The bundled official WireGuardNT driver is independently signed by its vendor. Website: https://waxvpn.pages.dev/.

## Changes since v0.1.44

- Reduced scheduled steady-state visible polling from 33ms to 250ms; connecting/disconnecting remains 33ms, hidden windows 1000ms. Input and animation may wake earlier. Removed an unnecessary per-frame cleanup-warning clone. These are code-level changes, not measured speedup claims.
- Network now has full-width cards, Received/Sent legends, Mbps/ms graph scales, a direct measurement-settings button and a clear disabled-speed hint. Added Ctrl+6 for Network and shortcut hints; existing Ctrl+1 through Ctrl+5 mappings are unchanged.
- Profile ordering caches lowercase sort keys instead of recomputing them for comparisons; ordering semantics are unchanged.
- Windows CLI Ctrl+C and Ctrl+Break request the existing orderly shutdown path. Enter and input EOF still request shutdown. Handler setup happens before tunnel creation and fails early if unavailable. The handler callback has a Windows unit test; a real active-tunnel Ctrl+C exercise remains unverified.
- CLI WARP unregister now returns a remote failure and retains the local account and reserved profile so unregister can be retried. No live WARP calls were made for this release.
- Uninstall removes the owned LICENSE-astryx.txt file. The clean-runner installer check now requires that file and the install directory to be absent after uninstall.
- Removed 302 lines of retired, uncompiled Gargantua scene code. No active scene was removed.

Review covered all six Rust crates, installer, build/verification scripts, workflow, README and current documentation. [Review scope and deferred findings](https://github.com/vio137/wax-vpn/blob/main/docs/V1-REVIEW.md). This was a development review, not an independent security audit or proof of every runtime path. Production tunnel routing, DNS, MTU, status/recovery and measurement-request behavior were not changed.

## Network observation and privacy limits

Public-IP, HTTPS latency and manual quick-speed checks remain off by default; the local adapter-counter graph remains on and makes no extra network requests. Optional checks contact the configured HTTPS service (Cloudflare by default), which sees your public IP. Endpoints, latency interval and separate toggles are editable in Features > Network measurements. Speed testing is manual: one 8 MB download and 2 MB upload plus protocol overhead, approximate single-request speed rather than line capacity. Latency includes DNS/TCP/TLS where needed, not ICMP ping. Automatic latency checks run only while Network is visible and connected. No signup or API key is needed for the defaults.

IP/results stay in memory, are not logged or persisted, and are excluded from local diagnostics. No analytics/result-upload API is called. Redirects are not followed. Requests have bounded timeouts; stale results are discarded after settings or route changes. An in-flight request can finish after its toggle is disabled, within its timeout. These are observations on the current OS route, not leak tests or proof that every destination uses the tunnel. Split routing can send another destination over a different route. An already-running tunnel has no original-IP baseline.

No kill switch is included. Traffic can use the normal connection after the tunnel drops. IPv6 leak protection and physical-network roaming are not claimed.

## Validation

Exact v1.0.0 [release run 43042035431](https://github.com/vio137/wax-vpn/actions/runs/43042035431) passed build, demo/GUI, fresh-runner installer and publication at source `8b1e4178494829e130a3fa47986ee4da22cc9b34`. Exact Windows tests: 102 passed, one manual external-service test ignored, with no Rust warning/error lines in the test or release-build logs. The downloaded installer is 11,062,776 bytes, SHA-256 `cffdfc167ec6b9afe95370a8b6dbbff067cf591a82ae9f9796bef9dfb285d19e`, matching SHA256SUMS and the release API asset digest.

Raw exact runtime passed GUI connect, scoped split routes, adapter-loss recovery in 6s, disabling Auto-reconnect after an observed pending retry (48 adapter-absence samples over 12s), explicit reconnect while disabled, original main HWND/PID visible rail restore, graceful quit, partial-profile warning/reload recovery and malformed/BOM settings checks. At 3s after simulated owner termination, adapter was gone and tunnel DNS absent, followed by explicit cleanup. IPv6 leak-protection test was skipped because there was no proven baseline and no kill switch; it is not a pass. No console was observed in 440 GUI samples about 100ms or 545/658/330 fresh/upgrade/uninstall samples requested 10ms. Sampling limits below apply. Actual exact Windows Network and connected local graph pixels were inspected; the exact installed view is readable but its bottom edge is taskbar-covered. Lower Network and expanded measurement settings remain local-native-only visual checks. Candidate [43039930525](https://github.com/vio137/wax-vpn/actions/runs/43039930525) passed build, demo/GUI and fresh-runner installer at source 8a219a7fd2aaad53cc88fd42f0d35fe4637ef5c1; publication skipped. Raw Windows workspace tests: 102 passed, one manual external-service test ignored; no Rust warning/error lines in test or release-build logs. Local workspace: 88 passed, one manual external-service test ignored.

Candidate runtime passed CLI tunnel handshake/ping, DNS restoration and configured MTU, GUI connect/local-counter graph/scoped split routes, adapter-loss recovery in 4s, original main HWND/PID and visible rail, close-to-tray and graceful quit. Fresh install, upgrade and uninstall assertions passed. No console was observed in 202 GUI samples about 100ms or 208/505/345 installer samples requested 10ms. Sampling can miss shorter flashes or other host classes. Push builds skip extended crash/IPv6/settings legs; the exact manual release runs them.

Actual Windows Network and connected local graph pixels inspected. Local native top/lower Network and measurement settings were inspected at 880x600. A fresh empty configuration was checked in actual pixels and code: all three external measurements off, local throughput on, with no settings file before or after. The installed candidate capture has a taskbar covering the bottom edge; it is not full-window pixel proof.

Remaining limits: real Windows pre/post-connection public-IP observation through a production Internet tunnel; external-observer IPv6/DNS leak testing; consumer SmartScreen/interactive installer behavior; physical-network roaming; tray-menu Open/Quit in a real notification area; exact Windows lower speed card and expanded measurement-settings capture. Existing crash evidence is a three-second snapshot followed by explicit cleanup, not continuous or general crash-cleanup proof. Service availability and compatible custom endpoints are not guaranteed.

Deferred review findings: adapter query errors can resemble down/absent state; direct profile/account writes are not crash-atomic; CLI WARP register/generate lacks the GUI reserved-profile overwrite guard; upstream driver download is HTTPS but not checksum-pinned. These remain disclosed follow-ups, not silently changed behavior in this release.
