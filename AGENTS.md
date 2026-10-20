# Instructions for coding agents

This repository is the Rust mini-PC player for [Songverse](https://github.com/bricedupuy/Songverse)
(its issues #62 and #247). Read [CLAUDE.md](CLAUDE.md) before changing anything: it is this
repository's rulebook for every agent, whatever its name, with its layout, the end-to-end check of
the screen bundle and the contracts with Songverse. Songverse's own rules apply here too. In short:

## Before every push

CI runs these; run them first and push only when all pass:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 -m py_compile tools/make-screen-manifest.py
```

## How to work

- **An issue first,** in Songverse (the player's plan lives there: #247, #62) or here. Commits reference
  it.
- **One commit per change,** made with git (not file by file through the GitHub API), with a message
  that says what changed and why. Never several commits with the same message.
- **Don't claim what you haven't run.** If you didn't build, run or test something, say so in the
  README and the commit. If you did, say what you ran.
- **Don't fork Songverse.** The screen is Songverse's `ScreenStage`, built by Songverse
  (`apps/web/src/player-screen/`, `pnpm --filter @songverse/web build:player-screen`). Changes it
  needs are made there, in their own commit with its own issue, and must pass Songverse's checks:
  `pnpm lint`, `pnpm type-check`, `pnpm test` and `e2e/web/player-screen.test.mjs`.
- **Contracts between the two repositories** (the local API the screen reads, the bundle layout)
  are written down in `docs/` here and in the player screen's README in Songverse, and changed in
  both at once.
