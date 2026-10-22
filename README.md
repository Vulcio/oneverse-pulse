# Oneverse Pulse

**The heartbeat of your live experience.**

**Oneverse Pulse** is a Rust-first local execution and resilience platform for the [Oneverse ecosystem](https://github.com/Vulcio/Oneverse). Designed for dedicated Debian mini PCs, Pulse brings critical event services closer to the venue: screens, synchronized playback, local control, and eventually instruments, lighting, and automation.

Its purpose is simple: **keep the experience running when the internet doesn't.**

Pulse is more than a player. It is intended to be the **on-site runtime and fallback node** for applications including Songverse and, in the future, Slideverse and other Oneverse products.

> **Development status:** Pulse is an early prototype. A Rust daemon and a standalone build of Songverse's real `ScreenStage` have been demonstrated together; production offline continuity, audio, NDI, MIDI hosting, and lighting control are not yet implemented. The Debian HDMI kiosk has not been validated on target hardware.

## What Pulse is becoming

| Capability | Direction | Current status |
| --- | --- | --- |
| **Resilient local services** | Cache authorized event data, serve local APIs, recover gracefully from network loss, and reconcile with Oneverse when connectivity returns | Local loopback API and verified frontend bundle; real offline data sync pending |
| **Screens and presentation** | Reuse Oneverse/Songverse screen rendering for venue displays, with independent HDMI and eventually NDI outputs | Upstream `ScreenStage` bundle demonstrated; kiosk configuration available, hardware validation pending |
| **Audio and timing** | Backing tracks, stems, metronome, count-ins, spoken cues, multichannel outputs and synchronized playback | Planned; no production audio engine yet |
| **Local orchestration** | Device coordination and event control over a resilient LAN | Planned |
| **MIDI and instruments** | MIDI routing, external devices and potential hosting of virtual instruments | Future exploration |
| **Lighting and show control** | Lighting cues and integration with DMX, Art-Net and sACN equipment | Future exploration |

These are **product directions**, not claims of shipped features. Pulse should remain useful outside music-focused contexts as Oneverse grows.

## Architecture

Pulse separates the event's **control and execution plane** from the **visual renderer**:

```text
               Oneverse cloud and applications
          (Songverse · Slideverse · future apps)
                            │
                     secure synchronization
                            │
                   ┌────────▼────────┐
                   │  Oneverse Pulse │
                   │  Rust runtime   │
                   │  Local services │
                   │  Offline state  │
                   └────────┬────────┘
                            │
            ┌───────────────┼────────────────┐
            │               │                │
      Screen renderer   Audio & timing   MIDI / lighting
      Songverse's own   (future Rust     (future modules)
       ScreenStage       engine)
            │
      Chromium / Wayland
            │
           HDMI            NDI (future)
```

**One shared renderer:** Pulse does not fork Songverse's screens. The standalone frontend is built in [Oneverse's `apps/web/src/player-screen/`](https://github.com/Vulcio/Oneverse/tree/main/apps/web/src/player-screen), reusing the actual `ScreenStage`, themes, CSS, and translations. Chromium runs that HTML/CSS/JavaScript output. Rust handles the local service boundary and will eventually handle cache, synchronization, device operations, and real-time audio.

The future MIDI and lighting modules must not interfere with time-critical audio, presentation, or fallback reliability.

See [architecture](docs/architecture.md), [roadmap](docs/roadmap.md) and [frontend integration](docs/frontend-build.md).

## What works today

- A Rust/Axum daemon serving a mock set and local navigation controls on **loopback only**.
- A verified frontend bundle server: SHA-256 manifest checks at startup and per request.
- A standalone client-only build of **Songverse's real `ScreenStage`**, packaged and served by the daemon.
- A prototype Chromium/Cage kiosk configuration for direct HDMI output.

**Previously verified (2026-10-20):** with Oneverse/Songverse source at `e8c3e55`, the standalone frontend built, the packaging script produced a bundle, the Rust daemon verified and served it, and Chromium displayed the demo set with working arrow-key navigation. This is a record of that test, **not** a claim of fresh verification on the present repository revision.

**Not yet delivered:** persistent real set/theme/media downloads; offline pairing and failover; authenticated LAN control; production sound; HDMI hardware qualification; NDI; MIDI virtual instruments; lighting control; a reproducible appliance image.

## Run the Rust development prototype

Rust stable is required. The Cargo package and binary **still use the transitional name `songverse-player`**; the repository/product rename does not automatically rename its executable or service units.

```sh
cargo run -p songverse-player
# Open http://127.0.0.1:8787/screen
curl http://127.0.0.1:8787/health
curl -X POST http://127.0.0.1:8787/api/next
cargo test --workspace
```

This route uses a **mock HTML screen**, not Songverse's production renderer.

### Build and serve the real Songverse stage locally

Checkout [Vulcio/Oneverse](https://github.com/Vulcio/Oneverse) alongside this repository, with its workspace dependencies installed:

```sh
cd ../Oneverse
pnpm --filter @oneverse/web build:player-screen
cd ../oneverse-pulse
REV=$(git -C ../Oneverse rev-parse HEAD)
sh tools/package-upstream-screen.sh ../Oneverse "/tmp/oneverse-pulse-screen-$REV" "$REV"
SONGVERSE_BUNDLE_DIR="/tmp/oneverse-pulse-screen-$REV" cargo run -p songverse-player
```

Then open **http://127.0.0.1:8787/offline/screen**. This frontend renders the **mock local state using the real `ScreenStage`**. It does not yet download or play a live show's content offline.

The packaging contract is documented in [docs/offline-bundle.md](docs/offline-bundle.md). See [docs/renderer-integration.md](docs/renderer-integration.md) for the online screen and HDMI kiosk approach.

## Development layout

| Path | Purpose |
| --- | --- |
| `crates/core/` | Rust state and navigation primitives |
| `crates/player/` | Local loopback daemon and verified bundle serving |
| `web/renderer/` | Isolated **mock** presentation screen for Rust development |
| `appliance/kiosk/` | Chromium/Cage kiosk launcher and configuration |
| `appliance/systemd/` | Prototype systemd service units (transitional names) |
| `appliance/debian/` | Manual Debian appliance setup notes |
| `tools/` | Upstream screen bundle packaging and manifest generation |
| `docs/` | Architecture, contracts, integration details, roadmap |

The long-term component and binary naming will transition from `songverse-player` to **Oneverse Pulse** in separate, tested changes. Existing environment variables and script paths intentionally remain unchanged for compatibility until then.

## Delivery plan

1. **Oneverse Pulse identity:** repository and product naming; then migrate executable, service, packaging, and configuration identifiers with tests.
2. **Real offline presentations:** versioned local screen-state contract, authorized cached sets/themes/media, and predictable state restoration.
3. **Resilient event coordination:** device pairing, authenticated LAN control, reliable timing, cloud reconnect and reconciliation.
4. **Production performance outputs:** native audio, calibrated hardware outputs, independent HDMI/NDI paths.
5. **Extensible show control:** investigate MIDI instrument hosting and lighting protocols without weakening core reliability.

The immediate implementation track follows [Vulcio/Oneverse issue #247](https://github.com/Vulcio/Oneverse/issues/247) (on-site/offline device). Audio timing, click, cues and tracks are tracked under [issue #62](https://github.com/Vulcio/Oneverse/issues/62).

## Security and licensing

The current HTTP API binds to `127.0.0.1` and **has no authentication**. Do not expose it to a venue LAN or the public internet. A SHA-256 manifest verifies bundle integrity but **does not authenticate its publisher**; secure signing and managed updates are still required. Keep paired-screen Chromium profiles private.

The workspace declares `AGPL-3.0-or-later`; confirm licensing across the Oneverse ecosystem and evaluate third-party NDI SDK terms before distributing an NDI-enabled build.

Contributors and coding agents must read [AGENTS.md](AGENTS.md) and [CLAUDE.md](CLAUDE.md) before making changes, link commits to an issue, and run all required checks before pushing.
