# Manual Debian HDMI kiosk setup (prototype)

**Not a bootable image yet.** Validate on a dedicated Debian stable x86-64 mini PC with working GPU drivers, monitor and VT/logind access. Do not install on a production show computer without testing recovery.

## Packages and account

On Debian, install `cage`, `chromium`, `dbus-user-session` and any GPU/mesa drivers required by the machine.

```sh
sudo apt update
sudo apt install --no-install-recommends cage chromium dbus-user-session
sudo useradd --system --create-home --home-dir /var/lib/songverse --shell /usr/sbin/nologin songverse
sudo usermod -aG video,render,input songverse
sudo install -d -o songverse -g songverse -m 0700 /var/lib/songverse/chromium
sudo install -d -m 0755 /usr/local/lib/songverse /etc/songverse
sudo install -m 0755 appliance/kiosk/launch.sh /usr/local/lib/songverse/launch-kiosk.sh
sudo install -m 0644 appliance/kiosk/kiosk.env.example /etc/songverse/kiosk.env
sudo install -m 0644 appliance/systemd/songverse-kiosk.service /etc/systemd/system/
```

Set `SONGVERSE_SCREEN_URL` in `/etc/songverse/kiosk.env`: default `https://songverse.one/screen` uses Songverse's **real** paired screen; `http://127.0.0.1:8787/screen` runs the separate Rust demo (requires Rust player daemon running).

**Important:** Cage uses a local seat and VT. The example kiosk service is not yet hardware-tested; a headless SSH shell alone is insufficient. Disable the display manager on a dedicated appliance before enabling the kiosk service, and ensure tty1 isn't simultaneously owned by a getty. Be prepared to recover through SSH/another TTY.

```sh
sudo systemctl daemon-reload
sudo systemctl enable --now songverse-kiosk.service
journalctl -u songverse-kiosk.service -f
```

Do not add remote debugging to Chromium or publish the mock server API to a LAN. The Chromium profile contains device pairing data.

The player daemon service in `appliance/systemd/songverse-player.service` is a separate unit and requires installing the built Rust binary.

Future: image builder, compositor integration tests, session/seat hardening, watchdog, update rollback and verified offline frontend bundles.
