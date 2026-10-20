# End-to-end standalone renderer development path

The source of the real renderer remains [Songverse](https://github.com/bricedupuy/Songverse).

## Build the real ScreenStage frontend

In a checked-out, dependency-installed Songverse workspace:

```sh
cd apps/web
vp build --config vite.player.config.ts
```

It emits the standalone, client-only screen entry in `dist/player-screen`, importing Songverse's actual `ScreenStage` and styles. This build does **not** invoke the production TanStack Start SSR route.

From the `songverse-player` repository, package the output as a Rust-compatible bundle:

```sh
REV=$(git -C ../Songverse rev-parse HEAD)
sh tools/package-upstream-screen.sh ../Songverse /tmp/songverse-screen-$REV "$REV"
SONGVERSE_BUNDLE_DIR=/tmp/songverse-screen-$REV cargo run -p songverse-player
```

Open `http://127.0.0.1:8787/offline/screen` in Chromium. The page polls the Rust mock `/api/state`, maps mock slides into proper `ScreenSlide` values, and displays them with **the actual Songverse ScreenStage component**. Left/right navigation uses the existing local API.

## What this proves

- Songverse's existing theme/CSS/JS React renderer can run as a client-only bundle served by Rust.
- No second CSS/HTML slide implementation is needed in the player.
- The Rust local asset server uses checksums and a versioned source revision.
- Chromium can render that route directly to HDMI through the kiosk launcher.

## What it does not yet prove

- This repository's CI doesn't build the upstream bundle; Songverse's own e2e suite (`e2e/web/player-screen.test.mjs`) builds it and checks it renders against a stand-in for this player's `/api/state`.
- The first offline screen shows a mocked local set, not real downloaded Songverse content.
- Remote pairing and session authorization are not available offline.
- Real custom themes, assets, chart data, embedded videos, widgets, synchronization and failover require an authoritative local snapshot contract.
- The script creates a file-integrity manifest, not signed or trusted release metadata.
- Hardware GPU/HDMI operation has not been validated on the mini PC.

## Next integration contract

Agree on a versioned `GET /api/local/screen-state` response representing the upstream `ScreenCurrent`, `SyncPresenting`, `SyncStage` and asset references. The Rust player caches set/theme/media content and serves local URLs; the frontend uses the same `renderChart`, `lyricSlides` and `ScreenStage` pipeline currently used by Songverse's `/screen` route.

Do not persist browser screen tokens in unencrypted frontend bundles or commit downloaded proprietary song content.
