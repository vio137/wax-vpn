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
| P1 | Redacted local diagnostics/status export | Separate warp_diag binary plus official diagnostic-log documentation | Shared GUI/CLI allowlist; Windows31f09d0 gate42769693672 confirms generate/clear and fixed schema. Generated-report pixels inspected. Pre-report privacy-note width fix8863f19 inspected on Windows. Gate42771027828 first attempt failed fresh-install quit15second bound after launch PASS; uninstall not reached. Separate tunnel/GUI quit passed. Same-source failed-job rerun attempt2 passed install, quit and uninstall (exact artifact13741628083). First intermittent quit failure remains unresolved; prepared fixed-stage logs will help diagnose recurrence. User-triggered snapshot/copy/clear only, no profiles/logs/accounts/network calls |
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

Cloudflare One enterprise documentation is not interchangeable with consumer availability. No signup or terms acceptance was needed for static extraction/inspection. Latest stable v0.1.7 (848a68a, run42791204943) includes MTU, diagnostics, state clarity and startup observer fixes. Releases remain unsigned, with no kill switch or IPv6 leak-protection claim.

## Package findings

Read-only MSI ServiceInstall rows declare the main CloudflareWARP service automatic and the updater demand-start, both LocalSystem. This describes installer configuration, not a tested service installation. Do not copy that privilege model into waX without a separate least-privilege design. The Flutter launcher is only412KB while app.so is10.6MB and the engine18.5MB; launcher inspection alone cannot establish UI behavior. The REA Dart AOT inspector accepts APK containers, not this Windows payload, so direct Dart reconstruction remains unsupported in this study.

## Observed CLI help surface

Publisher-verified official CLI help describes maintained connection intent; split tunnel host/IP routing and route dumps; aggregate/enhanced stats; DNS logging/fallback/families; key rotation; trusted-network disconnect; WireGuard/MASQUE preference; and tunnel/DNS/proxy modes. These are help descriptions, not verified networking behavior. waX currently uses WireGuardNT and profile DNS configuration, not Cloudflare's DNS proxy, MASQUE, enterprise policy or account services. Implementing labels without the corresponding behavior would be misleading.

## Follow-up source audit

GUI startup detected an already-running adapter but its stats thread exited because that tunnel had no owned handle in the GUI. Independent fix uses read-only status polling for the external tunnel, reflects stop without reconnecting a guessed profile, and uses fixed query errors. Targeted Windows verification green8b123c1/run42766598088: GUI detects externally owned tunnel stop without reconnect. This does not add a service or network probe.

## v0.1.3 checkpoint

Released https://github.com/vio137/wax-vpn/releases/tag/v0.1.3 after full manual gate. Fresh installer SHA25626b1ff1055362b5d98825df6ff7ac2f564400adab2231d0b38989e1875ac6668,10,922,699bytes. Connected/About/installed pixels inspected, all0.1.3. Crash snapshot before cleanup showed adapter gone,DNS none; IPv6 test skipped without proven baseline. Real tray menu remains manual-only. Reduced study evidence https://github.com/vio137/wax-vpn/actions/runs/42767639636 supports static MTU/PMTUD and staged connection/error categories, not runtime behavior or implementation copying.

## v0.1.4 checkpoint

Released https://github.com/vio137/wax-vpn/releases/tag/v0.1.4 sourcee84e2c3 after full stable gate42773347398. SHA256090a469c7f81ad483a4cf1218a1f389fbb78149c576aaa535935a6398f4a97dc,10,942,141bytes. Stable Diagnostics/Connected/installed pixels inspected. Actual adapter-loss recovery4seconds/original process alive; fresh install/quit/uninstall pass; crash snapshot3seconds adaptergone/DNSnone/rules0 before cleanup, cleanup pass. IPv6SKIP no baseline/no kill switch. Earlier fresh-install quit timeout remains unresolved intermittent history. The Connect stats/state layout feedback is addressed separately in1a39ed3, pending Windows multi-state/multi-size pixel checks forv0.1.5.


## v0.1.5 layout preparation

Follow-up 2ab0d2a passed Windows gate42775144956. Session/Received/Sent label bounds match exactly across Disconnected/Connecting/Connected at760x540,880x600and960x700. Actual880and960Connecting pixels inspected. Smallest Connecting screenshot still shows the previous Disconnected paint despite UIA state, so it is not counted as Connecting pixel proof. Final gate increases compositor wait to500ms and reserves the optional16pxprofile hint too. Exact-version release verification is pending.

## Astryx redesign request

User requested https://github.com/meta/astryx on10November13:02IST. Verified repository and MIT license; installed CLI/core/neutral-theme0.7.11 in a separate design workspace. Astryx components require React19+/StyleX, so they are not directly native Rust/egui controls. Use actual Astryx tooling/components as the redesign reference and independently implement native equivalents without silently introducing a web runtime. This full redesign is separate from the v0.1.5 alignment checkpoint. No new service, signup, spending or terms acceptance.


## v0.1.5 released

Exact source cd614c4, full gate42776110039 passed. Fresh installer10,943,511bytes, SHA25600ac3ccb467cabcc606d36c0cfaa48eb77f1a4c6ec5361159b5d3fc32a14f6ed. Actual stable pixels inspected at all three states and760x540/880x600/960x700; the final smallest Connecting frame is real. UIA anchors match across states. No roaming/IPv6/Internet claim follows. Parent received stable handoff13:11IST.

## Astryx native preview

Source433f867/full manual gate42777283654 passed. Actual Windows pages and all state/size captures were inspected before the exactv0.1.6stable gate. The separate React reference is illustrative only and labeled as not a live tunnel. No new product runtime or service.

### v0.1.6 stable checkpoint
- Source cf976cf; exact full gate 42778052338 passed build, verification, fresh-runner install/uninstall and publish.
- All three actual Windows states at760/880/960 inspected; counter anchors stable. Features toggle contrast and all page/diagnostics/installed images inspected.
- Fresh released installer SHA256 e7d3ee08b346044805f128b989523ca434b883588ae0fda894b39850798f8a4e, 10,943,260 bytes.
- Adapter-loss recovery5seconds with original process. Internet/IPv6/physical-roaming/real-tray limits and unresolved earlier intermittent quit retained in release notes.

## v0.1.7 maintenance checkpoint

Exact source848a68a/full stable gate42791204943 passed build/runtime/installer/publish. Fresh installer10,944,296bytesSHA2569ff8c7598442a69a34cfdfd9ca052075017e3434c608a80ade7e9a502e3c991d. Stable Connected/installed/diagnostics pixels inspected. Native adapter-presence query replaces one subprocess without skipping DNS or narrow legacy-rule cleanup. Recovery-disable cancels in-flight automatic attempts via generation invalidation (policy-tested, Windows failure sequence unproven). Route/address/interface groundwork remains test-only, not physical roaming. Crash3sadaptergoneDNSnone/rules0beforecleanup;cleanupPASS;IPv6SKIP;realtraymanual. Earlier intermittent15squit failure remains undiagnosed; stable installed cleanup4654ms/nativeabsence2ms.
