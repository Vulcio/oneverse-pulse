# Debian 13 mini-PC installation

**Not run on the mini PC. Not a bootable image yet.** These are installation steps
for Debian stable (Trixie), x86-64, with working GPU drivers, a monitor and VT/logind access.

## Recommended first installation

Install Debian 13 x86-64 with SSH available for recovery. Obtain the
`oneverse-pulse-debian13-x86_64` CI artifact for a trusted build. Extract the artifact's
`oneverse-pulse-debian13-x86_64.tar.gz` on the mini PC, then:

```sh
cd oneverse-pulse
sudo ./install.sh https://your-oneverse-api.example pulse-hall-1
journalctl -u oneverse-pulse.service -f
```

The installer checks the archive files and engine manifest, installs the small Debian runtime
prerequisites, creates `pulse` and its private data directory, installs the engine bundle as
root-owned files, then enables `oneverse-pulse.service`. Node, compiled engine code, production
modules and the console are already in the archive; no build tools or pnpm are needed here.
Supply the API's HTTPS origin and a unique 8–64 character device id (letters, digits, `-`, `_`).
The Node executable is pinned in the bundle manifest, built for this OS and architecture.

Use the pairing code in the journal (also available in JSON at
`http://127.0.0.1:8787/health`, under `engine.health.pairing`) in Oneverse's **Screens > Pair a
screen**, then assign the Pulse to a set or location. Health returns 503 while starting or
restarting the engine; a 200 response alone does not prove that it is paired or online.
The operator console is served by the engine on port 8788, entered with a cloud-signed pass.
Keep the daemon's own port 8787 on loopback.

Configuration is `/etc/oneverse-pulse/pulsed.env`; state and the engine's token are in
`/var/lib/oneverse-pulse/engine` (0700). The installer never overwrites an existing config or
paired identity. On a later manual installation, review the new bundle and set
`PULSE_ENGINE_DIR=/opt/oneverse-pulse/engines/<Oneverse-revision>` in that existing config, then
restart the service. A revision already installed with different bytes is refused. Stop and
disable the old `songverse-player.service` before migrating from P0's predecessor.
Automatic updates, signatures and rollback remain P4; only install releases from a trusted
source. The manifest does not authenticate a publisher.

HDMI kiosk setup below is optional and separate from installing the daemon/engine. Its
current online screen still needs internet; the offline session screen is roadmap P2.

## Build an archive yourself

On Debian 13 x86-64 with Rust, Python 3.11+, Node 22.18+ and Oneverse's pinned pnpm installed,
check out Oneverse beside Pulse at the wanted clean revision:

```sh
revision=$(git -C ../Oneverse rev-parse HEAD)
bash tools/package-appliance.sh ../Oneverse /tmp/oneverse-pulse-debian13-x86_64.tar.gz "$revision"
```

CI pins both the Oneverse revision and exact Node version. For a different installed Node
executable use `PULSE_NODE_BIN`. Build on the target OS/architecture; the runtime is copied,
not cross-compiled. See [engine runtime](../../docs/engine-runtime.md) for the bundle format,
supervisor behavior and local integration checks. The extracted archive includes this guide
as `INSTALL.md`; the source links in that copy refer to this repository.

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

The service starts without mocks. Set `PULSE_ENGINE_DIR` to supervise the packaged Node engine;
see `appliance/pulsed.env.example` and the recommended installation above.
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

## Required mini-PC acceptance (still outstanding)

Record the hardware model and installed release before claiming P1 hardware completion:

1. Reboot and verify the engine starts under systemd and can be paired.
2. Assign a set/location and reach its operator console from a phone on the same network.
3. Disconnect the venue's internet while retaining the LAN; follow **Continue on Pulse**.
4. Present a slide, restore internet, and observe the handback in Live.
5. Record pairing, outage/handback, and service restart results in the README/issue #2.

No HDMI, GPU, NDI or audio behavior is qualified by the local stand-in-cloud tests.
