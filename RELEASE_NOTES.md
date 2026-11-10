# waX VPN v0.1.7

Unsigned Windows x64 maintenance checkpoint. Windows may show an unknown-publisher warning. The official WireGuardNT driver is independently signed by its vendor. No kill switch: traffic can use the normal connection if the tunnel drops.

## Changes

- Replace the PowerShell adapter-presence probe with a read-only native Windows interface-table query. Only a successful query establishes absence; errors remain errors. DNS resets and narrow old-preview-rule cleanup are unchanged.
- Cancel a running automatic recovery attempt when Auto-reconnect is disabled, using the existing connection generation before queued results are consumed. Established and manual connections stay unchanged.
- Add fixed local cleanup substages with elapsed timings to installer evidence. No profile, endpoint or address values are added.
- Add test-only local network-change groundwork: notification lifetime/cancellation, burst filtering and private route/address/interface fingerprints. This is not wired to production reconnect and is not roaming support.

## Verification

Full manual dd34010 checkpoint42787586767 passed Windows runtime, extended crash cleanup and fresh install/quit/uninstall. Later source429a024 push42790476717 passed Windows build/tests, tunnel/GUI and fresh installer. Exact-version stable source848a68a/run42791204943 passed all jobs, including extended crash cleanup and fresh install/quit/uninstall. Actual stable Connected, installed and generated-diagnostics pixels inspected. Fresh released installer10,944,296bytes, SHA2569ff8c7598442a69a34cfdfd9ca052075017e3434c608a80ade7e9a502e3c991d. Stable installed cleanup4654ms, native absence2ms; adapter-loss recovery5seconds/originalprocess. Native adapter absence checks measured1-2ms instead of an earlier3.93second subprocess, but remaining legacy-rule cleanup varies. This is not a general performance or timeout-fix guarantee.

## Scope and limits

An earlier2542baf fresh installer timed out after15seconds waiting for quit. Fixed logs show the signal was consumed and leftover cleanup began with a PowerShell child still active. Later passes do not diagnose the first failure. The timeout was not widened and cleanup was not skipped. Fixed substage evidence stays in place.

Peer handshake and packets to the demo server tunnel-side address do not prove Internet reachability. Adapter-loss recovery does not establish physical-network roaming. Retry-exhaustion and recovery-disable cancellation have policy tests, not a full Windows failure-sequence demonstration. Crash evidence is a3second snapshot, not continuous leak observation. IPv6 was skipped without a proven runner baseline. Real notification-area tray menu remains manual-only.

No new product network service, Internet probe, signup, WARP registration, kill switch or IPv6 leak-protection claim. The Astryx-informed interface remains native Rust/egui, with its MIT notice retained in the installer. No React, JavaScript or WebView runtime is embedded. No Cloudflare implementation, assets, credentials or keys are copied.
