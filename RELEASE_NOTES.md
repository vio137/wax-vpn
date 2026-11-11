# waX VPN v1.0.1

Unsigned Windows x64 safety patch. Windows may show an unknown-publisher warning. Website: https://waxvpn.pages.dev/.

## Changes since v1.0.0

- CLI WARP Register and Generate now reject an occupied reserved Cloudflare WARP profile path. No replacement option is included. Registration checks before creating a client or making a remote request; exclusive creation also protects against another process creating the profile after that check. On a profile-save failure after registration, the local account is retained and the error explains recovery with `wax warp generate` after keeping or moving the occupied profile. No automatic remote rollback is attempted.
- Added exact-name, non-replacing profile creation for those CLI paths. Existing explicit save/update and additive-import behavior is unchanged. Unit fixtures check absent, occupied-file and occupied-directory paths and preserve the original bytes on a duplicate create.
- CI verifies the unchanged official WireGuardNT1.1 archive SHA-256 before extraction or packaging, and fails closed on mismatch. A deliberate bad-archive fixture exercises rejection. Fresh download hash `dceb30a9bc4be48cce0f74160fc88a585a2c2627366e8f846fc6658f9038dace` was independently checked against [WireGuard's own pinned build recipe](https://github.com/WireGuard/wireguard-windows/blob/6ece77bc487c8aa697e3c092197621c4f3e5ccb8/build.bat). The update procedure is recorded beside the CI pin; no automatic digest updates or alternate source is used.

Only the approved reserved-profile guard and driver archive pin are included. No UI, tunnel routing, DNS, MTU, status/recovery or measurement-request change. No live WARP calls were made. Existing profile/account update writes remain non-atomic; adapter-query uncertainty remains a separate held finding. [v1.0 review](https://github.com/vio137/wax-vpn/blob/main/docs/V1-REVIEW.md).

## Network observation and privacy limits

Public-IP, HTTPS latency and manual quick-speed checks remain off by default; the local adapter-counter graph remains on and makes no extra network requests. Optional checks contact the configured HTTPS service (Cloudflare by default), which sees your public IP. Endpoints, latency interval and separate toggles are editable in Features > Network measurements. Speed testing is manual: one 8 MB download and 2 MB upload plus protocol overhead, approximate single-request speed rather than line capacity. Latency includes DNS/TCP/TLS where needed, not ICMP ping. Automatic latency checks run only while Network is visible and connected. No signup or API key is needed for the defaults.

IP/results stay in memory, are not logged or persisted, and are excluded from local diagnostics. No analytics/result-upload API is called. Redirects are not followed. Requests have bounded timeouts; stale results are discarded after settings or route changes. An in-flight request can finish after its toggle is disabled, within its timeout. These are observations on the current OS route, not leak tests or proof that every destination uses the tunnel. Split routing can send another destination over a different route. An already-running tunnel has no original-IP baseline.

No kill switch is included. Traffic can use the normal connection after the tunnel drops. IPv6 leak protection and physical-network roaming are not claimed.

## Validation

Exact v1.0.1 Windows release gate is pending. Local focused tests passed: 41 core tests and one CLI guard test. The Windows-only Ctrl+C callback test runs in the exact Windows gate. No WARP registration, deletion or other live WARP API call was used for this change.

Remaining limits: real Windows pre/post-connection public-IP observation through a production Internet tunnel; external-observer IPv6/DNS leak testing; consumer SmartScreen/interactive installer behavior; physical-network roaming; tray-menu Open/Quit in a real notification area; real active-tunnel Ctrl+C handling. Crash evidence is a three-second snapshot followed by explicit cleanup, not continuous or general crash-cleanup proof. Console sampling can miss shorter flashes and other console-host classes. Lower Network and expanded measurement settings were inspected in local native pixels, not exact Windows captures. Service availability and compatible custom endpoints are not guaranteed. The app and installer remain unsigned; the bundled vendor driver is separately signed.
