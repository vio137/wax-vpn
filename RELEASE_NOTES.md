# waX VPN v1.0.2

Unsigned Windows x64 startup compatibility and diagnostics update. Windows may show an unknown-publisher warning. Website: https://waxvpn.pages.dev/.

## Startup issue and mitigation

A user reported that v1.0.0 opens a window and then closes. The cause on that machine has not yet been reproduced or identified. No claim is made that this build fixes that exact crash. v1.0.1 retained the same GUI startup path, but the user has not confirmed the issue on that version.

- Added bounded local `%APPDATA%\wax-vpn\startup.log` markers for graphics/app creation, settings/profile loading, tray setup, first/ten UI frames, explicit quit sources and normal event-loop return. The file is capped by replacement after 64 KiB; it contains no profile/account/settings values. Nothing is uploaded.
- Rust panic handling records the source location, not the panic payload, and shows a visible Windows error dialog. Returned window/graphics setup errors are logged and shown rather than disappearing silently. Native driver faults may bypass the Rust hook and still require Windows fault-module evidence.
- A Start-menu Safe graphics shortcut (or `"waX VPN.exe" --safe-graphics`) uses OpenGL as an alternative to the normal wgpu path. This is an explicit compatibility path, not a software-rendering guarantee or automatic crash recovery. Normal renderer and tunnel behavior are unchanged. If a normal instance remains running, quit it before using Safe graphics.
- Added runner startup checks for 15 s survival, visible rail, ten UI frames, no tunnel adapter and graceful quit in both normal and Safe graphics modes. Logs, screenshots, runner graphics details and matching Windows application events are retained as evidence. Runner success does not prove compatibility with every consumer GPU or non-elevated setup.

## Runtime dependency change

The Windows GUI and CLI now build with a static C runtime. The previous build imported VCRUNTIME140.dll and api-ms-win-crt-* DLLs. A packaging gate audits both executables and every bundled DLL for VC++/CRT, managed-runtime and directly imported shader/Vulkan redistributables. This is an import-level check, not a clean bare-Windows installation test. No .NET installation is required by the native app.

Windows system libraries and a compatible graphics driver are still needed. wgpu can try a dynamically loaded DXC compiler and then the Windows D3D compiler; a Vulkan loader is not guaranteed present. Missing or unsupported graphics setup is reported in the local log and visible error dialog. The explicit OpenGL alternative also needs a compatible driver. No bundled or separately installed VC++ redistributable is assumed by the new Rust executable imports; final exact audit results are below.

## Included v1.0.1 safety changes

CLI WARP Register/Generate reject occupied reserved profile paths before remote registration, and use exclusive exact-name creation to prevent replacement races. No replace option or live WARP calls. Failed profile creation retains the local registration for recovery. CI pins the unchanged official WireGuardNT 1.1 archive SHA-256 before extraction, with a deliberate mismatch test. Digest `dceb30a9bc4be48cce0f74160fc88a585a2c2627366e8f846fc6658f9038dace` independently matches [the vendor build recipe](https://github.com/WireGuard/wireguard-windows/blob/6ece77bc487c8aa697e3c092197621c4f3e5ccb8/build.bat).

No production tunnel routing, DNS, MTU, status/recovery or measurement-request changes. Profile/account crash-atomic writes and adapter-query uncertainty remain held follow-ups.

## Network observation and privacy limits

Public-IP, HTTPS latency and manual quick-speed checks remain off by default; the local adapter-counter graph remains on and makes no extra network requests. Optional checks contact the configured HTTPS service (Cloudflare by default), which sees your public IP. Endpoints, latency interval and separate toggles are editable in Features > Network measurements. Speed testing is manual: one 8 MB download and 2 MB upload plus protocol overhead, approximate single-request speed rather than line capacity. Latency includes DNS/TCP/TLS where needed, not ICMP ping. Automatic latency checks run only while Network is visible and connected. No signup or API key is needed for the defaults.

IP/results stay in memory, are not logged or persisted, and are excluded from local diagnostics. No analytics/result-upload API is called. Redirects are not followed. Requests have bounded timeouts; stale results are discarded after settings or route changes. An in-flight request can finish after its toggle is disabled, within its timeout. These are observations on the current OS route, not leak tests or proof that every destination uses the tunnel. Split routing can send another destination over a different route. An already-running tunnel has no original-IP baseline.

No kill switch is included. Traffic can use the normal connection after the tunnel drops. IPv6 leak protection and physical-network roaming are not claimed.

## Validation

Exact [v1.0.2 gate](https://github.com/vio137/wax-vpn/actions/runs/43056712982) passed build, runtime, clean-runner installer and publication for source `93528327cf4c1aff473bd198dcb7557467fdeb3a`. Windows tests: 104 passed, one manual external-service test ignored. No Rust warnings/errors in the exact build log.

- Both Rust executables' exact PE imports contain no VCRUNTIME140, api-ms-win-crt, other VC++/CRT redistributable or managed-runtime import. The bundled vendor DLL imports only KERNEL32 and ntdll. Direct/delay import audit passed for all three payload PE files; independently checked with objdump. Dynamic graphics loading and bare-image compatibility remain separate limits.
- Normal wgpu startup survived 15 seconds, reached ten UI frames, showed the rail, created no adapter and quit on request. The runner's Microsoft Hyper-V Video driver does not provide OpenGL 2.0. Safe graphics showed the expected visible error dialog and local log without creating an adapter. Rendering and dialog dismissal were explicitly skipped; that process was killed as test cleanup. No successful OpenGL rendering claim is made.
- CLI and UI-driven demo tunnel handshake, ping to the peer tunnel address, DNS apply/restore and configured IPv4 MTU passed. This does not prove general Internet routing. Split-route fixture passed. Adapter-loss recovery re-handshook after 6 seconds. Retry-disable held in 48 adapter-absence samples over 12 seconds; manual reconnect still worked. Original-process restore, close-to-tray, graceful quit/adapter removal, profile-read warnings and malformed/BOM settings checks passed. IPv6 leak protection was explicitly skipped.
- No new visible ConsoleWindowClass was observed in 427 GUI-connect samples at about 100 ms or fresh-install/upgrade/uninstall samples of 221/502/344 at requested 10 ms. Sampling can miss shorter flashes or other classes. Installer launch, same-version upgrade, installed GUI rail, quit and uninstall passed on a separate runner. Exact normal startup, error dialog and installed GUI pixels were inspected.

Installer: `waX-VPN-Setup-v1.0.2-x64.exe`, 11,466,250 bytes. Fresh download, release API digest and SHA256SUMS agree: `38df157cb730c5c0ce87e521abb20b26de5f255a9e2d45189b182bc32a4a056d`.

The user's other PC has not yet been retested with this installer. The confirmed previous VC++ dependency is removed; the exact reported crash remains unconfirmed.

Remaining limits: real Windows pre/post-connection public-IP observation through a production Internet tunnel; external-observer IPv6/DNS leak testing; consumer SmartScreen/interactive installer behavior; physical-network roaming; tray-menu Open/Quit in a real notification area; real active-tunnel Ctrl+C handling. Crash evidence is a three-second snapshot followed by explicit cleanup, not continuous or general crash-cleanup proof. Console sampling can miss shorter flashes and other console-host classes. Lower Network and expanded measurement settings were inspected in local native pixels, not exact Windows captures. Service availability and compatible custom endpoints are not guaranteed. The app and installer remain unsigned; the bundled vendor driver is separately signed.
