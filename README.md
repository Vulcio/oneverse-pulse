# Songverse Player

Rust-first, offline-capable Debian mini-PC appliance for Songverse live presentations and synchronized playback.

Based on [Songverse #62](https://github.com/bricedupuy/Songverse/issues/62) and [#247](https://github.com/bricedupuy/Songverse/issues/247).

## Current milestone: HDMI vertical slice

A Rust/Axum daemon serves a **mock offline set** and a local HTML/CSS/JS screen at `http://127.0.0.1:8787/screen`. Chromium can run in kiosk mode on the attached display. Next/previous slide controls work through a local API.

This is a prototype, **not** a production-ready offline failover, audio, NDI or bootable OS image.

## Development

Requires Rust stable.

```sh
cargo run -p songverse-player
# open http://127.0.0.1:8787/screen in Chromium
curl http://127.0.0.1:8787/health
curl -X POST http://127.0.0.1:8787/api/next
cargo test --workspace
```

For a local HDMI kiosk (requires a working graphical session and Chromium):
```sh
chromium --kiosk --no-first-run --disable-session-crashed-bubble http://127.0.0.1:8787/screen
```

## Architecture

See [docs/architecture.md](docs/architecture.md) and [docs/roadmap.md](docs/roadmap.md).

The initial renderer is an isolated **mock**: production must reuse Songverse's existing screen frontend rather than reimplementing its themes. Rust handles orchestration, local cache, device identity, and eventually real-time audio. Chromium owns HTML/CSS/JS rendering and direct HDMI display; NDI will use a separate captured-frame backend.

## Security

The HTTP API binds to loopback only. Do not expose the prototype to a LAN: it has no authentication or authorization yet. Device pairing, signed offline permissions and secure LAN access are future work.

## Licensing

Licensing is to be decided with the Songverse maintainers before redistributing the appliance and optional NDI components.
