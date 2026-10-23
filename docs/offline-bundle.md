# Verified local frontend bundle — first infrastructure increment

> **Prototype.** This describes the prototype (mock `/api/state`, the player-screen build). Where Pulse
> is going is in [architecture.md](architecture.md) and [roadmap.md](roadmap.md): the screen page
> joins the room's `/sync` instead (roadmap P2).

## Scope

The Rust daemon now supports read-only delivery of an **externally prepared** screen frontend bundle from a local directory. It verifies a SHA-256 digest manifest on startup and verifies each response's bytes. It only serves `index.html` at `/offline/screen` and manifest-listed assets under `/assets/*`, bound to `127.0.0.1:8787`.

The daemon only serves what it's given; Songverse's client-only stage build, packaged by `tools/package-upstream-screen.sh`, is what it's given (see [frontend build](frontend-build.md)). It is separate from the mock screen at `/screen`.

## Bundle layout

```text
/var/lib/songverse/bundles/<version>/
  index.html
  manifest.json
  assets/
    (hashed JS, CSS, fonts, images, ...)
```

The manifest is `{ "format":1, "source_revision":"<upstream-commit>", "files": { "index.html":"<sha256>", "assets/app.js":"<sha256>" } }`.

For a trusted, already-built compatible bundle:
```sh
python3 tools/make-screen-manifest.py /path/to/bundle --revision <songverse-commit>
SONGVERSE_BUNDLE_DIR=/path/to/bundle cargo run -p songverse-player
# http://127.0.0.1:8787/offline/screen
```
Set `SONGVERSE_SCREEN_URL=http://127.0.0.1:8787/offline/screen` in `/etc/songverse/kiosk.env` only after installing a compatible frontend bundle.

## Missing frontend work

Upstream `apps/web/src/routes/screen.tsx` uses TanStack Start and a server-side `beforeLoad` locale initializer, and its pairing/state hook talks to the cloud API via `getApiUrl()`. Simply copying `dist/client` is **not** enough for disconnected operation.

A dedicated upstream screen build must:
1. Emit a self-bootstrapping offline `index.html` containing the real `ScreenStage` React renderer and styles, sourced from the same Songverse commit.
2. Remove/replace server-side loaders required for the screen's initial render.
3. Use a local, authenticated screen-state adapter and locally mirrored API data when offline.
4. Include all required assets and absolute `/assets/` references compatible with the daemon.
5. Test fonts, shaders, transitions, videos, masking and template parity offline.

## Security limitations

A manifest is an **integrity check only**, not an authenticity guarantee: someone able to replace both assets and their manifest can execute arbitrary JS in Chromium. The bundle directory must be managed by a privileged updater and must not be writable by the player service user. Add signed release metadata, rollback protection and atomic active-version switching before production.

The Rust server rechecks checksums on each request; this is conservative and may be costly for large videos. Use a verified immutable content store and efficient streaming in production. Non-bundled external URLs are not intercepted or cached.
