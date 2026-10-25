# Oneverse Pulse

**The heartbeat of your live experience.**

Oneverse Pulse is the on-site runtime and fallback node for the
[Oneverse ecosystem](https://github.com/Vulcio/Oneverse): Songverse, and later Slideverse.
It is intended for a Debian mini PC at the venue, keeping a live set running when the internet
fails and driving the room's screens, later audio, MIDI and lighting.

**Development status:** the Rust daemon serves verified bundles and, with `--dev`, a mock set
and screen. The Chromium/Cage kiosk and systemd units are written but **have never run on the
target hardware**. This repository does not yet install or supervise the live engine.

## Two programs, two repositories

| Part | Repository | Responsibility |
| --- | --- | --- |
| Live engine (`@oneverse/pulse`, Node) | Oneverse, `apps/pulse` | Pairing, cloud connection, `LiveEngine`, session mirror/takeover/handback, set copy, room server and operator console |
| `pulsed` and Debian appliance | This repository | Bundle serving today; process supervision, aggregate health, updates and outputs next |
| Screen page (`ScreenStage`) | Oneverse, `packages/screen` | Shared rendering for every screen; Chromium renders it for HDMI |

The Node engine owns the live session. Rust will follow the room server as a local client;
it never keeps a second session or implements the cloud's reducer. The engine's pairing,
mirror, takeover, handback, room server and console have been tested in Oneverse's CI against
the API, **not on the mini PC**. Connecting this daemon to it is roadmap P1.

The current standalone frontend is Oneverse's prototype player-screen build. It reads mock
`/api/state`, so it needs `--dev`. Roadmap P2 replaces it with a screen page that joins the
Node room server and reuses the same `ScreenStage`. No renderer is copied into this repository.

Read [the architecture](docs/architecture.md), [roadmap](docs/roadmap.md) and
[agent instructions](AGENTS.md) before contributing.

Future instrument hosting, native web preset editing, rack snapshots and song/section scenes
are scoped in the [P7 plan](docs/plugin-hosting.md), after P6 synchronized audio and LTC.
They are planned capabilities, not current runtime features.

## Run the daemon

Rust stable is required. The package and binary are `pulsed`.

```sh
cargo run -p pulsed
# Health only, plus a verified bundle if configured; no mock routes.
curl http://127.0.0.1:8787/health
```

The API always binds to `127.0.0.1:8787`. `/health` currently returns `ok`; aggregate engine and
box health comes in P1. It does not yet indicate readiness for a live event.

For the development prototype:

```sh
cargo run -p pulsed -- --dev
# Open http://127.0.0.1:8787/screen
curl http://127.0.0.1:8787/api/state
curl -X POST http://127.0.0.1:8787/api/next
curl -X POST http://127.0.0.1:8787/api/previous
```

Without `--dev`, `/screen`, `/api/state`, `/api/next` and `/api/previous` return 404, and the mock
set is not constructed. `--help` and `--version` work without starting the server.

### Build and serve the real ScreenStage prototype

Check out [Vulcio/Oneverse](https://github.com/Vulcio/Oneverse) beside this repository and install
its workspace dependencies:

```sh
cd ../Oneverse
pnpm --filter @oneverse/web build:player-screen
cd ../oneverse-pulse
REV=$(git -C ../Oneverse rev-parse HEAD)
sh tools/package-upstream-screen.sh ../Oneverse "/tmp/oneverse-pulse-screen-$REV" "$REV"
PULSE_BUNDLE_DIR="/tmp/oneverse-pulse-screen-$REV" cargo run -p pulsed -- --dev
```

Open **http://127.0.0.1:8787/offline/screen**. This renders the mock set with the real
`ScreenStage`; it does not download a live show's content. Bundle serving at `/offline/screen`
and `/assets/*` works independently of `--dev`, with checksums verified at startup and per
request. An invalid configured bundle prevents startup.

See [frontend build](docs/frontend-build.md), [bundle format](docs/offline-bundle.md), and
[Debian installation and migration](appliance/debian/README.md).

## Configuration and naming migration

| Current variable | Previous name (one release only) | Default |
| --- | --- | --- |
| `PULSE_BUNDLE_DIR` | `SONGVERSE_BUNDLE_DIR` | No bundle |
| `PULSE_SCREEN_URL` | `SONGVERSE_SCREEN_URL` | `https://screen.oneverse.top` (online screen) |
| `PULSE_CHROMIUM_PROFILE` | `SONGVERSE_CHROMIUM_PROFILE` | `/var/lib/oneverse-pulse/chromium` |

The daemon reads the bundle directory; the kiosk launcher reads the other two settings.
Legacy variables emit warnings. A present `PULSE_*` variable takes precedence, including an
explicit empty value, which is rejected rather than silently using an old value.

The service user is `pulse`; state lives in `/var/lib/oneverse-pulse`, configuration in
`/etc/oneverse-pulse`. Units are `oneverse-pulse.service` and `oneverse-pulse-kiosk.service`.
Old users, files and units require [manual migration](appliance/debian/README.md#migration-from-the-prototype).
The default screen URL has also moved from `https://songverse.one/screen` to
`https://screen.oneverse.top`; explicitly configured URLs are retained.

## Development layout and checks

| Path | Purpose |
| --- | --- |
| `crates/pulse-core/` | Development-only mock state and navigation, removed in P2 |
| `crates/pulsed/` | Loopback daemon, CLI, mock routes and verified bundle serving |
| `web/renderer/` | Mock screen for daemon development |
| `appliance/` | Chromium/Cage launcher, systemd units and Debian setup notes |
| `tools/` | Screen packaging, manifest generation and launcher tests |
| `docs/` | Architecture, integration details and ordered roadmap |

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 -m py_compile tools/make-screen-manifest.py
sh -n appliance/kiosk/launch.sh tools/package-upstream-screen.sh
python3 -m unittest discover -s tools/tests
```

CI runs these checks and builds an `x86_64-unknown-linux-gnu` release binary in Debian stable
(Trixie), uploaded as `pulsed-debian-trixie-x86_64`. That build does not qualify GPU, HDMI, NDI
or audio operation on the appliance.

**Verified on 2026-10-23:** in an Ubuntu 24.04 Linux development sandbox with Rust 1.99.1,
formatting, Clippy, 7 Rust tests, 5 launcher tests, Python compilation and shell syntax checks
passed. The release binary built for `x86_64-unknown-linux-gnu`. HTTP smoke checks exercised
default route isolation, `--dev` navigation, verified bundle delivery, legacy configuration,
new-variable precedence and corrupt-bundle startup rejection. No browser or real Oneverse
frontend was rebuilt or run in that check; no target hardware was used.

**Previously verified (2026-10-20):** on a laptop, Oneverse source at `e8c3e55` built a
standalone frontend, packaging produced a verified bundle, and Chromium displayed its mock
set with arrow-key navigation. This records that earlier test, not fresh browser verification.

## Security and licensing

The daemon's loopback API has no authentication. The Node room server is the intended LAN
entry point, using passes signed by the cloud. Keep the Chromium profile private; do not
enable remote debugging. Bundle manifests prove integrity, not who published a bundle;
signed releases and rollback are roadmap P4.

The workspace is AGPL-3.0-or-later. NDI, audio, MIDI and lighting are not implemented.
NDI remains a future opt-in build feature, subject to SDK licence review and approval.
