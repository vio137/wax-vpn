# Stable release preparation

Stable releases are currently unsigned by explicit maintainer approval while a certificate is pending. Do not call a build signed or release it merely because a job is green.

## Release gate

Publication waits for build/unit checks, scoped tunnel/GUI verification, and a separate fresh Windows runner installer launch/uninstall check. Hosted runners are not consumer Windows or SmartScreen-reputation tests.

## SignPath Foundation hook

The user selected the SignPath Foundation route. Application/setup is handled separately. No signing request is sent by default. Set WAX_SIGNPATH_ENABLED=true only after approved service setup, with SIGNPATH_API_TOKEN in repository secrets and organization/project/policy/artifact-configuration slugs in repository variables. These are unset today.

The disabled hook uploads the unsigned installer as a GitHub artifact, submits its artifact ID, waits for signing, downloads the result, validates the outer Authenticode signature, and hashes the signed bytes before release. Failure blocks publication, with no unsigned fallback when enabled.

Before activation, configure and verify recursive signing of the NSIS embedded application, CLI and uninstaller. The current hook only validates the outer installer signature. Outer signing alone is not proof of signed installed executables. The vendor WireGuard DLL must remain unchanged. A signed-artifact installer smoke check is also needed once signing is configured; the current installer smoke job checks the unsigned input. Newly signed software can still show SmartScreen warnings.

Source: https://docs.signpath.io/trusted-build-systems/github

## Stable version path

Use a reviewed SemVer tag such as v0.1.0, not a CI run counter. Derive displayed version, installer filename, uninstall registry version and release tag from that tag. Publish without prerelease flag only after required checks and signing decisions. Existing preview tags remain immutable. Automatic builds validate and retain artifacts without publication; stable publication requires an explicit manual dispatch. The manual stable_version input enforces exact workspace SemVer and requires either SignPath enabled or the explicit unsigned_stable manual input. The maintainer approved release without certificate while SignPath is pending. An empty input keeps previews. The manual path has produced stablev0.1.0 throughv0.1.2; v0.1.3 is in preparation.

## Remaining gaps

- No kill switch is included; traffic is not blocked if the tunnel drops
- IPv6 leak probe with working baseline, not absence of IPv6 connectivity
- Real-desktop tray actions and consumer installer/signature checks
- WARP registration remains explicit-consent gated
