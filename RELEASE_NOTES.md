# waX VPN v1.0.23

Dependency review and one scoped patch update: crc32fast 1.5.4 -> 1.5.5,
used by image/PNG decoding through eframe. No Windows x64 speedup is claimed.
No networking/TLS/certificate package was updated.

cargo-audit 0.23.0 scanned 571 lockfile dependencies against RustSec DB commit
fd6f39814c4d3c359fdbe223d0280dec4933f78f (2026-11-11). It reported zero
vulnerability entries but six warnings, unchanged after the update: glib0.18.5
unsoundness and five unmaintained crates. None appears in the default Windows
x64 target dependency graph; all remain in the cross-platform lockfile.
This is not a clean-security claim. See docs/dependencies/v1.0.23-review.md
and raw before/after reports for warnings, deferred updates and scan limits.
No tunnel, DNS, firewall or service behavior changed.

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
runtimes and a virtual display, not a clean Windows 10/11 PC. It does not prove
the reported consumer-PC crash is fixed. A peer handshake and tunnel-address
ping do not prove Internet reachability. Tray-menu clicks need real Windows
review. Runner OpenGL rendering/dismissal remain unverified. Console-window
sampling can miss a shorter flash.

## Release verification

Local workspace tests passed after the patch: 104 passed, one manual test ignored.
Preview: https://github.com/vio137/wax-vpn/actions/runs/43164420215
Source: `17920a2c1a2bb29cdcfd808795f0d7bf77485d23`.
All three execution jobs passed. Actual Windows proof passed prior profile,
summary, demo connection/recovery and installer gates. Installed-GUI and summary
pixels were inspected.

Stable: https://github.com/vio137/wax-vpn/actions/runs/43166024496
Source: `1069d029c4864fd8441a3c1432c635415bf66dcb`. All four jobs passed.
Actual artifacts passed demo registration, peer handshake/tunnel-address ping,
DNS/MTU restore, split-route checks, recovery after 3 seconds, recovery-disable
with 48 samples over 12 seconds, manual reconnect, original HWND/PID restore,
and graceful quit. No console was observed in 327 samples at about 100 ms;
shorter flashes can be missed. All profile and key-free summary checks passed.
Installed-GUI and summary pixels were inspected. Fresh-runner installer passed.

Published installer (11,711,595 bytes) was downloaded and independently hashed;
it matches SHA256SUMS-v1.0.23.txt:
`32ae6de02ed82d29e350f0cc7339b7337065c51728a8c6617e842611ae17f94f`.

Advisory scan covers known RustSec crate-version advisories,
not custom/vendored code, driver or installer binaries, runtime configuration,
exploitability or unknown issues.
