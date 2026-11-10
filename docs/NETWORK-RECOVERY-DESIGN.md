# Network-change recovery design

Status: bounded adapter-loss retry policy passed targeted Windows verification on9aea9b3, run42770444348. Route/address notification callbacks and physical-network recovery remain design only.

## Evidence and boundaries

The official Windows package imports route/address notification functions. That is a capability observation, not evidence of Cloudflare's reconnect algorithm. waX must implement its own behavior.

Microsoft documents `NotifyRouteChange2` and `NotifyUnicastIpAddressChange` as local route/address change subscriptions. Initial notifications are registration confirmation, not a network change. Callback rows are incomplete and owned by Windows. Cancellation from the callback, or while holding a resource the callback needs, can deadlock.

Sources checked November 10, 2026:
- https://learn.microsoft.com/en-us/windows/win32/api/netioapi/nf-netioapi-notifyroutechange2
- https://learn.microsoft.com/en-us/windows/win32/api/netioapi/nf-netioapi-notifyunicastipaddresschange

## Proposed behavior

1. Observe only while this GUI owns a tunnel and automatic recovery is enabled. An externally owned adapter stays read-only. No profile is guessed.
2. The Windows callback only copies the minimum stable identity needed for filtering and sets an atomic change flag. It never calls netsh, takes the tunnel mutex, resolves endpoints, logs addresses, or reconnects.
3. Ignore initial notifications and events from the waX adapter itself. Coalesce physical-interface events for a two-second quiet period, with a ten-second maximum burst window. This prevents an endless quiet-period extension on noisy networks.
4. Re-read local underlay route/interface state outside the callback. An event alone does not establish a broken tunnel. If no useful local change is established, keep the tunnel and show no alarm. A silent peer alone is not a reconnect trigger.
5. If recovery is needed, use the profile snapshot that created the owned tunnel, not the current list selection. Keep profile identifiers and addresses out of diagnostics. Re-resolving that configured endpoint is permitted tunnel setup, not an added reachability service.
6. Limit a recovery episode to three attempts, delayed 2, 4 and 8 seconds. A new UI connection intent gets a new generation; stale callbacks, timers and worker results cannot revive an old session. User disconnect, quit, or recovery-disable cancels the episode.
7. Do not reset the retry budget on adapter creation alone. A short-lived adapter must not create an infinite recovery loop. Reset after a stable session window or explicit user connection action, with that policy covered by deterministic tests.
8. Registration failure leaves normal tunnel use available. Report a fixed local code, not raw system text. No service installation, firewall rule, Internet probe, WARP registration, or upload is introduced.
9. On shutdown, cancel notifications from the worker outside the callback and outside any mutex it could require. Keep callback context alive until cancellation completes. Explicitly release both handles, including partial-registration failure.

## Implementation sequence

- First add and test a pure recovery policy: coalescing, bounded delay, cancellation generations, stable-session budget reset, own-adapter filtering, external-owner prohibition.
- Then add the Windows notification owner behind a small local API with explicit shutdown.
- Then integrate into the GUI using the existing event loop. Keep route handling and profile selection separate.
- Targeted Windows checks: initial notification does not reconnect; own adapter connect/disconnect does not recurse; burst coalesces; external tunnel remains observer-only; disconnect cancels a pending timer; notifications stop before callback context is freed.
- Verify actual physical-network changes before calling roaming support verified. A synthetic event test is policy evidence only. No IPv6 leak-protection or Internet-routing claim follows from this work.

## Existing source risks to address first

At source31f09d0, adapter-loss recovery called the next attempt immediately and reset its attempt counter when an adapter was created. The9aea9b3 change uses2/4/8-second delays, retains the starting profile plus split exclusions, tags worker events with a connection generation, cancels pending retries on disconnect/disable/quit, and requires60seconds up to reset the budget. Four pure-policy tests cover timing, flapping, stale generations and external-owner/disabled limits. Targeted Windows adapter-loss verification passed: a fresh handshake after4seconds with original GUI process alive. The multi-failure retry limit and cancel behavior still have policy-only evidence, not a full Windows failure-sequence demonstration. None of this establishes physical-network roaming support.

## Separate installer checkpoint

Width-only diagnostics source8863f19/run42771027828 passed the tunnel/GUI/recovery job, but its first fresh-install job did not quit within15seconds after GUI launch. Uninstall was not reached. The same-source job is being rerun; do not treat this source as fully gate-green. This is separate from physical-network recovery.
