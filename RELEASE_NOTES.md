# waX VPN v1.0.2

Unsigned Windows x64 startup compatibility and diagnostics update. Windows may show an unknown-publisher warning. Website: https://waxvpn.pages.dev/.

## Startup issue and mitigation

A user reported that v1.0.0 opens a window and then closes. The cause on that machine has not yet been reproduced or identified. No claim is made that this build fixes that exact crash. v1.0.1 retained the same GUI startup path, but the user has not confirmed the issue on that version.

- Added bounded local `%APPDATA%\wax-vpn\startup.log` markers for graphics/app creation, settings/profile loading, tray setup, first/ten UI frames, explicit quit sources and normal event-loop return. The file is capped by replacement after64KiB; it contains no profile/account/settings values. Nothing is uploaded.
- Rust panic handling records the source location, not the panic payload, and shows a visible Windows error dialog. Returned window/graphics setup errors are logged and shown rather than disappearing silently. Native driver faults may bypass the Rust hook and still require Windows fault-module evidence.
- A Start-menu Safe graphics shortcut (or `"waX VPN.exe" --safe-graphics`) uses OpenGL as an alternative to the normal wgpu path. This is an explicit compatibility path, not a software-rendering guarantee or automatic crash recovery. Normal renderer and tunnel behavior are unchanged. If a normal instance remains running, quit it before using Safe graphics.
- Added runner startup checks for15s survival, visible rail, ten UI frames, no tunnel adapter and graceful quit in both normal and Safe graphics modes. Logs, screenshots, runner graphics details and matching Windows application events are retained as evidence. Runner success does not prove compatibility with every consumer GPU or non-elevated setup.

## Included v1.0.1 safety changes

CLI WARP Register/Generate reject occupied reserved profile paths before remote registration, and use exclusive exact-name creation to prevent replacement races. No replace option or live WARP calls. Failed profile creation retains the local registration for recovery. CI pins the unchanged official WireGuardNT1.1 archive SHA-256 before extraction, with a deliberate mismatch test. Digest `dceb30a9bc4be48cce0f74160fc88a585a2c2627366e8f846fc6658f9038dace` independently matches [the vendor build recipe](https://github.com/WireGuard/wireguard-windows/blob/6ece77bc487c8aa697e3c092197621c4f3e5ccb8/build.bat).

No production tunnel routing, DNS, MTU, status/recovery or measurement-request changes. Profile/account crash-atomic writes and adapter-query uncertainty remain held follow-ups.

## Network observation and privacy limits

Public-IP, HTTPS latency and manual quick-speed checks remain off by default; the local adapter-counter graph remains on and makes no extra network requests. Optional checks contact the configured HTTPS service (Cloudflare by default), which sees your public IP. Endpoints, latency interval and separate toggles are editable in Features > Network measurements. Speed testing is manual: one 8 MB download and 2 MB upload plus protocol overhead, approximate single-request speed rather than line capacity. Latency includes DNS/TCP/TLS where needed, not ICMP ping. Automatic latency checks run only while Network is visible and connected. No signup or API key is needed for the defaults.

IP/results stay in memory, are not logged or persisted, and are excluded from local diagnostics. No analytics/result-upload API is called. Redirects are not followed. Requests have bounded timeouts; stale results are discarded after settings or route changes. An in-flight request can finish after its toggle is disabled, within its timeout. These are observations on the current OS route, not leak tests or proof that every destination uses the tunnel. Split routing can send another destination over a different route. An already-running tunnel has no original-IP baseline.

No kill switch is included. Traffic can use the normal connection after the tunnel drops. IPv6 leak protection and physical-network roaming are not claimed.

## Validation

Exact v1.0.2 Windows gate is pending. Local app tests:38 passed, one manual external-service test ignored. Local compilation with both renderer features passed. Exact1.0.1 previously passed104 Windows tests and extended runtime/installer checks; those are not proof of the1.0.2 startup paths.

Remaining limits: real Windows pre/post-connection public-IP observation through a production Internet tunnel; external-observer IPv6/DNS leak testing; consumer SmartScreen/interactive installer behavior; physical-network roaming; tray-menu Open/Quit in a real notification area; real active-tunnel Ctrl+C handling. Crash evidence is a three-second snapshot followed by explicit cleanup, not continuous or general crash-cleanup proof. Console sampling can miss shorter flashes and other console-host classes. Lower Network and expanded measurement settings were inspected in local native pixels, not exact Windows captures. Service availability and compatible custom endpoints are not guaranteed. The app and installer remain unsigned; the bundled vendor driver is separately signed.
