# Architecture

## Responsibilities

- **Rust player daemon**: lifecycle, device identity, offline set cache, local control, session and timing coordination.
- **Rust audio engine (future)**: local real-time click, cues, backing tracks and multichannel routing; no browser audio dependency.
- **Chromium renderer**: reuse Songverse's actual screen web bundle and assets. This repository's current HTML is only a mock vertical slice.
- **HDMI**: Chromium kiosk on a Wayland compositor, GPU output directly to connected displays.
- **NDI (future)**: separate offscreen rendering/capture backend, RGBA/alpha support where possible, SDK license review before distribution.
- **LAN fallback (future)**: locally served sync protocol, signed authorization grants, clock handover, cloud reconciliation.

## Data flow

Cloud Songverse API → paired device → cached sets and assets → Rust session state → local screen frontend → Chromium → HDMI.

A second rendering instance can feed a native capture pipeline → NDI sender. Never capture HDMI frames unnecessarily for direct output.

## Prototype constraints

- Only a mock set in memory; restart resets position.
- HTTP listens only on loopback and is not authenticated.
- Navigation uses polling, not the production sync protocol.
- No actual offline downloads, Chromium supervisor, NDI or audio playback yet.
- Multi-output design should preserve independent screens and themes.
- Evaluate the upstream `@songverse/core` sync protocol before defining Rust-compatible message contracts.

## Production principles

Atomic cache updates, checksum-verified media, bounded disk usage, no audio timing on network ticks, restart watchdog, local controls during WAN failure, deterministic session recovery, and explicit authorization for every control operation.
