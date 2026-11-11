# waX VPN 1.0 review

Review scope: all six Rust crates, Windows-only modules, installer, build scripts and workflows, current README/release documentation, and the separate illustrative Astryx reference. Expected use is one desktop user with GUI/CLI processes sharing local profile files, not a multi-user network service. This is source review plus the checks named below, not a penetration test or proof of every runtime path.

## Shipped changes selected from the review

- CLI `connect`: register a native Windows Ctrl+C/Ctrl+Break handler before creating a tunnel. The callback sets an atomic stop flag; the existing wait loop reaches the existing shutdown path. Enter/EOF remains an exit. Registration failure stops before tunnel creation. Closing a console, logoff and Windows shutdown are not advertised as graceful cleanup paths.
- CLI WARP unregister: a failed remote delete now returns failure and keeps the local account/profile available for retry. No account registration or live WARP API test was performed.
- Installer: delete the owned `LICENSE-astryx.txt` file during uninstall. The clean-runner check now requires that file and the installation directory to be gone. User profiles/settings remain untouched.
- Profile loading: cache each case-insensitive sorting key once. Name order and parsing/storage behavior stay the same.
- Visible steady-state periodic polling: 250ms instead of 33ms; connecting/disconnecting stays 33ms and hidden stays 1000ms. egui input and animation can request earlier frames. This is a polling policy change, not a measured CPU/frame-rate claim.
- Remove an unnecessary per-frame cleanup-message clone.
- Network page: consistent full-width panels, received/sent color legends, numeric chart scales with Mbps/ms units, and a direct Measurement settings button that opens the expanded settings section. Ctrl+6 opens Network, leaving Ctrl+1..5 unchanged. Hover hints show the shortcuts.
- Delete the unused, uncompiled Gargantua visual module. It was not part of the binary, so its deletion is a maintenance reduction, not a binary-size or runtime-speed improvement.

Production tunnel routing, DNS, MTU, adapter-loss recovery, measurement requests/destinations/sizes/timeouts and stored settings schema are unchanged. External measurement defaults remain off. Local throughput alone defaults on.

## Review findings intentionally not changed

- Adapter open/state errors can be interpreted as absent/down by existing tunnel-status code. This can make uncertainty look like adapter loss. Changing this also changes recovery behavior; held out of 1.0 pending a separate design and Windows evidence.
- Existing explicit profile saves and WARP-account saves use direct writes. Import uses exclusive creation; settings already use same-directory replacement. Crash-safe replacement for other secrets/profile updates needs a small independent storage change with failure tests, not an untested claim of durability.
- CLI WARP registration/profile generation can overwrite the reserved local profile name. GUI registration/restore checks that name first. Aligning these write semantics remains a follow-up; no live WARP operation was performed for this review.
- Read-only network-change subscription/comparison groundwork remains test-only. Physical-network roaming is not wired.
- Existing tunnel cleanup rollback and legacy-rule removal need separate network-behavior proposals before alteration. No broad cleanup or security weakening was added.
- The CI downloads official WireGuardNT over HTTPS but does not pin a separate checksum for that upstream artifact. Dependency-update and supply-chain hardening remain follow-up work; this release does not claim a security audit.

## Validation boundaries

Fresh local startup under an empty config directory showed Public IP, latency and manual speed OFF, local throughput ON. The empty directory still had no settings file after viewing Network and expanded settings. A separate explicit all-enabled local fixture was used to inspect the other panels while disconnected. No external measurement was run in that visual check.

Actual local 880x600 native pixels were inspected for Network top/lower cards, direct expanded settings and the Ctrl+6 route. Windows candidate and exact release checks must establish their own results; local native pixels do not prove Windows networking.

The Windows console callback has a focused test for Ctrl+C/Break versus unhandled close/logoff/shutdown codes. That test alone does not demonstrate a real console interrupt during an active tunnel. No live WARP API calls or registration were used for the unregister change.

Remaining product limits: no kill switch; normal traffic can continue after a drop; no external-observer IPv6/DNS leak-protection claim; no production Internet-tunnel public-IP-before/after proof; real Windows tray-menu and consumer SmartScreen/interactive installer checks remain separate. Prior intermittent quit and PowerShell-host failures were not diagnosed by later passing jobs. Crash evidence is a three-second snapshot followed by cleanup, not continuous crash-cleanup proof. Console-window sampling can miss shorter windows or other classes.
