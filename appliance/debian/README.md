# Debian minimal appliance

Target: Debian stable x86-64, systemd, Chromium and a minimal Wayland compositor (for example cage). This directory is a **design scaffold**, not a working disk image builder.

1. Install the compiled Rust daemon to `/usr/local/bin/songverse-player`.
2. Create an unprivileged `songverse` service account.
3. Install and enable `appliance/systemd/songverse-player.service`.
4. Configure a dedicated Wayland kiosk session with Chromium pointed to `http://127.0.0.1:8787/screen`.
5. Keep network control on loopback until pairing, authentication and authorization are implemented.

Future: reproducible image builds, boot watchdog, read-only root filesystem, update rollback, GPU/HDMI probing, audio interface setup.
