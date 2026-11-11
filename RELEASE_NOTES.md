# waX VPN v0.1.44

Unsigned Windows x64 polish and allocation-cleanup checkpoint. Windows may show an unknown-publisher warning. The bundled official WireGuardNT driver is independently signed by its vendor. Website: https://waxvpn.pages.dev/.

## Changes since v0.1.43

- Each optional measurement worker copies only its selected endpoint string, rather than all three endpoint strings and the full measurement settings object. Request destinations, sizes, timing and defaults are unchanged.
- A graph with one observation now shows a point instead of waiting for a second observation to draw a line. Checked in the actual local native UI after one manual HTTPS latency request. This development-host result is not a VPN benchmark.
- The Profiles split-exclusion field reuses one validation parse for the border and label on unchanged frames. Editing frames reparse immediately for the current label. Parsing rules, saved exclusions and tunnel routes are unchanged.
- Removed two unused settings compatibility wrappers that swallowed errors. Existing app call sites continue to use checked load/save paths; settings format and visible warning behavior are unchanged. The candidate Windows test and release-build logs have no Rust warning/error lines.

These are small code and UI changes, not a measured performance improvement. No new tunnel, routing or measurement-service behavior is included.

## Network observation and privacy limits

Public-IP, HTTPS latency and manual quick-speed checks remain off by default; the local adapter-counter graph remains on and makes no extra network requests. Optional checks contact the configured HTTPS service (Cloudflare by default), which sees your public IP. Endpoints, latency interval and separate toggles are editable in Features > Network measurements. Speed testing is manual: one 8 MB download and 2 MB upload plus protocol overhead, approximate single-request speed rather than line capacity. Latency includes DNS/TCP/TLS where needed, not ICMP ping. Automatic latency checks run only while Network is visible and connected. No signup or API key is needed for the defaults.

IP/results stay in memory, are not logged or persisted, and are excluded from local diagnostics. No analytics/result-upload API is called. Redirects are not followed. Requests have bounded timeouts; stale results are discarded after settings or route changes. An in-flight request can finish after its toggle is disabled, within its timeout. These are observations on the current OS route, not leak tests or proof that every destination uses the tunnel. Split routing can send another destination over a different route. An already-running tunnel has no original-IP baseline.

No kill switch is included. Traffic can use the normal connection after the tunnel drops. IPv6 leak protection and physical-network roaming are not claimed.

## Validation

Exact release run [43035591745](https://github.com/vio137/wax-vpn/actions/runs/43035591745) passed build, extended demo/GUI verification, installer and publication at source `3857e6b2a8394a3168eb9c6db8afb24328df2097`. Raw Windows workspace log: 101 tests passed, one manual external-service test ignored, no Rust warnings/errors in the test or release-build logs. Exact raw runtime passed GUI connect, local Network graph, scoped split routes, adapter-loss recovery (7 seconds), recovery-disable after an observed retry (48 samples over 12 seconds), explicit manual reconnect, original main HWND/PID with visible rail, close-to-tray, graceful quit and profile/settings warning paths. Crash state was a three-second snapshot then explicit cleanup; IPv6 leak testing was skipped.

Silent installer fresh install, same-version upgrade and uninstall passed. No new visible ConsoleWindowClass windows were observed in 162, 410 and 326 samples at a requested 10ms interval; GUI connect had none in 505 samples at about 100ms. Shorter flashes and other console-host classes can be missed. Actual exact Windows Network, Profiles/split-field, settings warning and installed-GUI screenshots were inspected at 880x600. The single-observation point was separately inspected in the actual local native UI; Windows external measurement tests were not run.

Published installer: 11,062,416 bytes. SHA256 `e6d3089a53419216acd428f8f450e8acf8553c1d9b50a76930ea68eecdcc5e52`, freshly downloaded and matched against SHA256SUMS and the release asset digest. Candidate run [43031797310](https://github.com/vio137/wax-vpn/actions/runs/43031797310) passed Windows build, demo/GUI and fresh-runner installer checks at source `041518e1d064fd9bbbfd7c244c5aa8a9baa92ef8`; publication was skipped. Raw Windows workspace log: 101 tests passed, one manual external-service test ignored. Focused local app tests: 38 passed, one manual external-service test ignored.

Candidate raw runtime passed GUI connect, connected Network local-counter visibility, scoped split routing, adapter-loss recovery (6 seconds), original main HWND/PID with visible Connect rail, close-to-tray and graceful quit. The actual Network and Profiles/split-field Windows screenshots were inspected at 880x600. GUI console observation found none in 253 samples at about 100ms. Push builds skip extended crash/IPv6 legs; the exact manual release runs them. Optional external checks remain disabled during the Windows demo-tunnel checks.

Remaining limits: real Windows pre/post-connection public-IP observation through a production Internet tunnel; external-observer IPv6/DNS leak testing; consumer SmartScreen/interactive installer behavior; physical-network roaming; tray-menu Open/Quit interaction in a real notification area. Existing crash evidence is a three-second snapshot followed by explicit cleanup, not continuous or general crash-cleanup proof. Console sampling can miss shorter flashes and other console-host classes. The Windows lower speed card and expanded measurement settings were not captured; their actual local native GUI was inspected in v0.1.43 development. Service availability and compatible custom-endpoint behavior are not guaranteed.
