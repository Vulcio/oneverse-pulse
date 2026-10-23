# Roadmap

This is the order of work for Oneverse Pulse, in the shape of [architecture.md](architecture.md).
Each step says:

- what it delivers;
- what it needs from Oneverse first;
- how to know it's done.

Take the steps in order: each one rests on the one before. Give each step an issue (in Oneverse or
here) before starting, and tick its boxes here in the commit that delivers them.

The requirements are in Oneverse's issues: [#247](https://github.com/Vulcio/Oneverse/issues/247)
(on-site runtime), [#62](https://github.com/Vulcio/Oneverse/issues/62) (audio),
[#196](https://github.com/Vulcio/Oneverse/issues/196) (NDI) and
[#320](https://github.com/Vulcio/Oneverse/issues/320) (pairing, locations). Also related: #195
(offline screen) and #13 (sync play).

## Done so far (prototype)

- [x] Cargo workspace, mock set and navigation, mock screen
- [x] SHA-256-verified bundle server (`crates/pulsed/src/offline.rs`)
- [x] Oneverse's real `ScreenStage` built client-only, packaged and served. It was seen in Chromium
      on a laptop on 2026-10-20, against the mock set.
- [x] Cage + Chromium kiosk and systemd units, written but **not run on the hardware**

In Oneverse (`apps/pulse`, tested in its CI against the API, never on the mini PC):

- pairing as a Pulse;
- following a set or a location;
- mirror, takeover and handback;
- the set's copy;
- the room's server with passes;
- Pulse's page (the console);
- "Continue on Pulse".

## P0 - One name, no mocks in the way

Tracked in [#1](https://github.com/Vulcio/oneverse-pulse/issues/1).

- [x] Rename `songverse-player` to `oneverse-pulse`, as one change:
  - crates `pulse-core` and `pulsed`, the binary `pulsed`;
  - units `oneverse-pulse.service` and `oneverse-pulse-kiosk.service`;
  - user `pulse`, paths `/var/lib/oneverse-pulse` and `/etc/oneverse-pulse`, variables `PULSE_*`.
  - The old `SONGVERSE_*` variables are still read for one release, with a warning.
- [x] Mark the mock set, mock screen and `/api/state` as development-only (behind a `--dev` flag).
      Keep the bundle server.
- [x] CI also checks the shell scripts (`sh -n`) and builds a release binary for `x86_64` Debian
      stable.

**Done when:** CI is green, and `pulsed --dev` runs what `songverse-player` ran.

## P1 - The engine runs on the box (first run on the mini PC)

Tracked in [#2](https://github.com/Vulcio/oneverse-pulse/issues/2). Software is exercised with
a local stand-in cloud; P1 remains open until the mini-PC acceptance run below.

- [x] `pulsed` starts the engine as a child process and restarts it with backoff:
  - from a pinned Node runtime and an installed `@oneverse/pulse` build (`dist/` plus production
    `node_modules`, or one bundled file);
  - with its environment (see AGENTS.md);
  - its stdout goes to the journal.
- [x] `pulsed`'s loopback API: `GET /health` gathers the engine's `/health` (pairing code, set,
      cloud, session, epoch) with versions, uptime, disk and temperature.
- [x] `tools/package-engine.sh`: builds `@oneverse/pulse` and `build:pulse` from an Oneverse
      checkout into a verified bundle, the same way as the screen bundle (manifest, source revision).
- [x] `appliance/`: archive builder, installer and install steps for Node, the engine bundle
  and the `pulsed` service. The privileged installer itself still needs the hardware run.
- [ ] **On the mini PC**, the manual run below works, and the result is written in the README with
      the hardware's model:
  1. pair the box from Screens;
  2. put it on a set;
  3. open "Continue on Pulse" from a phone with the box's Ethernet unplugged from the internet;
  4. present a slide;
  5. plug it back in, and see the handback in Live.

**Needs from Oneverse:** #320 merged (the engine pairs itself).

**Done when:** that run has been done on the hardware, and an automated test here starts a real
engine (from a built Oneverse checkout) against a stand-in cloud, and sees it ask for a pairing code.

## P2 - The room's screens, offline (HDMI)

**First in Oneverse (its own issue, contracts written in both repositories):**

- [ ] A local credential for the box's own clients:
  - the engine writes a random key to `PULSE_DATA_DIR` (mode `0600`) at start;
  - a client on loopback enters the room with `{type: "enter", local: <key>}`;
  - it is accepted from `127.0.0.1` only;
  - it gives a read-only screen's rights.
- [ ] Pulse's screen page, built by the screen app next to `build:pulse`. It:
  - joins the room as a screen;
  - reads the set from the room (`GET /set`);
  - renders it with `ScreenStage` and the theme's files from the copy;
  - shows the pairing code while the box isn't paired.
  - An e2e suite like `web/pulse-console` covers it.
- [ ] The theme's and the set's files (images, fonts, videos) in the engine's copy. They are fetched
      with checksums and capped on disk.
- [ ] Retire the player-screen build and its `/api/state` contract in Oneverse
      (`apps/web/src/player-screen`, `e2e/web/player-screen.test.mjs`) once the new page replaces
      it.

**Then here:**

- [ ] `pulsed` installs the screen page as a bundle, serves it on loopback, and starts the kiosk on
      it.
- [ ] One kiosk per connected HDMI output, each its own screen.
- [ ] Remove the mock screen and `crates/pulse-core`'s mock state.
- [ ] **On the mini PC:** a screen on HDMI follows the set online, keeps following it with the
      internet unplugged, and survives a reboot in the middle of it (same song, same slide).

**Done when:** the reboot test passes on the hardware, and is written down.

## P3 - A box that looks after itself

- [ ] A systemd watchdog for `pulsed`. `pulsed` restarts the engine and the kiosks, and reports
      what it restarted, and why, in `/health`.
- [ ] Disk policy: old bundles and sets pruned, with a floor of free space kept.
- [ ] Logs bounded in the journal. A "send diagnostics" bundle, with no secrets in it.
- [ ] The Debian install as a script, then as an image (debos or mkosi): unattended, reproducible,
      with a recovery path documented.

**Done when:** a fresh box goes from the image to "showing a pairing code" with no keyboard
attached.

## P4 - Updates that can't brick a show

- [ ] Releases signed (minisign or Ed25519), and checked by `pulsed` before a bundle is installed.
- [ ] A/B install: the new release runs, reports healthy, and only then becomes current. Otherwise
      it rolls back.
- [ ] Never update during an event. The box checks its session and the location's events first.
- [ ] Where updates come from, and who may publish them: designed in an issue first.

## P5 - NDI output (#196)

- [ ] **Licence:** read the NDI SDK's terms, write down what they allow for an AGPL project
      distributed on a box, and get brice's yes before linking anything. NDI is a Cargo feature,
      off by default.
- [ ] The screen page rendered offscreen (headless Chromium, or a compositor's output), its frames
      sent as an NDI source named after the box and the screen. Alpha is kept where the theme has
      it.
- [ ] The NDI outputs chosen in Oneverse's Screens like any screen. That is a contract to agree
      there.

**Done when:** an NDI receiver (Oneverse Screen or a video mixer) shows the box's output, frame
pacing is measured, and the results are written with the hardware's model.

## P6 - Audio (#62)

- [ ] An audio engine in Rust (cpal or JACK/PipeWire, chosen in the issue): click, count-in and
      spoken cues, scheduled on the box's audio clock from the session's timeline.
- [ ] Backing tracks and stems from the set's copy, routed to the interface's channels.
- [ ] Latency and drift measured against a phone running Songverse's metronome in the same session,
      and written down.

## Later

MIDI (routing, program changes on song change), lighting (DMX, Art-Net, sACN cues on slides or
sections), and more than one Pulse in a room. Each gets its own issue first, and none may cost
audio or screen timing.
