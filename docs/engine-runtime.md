# Supervised Node engine (P1)

Tracked in [#2](https://github.com/Vulcio/oneverse-pulse/issues/2). This is the software
installation and supervision path, tested on Linux with a local cloud stand-in. **The mini-PC
pairing/outage/handback acceptance test is still outstanding.**

The engine remains Oneverse's `apps/pulse`. Rust starts a process and polls its existing
`GET /health`; it does not read the token or set files, maintain a session, or reinterpret
the cloud protocol. No Oneverse engine contract is changed by this work.

## Build a portable engine bundle

Use a clean Oneverse checkout beside Pulse, on the target Linux architecture, with Node
22.18 or newer, the pnpm version pinned by Oneverse and Python 3.11 or newer:

```sh
revision=$(git -C ../Oneverse rev-parse HEAD)
bash tools/package-engine.sh ../Oneverse /tmp/pulse-engine "$revision"
```

The script installs with the frozen lockfile, builds core, the engine and the Pulse console,
and deploys production dependencies with pnpm. It includes only compiled engine files,
production modules and the console, plus the exact Node executable used to build it.
Set `PULSE_NODE_BIN` to select a different installed Node executable. There is no Node or
package-manager download on the mini PC. Build on Debian 13 x86-64 for that appliance;
copying a runtime built for another architecture or newer libc is not supported.
Packaging fetches Node's licence and bundled third-party notices from its exact release tag
at `raw.githubusercontent.com`; TLS verification stays enabled.

`manifest.json` records the Oneverse commit, exact Node version, OS/architecture and SHA-256
of **every** file. Files absent from the manifest, changed files, links and special files
are rejected before execution and again before every child restart. Install bundles
readable but unwritable by the `pulse` service user. The manifest checks integrity, not
publisher identity: distribute the archive through a trusted channel. Signed updates and
automatic rollback remain P4.

```text
bundle/
  manifest.json
  runtime/bin/node
  engine/package.json
  engine/dist/main.js
  engine/node_modules/...
  console/pulse.html
  console/assets/...
```

## Start and supervise

```sh
PULSE_ENGINE_DIR=/tmp/pulse-engine \
PULSE_DATA_DIR=/tmp/pulse-private-data \
ONEVERSE_API_URL=http://127.0.0.1:3001 \
PULSE_NODE_ID=pulse-dev-1 \
PULSE_ROOM_HOST=127.0.0.1 \
cargo run --locked -p pulsed
```

The upstream engine receives `ONEVERSE_API_URL`, `PULSE_NODE_ID`, `PULSE_ROOM_HOST`,
`PULSE_ROOM_URL` and any other existing engine settings through its inherited environment.
The daemon sets `PULSE_CONSOLE_DIR` to the packaged console, `PULSE_ROOM_PORT` to 8788 by
default, and `PULSE_DATA_DIR` to the configured private directory (default
`/var/lib/oneverse-pulse/engine`, mode 0700). That directory must be outside the bundle.
Use the engine's normal pairing flow; no token is put into an archive or daemon health.

Stdout and stderr are inherited, so systemd sends both the daemon and engine logs to the
journal. A child exit or spawn failure is retried after 1, 2, 4, 8, 16, then at most 30 seconds;
the delay resets after a child has run for a minute. A configured but invalid bundle prevents
initial daemon startup. If a bundle changes later, restart attempts refuse to execute it.
An occupied room port is reported as a failure instead of adopting an unrelated server.
Selecting a different manifest at the same path requires a daemon restart; restart attempts
remain pinned to the manifest verified when that daemon started.

On SIGINT/SIGTERM, `pulsed` stops accepting requests, sends SIGTERM to its child, waits up
to five seconds, then kills and reaps it if necessary. The unit uses a control group so
systemd also cleans up on daemon crashes. A live engine with failing health is reported as
unhealthy; it is not killed merely because health fails. Hang recovery/watchdogs are P3.

Without `PULSE_ENGINE_DIR`, the daemon remains usable as a standalone verified screen-bundle
server. Development mock routes still require `--dev`. The screen bundle in
`PULSE_BUNDLE_DIR` is separate from the executable engine bundle.

## Loopback health

`GET http://127.0.0.1:8787/health` now returns JSON (replacing P0's plain `ok`):

- daemon service/version, uptime, available disk bytes, and thermal-zone temperature where
  Linux exposes it (otherwise `null`);
- pinned Oneverse revision and Node version when an engine is configured;
- child state, PID, restart count, last exit and configured retry delay;
- the engine's pairing code, name, set, cloud state, session and epoch from its health
  endpoint, or an error with stale health cleared.

HTTP 503 means a configured engine is starting, stopped, backing off or not answering valid
health. HTTP 200 means it answers health, or that engine supervision was not configured.
This does **not** prove pairing, cloud connectivity, offline handback or event readiness.
`engine.health.cloud` and pairing state must be assessed separately. The probe goes directly
to loopback with a one-second timeout, no redirects and a 64-KiB body limit.

`PULSE_DAEMON_PORT` changes the port for development/tests (default 8787); the daemon always
binds to IPv4 loopback. `PULSE_ROOM_PORT` must be nonzero and available. Keep the room bound
to all interfaces or loopback so the local health probe can reach it.

## Checks

```sh
cargo build --locked -p pulsed
PULSE_TEST_ENGINE_DIR=/tmp/pulse-engine \
PULSE_TEST_BINARY="$PWD/target/debug/pulsed" \
python3 -m unittest discover -s tools/tests -v
```

The integration test starts the actual packaged engine against a local HTTP stand-in,
observes its PULSE pairing request and code through aggregate health, serves the real console,
kills the child to check degraded health and restart, checks shutdown reaps the child, and
rejects an incomplete bundle. It never calls production. Ordinary Python checks skip this
test unless a package is supplied; CI's engine job explicitly builds and tests the package.
That job pins Oneverse and Node and uploads the tested Debian 13 archive.

New direct Rust dependencies: `libc` and `reqwest`, both MIT OR Apache-2.0. Reqwest has its
default features disabled; it only probes the engine's local HTTP health endpoint.
