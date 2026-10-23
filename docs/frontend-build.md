# End-to-end standalone renderer development path

> **Prototype.** This describes the prototype (mock `/api/state`, the player-screen build). Where Pulse
> is going is in [architecture.md](architecture.md) and [roadmap.md](roadmap.md): the screen page
> joins the room's `/sync` instead (roadmap P2).

The source of the real renderer remains [Songverse](https://github.com/Vulcio/Oneverse).

## Build the real ScreenStage frontend

In a checked-out, dependency-installed Songverse workspace:

```sh
cd apps/web
vp build --config vite.player.config.ts
```

It emits the standalone, client-only screen entry in `dist/player-screen`, importing Songverse's actual `ScreenStage` and styles. This build does **not** invoke the production TanStack Start SSR route.

From the `oneverse-pulse` repository, package the output as a Rust-compatible bundle:

```sh
REV=$(git -C ../Oneverse rev-parse HEAD)
sh tools/package-upstream-screen.sh ../Oneverse /tmp/songverse-screen-$REV "$REV"
PULSE_BUNDLE_DIR=/tmp/songverse-screen-$REV cargo run -p pulsed -- --dev
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

## Former integration proposal (superseded)

The former `GET /api/local/screen-state` proposal is superseded by roadmap P2:
Oneverse's Node engine owns the session and set copy; the screen joins its room server.
Rust never caches a second session or implements a screen-state API.

Do not persist browser screen tokens in unencrypted frontend bundles or commit downloaded proprietary song content.
