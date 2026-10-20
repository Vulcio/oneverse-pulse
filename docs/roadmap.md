# Roadmap

Requirements: [#62](https://github.com/bricedupuy/Songverse/issues/62), [#247](https://github.com/bricedupuy/Songverse/issues/247). Related: Songverse #195 (offline screens), #196 (NDI), #13 (sync).

## M0 — Rust + local HDMI demo (initial scaffold)
- [x] Cargo workspace and Rust server
- [x] Local mock set and slide navigation API
- [x] Browser-rendered screen with keyboard navigation
- [ ] Validate cargo build/tests in CI and on target hardware
- [ ] Chromium kiosk and compositor startup under systemd

## M1 — Paired device
- [ ] Pairing and revocable device credentials
- [ ] Signed local identity and encrypted credential storage
- [ ] Health, diagnostics, hardware detection and last-sync status

## M2 — Offline presentation
- [ ] Download and verify sets, themes, fonts and media
- [ ] Replace mock renderer with actual Songverse screen frontend
- [ ] Persist presentation state and support keyboard/clicker
- [ ] Debian minimal image with automatic startup and recovery

## M3 — Local session continuity
- [ ] Implement compatible local sync and clock
- [ ] LAN discovery, authorized control and WAN failover
- [ ] Safe reconciliation on cloud reconnect

## M4 — Audio and production video
- [ ] Real-time audio engine, stems, click and cues
- [ ] Multichannel interface and latency calibration
- [ ] Independent NDI outputs, alpha and SDK compliance review
- [ ] Multi-screen orchestration, telemetry and secure updates
