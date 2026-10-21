# Roadmap

Requirements: [#62](https://github.com/Vulcio/Oneverse/issues/62) and [#247](https://github.com/Vulcio/Oneverse/issues/247). Related Songverse issues #195, #196 and #13.

## M0 — Rust and HDMI vertical slice
- [x] Cargo workspace and local Rust demo server
- [x] Mock set, navigation and browser display
- [x] Identify upstream Songverse `/screen` and `ScreenStage` renderer
- [x] Add Debian Cage/Chromium kiosk launcher and systemd unit
- [ ] Validate CI build and tests
- [ ] Validate dedicated-VT kiosk startup on target Debian mini PC
- [ ] Verify screen pairing, theme changes and real HDMI rendering

## M1 — Device integration
- [ ] Device pairing and revocable identity managed by Rust
- [ ] Health, hardware and storage diagnostics
- [ ] Browser process supervisor, failure recovery and sealed credentials
- [ ] Harden and automate reproducible Debian appliance image

## M2 — Actual offline screen
- [x] Implement SHA-256-verified read-only local frontend bundle serving
- [ ] Package upstream ScreenStage web bundle with compatible `@oneverse/core`
- [ ] Bundle fonts, images, video and required theme assets
- [ ] Local authenticated API for cached screen state and controls
- [ ] Atomic set/media downloads, checksums and disk policies
- [ ] Persist screen and presentation state through restart

## M3 — Resilient synchronized control
- [ ] Compatible Rust sync protocol and stable clock
- [ ] LAN discovery and authorized local controls
- [ ] Deterministic WAN failover and reconnect reconciliation

## M4 — Audio and NDI
- [ ] Native real-time stems/click/cues and routing
- [ ] Clock calibration and multidevice synchronization
- [ ] Independent captured-frame output with NDI SDK and alpha validation
- [ ] Multi-screen management, diagnostics and secure updates
