# Songverse Player

Rust-first Debian mini-PC appliance for Songverse stage screens and synchronized playback.

Design inputs: [Songverse #62](https://github.com/bricedupuy/Songverse/issues/62) (headless audio/cues) and [#247](https://github.com/bricedupuy/Songverse/issues/247) (mini PC, HDMI/NDI, offline LAN fallback).

## HDMI milestone: real Songverse screen

The **real** Songverse HTML/CSS/JS screen is already implemented upstream in [Songverse `/screen`](https://github.com/bricedupuy/Songverse/blob/main/apps/web/src/routes/screen.tsx). On Debian, the kiosk can launch Chromium in the Cage Wayland compositor at `https://songverse.one/screen`, so pairing, themes and presentation behavior remain in the existing application.

**This is currently an online integration:** kiosk launch files are committed but have not been validated on a physical mini PC. The screen is *not* bundled for offline usage yet.

See [renderer integration](docs/renderer-integration.md) and [Debian setup](appliance/debian/README.md).

## Verified local screen bundle (infrastructure)

The Rust daemon can verify a SHA-256 manifest for an externally built frontend and serve `index.html` at `http://127.0.0.1:8787/offline/screen`. Run `python3 tools/make-screen-manifest.py /path/to/bundle --revision <songverse-commit>` and set `SONGVERSE_BUNDLE_DIR=/path/to/bundle` before launching the daemon. See [offline bundle design](docs/offline-bundle.md).

**Important:** the actual upstream Songverse frontend has **not** been exported into this bundle format yet; disconnected rendering and device sync remain incomplete. A manifest detects file modification but does not authenticate its publisher.

## Real Songverse ScreenStage offline-bundle demo

A **client-only build of the actual Songverse ScreenStage** is now defined upstream in [Songverse `apps/web/src/player-screen/`](https://github.com/bricedupuy/Songverse/tree/main/apps/web/src/player-screen). Build it from the main Songverse workspace:

```sh
cd ../Songverse
pnpm --filter @songverse/web build:player-screen
cd ../songverse-player
REV=$(git -C ../Songverse rev-parse HEAD)
sh tools/package-upstream-screen.sh ../Songverse "/tmp/songverse-screen-$REV" "$REV"
SONGVERSE_BUNDLE_DIR="/tmp/songverse-screen-$REV" cargo run -p songverse-player
```

Open `http://127.0.0.1:8787/offline/screen` to render Rust mock presentation state through the **actual upstream React stage**. See [frontend build integration](docs/frontend-build.md).

This has **not yet been build-verified**, and it is not the complete offline paired Songverse screen. Real cached song/set/theme data and synchronization are separate milestones.

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
- `tools`: verified bundle manifest generator
- `docs`: architecture, renderer integration, offline bundle contract and roadmap

## Status and security

No production audio, NDI, offline caching, LAN sync/failover or reproducible Debian image yet. The mock API has no authentication and is deliberately bound to loopback. The upstream browser screen retains its pairing token in a persistent Chromium profile, requiring device access controls.

Read [architecture](docs/architecture.md) and [roadmap](docs/roadmap.md).

## Licensing

Rust workspace metadata is provisional; review final application licensing and third-party NDI SDK distribution terms before release.
