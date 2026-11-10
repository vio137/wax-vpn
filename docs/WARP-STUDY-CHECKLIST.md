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
- [ ] Deep CLI feature tracing and native relationships, in an isolated research workflow.
- [x] Resolve MSI File/Component tables: Flutter engine plus x64 ELF app.so (Dart AOT payload), separate service/CLI/diagnostics/DEX/updater. File sizes and SHA256 recorded locally. No assets copied.
- [ ] Trace Flutter settings/state candidates. Supplemental strings show reconnect/package-update/captive-portal/connectivity names, not runtime behavior.
- [ ] Runtime CLI help/version on an isolated Windows runner, validating publisher before execution. No MSI install/service/registration/connection. Full UI behavior still unknown.
- [ ] Reconcile static evidence, runtime observations and documentation into final findings.

## Independent waX priorities

| Priority | Item | Basis | State / boundary |
| --- | --- | --- | --- |
| P0 | Honor profile MTU | waX source audit found it parsed but unused | IPv4 1380 applied and checked on Windows; IPv6 branch not runtime-proven |
| P1 | Redacted local diagnostics/status export | Separate warp_diag binary plus official diagnostic-log documentation | CLI implementation in validation; allowlist only, no profiles/logs/accounts/network calls |
| P1 | Clear tunnel state and actionable error categories | Separate service/CLI structure; detailed state documented in diagnostics | Investigate exact state model before changes |
| P2 | Network-change reconnect handling | Service imports NotifyRouteChange2 / NotifyUnicastIpAddressChange | Static capability observed; runtime semantics unknown |
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

Cloudflare One enterprise documentation is not interchangeable with consumer availability. No signup or terms acceptance was needed for static extraction/inspection. Existing waX stable releases remain unsigned, with no kill switch or IPv6 leak-protection claim.

## Package findings

Read-only MSI ServiceInstall rows declare the main CloudflareWARP service automatic and the updater demand-start, both LocalSystem. This describes installer configuration, not a tested service installation. Do not copy that privilege model into waX without a separate least-privilege design. The Flutter launcher is only412KB while app.so is10.6MB and the engine18.5MB; launcher inspection alone cannot establish UI behavior. The REA Dart AOT inspector accepts APK containers, not this Windows payload, so direct Dart reconstruction remains unsupported in this study.
