//! Built-in drum machine: 8 synthesised voices (no samples) driven by a 16-step sequencer.
//!
//! * Step data lives in the parameter store as 16-bit masks (`DR_T1..DR_T8`, bit 0 = step 1), so
//!   patterns are lock-free shared with the UI and are saved in `.scpreset` files.
//! * Runs on the audio thread: fixed arrays only, no allocation, bounded loops, NaN-safe voices.
//! * Tempo comes from the global `BPM`; swing and the shared Humanize controls (late timing, velocity
//!   spread) are applied per hit.

use crate::params::*;

pub const TRACKS: usize = 8;
pub const BLOCK: usize = 16;
pub const NAMES: [&str; TRACKS] = ["Kick", "Snare", "Clap", "Hat C", "Hat O", "Tom", "Rim", "Perc"];
/// Constant-power-ish static pan per track (-1..1).
const PAN: [f32; TRACKS] = [0.0, 0.0, 0.05, 0.25, -0.25, 0.15, -0.2, 0.3];

pub struct Pat { pub name: &'static str, pub t: [&'static str; TRACKS] }
/// 16-step patterns, tracks in `NAMES` order ('x' = hit).
pub static PATTERNS: [Pat; 16] = [
    Pat { name: "House",          t: ["x...x...x...x...", "................", "....x.......x...", "x...x...x...x...", "..x...x...x...x.", "................", "................", "................"] },
    Pat { name: "Techno",         t: ["x...x...x...x...", "................", "................", "x.xxx.xxx.xxx.xx", "..x...x...x...x.", "................", "...x.......x....", ".......x........"] },
    Pat { name: "Hip-Hop Boom Bap", t: ["x.....x...x.....", "....x.......x...", "................", "x.x.x.x.x.x.x.x.", "..............x.", "................", "................", "................"] },
    Pat { name: "Trap",           t: ["x.......x..x....", "........x.......", "........x.......", "x.x.xxx.x.x.xxxx", "......x.......x.", "................", "................", "................"] },
    Pat { name: "Rock",           t: ["x.......x.x.....", "....x.......x...", "................", "x.x.x.x.x.x.x.x.", "................", "................", "................", "................"] },
    Pat { name: "Disco",          t: ["x...x...x...x...", "....x.......x...", "....x.......x...", "x...x...x...x...", "..x...x...x...x.", "................", "................", "................"] },
    Pat { name: "Funk",           t: ["x..x..x...x..x..", "....x..x.x..x...", "................", "x.x.x.x.x.x.x.x.", "......x.......x.", "................", "................", "................"] },
    Pat { name: "Breakbeat",      t: ["x.x.......x.x...", "....x..x.x..x...", "................", "x.x.x.x.x.x.x.x.", "................", "................", "................", "................"] },
    Pat { name: "Drum & Bass",    t: ["x.........x.....", "....x.......x...", "................", "x.x.x.x.x.x.x.x.", "..............x.", "................", "................", "................"] },
    Pat { name: "Reggaeton",      t: ["x...x...x...x...", "...x..x....x..x.", "................", "x.x.x.x.x.x.x.x.", "................", "................", "................", "................"] },
    Pat { name: "Electro",        t: ["x.....x...x.....", "....x.......x...", "....x.......x...", "x.x.x.x.x.x.x.x.", "................", "................", "................", "..x.......x....."] },
    Pat { name: "80s Pop",        t: ["x.....x.x.......", "....x.......x...", "................", "x.x.x.x.x.x.x.x.", "................", "..............xx", "................", "................"] },
    Pat { name: "Lo-Fi",          t: ["x.....x...x.....", "....x.......x...", "................", "x.x.x.x.x.x.x.x.", "................", "................", "..x.....x.......", "................"] },
    Pat { name: "Afrobeat",       t: ["x..x..x...x..x..", "......x.......x.", "................", "x.x.x.x.x.x.x.x.", "................", "................", "..x..x..x..x..x.", "x.x..x.x..x.x..x"] },
    Pat { name: "Half-Time",      t: ["x.......x.......", "........x.......", "................", "x.x.x.x.x.x.x.x.", "..............x.", "................", "................", "................"] },
    Pat { name: "Minimal Clicks", t: ["x.......x.....x.", "................", "................", "..x...x...x...x.", "................", "................", "x.x..x..x.x.x...", ".......x........"] },
];

/// "x..x" -> bit mask (bit 0 = first character). Any char other than '.', ' ' and '-' counts as a hit.
pub fn mask(s: &str) -> u16 { s.chars().take(16).enumerate().fold(0u16, |m, (i, c)| if matches!(c, '.' | ' ' | '-') { m } else { m | (1 << i) }) }
pub fn pattern_masks(i: usize) -> [u16; TRACKS] { let p = &PATTERNS[i.min(PATTERNS.len() - 1)]; let mut m = [0u16; TRACKS]; for t in 0..TRACKS { m[t] = mask(p.t[t]); } m }

const TAU: f32 = std::f32::consts::TAU;
#[inline] fn xorshift(s: &mut u32) -> f32 { let mut x = *s; x ^= x << 13; x ^= x >> 17; x ^= x << 5; *s = x; (x >> 8) as f32 / 8388608.0 - 1.0 }
#[inline] fn fin(x: f32) -> f32 { if x.is_finite() { x } else { 0.0 } }

#[derive(Clone, Copy)]
struct Voice { on: bool, t: f32, vel: f32, ph: f32, ph2: f32, hp: f32, lp: f32, choke: f32, noise: u32 }
impl Voice {
    const fn new(seed: u32) -> Voice { Voice { on: false, t: 0.0, vel: 0.0, ph: 0.0, ph2: 0.0, hp: 0.0, lp: 0.0, choke: 1.0, noise: seed } }
    fn trigger(&mut self, vel: f32) { self.on = true; self.t = 0.0; self.vel = vel; self.ph = 0.0; self.ph2 = 0.0; self.hp = 0.0; self.lp = 0.0; self.choke = 1.0; }

    /// One sample of voice `k` (see NAMES). `tune` in -1..1, `dec` in 0..1.
    #[inline] fn tick(&mut self, k: usize, tune: f32, dec: f32, sr: f32) -> f32 {
        if !self.on { return 0.0; }
        let dt = 1.0 / sr;
        let t = self.t;
        let ratio = (tune * 0.7).exp2();
        let x = match k {
            0 => { // kick: sine with fast pitch drop + click
                let f = 50.0 * ratio * (1.0 + 5.0 * (-t * 38.0).exp());
                self.ph = (self.ph + f * dt).fract();
                let body = (self.ph * TAU).sin() * (-t / (0.10 + 0.55 * dec)).exp();
                let click = xorshift(&mut self.noise) * (-t * 700.0).exp() * 0.25;
                ((body + click) * 1.6).tanh() * 0.9
            }
            1 => { // snare: two body sines + high-passed noise
                self.ph = (self.ph + 185.0 * ratio * dt).fract(); self.ph2 = (self.ph2 + 330.0 * ratio * dt).fract();
                let tone = ((self.ph * TAU).sin() * 0.6 + (self.ph2 * TAU).sin() * 0.3) * (-t / 0.07).exp();
                let n = xorshift(&mut self.noise); let hp = n - self.lp; self.lp += 0.45 * (n - self.lp);
                tone * 0.7 + hp * (-t / (0.05 + 0.28 * dec)).exp() * 0.8
            }
            2 => { // clap: three fast noise bursts then a tail
                let n = xorshift(&mut self.noise); let bp = { let h = n - self.hp; self.hp += 0.3 * (n - self.hp); self.lp += 0.5 * (h - self.lp); self.lp };
                let burst = [0.0f32, 0.011, 0.022].iter().map(|&o| if t >= o { (-(t - o) * 260.0).exp() } else { 0.0 }).sum::<f32>();
                let tail = if t > 0.033 { (-(t - 0.033) / (0.07 + 0.3 * dec)).exp() } else { 0.0 };
                bp * (burst * 0.5 + tail) * 1.6
            }
            3 | 4 => { // hats: six detuned squares (metallic) + noise, high-passed
                const F: [f32; 6] = [263.0, 400.0, 421.0, 474.0, 587.0, 845.0];
                let mut s = 0.0;
                for (i, f) in F.iter().enumerate() { let ph = (t * f * 2.7 * ratio + i as f32 * 0.17).fract(); s += if ph < 0.5 { 1.0 } else { -1.0 }; }
                let n = xorshift(&mut self.noise);
                let raw = s * 0.12 + n * 0.5; let hp = raw - self.hp; self.hp += 0.55 * (raw - self.hp);
                let d = if k == 3 { 0.015 + 0.07 * dec } else { 0.12 + 0.55 * dec };
                if self.choke < 1.0 { self.choke *= 0.97; }
                hp * (-t / d).exp() * self.choke * 0.9
            }
            5 => { // tom
                let f = 110.0 * ratio * (1.0 + 0.6 * (-t * 18.0).exp());
                self.ph = (self.ph + f * dt).fract();
                (self.ph * TAU).sin() * (-t / (0.12 + 0.45 * dec)).exp() * 0.9
            }
            6 => { // rim shot: short two-tone click
                self.ph = (self.ph + 1700.0 * ratio * dt).fract(); self.ph2 = (self.ph2 + 480.0 * ratio * dt).fract();
                ((self.ph * TAU).sin() * 0.5 + (self.ph2 * TAU).sin() * 0.5 + xorshift(&mut self.noise) * 0.2) * (-t / (0.008 + 0.025 * dec)).exp()
            }
            _ => { // cowbell-style perc: two squares through a low-pass
                self.ph = (self.ph + 587.0 * ratio * dt).fract(); self.ph2 = (self.ph2 + 845.0 * ratio * dt).fract();
                let raw = (if self.ph < 0.5 { 1.0 } else { -1.0 }) + (if self.ph2 < 0.5 { 1.0 } else { -1.0 });
                self.lp += 0.28 * (raw - self.lp);
                self.lp * 0.4 * ((-t / 0.025).exp() * 0.6 + (-t / (0.06 + 0.3 * dec)).exp() * 0.4)
            }
        };
        self.t += dt;
        if self.t > 3.0 || (self.t > 0.02 && x.abs() < 1e-5 && self.choke < 0.01) { self.on = false; }
        if self.t > 0.5 && x.abs() < 1e-6 { self.on = false; }
        fin(x) * self.vel
    }
}

pub struct Drums {
    v: [Voice; TRACKS],
    step: u32, beat: f64, start: f64, running: bool, rnd: u32,
    pend: [(i32, u8, f32); 48], npend: usize,
    pub hits: [u32; TRACKS],
    pub cur_step: i32, // -1 when stopped (for the UI playhead)
}

impl Drums {
    pub fn new() -> Drums {
        Drums {
            v: std::array::from_fn(|i| Voice::new(0x9E3779B9u32.wrapping_mul(i as u32 + 7))),
            step: 0, beat: 0.0, start: 0.0, running: false, rnd: 0x1234567, pend: [(0, 0, 0.0); 48], npend: 0,
            hits: [0; TRACKS], cur_step: -1,
        }
    }

    pub fn trigger(&mut self, track: usize, vel: f32) {
        let track = track.min(TRACKS - 1);
        if track == 3 { self.v[4].choke = 0.999; } // closed hat chokes the open hat
        self.v[track].trigger(vel.clamp(0.0, 1.5));
        self.hits[track] = self.hits[track].wrapping_add(1);
    }

    #[inline] fn rnd01(&mut self) -> f32 { let mut x = self.rnd; x ^= x << 13; x ^= x >> 17; x ^= x << 5; self.rnd = x; (x >> 8) as f32 / 16777216.0 }

    /// Render one block of BLOCK samples (additive into `ol`/`or`). Advances the sequencer.
    pub fn process(&mut self, s: &[f32], bpm: f32, sr: f32, ol: &mut [f32; BLOCK], or: &mut [f32; BLOCK]) {
        let dt = (bpm.clamp(20.0, 300.0) as f64 / 60.0) * BLOCK as f64 / (sr.max(8000.0) as f64);
        let run = s[DR_RUN] >= 0.5;
        // humanized hits whose delay elapsed
        let mut i = 0;
        while i < self.npend {
            self.pend[i].0 -= BLOCK as i32;
            if self.pend[i].0 <= 0 { let (_, t, v) = self.pend[i]; self.pend.copy_within(i + 1..self.npend, i); self.npend -= 1; self.trigger(t as usize, v); } else { i += 1; }
        }
        if run {
            if !self.running { self.running = true; self.start = self.beat; self.step = 0; self.npend = 0; }
            let end = self.beat + dt;
            let swing = s[DR_SWING] as f64;
            let st = self.start;
            let t_of = move |n: u32| st + n as f64 * 0.25 + if n & 1 == 1 { swing * 0.125 } else { 0.0 };
            let mut fired = 0;
            while t_of(self.step) < end && fired < 4 {
                fired += 1;
                let bit = self.step % 16;
                self.cur_step = bit as i32;
                for k in 0..TRACKS {
                    let m = s[DR_T1 + k * 5] as u32 as u16;
                    if m & (1 << bit) == 0 || s[DR_M1 + k * 5] >= 0.5 { continue; }
                    let accent = if bit % 4 == 0 { 1.0 } else { 0.88 };
                    let mut vel = s[DR_V1 + k * 5] * accent;
                    let hv = s[HUM_VEL];
                    if hv > 0.0 { vel *= 1.0 + (self.rnd01() - 0.5) * 2.0 * hv * 0.35; }
                    let ht = s[HUM_TIME];
                    let delay = if ht > 0.0 { self.rnd01() * ht * 0.025 * sr } else { 0.0 };
                    if delay < BLOCK as f32 || self.npend >= self.pend.len() { self.trigger(k, vel); }
                    else { self.pend[self.npend] = (delay as i32, k as u8, vel); self.npend += 1; }
                }
                self.step = self.step.wrapping_add(1);
            }
            self.beat = end;
        } else {
            if self.running { self.running = false; self.npend = 0; }
            self.cur_step = -1;
            self.beat += dt;
        }
        let vol = s[DR_VOL] * s[DR_VOL] * 1.4;
        for k in 0..TRACKS {
            if !self.v[k].on { continue; }
            let (tune, dec) = (s[DR_TUNE1 + k * 5], s[DR_DEC1 + k * 5]);
            let (gl, gr) = (1.0 - PAN[k].max(0.0), 1.0 + PAN[k].min(0.0));
            for n in 0..BLOCK {
                // per-hit velocity (incl. the track fader) is baked into the voice at trigger time
                let x = self.v[k].tick(k, tune, dec, sr) * vol;
                ol[n] += x * gl; or[n] += x * gr;
            }
        }
    }
}

#[cfg(test)]
#[allow(unused_assignments)]
mod tests {
    use super::*;

    #[test]
    fn masks_and_patterns() {
        assert_eq!(mask("x...x...x...x..."), 0b0001_0001_0001_0001);
        assert_eq!(mask("................"), 0);
        assert_eq!(mask("xxxxxxxxxxxxxxxxxxxx"), 0xFFFF); // extra characters ignored
        for (i, p) in PATTERNS.iter().enumerate() {
            for t in 0..TRACKS { assert_eq!(p.t[t].len(), 16, "{} track {}", p.name, t); }
            assert!(pattern_masks(i).iter().any(|&m| m != 0));
        }
    }

    #[test]
    fn one_bar_triggers_expected_hits_and_stays_finite() {
        let mut s = default_patch();
        s[DR_RUN] = 1.0;
        let m = pattern_masks(0);
        for k in 0..TRACKS { s[DR_T1 + k * 5] = m[k] as f32; }
        let mut d = Drums::new();
        let (mut l, mut r) = ([0.0f32; BLOCK], [0.0f32; BLOCK]);
        let blocks = (2.0 * 48000.0 / 16.0) as usize - 1; // one bar @120 bpm, minus a hair
        let mut peak = 0.0f32;
        for _ in 0..blocks {
            l = [0.0; BLOCK]; r = [0.0; BLOCK];
            d.process(&s, 120.0, 48000.0, &mut l, &mut r);
            for x in l.iter().chain(r.iter()) { assert!(x.is_finite()); peak = peak.max(x.abs()); }
        }
        assert_eq!(d.hits[0], 4, "four kicks per house bar");
        assert_eq!(d.hits[2], 2, "two claps");
        assert!(peak > 0.1 && peak < 2.5, "peak {peak}");
        // muting and stopping
        s[DR_M1] = 1.0; s[DR_RUN] = 0.0;
        let before = d.hits[0];
        for _ in 0..2000 { l = [0.0; BLOCK]; r = [0.0; BLOCK]; d.process(&s, 120.0, 48000.0, &mut l, &mut r); }
        assert_eq!(d.hits[0], before, "no hits once stopped");
        assert_eq!(d.cur_step, -1);
    }

    #[test]
    fn every_voice_sounds_and_decays() {
        for k in 0..TRACKS {
            let mut d = Drums::new();
            d.trigger(k, 1.0);
            let s = default_patch();
            let (mut energy, mut tail) = (0.0f32, 0.0f32);
            for b in 0..(48000 * 4 / 16) {
                let (mut l, mut r) = ([0.0f32; BLOCK], [0.0f32; BLOCK]);
                d.process(&s, 120.0, 48000.0, &mut l, &mut r);
                let e: f32 = l.iter().map(|x| x * x).sum();
                if b < 3000 { energy += e } else if b > 9600 { tail += e }
            }
            assert!(energy > 0.01, "{} is silent", NAMES[k]);
            assert!(tail < 1e-6, "{} never decays", NAMES[k]);
        }
    }

    #[test]
    fn humanize_never_drops_or_duplicates_hits() {
        let mut s = default_patch();
        s[DR_RUN] = 1.0; s[HUM_TIME] = 1.0; s[HUM_VEL] = 1.0;
        let m = pattern_masks(3);
        for k in 0..TRACKS { s[DR_T1 + k * 5] = m[k] as f32; }
        let mut d = Drums::new();
        let (mut l, mut r) = ([0.0f32; BLOCK], [0.0f32; BLOCK]);
        for _ in 0..(48000 * 8 / 16) { l = [0.0; BLOCK]; r = [0.0; BLOCK]; d.process(&s, 120.0, 48000.0, &mut l, &mut r); }
        // 4 bars of trap: kicks per bar = 3
        assert!(d.hits[0] >= 11 && d.hits[0] <= 12, "kick hits {}", d.hits[0]);
    }
}
