# Integrating the real Songverse renderer

## Source of truth

The upstream implementation already exists:
- [Songverse `apps/web/src/routes/screen.tsx`](https://github.com/bricedupuy/Songverse/blob/main/apps/web/src/routes/screen.tsx) owns the screen page and presentation assembly.
- [`apps/web/src/components/screen-stage.tsx`](https://github.com/bricedupuy/Songverse/blob/main/apps/web/src/components/screen-stage.tsx) owns staged visual output.
- [`apps/web/src/lib/screen-display.ts`](https://github.com/bricedupuy/Songverse/blob/main/apps/web/src/lib/screen-display.ts) handles code pairing, screen tokens, loading and sync.
- [`packages/core/src/screens/`](https://github.com/bricedupuy/Songverse/tree/main/packages/core/src/screens) defines themes and slide behavior.

Do **not** fork the production screen renderer into this Rust repository. The JavaScript bundle must be produced from the upstream Songverse web application, using compatible versions of `@songverse/core` and the API.

## First vertical slice (online)

1. The mini PC boots Debian.
2. systemd launches Cage (single-app Wayland compositor).
3. Cage starts Chromium fullscreen on `https://songverse.one/screen`.
4. The existing Songverse screen UI shows its pairing code; pair the screen in Songverse.
5. Songverse's real HTML/CSS/JS, themes and screen session render directly through the GPU to HDMI.

This path **requires the web app and API to be reachable**. It does not yet route remote web calls through Rust. Rust's local mock demo remains available at `http://127.0.0.1:8787/screen`.

## Target offline mode (not implemented)

Build/version the actual Songverse screen frontend as a deployable static or locally hosted bundle. The Rust agent would verify and activate it atomically, expose authorized offline state and assets through a local API, and coordinate sync handover. Refactor any SSR-only screen initialization to support the local runtime. The same ScreenStage HTML/CSS/JS remains the renderer—only the data source changes.

## HDMI and NDI

Direct HDMI: Chromium → GPU → Wayland/Cage → DRM/KMS → connector. Avoid frame capture here.

NDI: independent offscreen Chromium instance or capture compositor → RGBA frames → native NDI SDK integration. Alpha, transport format, frame pacing and redistribution licensing require validation. NDI is **not implemented** by the kiosk service.

## Security and operations

Browser profile retains a screen token. Restrict permissions on `/var/lib/songverse/chromium`; a compromise of that account can expose the paired screen. Do not enable unauthenticated remote debugging. The kiosk requires local-seat access (logind/PAM, VT and graphics devices); validate service startup on actual target hardware. Configure firewall and unattended updates as separate production work.
