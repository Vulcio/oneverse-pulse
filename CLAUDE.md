# songverse-player - notes for Claude

The Rust player for a mini PC on stage (Debian, HDMI and later NDI, offline fallback), part of
[Songverse](https://github.com/bricedupuy/Songverse). Its plan lives in Songverse's issues: #247 (the
on-site player: screens, offline copy of the sets, local control) and #62 (headless audio: click,
cues, backing tracks). Songverse's own rules (its `CLAUDE.md`) apply here too.

## Layout

- `crates/core`: the player's state and navigation, no I/O, unit-tested.
- `crates/player`: the Axum daemon on `127.0.0.1:8787`: the mock set (`/api/state`, `/api/next`,
  `/api/previous`), the mock screen (`/screen`), and the verified screen bundle (`offline.rs`,
  `/offline/screen` and `/assets/*`, checked against a SHA-256 manifest on start and on each request).
- `tools/`: `package-upstream-screen.sh` packages Songverse's player-screen build into a bundle;
  `make-screen-manifest.py` writes its manifest.
- `appliance/`: the kiosk (Cage + Chromium) and systemd units. They have not been run on the target
  hardware yet: say so wherever it matters.
- `web/renderer/`: the mock screen only, for working on the daemon. Never the real screen.

## Before every push

CI runs these on every push; run them first and push only when all pass:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 -m py_compile tools/make-screen-manifest.py
```

When the change touches the screen bundle, also check it end to end, with Songverse checked out
beside this repository:

```sh
pnpm --dir ../Songverse --filter @songverse/web build:player-screen
REV=$(git -C ../Songverse rev-parse HEAD)
sh tools/package-upstream-screen.sh ../Songverse /tmp/songverse-screen-$REV "$REV"
SONGVERSE_BUNDLE_DIR=/tmp/songverse-screen-$REV cargo run -p songverse-player
# open http://127.0.0.1:8787/offline/screen in Chromium: the set shows, the arrow keys move it
```

## The screen is Songverse's

- What the player shows on a screen is Songverse's `ScreenStage`, built by Songverse
  (`apps/web/src/player-screen/`, `pnpm --filter @songverse/web build:player-screen`). Never write a
  second stage, theme or slide renderer here; never copy Songverse's source into this repository.
- What the screen needs from the player (data, routes) changes in Songverse's player screen in its
  own commit, with its own issue, passing Songverse's checks (`pnpm lint`, `pnpm type-check`,
  `pnpm test`, and `e2e/web/player-screen.test.mjs`, which stands in for this daemon's API).
- The contracts between the two - the local API the screen reads (`GET /api/state`: a set's title
  and slides, the slide shown; `POST /api/next`, `/api/previous`) and the bundle's layout
  (`index.html`, `assets/`, `manifest.json`) - are written in `docs/` here and in the player
  screen's README in Songverse, and changed in both at once. The next one, a versioned local
  snapshot of a screen's state, is agreed in #247 before it's built.

## How to work

- **An issue first,** in Songverse or here, referenced by every commit (`Refs bricedupuy/Songverse#247`).
- **One commit per change,** made with git, not file by file through the GitHub API, with a message
  that says what changed and why. Never the same message on several commits.
- **Say only what you verified.** The README and docs say what was built, run and seen working, and
  what wasn't (hardware, GPU, HDMI, NDI). Update them when that changes, either way.
- **Security:** the daemon listens on loopback only and has no authentication yet; don't widen
  either without an issue that designs it. The bundle manifest checks integrity, not who made it.
- **Licences:** this repository is AGPL-3.0-or-later like Songverse; NDI's SDK has its own terms,
  to check before anything links to it.
