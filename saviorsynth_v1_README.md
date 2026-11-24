# saviorsynth_v1 — wavetable synthesizer (Rust + FLTK)

A standalone wavetable synth for **Windows (x64)** and **Linux (x86_64)**.
Single self-contained executable, no runtime dependencies besides the system audio/GUI libraries
(WASAPI/Win32 on Windows, ALSA/X11 on Linux). Not a VST/AU plugin — it opens its own window and audio device.

> Independent project. The 80 factory presets are
> original patches that recreate the *sound archetypes* popular wavetable synths are known for (supersaw,
> reese, wobble, 303 acid, pluck, e-piano, riser, …).

## Quick start

| Platform | Run |
|---|---|
| Windows 10/11 x64 | double-click `saviorsynth_v1_windows_x64.exe` |
| Linux   | `chmod +x saviorsynth_v1_linux_x86_64 && ./saviorsynth_v1_linux_x86_64` (needs the usual desktop libraries: X11, Xinerama/Xcursor/Xfixes, pango, cairo, ALSA — present on any desktop distro; binary needs glibc 2.38+ = Ubuntu 24.04, Debian 13, Fedora 39 or newer; on an older distro just run `./build_all.sh`) |

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

## Presets (80)

Init (3) · Bass (10: Reese, Wobble, Sub, Acid 303, FM Growl, Pluck, Rubber, Dubstep Growl, 808 Sub Boom, Wide Detuned) ·
Lead (10: Supersaw, Sync, Screamer, Square, Hoover, Portamento, Trance, EDM Brass, Chiptune, Vocal) ·
Pad (9) · Pluck (8) · Keys (8: E-Piano, Drawbar Organ, Clav, Tubular Bell, Vibraphone, …) · Stab (5) · Brass (2) ·
Gate (2) · FX (7: Noise Riser, Downlifter, Laser Zap, Alien Whale, Wind, Impact Boom, Sci-Fi Sweep) ·
Wavetables (16, one showcase patch per wavetable).
`Save` / `Load` write plain-text `.scpreset` files (loaded files appear under the *User* category). `Random` makes a musical random patch.

## Command line

```
saviorsynth_v1 --list                         list presets
saviorsynth_v1 --render "supersaw" out.wav    render one preset to a 16-bit WAV (--note 48 --seconds 4)
saviorsynth_v1 --render all outdir/           render every preset
saviorsynth_v1 --selftest                     headless regression test: NaN / silence / clipping / CPU for all presets
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
| `src/presets.rs` | preset DSL + 80 factory presets + defensive file parser |
| `src/audio.rs`, `src/midi.rs` | cpal output stream, midir input (optional feature `midi`) |
| `src/ui.rs` | FLTK interface (custom knobs/sliders/views/keyboard) |
| `src/offline.rs` | WAV export + self-test |

Real-time rules: the audio callback never locks, allocates or does I/O; the UI and MIDI threads talk to it only through
atomics and a bounded lock-free queue.

## Versioning

The file/version tag lives in `Cargo.toml` (`name = "saviorsynth_vN"`); the window title, `--help` and preset headers are
derived from it. `tools/bump_version.sh N` rewrites every reference (including this README's name) and renames companion files.
