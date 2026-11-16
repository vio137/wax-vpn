# waX VPN v1.0.31

Fix: the installer could freeze right after "Extract: wireguard.dll... 100%"
and then show "Installation Aborted - Setup was not completed successfully"
when cancelled. Reported on a user PC on 2026-11-16.

Root cause: after extracting its files, setup runs two hidden cleanup commands
from the bundled helper (wax-installer.exe): stop a running waX VPN app, then
clean up leftover tunnel state. Stopping the app was already time-bounded, but
the cleanup step called the WireGuard-NT driver, the DNS configuration API and
the Windows Firewall COM service with no time limit. On a PC where one of
those driver or service calls sticks (a leftover or mid-teardown tunnel
adapter, a wedged firewall or network service, or antivirus interference with
the freshly extracted helper), the cleanup never returned, the installer
waited on it forever, and the setup log showed nothing after the extract
lines. Cancelling the frozen setup produced "Installation Aborted".

Changes:

- Both cleanup commands now run under hard deadlines inside the helper:
  30 seconds for stopping the installed app, 45 seconds for leftover tunnel
  cleanup. A stuck call fails the step instead of freezing setup.
- Leftover-state cleanup is now best-effort for setup and uninstall: a failed
  or timed-out cleanup is noted and installation continues, since leftover
  tunnel state never blocks new files from being written.
- The setup log now narrates these steps ("Stopping any running waX VPN
  app...", "Cleaning up leftover tunnel state..."), so a future stall names
  the step it is stuck on.
- The app's own `wax disconnect` command gains the same 45-second bound, so a
  stuck driver call can no longer hang an interactive terminal either.

No VPN, UI, profile, firewall or networking behavior changed apart from the
bounded cleanup described above. The build remains unsigned, so SmartScreen
and some antivirus products may still warn; verify the SHA-256 in
SHA256SUMS-v1.0.31.txt against the downloaded file.

## Compatibility and limits

CPU software rendering through Windows GDI remains the default. GPU rendering
is opt-in. No separate VC++/.NET/WebView2 installation is required, but Windows
system libraries, administrator consent, networking and a valid profile still
are. The bundled WireGuardNT driver is signed by its vendor. The app and
installer are unsigned and Windows may show an unknown-publisher warning.

DNS uses the native Windows API when available. netsh remains only for older
Windows builds without that API; the API-absent fallback is untested. IPv6
DNS/MTU settings checks do not prove IPv6 traffic routing or leak protection.
No kill switch is provided. Traffic can use the normal connection when the
tunnel is off. Live WARP registration is not verified.

The release gate runs on a hosted Windows Server 2025 runner with installed
runtimes and a virtual display, not a clean Windows 10/11 PC. The gate covers
fresh install, running-app upgrade, uninstall and bounded-cleanup fixtures; it
cannot reproduce every consumer-PC driver or antivirus state, including the
exact stuck state reported for this fix. The 30s/45s deadlines bound the
failure regardless of its cause.

## Release verification

Hosted-gate results (stable run 43826517343, Windows Server 2025, November
16, 2026): build passed; workspace tests passed, including the new cleanup
deadline tests; demo-tunnel and GUI verification passed; fresh-runner install,
running-app upgrade, quit and uninstall passed; the bounded-cleanup fixtures
passed; the published installer SHA-256 matches SHA256SUMS-v1.0.31.txt
(93a0cc9b2de0646a2e57e92a3dc2d999db151d13502b510d3db7a496fa094afb).

