# WARP study and waX improvement checklist

Study target: official Windows release at https://1111-releases.cloudflareclient.com/win/latest, retrieved November 10, 2026. Redirect: 2026.8.2115.0. MSI SHA256: ab3138dbebdf132af076b5ebcb25b42345d29258acb76267c3bd1f8cfea57a53.

This is an independent behavior/structure study with REA 6.8.0 and Ghidra 12.1.4. No Cloudflare implementation, assets, branding or credentials are copied into waX. Static imports and strings are observations, not proof that a runtime path executes.

## Investigation

- [x] Acquire and fingerprint official MSI.
- [x] REA bounded desktop inventory: 601 entries, eight header-probed files. Extraction names are partly opaque, so this is not a complete installed-layout model.
- [x] REA native UI launcher overview: 485 procedures, 339 defined strings. Flutter/plugin references observed; this launcher alone is not the full UI implementation.
- [x] REA service imports: named-pipe/service APIs, route/address notifications, WFP transaction/filter APIs. These support architecture hypotheses, not complete feature reconstruction.
- [x] REA certificate identities: Cloudflare signer certificate embedded in service. This inspection does not validate Authenticode trust.
- [x] REA Rust metadata probe: compiler path marker found, no cargo-auditable dependency section. Dependencies remain unknown.
- [x] Reduced CLI overview/search/literal trace completed:31503procedures,13665defined strings, registration57matches/47references. Default analysis timed out after20minutes; completed pass disables Decompiler Switch Analysis. Switch recovery/complete calls remain unknown.
- [x] Resolve MSI File/Component tables: Flutter engine plus x64 ELF app.so (Dart AOT payload), separate service/CLI/diagnostics/DEX/updater. File sizes and SHA256 recorded locally. No assets copied.
- [ ] Trace Flutter settings/state candidates. Supplemental strings show reconnect/package-update/captive-portal/connectivity names, not runtime behavior.
- [x] Runtime CLI help/version on an isolated Windows runner: Authenticode valid Cloudflare, Inc., version2026.8.2115.0. No MSI install/service/registration/connection. One unsupported exploratory help command caused final harness exit2, not a signature failure; corrected rerun green42765965388. Full UI behavior still unknown.
- [x] Checkpoint findings reconcile static structure with signed help-only runtime and docs. Full GUI/network behavior reconstruction remains open.

## Independent waX priorities

| Priority | Item | Basis | State / boundary |
| --- | --- | --- | --- |
| P0 | Honor profile MTU | waX source audit found it parsed but unused | IPv4 1380 applied and checked on Windows; IPv6 branch not runtime-proven |
| P1 | Redacted local diagnostics/status export | Separate warp_diag binary plus official diagnostic-log documentation | Shared GUI/CLI allowlist; Windows31f09d0 gate42769693672 confirms generate/clear and fixed schema. Generated-report pixels inspected. Pre-report privacy-note width fix8863f19 inspected on Windows. Gate42771027828 first attempt failed fresh-install quit15second bound after launch PASS; uninstall not reached. Separate tunnel/GUI quit passed. Same-source failed-job rerun pending. User-triggered snapshot/copy/clear only, no profiles/logs/accounts/network calls |
| P1 | Clear tunnel state and actionable error categories | Separate service/CLI structure; detailed state documented in diagnostics | Prepared: distinguish adapter-ready/no handshake, recent peer handshake, older handshake; never assert Internet reachability. Recent-handshake pixels inspected and gates green168c994. No/older-handshake runtime pixels still pending |
| P2 | Network-change reconnect handling | Service imports NotifyRouteChange2 / NotifyUnicastIpAddressChange | Static capability observed; runtime semantics unknown. Bounded same-profile adapter-loss policy9aea9b3 passed targeted Windows gate42770444348: fresh handshake after4seconds, original GUI process alive. Retry bounds/flapping/cancellation have pure-policy tests, not full Windows failure-sequence proof. Physical roaming and route/address hooks remain design-only in NETWORK-RECOVERY-DESIGN.md |
| P2 | Optional background service | Distinct service and named-pipe imports | Design first; least-privilege IPC and install/uninstall work needed |
| P2 | Update verification and explicit install choice | Updater binary, launcher updater plugin/WinSparkle reference | No silent auto-update or new service |
| P2 | Connection/DNS mode clarity | Official consumer connection-preferences documentation | No claim that waX supports DNS-only/DoH/DoT modes |
| Excluded | Kill switch | Explicit product scope | Removed; do not restore |
| Excluded | Enterprise enrollment, posture, MASQUE/private APIs | Outside current scope | Do not implement or imitate private protocols |
| Excluded | Account signup / WARP registration | Current study scope | No signup; live registration remains unverified |

## Sources and scope

- https://github.com/morluto/rea (tool source; pinned npm 6.8.0)
- https://developers.cloudflare.com/warp-client/get-started/windows/ (consumer modes/preferences)
- https://developers.cloudflare.com/cloudflare-one/team-and-resources/devices/cloudflare-one-client/ (GUI, daemon, CLI and diagnostics roles)
- https://developers.cloudflare.com/cloudflare-one/team-and-resources/devices/cloudflare-one-client/troubleshooting/diagnostic-logs/ (diagnostic categories)
- https://github.com/NationalSecurityAgency/ghidra/releases/tag/Ghidra_12.1.4_build (native analysis engine)

Cloudflare One enterprise documentation is not interchangeable with consumer availability. No signup or terms acceptance was needed for static extraction/inspection. Latest stablev0.1.3 (41cbe58,run42768192198) includes MTU, diagnostics, state clarity and startup observer fixes. Releases remain unsigned, with no kill switch or IPv6 leak-protection claim.

## Package findings

Read-only MSI ServiceInstall rows declare the main CloudflareWARP service automatic and the updater demand-start, both LocalSystem. This describes installer configuration, not a tested service installation. Do not copy that privilege model into waX without a separate least-privilege design. The Flutter launcher is only412KB while app.so is10.6MB and the engine18.5MB; launcher inspection alone cannot establish UI behavior. The REA Dart AOT inspector accepts APK containers, not this Windows payload, so direct Dart reconstruction remains unsupported in this study.

## Observed CLI help surface

Publisher-verified official CLI help describes maintained connection intent; split tunnel host/IP routing and route dumps; aggregate/enhanced stats; DNS logging/fallback/families; key rotation; trusted-network disconnect; WireGuard/MASQUE preference; and tunnel/DNS/proxy modes. These are help descriptions, not verified networking behavior. waX currently uses WireGuardNT and profile DNS configuration, not Cloudflare's DNS proxy, MASQUE, enterprise policy or account services. Implementing labels without the corresponding behavior would be misleading.

## Follow-up source audit

GUI startup detected an already-running adapter but its stats thread exited because that tunnel had no owned handle in the GUI. Independent fix uses read-only status polling for the external tunnel, reflects stop without reconnecting a guessed profile, and uses fixed query errors. Targeted Windows verification green8b123c1/run42766598088: GUI detects externally owned tunnel stop without reconnect. This does not add a service or network probe.

## v0.1.3 checkpoint

Released https://github.com/vio137/wax-vpn/releases/tag/v0.1.3 after full manual gate. Fresh installer SHA25626b1ff1055362b5d98825df6ff7ac2f564400adab2231d0b38989e1875ac6668,10,922,699bytes. Connected/About/installed pixels inspected, all0.1.3. Crash snapshot before cleanup showed adapter gone,DNS none; IPv6 test skipped without proven baseline. Real tray menu remains manual-only. Reduced study evidence https://github.com/vio137/wax-vpn/actions/runs/42767639636 supports static MTU/PMTUD and staged connection/error categories, not runtime behavior or implementation copying.
