# waX VPN v0.1.43

Unsigned Windows x64 network-observation checkpoint. Windows may show an unknown-publisher warning. The bundled official WireGuardNT driver is independently signed by its vendor. Website: https://waxvpn.pages.dev/.

## Network page

- Public IP before and after connection. When enabled, a bounded lookup runs before initial tunnel creation and once after connection; refresh is manual. A failed lookup does not prevent connection. An already-running tunnel has no original-IP baseline. Only one address family and the configured service's route are observed: split routing or other destinations can use another route.
- Live received/sent throughput graph from local adapter counters. Up to 60 one-second samples, with session reset handling. This makes no extra network requests.
- HTTPS response-latency graph. When enabled, automatic probes run only while the Network page is visible and connected, every 10-300 seconds (30 by default); manual testing is available while connected or disconnected. Timing includes DNS, TCP and TLS where needed, not ICMP ping. The graph shows the latest 60 successful observations, not a continuous outage timeline.
- Manual quick speed test: one 8 MB download and 2 MB upload against a compatible HTTPS service. Approximate single-request speed, not line capacity, and may affect other traffic. No automatic speed tests.

Features > Network measurements has separate toggles and editable IP/latency endpoints, speed-service origin and latency interval. Public IP, latency and speed tests are off by default; the local throughput graph is on. Cloudflare is the default service. Optional external checks disclose your public IP to that service, as stated in the UI. No signup or API key is needed; custom endpoints must use HTTPS without embedded credentials. Redirects are not followed. IP values and results stay in memory, are not logged or persisted, and are excluded from local diagnostics. No analytics/result-upload API is called. Requests are bounded, run off the UI thread and old results are discarded after route/settings changes. An in-flight request can finish after a toggle is disabled, within its timeout.

These are observations on the current OS route, not proof that all traffic uses the tunnel and not leak tests. No kill switch is included; traffic can use the normal connection after the tunnel drops. IPv6 leak protection is not claimed. Physical-network roaming is not supported.

## Code and documentation

- Profile picker/list display strings are borrowed rather than cloned on each repaint; owned selection names are created only when saved. No measured speedup claim.
- README rebuilt with a banner, real historical screenshots and scoped feature/verification information. Its image assets are hosted at https://waxvpn.pages.dev/ and rendered through GitHub's image proxy. Desktop and 390px mobile layout checked; historical screenshot versions are captioned.
- Existing no-console installer cleanup and application helper behavior retained.

## Validation

Exact release run [43026258205](https://github.com/vio137/wax-vpn/actions/runs/43026258205) passed build, installer, extended demo/GUI verification and publication at source `28591e7d4ea60f079c03b177b8d1b2574ae16745`. Raw Windows workspace log: 101 tests passed, one manual external-service test ignored. Demo evidence passed handshake/ping, DNS/MTU, GUI connect, connected Network counter-panel visibility, scoped split routing, adapter-loss recovery, recovery-disable after an observed retry (48 samples over 12 seconds), explicit manual reconnect, original main HWND/PID plus visible Connect rail, close-to-tray, graceful quit and profile/settings warning paths. Crash state remains a three-second snapshot then explicit cleanup; IPv6 leak test was skipped.

Actual Windows opt-in Network top, connected counter graph, Features, Connect and installed-GUI screenshots were inspected at 880x600. The Windows lower speed card and expanded measurement settings were not captured; their actual local native GUI was inspected separately. Optional external checks were disabled in the Windows demo-tunnel run; this is not a Windows public-Internet IP-change or speed-test result.

Silent fresh install, same-version upgrade and uninstall passed. No new visible ConsoleWindowClass windows were observed in 169, 426 and 325 samples at a requested 10ms interval respectively; GUI connect had none in 478 samples at about 100ms. This sampling can miss shorter flashes and other console-host classes. Fresh published installer: 11,061,755 bytes, SHA256 `0adf9290a1d51bbafe5e443bda9cae673831ed7743a31d656165dbd9c533f4bf`, downloaded and matched against SHA256SUMS and the release asset digest.

Exact-source local tests: 38 passed, one manual external-service test ignored by default. That manual test was separately run and passed IP parsing, HTTPS latency and the bounded download/upload request path against Cloudflare from the development host. Actual local GUI latency and speed buttons were exercised and their results inspected, not promoted as VPN benchmarks. Tests cover opt-in defaults, HTTPS URL restrictions, IP parsing, units, bounded histories, session reset, settings compatibility and stale-result rejection.

Remaining limits: real Windows pre/post-connection public-IP observation through a production Internet tunnel; external-observer IPv6/DNS leak testing; consumer SmartScreen/interactive installer behavior; physical-network roaming; tray-menu Open/Quit interaction in a real notification area. Existing crash evidence is a three-second snapshot followed by explicit cleanup, not continuous or general crash-cleanup proof. Console sampling can miss shorter flashes and other console-host classes. Service availability and compatible custom-endpoint behavior are not guaranteed.
