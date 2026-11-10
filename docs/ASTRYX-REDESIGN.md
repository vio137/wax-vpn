# Astryx-informed native redesign

Source: https://github.com/meta/astryx
Study/install: 10 November 2026. CLI/core/theme-neutral version 0.7.11, MIT.

Astryx ships React 19+/StyleX components, not native Rust controls. A separate local reference uses its actual Theme, Button, Heading, Text, StatusDot and Divider. The shipped Windows app remains Rust/egui: no React, WebView, JavaScript or network service is added. This is a native design adaptation, not a claim to run Astryx components in egui.

Design mapping:

- Frame-first: 176px text sidebar, reduced to 152px below840px; one capped480pxConnect region. Other pages fill the remaining region and scroll.
- Neutral body/surface layers, semantic primary/secondary text and status colors. Keep one primary action per region.
- Four-pixel spacing rhythm. Profile facts grouped as a row; counters use three equal centered columns.
- Stable slots for detail, profile hint, connection stage and evidence. No status animation moves surrounding content.
- Short color/surface changes. Reduced-motion setting keeps the tab transition immediate. No continuous decorative starfield.
- Existing network, profile, diagnostics, recovery, registration consent and cleanup behavior is retained. No signup is performed by this work.

Validation before release: local pixels at760x540/880x600/960x700; actual Windows state and page pixels; focused tunnel/cleanup/installer gate. Visual reference values are illustrative and explicitly labeled, never runtime evidence.

MIT notice is retained in docs/vendor-licenses/astryx-MIT.txt. No Cloudflare assets or implementation are used.
