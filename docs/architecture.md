# Architecture

This is the shape Oneverse Pulse is being built to. The direction was decided on 2026-10-22/23, in
Oneverse's `docs/oneverse-architecture.md` and `docs/live-session.md`. What exists today is said
in each part.

## What Pulse is for

The cloud (`api.oneverse.top`) runs a set's **live session**: the song and slide on the screens,
the metronome's timeline, and who may present. Phones, tablets, the operator's console and the
screens follow it.

Pulse is a mini PC at the venue. It follows that session too, keeps a copy of the set, and if the
venue loses the internet, it carries on as the room's authority:

1. **Online:** Pulse is a member of the set's session, signed in with its own token. It was paired
   in Oneverse's Screens like a screen, and is shown there as a Pulse. It mirrors the session.
   Pulse is on one set at a time: the set put on it, or the set of the event on now at a location
   it follows (#320).
2. **The cloud is gone for 10 s:** Pulse takes the session over, one epoch up
   (`authority: "pulse:<node id>"`). The room's devices reach it through **Continue on Pulse**, a
   link to Pulse's page on the venue's network that carries a pass the cloud signed earlier.
   Pulse checks that pass offline.
3. **The cloud is back:** Pulse hands its session and record back. The room's state wins. Then
   Pulse mirrors the cloud again.

Meanwhile Pulse drives the room's outputs: screens over HDMI, and later NDI, audio and MIDI/DMX.
The outputs follow the session whether the cloud or the room runs it.

## The parts

```text
                     Oneverse cloud (api.oneverse.top)
                                  │ wss /sync (Pulse's own token)
┌─────────────────────────────── mini PC ────────────────────────────────┐
│                                                                        │
│  live engine (Node, Oneverse's apps/pulse)                             │
│    pairing · cloud link · LiveEngine · set copy                        │
│    room server :8788 (venue LAN, passes) · Pulse's page (console)      │
│        ▲ loopback /sync (local credential, P2)                         │
│        │                                                               │
│  pulsed (Rust, this repository)                                        │
│    supervisor · health · bundles/updates · outputs                     │
│    ├─ HDMI: Cage + Chromium kiosk ─► screen page (Oneverse build)      │
│    ├─ NDI: offscreen render ─► NDI sender (P5)                         │
│    ├─ audio: click, cues, stems on its own clock (P6)                  │
│    └─ MIDI / DMX (later)                                               │
└────────────────────────────────────────────────────────────────────────┘
```

### The live engine (Oneverse, `apps/pulse`)

TypeScript in a Node process, because it must decide exactly what the cloud decides. It runs
`@oneverse/core`'s `LiveEngine`, the reducer the cloud runs, and is tested against the same
conformance cases.

What it does today is in its README. It pairs, mirrors, takes over and hands back. It copies the
set to disk, runs the room's `/sync` with passes, and serves the console as Pulse's page.

It has been tested in Oneverse's CI against the API only. It has never run on the mini PC.

This repository installs, starts and watches the engine. It never reimplements it.

### pulsed (Rust, this repository)

`pulsed` is the box's daemon, renamed from the prototype `songverse-player` in P0.
Today it serves `/health` and verified bundles on loopback; the mock screen and navigation
require `--dev`. Supervision, aggregate health and the outputs below are still the target shape.

- **Supervisor:**
  - starts the engine (a pinned Node runtime and a pinned `@oneverse/pulse` build) and the kiosk
    with the right environment, and restarts them if they die;
  - runs under systemd, with a watchdog.
- **Health:** a loopback API that gathers the engine's `GET /health` with the box's own state
  (disk, temperature, outputs, versions). Its pairing code is what the screen shows until Pulse is
  paired.
- **Bundles and updates:**
  - installs what Oneverse builds (the engine, Pulse's page, the screen page) as verified bundles,
    pinned to one Oneverse commit, and switches between them atomically;
  - signed releases and rollback come later. The prototype's `offline.rs` is the start of this.
- **Outputs.** Each follows the session as the room's `/sync` tells it, through one loopback
  client in `pulsed`:
  - **HDMI:** Cage and Chromium in kiosk mode, showing the screen page. Chromium renders, and the
    GPU drives the connector. No frame capture.
  - **NDI (#196):** the same screen page rendered offscreen, its frames sent with the NDI SDK. Pulse
    sends NDI only and never takes it in. A Screen's camera or capture goes to a video mixer
    instead. Oneverse Screen apps can show Pulse's NDI output.
  - **Audio (#62):** a native engine (click, count-ins, spoken cues, backing tracks, stems) that
    plays the session's timeline ("beat N at time T") on the box's clock. It never runs on network
    ticks.
  - **MIDI and lighting:** later, and never at the cost of audio or screen timing.

### Instruments and plugin control (planned P7)

After P6's synchronized audio and LTC foundation, [P7](plugin-hosting.md) adds supervised native
engine workers with common parameter metadata, sound presets, rack snapshots and song/section
scenes. Oneverse owns their schemas, web UI, permissions and session decisions; Rust owns native
DSP/MIDI, local device mapping and scheduled execution. Audio stays on Pulse, with cached state
for offline recall. The browser controls it through the authenticated room, not an exposed host
socket. Host selection, supported Linux plugins and latency budgets remain qualification work.

### The screen page (Oneverse)

What every output shows is Oneverse's `ScreenStage` (`packages/screen`), the same as on any screen.
On Pulse it needs a page that:

- loads from Pulse;
- joins the room's `/sync` as a screen of Pulse's own;
- reads the set from Pulse's copy.

That page doesn't exist yet. Oneverse's `apps/web` player-screen build reads the prototype's mock
`/api/state` instead. The page is built in Oneverse (roadmap P2), and the mock contract is then
removed from both repositories.

## Who listens where

| Listener | Address | Who reaches it | How |
| --- | --- | --- | --- |
| Engine: room server and Pulse's page | `0.0.0.0:8788` | The venue's devices | A pass the cloud signed |
| Engine: `/health` | Same port | Anyone on the LAN (no secrets in it) | None |
| `pulsed` API | `127.0.0.1` only | The box itself | None, loopback only |
| Chromium | Not listening | Nobody | No remote debugging, ever |

## Principles

- One session, decided in one place: Rust follows it and never keeps its own.
- One renderer: Oneverse's `ScreenStage`.
- Timing on the box's clock: audio and transitions never wait on the network.
- Every step recovers on its own. A crashed process is restarted, a restart restores where the
  session was, and a dropped cloud is retried.
- Bounded disk: sets, media and bundles are verified by checksum and pruned.
- Say what was verified, and on what hardware.
