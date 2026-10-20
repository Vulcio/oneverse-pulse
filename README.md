# Songverse Player

Rust-first Debian mini-PC appliance for Songverse stage screens and synchronized playback.

Design inputs: [Songverse #62](https://github.com/bricedupuy/Songverse/issues/62) (headless audio/cues) and [#247](https://github.com/bricedupuy/Songverse/issues/247) (mini PC, HDMI/NDI, offline LAN fallback).

## HDMI milestone: real Songverse screen

The **real** Songverse HTML/CSS/JS screen is already implemented upstream in [Songverse `/screen`](https://github.com/bricedupuy/Songverse/blob/main/apps/web/src/routes/screen.tsx). On Debian, the kiosk can launch Chromium in the Cage Wayland compositor at `https://songverse.one/screen`, so pairing, themes and presentation behavior remain in the existing application.

**This is currently an online integration:** kiosk launch files are committed but have not been validated on a physical mini PC. The screen is *not* bundled for offline usage yet.

See [renderer integration](docs/renderer-integration.md) and [Debian setup](appliance/debian/README.md).

## Rust development demo

A separate mock set and local screen are useful for Rust orchestration development (they are **not** the upstream Songverse renderer).

```sh
cargo run -p songverse-player
# http://127.0.0.1:8787/screen
curl http://127.0.0.1:8787/health
curl -X POST http://127.0.0.1:8787/api/next
cargo test --workspace
```

## Repository layout

- `crates/core`: state model and navigation functions
- `crates/player`: loopback-only Rust/Axum demo runtime
- `web/renderer`: mock test presentation screen
- `appliance/kiosk`: Cage + Chromium launch script and configuration
- `appliance/systemd`: player and kiosk services
- `appliance/debian`: manual setup guidance; image builder pending
- `docs`: architecture, renderer integration and roadmap

## Status and security

No production audio, NDI, offline caching, LAN sync/failover or reproducible Debian image yet. The mock API has no authentication and is deliberately bound to loopback. The upstream browser screen retains its pairing token in a persistent Chromium profile, requiring device access controls.

Read [architecture](docs/architecture.md) and [roadmap](docs/roadmap.md).

## Licensing

Rust workspace metadata is provisional; review final application licensing and third-party NDI SDK distribution terms before release.
