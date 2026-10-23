# Oneverse Pulse - instructions for coding agents

Oneverse Pulse is the on-site runtime of [Oneverse](https://github.com/Vulcio/Oneverse) (Songverse,
later Slideverse): a Debian mini PC at the venue that keeps a live set running when the internet
doesn't. It mirrors the cloud's live session, takes it over after 10 s without the cloud, hands it
back when the cloud returns, drives the room's screens over HDMI (later NDI), and later plays audio
(click, cues, stems) and drives MIDI and lighting. Its plan lives in Oneverse's issues: #247 (the
on-site runtime), #62 (headless audio), #196 (NDI), #320 (pairing, locations). Oneverse's own rules
(its `CLAUDE.md`, written for every agent despite its name) apply here too.
This file is the rulebook for every agent, whatever its tool or model.

**Read [docs/architecture.md](docs/architecture.md) and [docs/roadmap.md](docs/roadmap.md) before
building anything.** They say what Pulse is made of, which repository each part lives in, and the
order of work.

## Pulse is two programs, in two repositories

Decided by brice on 2026-10-22. Don't move the line between them without asking him.

- **The live engine is TypeScript, in Oneverse** (`apps/pulse`, `@oneverse/pulse`, a Node process).
  It handles the pairing, the link to the cloud, the session (`LiveEngine` from `@oneverse/core`, the
  same reducer the cloud runs), and the takeover and handback. It also runs the room's server (the
  cloud's `/sync` for the venue's network, entered with a pass the cloud signed), the copy of the
  set, and Pulse's page (the operator's console). **Never rewrite any of it in Rust here.** A change
  to it is made in Oneverse, with its issue and its checks.
- **This repository holds everything else on the box.** That is the appliance (the Debian install,
  services, kiosk, updates) and `pulsed`, the Rust daemon. `pulsed` supervises the processes and
  reports the box's health. It owns the outputs: HDMI through Chromium, NDI through an offscreen
  renderer and the NDI SDK. Later it also runs what needs real time: audio, timing, MIDI and DMX.
  Rust follows the session as a **local client of the Node room server**. It never keeps a second
  copy of the session state.

What a screen shows is Oneverse's `ScreenStage` (`packages/screen`), built by Oneverse. Never write
a second stage, theme or slide renderer here, and never copy Oneverse's source into this repository.

## Layout

The layout today is the prototype. It is moving to the shape in `docs/architecture.md`.

- `crates/pulse-core`: development-only mock state and navigation, with no I/O. It is due to be
  removed in roadmap P2.
- `crates/pulsed`: the daemon (`pulsed`, `127.0.0.1:8787`):
  - only with `--dev`: the mock set (`/api/state`, `/api/next`, `/api/previous`) and mock screen (`/screen`);
  - the verified bundle server (`offline.rs`, `/offline/screen`, `/assets/*`, checked against a
    SHA-256 manifest). Keep the bundle server; it becomes how `pulsed` serves what it installs.
- `tools/`: packages Oneverse's player-screen build into a verified bundle.
- `appliance/`: the kiosk (Cage + Chromium) and the systemd units. **None of it has run on the
  target hardware yet.** Say so wherever it matters.
- `web/renderer/`: the mock screen, for working on the daemon only. It is never the real screen.
- `docs/`:
  - the architecture and roadmap (current);
  - `frontend-build.md`, `offline-bundle.md` and `renderer-integration.md`, which describe the
    prototype and say where they're superseded.

The P0 names are `pulse-core`, `pulsed`, `oneverse-pulse.service` and
`oneverse-pulse-kiosk.service`, user `pulse`, and `/var/lib/oneverse-pulse` and
`/etc/oneverse-pulse`. Configuration uses `PULSE_*`; the old `SONGVERSE_*` variables are still
read for one release with a warning. A present new variable takes precedence over the old one.

## Before every push

CI runs these on every push. Run them first, and push only when all pass:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 -m py_compile tools/make-screen-manifest.py
sh -n appliance/kiosk/launch.sh tools/package-upstream-screen.sh
python3 -m unittest discover -s tools/tests
```

A change that talks to the live engine is also checked against it, with Oneverse checked out beside
this repository:

```sh
pnpm --dir ../Oneverse install
pnpm --dir ../Oneverse --filter @oneverse/pulse build
pnpm --dir ../Oneverse --filter @oneverse/screen-app build:pulse
# a local API and its stack: ../Oneverse/e2e/README.md
```

Then run `pulsed` against that `apps/pulse`, and say in the commit what you saw. Tests that need the
engine start it in-process or as a child process. A test that needs the cloud is written in
Oneverse's `e2e/` instead (`pulse-link`, `pulse-room`, `pulse-pairing`, `web/pulse-console`
already exist). Tests never call `api.oneverse.top`.

## Oneverse beside this repository

Clone it next to this one: `git clone https://github.com/Vulcio/Oneverse ../Oneverse`. Its
`apps/pulse/README.md` and `docs/live-session.md` say what the engine does today. Read them before
relying on any of its behaviour.

When a step needs a change on Oneverse's side and you can't make it there, stop at the contract.
Write the change down in an issue on Vulcio/Oneverse: what Pulse needs, the messages or routes, and
how you'd test it. Then build this side against a stand-in that follows that write-up. Never work
around it here, for example by reading the engine's files, scraping its page, or keeping session
state in Rust.

## Contracts with Oneverse

Every contract below is written down in both repositories (`docs/` here, and `apps/pulse/README.md`
or `docs/live-session.md` there). Change it in both at once: the Oneverse side first, in its own
commit with its own issue, passing Oneverse's checks (`pnpm lint`, `pnpm type-check`, `pnpm test`,
and the e2e suites its `e2e/affected.mjs` picks).

- **The engine's environment and files.**
  - `ONEVERSE_API_URL`, `PULSE_NODE_ID`, `PULSE_DATA_DIR` (its token, `0600`, and the set's copy),
    `PULSE_CONSOLE_DIR`, `PULSE_ROOM_PORT` (8788), `PULSE_ROOM_HOST`, `PULSE_ROOM_URL`.
  - `GET /health` (`pairing`, `name`, `set`, `cloud`, `session`, `epoch`).
  - `pulsed` starts the engine with these and reads its health. It doesn't read the engine's files.
- **The room's `/sync`.** It uses the same messages as the cloud's (`@oneverse/core`'s `sync/` and
  `live/`). `pulsed` joins it on loopback to hear the session: the slide shown and the metronome's
  timeline. That needs a local credential, which doesn't exist yet. It is designed in Oneverse
  first (roadmap P2).
- **The screen page.** What HDMI and NDI show is a page Oneverse builds. It joins the room as a
  screen. `pulsed` serves it from a verified bundle and points Chromium at it. That page doesn't
  exist yet either (roadmap P2). The old player-screen build, with its `/api/state` contract, goes
  away with it.
- **The bundle** (`index.html`, `assets/`, `manifest.json`: [docs/offline-bundle.md](docs/offline-bundle.md)).
  It is pinned to the Oneverse commit it was built from.

## How to work

- **An issue first,** in Oneverse or here, referenced by every commit (`Refs Vulcio/Oneverse#247`).
  Anything planned but not started gets an issue, so nothing lives only in a conversation.
- **One commit per change,** made with git, not file by file through the GitHub API. Its message says
  what changed and why. Never put the same message on several commits.
- **Say only what you verified.** The README and docs say what was built, run and seen working, on
  what (a laptop, CI, the mini PC), and what wasn't (hardware, GPU, HDMI, NDI, audio latency).
  Update them when that changes, either way.
- **Security:**
  - The room server (Node) is the only thing on the venue's network, and only with a pass.
  - `pulsed`'s own API stays on loopback with no authentication. Don't widen it without an issue
    that designs it.
  - The engine's token and the Chromium profile are readable by their service user only.
  - A manifest checks integrity, not who made the bundle. Signed releases come in roadmap P4.
- **Real time:** audio and timing never ride on network messages. Pulse plays the timeline ("beat N
  at time T") on its own clock, as every Oneverse device does. The audio thread never waits on I/O,
  the network or the session.
- **Licences:** this repository is AGPL-3.0-or-later like Oneverse. Check every new crate's licence.
  The NDI SDK has its own terms: read them and ask brice before anything links to it. NDI support
  stays a build feature that is off by default.
