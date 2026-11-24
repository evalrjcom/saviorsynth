# saviorsynth_v4 — wavetable synthesizer (Rust + FLTK)

A standalone wavetable synth with one-key chords, progressions and an arpeggiator for **Windows (x64)** and **Linux (x86_64)**.
Single self-contained executable, no runtime dependencies besides the system audio/GUI libraries
(WASAPI/Win32 on Windows, ALSA/X11 on Linux). Not a VST/AU plugin — it opens its own window and audio device.

> Independent project. The 297 factory presets are
> original patches that recreate the *sound archetypes* popular wavetable synths are known for (supersaw,
> reese, wobble, 303 acid, pluck, e-piano, riser, …).

## Quick start

| Platform | Run |
|---|---|
| Windows 10/11 x64 | double-click `saviorsynth_v4_windows_x64.exe` |
| Linux   | `chmod +x saviorsynth_v4_linux_x86_64 && ./saviorsynth_v4_linux_x86_64` (needs the usual desktop libraries: X11, Xinerama/Xcursor/Xfixes, pango, cairo, ALSA — present on any desktop distro; binary needs glibc 2.38+ = Ubuntu 24.04, Debian 13, Fedora 39 or newer; on an older distro just run `./build_all.sh`) |

Play with the on-screen keyboard (mouse; drag to glissando; click height = velocity), the computer keyboard,
or a MIDI keyboard (first MIDI port is auto-connected; choose another in the top-right menu).

**Computer keyboard:** `A W S E D F T G Y H U J K O L P ;` = one octave + a bit (white keys on the home row,
black keys above). `Z` / `X` = octave down / up (shown in the window title).
Double-click any knob to reset it · drag = change · **Shift+drag** = fine · mouse wheel = step · Panic button = all notes off.

## Sound engine

* **2 wavetable oscillators (A, B)** — 16 wavetables × 32 frames, band-limited (10 mip levels per frame, so no aliasing
  on high notes), per-osc: WT position, **warp** (Sync, Bend+, Bend−, PWM, Mirror, Quantize, FM / AM / Ring from the other osc),
  octave, semitone, fine, **unison 1–7** with detune / blend / stereo spread, start phase + randomisation, level, pan.
* **Sub oscillator** (sine/tri/square/saw, PolyBLEP) and **noise** (white/pink/brown).
* **Filter**: TPT state-variable, LP12 / LP24 / HP12 / HP24 / BP / Notch, resonance, drive, key-tracking, mix,
  dedicated Env-2 amount, per-source routing (A / B / Sub / Noise).
* **3 envelopes** (A-H-D-S-R; Env 1 = amp) and **4 LFOs** (7 shapes incl. smooth-random and S&H, free or tempo-synced
  1/1 … 1/16 incl. triplets and dotted, retrigger or free-run, fade-in).
* **Mod matrix, 8 slots**: sources Env 2/3, LFO 1–4, Velocity, Key, Mod wheel, Random → destinations A/B position, A/B warp,
  A/B pitch, A/B/Sub/Noise level, cutoff, resonance, drive, unison detune, pan, amp level (bipolar amounts).
* **Voice modes**: Poly (1–16 voices, quietest/oldest stealing), Mono, Legato, with portamento; pitch-bend range, sustain pedal.
* **FX chain**: distortion (soft, hard, fold, tube, bitcrush, decimate) → stereo chorus → ping-pong delay (free or synced)
  → reverb → DC blocker → soft limiter.

## Presets (297)

Init (3) · Bass (10: Reese, Wobble, Sub, Acid 303, FM Growl, Pluck, Rubber, Dubstep Growl, 808 Sub Boom, Wide Detuned) ·
Lead (10: Supersaw, Sync, Screamer, Square, Hoover, Portamento, Trance, EDM Brass, Chiptune, Vocal) ·
Pad (9) · Pluck (8) · Keys (8: E-Piano, Drawbar Organ, Clav, Tubular Bell, Vibraphone, …) · Stab (5) · Brass (2) ·
Gate (2) · FX (7: Noise Riser, Downlifter, Laser Zap, Alien Whale, Wind, Impact Boom, Sci-Fi Sweep) ·
Wavetables (16, one showcase patch per wavetable).

**Vintage banks (60)** — approximations of the *character* of classic hardware (no samples, no vendor presets):
- **70s Synths (29)** — Minimoog lead/bass/brass, ARP Odyssey sync + ring-mod, ARP 2600 drone, ARP String Ensemble, Moog modular sweep, Prophet-5 brass/pad, Oberheim SEM pad, CS-80 brass/pad, Polymoog Vox Humana, Mellotron strings/flute/choir, Korg MS-20 screamer/bass, Jupiter-4 strings, EMS VCS3 blips, Clavinet, Rhodes, Wurlitzer, Hammond B3, Pro-One, Taurus pedal, SH-5 …
- **80s Synths (31)** — DX7 e-piano/slap bass/marimba/bell/brass, Juno-60/106 pads and strings, Jupiter-8 brass/strings/sweep/arp pluck, OB-Xa stab, OB-8 brass pad, Prophet-600 pluck, Prophet VS vector pad, PPG glass pad + digital bell, Fairlight orchestra stab + ArcoPad, Synclavier pluck, D-50 pads, Korg M1 organ, Casio CZ bass/bell, SH-101, TB-303, Memorymoog, Emulator II strings …

**Hit Sounds (20)** — original patches approximating the signature *synth timbre* of famous chart-topping records
(e.g. "Jump-style Brass", "Billie Jean-style Bass", "Sweet Dreams-style Pulse Bass", "Take On Me-style Synth Lead",
"I Feel Love-style Sequence", "Blinding Lights-style Synth" …). Only the sound character is imitated: no melodies, lyrics,
samples or recordings are included, and the project is not affiliated with the artists.

**Playable banks (53)** — **Chords (17)**, **Progressions (14)** and **Arps (22)**, ready to play with one finger (see below).

**Style banks (72)** — original patches recreating the sound archetypes the most popular soft synths are known for.
They are *not* copies of those products' factory presets (which are proprietary); each is built from scratch on this engine.
- **Syl Style (25)** — Sylenth1-type subtractive saw/pulse stacks: trance gate lead, big-room saw, euphoric pluck arp, hoover, rave stab, house chord stab, uplifter/downlifter, acid, psy bass, 80s poly brass, …
- **Diva Style (24)** — warm analog-modelled voices: ladder-Moog bass, Taurus pedal, Minimoog lead, Jupiter brass/strings, Juno chorus pad + sub bass, Prophet sweep, Oberheim stack, ARP screamer, CS-style brass, PWM strings, clav, Rhodes, Vangelis swell, …
- **Serum Style (23)** — wavetable-first modern sound design: neuro growl, talking bass (formants), riddim, future-bass chord, vocal chop, trap bell, glitch lead, screech, evolving spectral pad, cinematic drone, lo-fi keys, wavetable pluck, FM glass arp, ring-mod robot, 808 glide, riser/noise FX, …
`Save` / `Load` write plain-text `.scpreset` files (loaded files appear under the *User* category). `Random` makes a musical random patch.

## Chords, progressions and the arpeggiator

The bottom panels sit between your keys and the voices. Play **one key** and the synth plays the rest.

**CHORDS & PROGRESSIONS** (`Mode`)
| Mode | What one key does |
|---|---|
| Off | normal playback |
| One-Key | the key is the chord **root**; `Quality` = Major / Minor / Dim / Aug |
| Degree | the key is the **tonic**; the chord is scale degree **I, II, III, IV, V, VI or VII** of the chosen `Scale` (diatonic quality) |
| Progression | the key is the tonic; the chord **steps through a progression** (I-IV-V-I, I-V-VI-IV, II-V-I, I-VI-IV-V, VI-IV-I-V, **I-II-IV-I**, I-IV-II-V, I-III-IV-V, I-II-IV-V, I-VII-VI-V, or **Custom** steps 1-4) every `Step Length` (2 bars / 1 bar / 1/2 bar / 1 beat, tempo from `BPM`) |

`Scale`: Major, Minor, Dorian, Mixolydian, Harmonic Minor. `Variation`: Triad, Sus2, Sus4, 6th, 7th (diatonic), Dom 7, 9th, Add9, Power (5th), Octave.
`Inversion`: Root / 1st / 2nd / 3rd. `Open` = drop-2 voicing. `Bass` = add the chord root an octave below.
The detected chord name (e.g. *Fmaj7*) is shown in the panel while a chord plays. In One-Key mode, several held keys each play their own chord.

**ARPEGGIATOR** — works on held keys *or* on the generated chords/progression.
- `Pattern`: Up, Down, Up-Down, Down-Up, Up+Down, Played (key order), Random, Converge, Diverge, Thumb Up, Pinky Up, Stairs, Alberti, Chord Pulse.
- `Rate`: 1/4, 1/8, 1/16, 1/32, triplets 1/8T, 1/16T, dotted 1/8., 1/16.   `Octaves`: 1-4.   `Gate`, `Swing`, `Accent`, `Latch`.
- `Inversion Cycle` (**inverted chord shapes**): Fixed, Cycle Up, Cycle Down or Ping-Pong — each completed pattern cycle moves the chord to its next inversion.
- The stage runs on the audio thread with fixed-size buffers (no allocation); toggling it mid-note never leaves stuck notes.

**TEMPO** — one global `BPM` (20-300) drives the arpeggiator, progressions, drum machine and looper. Set it with the BPM
slider (in the Chords and Arpeggiator panels and on the Drums page) or with **Tap** (average of your last 6 taps).

**HUMANIZE** (Arpeggiator panel, all default to 0 = off) — makes the machine-perfect stage feel played:
`Time` = notes land slightly late / strummed (up to ~35 ms for arp and chords, ~25 ms for drums),
`Velocity` = random velocity variation (up to about ±30%), `Gate` = random note-length variation.
It applies to arp notes, chord strums (one key -> chord notes arrive spread out) and drum hits. Delayed notes are cancelled
cleanly on key release, so humanizing never causes hanging notes.

## Drum machine and looper (page **DRUMS + LOOP**)

Switch pages with the `SYNTH` / `DRUMS + LOOP` buttons at the top; the chord/arp panels and keyboard stay visible on both pages.

**Drum machine** — 8 synthesized voices (Kick, Snare, Clap, Closed Hat, Open Hat, Tom, Rim, Perc), no samples.
- 16-step grid (one bar): click a step to toggle it, drag to paint. Clicking also auditions the sound. The playhead follows the transport.
- `RUN` starts/stops (runs in sync with the audio clock at `BPM`). `Swing` delays every second 16th. `Level` = drum bus.
- Per track: `M` mute, level slider; **Drum Sounds** panel: `Tune` and `Decay`. The closed hat chokes the open hat.
- `Load pattern`: House, Techno, Hip-Hop Boom Bap, Trap, Rock, Disco, Funk, Breakbeat, Drum & Bass, Reggaeton, Electro, 80s Pop, Lo-Fi, Afrobeat, Half-Time, Minimal Clicks (or clear all steps).

**Looper** — records the **live mix** (synth + drums, before the loop itself) so you can build up layers.
| Button | Action |
|---|---|
| REC / DUB | Empty: start recording now. Recording: close the loop and start playing. Playing: overdub. Overdubbing: back to play |
| PLAY / STOP | start / stop playback |
| UNDO DUB | removes all overdubs, keeps the first take (one level) |
| CLEAR | empties the loop |
`Length`: **Free** (press REC again to set the length) or 1 / 2 / 4 / 8 bars (length computed from `BPM`; recording closes by itself).
Maximum loop 32 s; buffers are allocated once at start (about 24 MB at 48 kHz), nothing is allocated on the audio thread.
Loop seams are cross-faded (128 samples). `Loop Level` sets playback volume.

**Beats presets (12)** — one-click demos: a sound + one-key progression or arpeggiator + a running drum pattern. Loading a *Beats*
or *User* preset replaces the session (tempo, drum pattern); other categories keep your BPM, drum, looper and humanize settings.

## Command line

```
saviorsynth_v4 --list                         list presets
saviorsynth_v4 --render "supersaw" out.wav    render one preset to a 16-bit WAV (--note 48 --seconds 4)
saviorsynth_v4 --render all outdir/           render every preset
saviorsynth_v4 --selftest                     headless regression test: NaN / silence / clipping / CPU for all presets
```

## Building

`./build_all.sh` builds both binaries into `dist/` (prerequisites are listed at the top of the script).
Cross-compiling for Windows uses mingw-w64 (POSIX threads); FLTK is compiled from source for that target and linked statically.

## Source layout

| file | role |
|---|---|
| `src/params.rs` | **generated** (`tools/gen_params.py`) parameter table, enum labels, lock-free atomic store |
| `src/wavetable.rs` | in-house FFT, 16 wavetable generators, mip-mapped band-limited bank |
| `src/engine.rs` | voices, oscillators/unison/warp, filter, envelopes, LFOs, mod matrix, voice allocation, render loop |
| `src/fx.rs` | distortion, chorus, delay, reverb, DC blocker |
| `src/presets.rs`, `src/presets/banks*.rs` | preset DSL + 297 factory presets + defensive file parser |
| `src/audio.rs`, `src/midi.rs` | cpal output stream, midir input (optional feature `midi`) |
| `src/ui.rs` | FLTK interface (custom knobs/sliders/views/keyboard) |
| `src/perform.rs` | one-key chords, scale degrees, progressions, arpeggiator patterns + inversion cycling (real-time safe, unit-tested) |
| `src/drums.rs` | drum synthesis (8 voices), 16-step sequencer, 16 patterns, swing/humanize (real-time safe, unit-tested) |
| `src/looper.rs` | audio looper: record / play / overdub / undo / clear, bar-synced length, seam fades |
| `src/offline.rs` | WAV export + self-test |

Real-time rules: the audio callback never locks, allocates or does I/O; the UI and MIDI threads talk to it only through
atomics and a bounded lock-free queue.

## Versioning

The file/version tag lives in `Cargo.toml` (`name = "saviorsynth_vN"`); the window title, `--help` and preset headers are
derived from it. `tools/bump_version.sh N` rewrites every reference (including this README's name) and renames companion files.
