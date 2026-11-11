# Stable release preparation

Current builds are unsigned. Windows SmartScreen may show an unknown-publisher warning. Do not call a build signed or release it merely because a job is green.

## Release gate

Publication waits for build/unit checks, scoped tunnel/GUI verification, and a separate fresh Windows runner installer launch/uninstall check. Hosted runners are not consumer Windows or SmartScreen-reputation tests.

## Stable version path

Use a reviewed SemVer tag, not a CI run counter. The workspace version, displayed version, installer filename, uninstall registry version and release tag must agree. Existing preview tags stay unchanged. Automatic builds validate and retain artifacts without publication; stable publication requires an explicit manual dispatch. The stable_version input must match the workspace SemVer. The unsigned_stable input explicitly selects an unsigned release. An empty stable_version input keeps previews. Verify the exact release build, runtime evidence, installer checksum and release notes before announcing it.

## Remaining gaps

- No kill switch is included; traffic is not blocked if the tunnel drops
- IPv6 leak probe with working baseline, not absence of IPv6 connectivity
- Real-desktop tray actions and consumer installer/SmartScreen checks
- WARP registration remains explicit-consent gated
