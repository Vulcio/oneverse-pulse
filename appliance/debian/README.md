# Manual Debian HDMI kiosk setup (prototype)

**Not run on the mini PC. Not a bootable image yet.** These are manual installation steps
for Debian stable (Trixie), x86-64, with working GPU drivers, a monitor and VT/logind access.

## Packages and account

```sh
sudo apt update
sudo apt install --no-install-recommends cage chromium dbus-user-session
sudo useradd --system --create-home --home-dir /var/lib/oneverse-pulse --shell /usr/sbin/nologin pulse
sudo usermod -aG video,render,input pulse
sudo install -d -o pulse -g pulse -m 0700 /var/lib/oneverse-pulse /var/lib/oneverse-pulse/chromium
sudo install -d -m 0755 /usr/local/lib/oneverse-pulse /etc/oneverse-pulse
sudo install -m 0755 appliance/kiosk/launch.sh /usr/local/lib/oneverse-pulse/launch-kiosk.sh
sudo install -m 0644 appliance/kiosk/kiosk.env.example /etc/oneverse-pulse/kiosk.env
sudo install -m 0644 appliance/systemd/oneverse-pulse-kiosk.service /etc/systemd/system/
```

Set `PULSE_SCREEN_URL` in `/etc/oneverse-pulse/kiosk.env`. The default
`https://screen.oneverse.top` opens Oneverse's real paired screen and requires internet access.
`http://127.0.0.1:8787/screen` opens the separate mock screen and requires `pulsed --dev`.
Keep the Chromium profile private; the launcher creates it with mode `0700`.

Cage needs a local seat and VT. The kiosk service is **not hardware-tested**; an SSH shell alone
is insufficient. On a dedicated appliance, disable the display manager and free tty1 from its
getty before enabling it. Keep SSH or another TTY available for recovery.

```sh
sudo systemctl daemon-reload
sudo systemctl enable --now oneverse-pulse-kiosk.service
journalctl -u oneverse-pulse-kiosk.service -f
```

## Daemon and verified bundles

Build on Debian stable, or use CI's `pulsed-debian-trixie-x86_64` artifact. The executable needs
its executable permission restored after downloading an artifact.

```sh
cargo build --locked --release -p pulsed
sudo install -m 0755 target/release/pulsed /usr/local/bin/pulsed
sudo install -m 0644 appliance/systemd/oneverse-pulse.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now oneverse-pulse.service
curl http://127.0.0.1:8787/health
```

The service starts without mocks. It does not start the Node engine yet (roadmap P1).
Optional daemon configuration lives in `/etc/oneverse-pulse/pulsed.env`, for example
`PULSE_BUNDLE_DIR=/opt/oneverse-pulse/bundles/<revision>`. Install a verified bundle there as
root, readable by `pulse`, and keep it unwritable by `pulse`. Restart the daemon after changing
its configuration. See [the bundle format](../../docs/offline-bundle.md).

The current prototype bundle needs mock `/api/state`. For development only, add a systemd
service override with `sudo systemctl edit oneverse-pulse.service`:

```ini
[Service]
ExecStart=
ExecStart=/usr/local/bin/pulsed --dev
```

Restart the daemon after adding or removing this override. Set the kiosk URL to
`http://127.0.0.1:8787/offline/screen` only after installing a compatible frontend bundle.

## Migration from the prototype

Stop and disable `songverse-player.service` and `songverse-kiosk.service` before installing
the renamed units, so the old daemon does not hold port 8787 or the kiosk's VT. The account,
paths and unit names are not automatically migrated.

Copy any retained kiosk profile from `/var/lib/songverse/chromium` to
`/var/lib/oneverse-pulse/chromium` while Chromium is stopped; make the copied profile owned by
`pulse:pulse` and mode `0700`. Copy wanted configuration from `/etc/songverse` to
`/etc/oneverse-pulse`; the renamed units only read the new paths. Move or reinstall bundles
separately and update their path in `pulsed.env`.

For one release, `SONGVERSE_BUNDLE_DIR`, `SONGVERSE_SCREEN_URL` and
`SONGVERSE_CHROMIUM_PROFILE` still work, with warnings. Prefer their `PULSE_*` names. If both
names are present, the new value wins, including an explicit empty value (which is invalid).
The default online screen URL has moved from `https://songverse.one/screen` to
`https://screen.oneverse.top`; an explicit URL still wins.

Do not enable Chromium remote debugging or expose the daemon's unauthenticated API to the LAN.
Hardware, image building, watchdogs and update rollback remain later roadmap work.
