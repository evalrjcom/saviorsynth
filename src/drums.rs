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

/// Kit character tables. All three kits are synthesized from scratch (no samples): they model the
/// *character* of the classic analog (808), digital/PCM (707) and hybrid (909) drum machines.
#[cfg(test)]
pub const KIT_NAMES: [&str; 3] = ["707 Kit", "808 Kit", "909 Kit"];
struct Kit {
    k_f: f32, k_drop: f32, k_rate: f32, k_dec: f32, k_decr: f32, k_click: f32, k_drive: f32,
    s_f1: f32, s_f2: f32, s_tone: f32, s_mix: f32, s_nlp: f32, s_ndec: f32, s_ndecr: f32,
    c_gap: f32, c_n: u8, c_rate: f32, c_tail: f32, c_tailr: f32, c_lp: f32,
    h_mul: f32, h_sq: f32, h_noise: f32, h_hp: f32, h_cd: f32, h_cdr: f32, h_od: f32, h_odr: f32,
    t_f: f32, t_drop: f32, t_rate: f32, t_dec: f32, t_decr: f32, t_click: f32,
    r_f1: f32, r_f2: f32, r_dec: f32, r_decr: f32, r_noise: f32,
    p_f1: f32, p_f2: f32, p_lp: f32, p_d1: f32, p_d2: f32, p_dr: f32,
    bits: u32, hold: u32, // lo-fi: bit depth (0 = off) and sample-hold factor (1 = off)
}
const HAT_F: [f32; 6] = [205.3, 304.4, 369.6, 522.7, 540.0, 800.0];
static KITS: [Kit; 3] = [
    // 707: digital, short, bright and slightly grainy (8-bit, sample-held)
    Kit { k_f: 58.0, k_drop: 2.8, k_rate: 40.0, k_dec: 0.09, k_decr: 0.30, k_click: 0.15, k_drive: 1.6,
          s_f1: 200.0, s_f2: 360.0, s_tone: 0.05, s_mix: 0.5, s_nlp: 0.5, s_ndec: 0.05, s_ndecr: 0.18,
          c_gap: 0.012, c_n: 3, c_rate: 280.0, c_tail: 0.05, c_tailr: 0.2, c_lp: 0.55,
          h_mul: 3.0, h_sq: 0.10, h_noise: 0.5, h_hp: 0.65, h_cd: 0.018, h_cdr: 0.05, h_od: 0.14, h_odr: 0.4,
          t_f: 150.0, t_drop: 0.5, t_rate: 20.0, t_dec: 0.10, t_decr: 0.3, t_click: 0.1,
          r_f1: 1500.0, r_f2: 440.0, r_dec: 0.005, r_decr: 0.02, r_noise: 0.15,
          p_f1: 560.0, p_f2: 830.0, p_lp: 0.4, p_d1: 0.02, p_d2: 0.07, p_dr: 0.25, bits: 8, hold: 2 },
    // 808: analog, long boomy kick, tonal snare, metallic square hats, cowbell
    Kit { k_f: 46.0, k_drop: 2.2, k_rate: 22.0, k_dec: 0.28, k_decr: 1.1, k_click: 0.06, k_drive: 1.2,
          s_f1: 180.0, s_f2: 330.0, s_tone: 0.09, s_mix: 0.8, s_nlp: 0.35, s_ndec: 0.08, s_ndecr: 0.3,
          c_gap: 0.010, c_n: 4, c_rate: 200.0, c_tail: 0.09, c_tailr: 0.35, c_lp: 0.5,
          h_mul: 2.7, h_sq: 0.13, h_noise: 0.3, h_hp: 0.5, h_cd: 0.03, h_cdr: 0.08, h_od: 0.22, h_odr: 0.6,
          t_f: 100.0, t_drop: 0.5, t_rate: 14.0, t_dec: 0.2, t_decr: 0.5, t_click: 0.0,
          r_f1: 1700.0, r_f2: 480.0, r_dec: 0.006, r_decr: 0.02, r_noise: 0.1,
          p_f1: 540.0, p_f2: 800.0, p_lp: 0.35, p_d1: 0.03, p_d2: 0.1, p_dr: 0.35, bits: 0, hold: 1 },
    // 909: punchy driven kick with a click, snappy noisy snare, crisp hats, bright ride-bell perc
    Kit { k_f: 55.0, k_drop: 4.0, k_rate: 40.0, k_dec: 0.12, k_decr: 0.5, k_click: 0.35, k_drive: 2.0,
          s_f1: 190.0, s_f2: 340.0, s_tone: 0.06, s_mix: 0.55, s_nlp: 0.5, s_ndec: 0.06, s_ndecr: 0.25,
          c_gap: 0.008, c_n: 4, c_rate: 300.0, c_tail: 0.06, c_tailr: 0.25, c_lp: 0.6,
          h_mul: 3.4, h_sq: 0.08, h_noise: 0.6, h_hp: 0.72, h_cd: 0.02, h_cdr: 0.06, h_od: 0.18, h_odr: 0.5,
          t_f: 130.0, t_drop: 0.9, t_rate: 25.0, t_dec: 0.12, t_decr: 0.35, t_click: 0.3,
          r_f1: 1900.0, r_f2: 520.0, r_dec: 0.007, r_decr: 0.02, r_noise: 0.3,
          p_f1: 760.0, p_f2: 1230.0, p_lp: 0.6, p_d1: 0.08, p_d2: 0.35, p_dr: 0.6, bits: 0, hold: 1 },
];

#[derive(Clone, Copy)]
struct Voice { on: bool, t: f32, vel: f32, ph: f32, ph2: f32, hp: f32, lp: f32, choke: f32, noise: u32, kit: u8, held: f32, hc: u32 }
impl Voice {
    const fn new(seed: u32) -> Voice { Voice { on: false, t: 0.0, vel: 0.0, ph: 0.0, ph2: 0.0, hp: 0.0, lp: 0.0, choke: 1.0, noise: seed, kit: 1, held: 0.0, hc: 0 } }
    fn trigger(&mut self, vel: f32, kit: u8) { self.on = true; self.t = 0.0; self.vel = vel; self.ph = 0.0; self.ph2 = 0.0; self.hp = 0.0; self.lp = 0.0; self.choke = 1.0; self.kit = kit.min(2); self.held = 0.0; self.hc = 0; }

    /// One sample of voice `k` (see NAMES). `tune` in -1..1, `dec` in 0..1. The kit is fixed per hit at trigger time.
    #[inline] fn tick(&mut self, k: usize, tune: f32, dec: f32, sr: f32) -> f32 {
        if !self.on { return 0.0; }
        let kp = &KITS[self.kit as usize];
        let dt = 1.0 / sr;
        let t = self.t;
        let ratio = (tune * 0.7).exp2();
        let x = match k {
            0 => { // kick: sine with fast pitch drop + click
                let f = kp.k_f * ratio * (1.0 + kp.k_drop * (-t * kp.k_rate).exp());
                self.ph = (self.ph + f * dt).fract();
                let body = (self.ph * TAU).sin() * (-t / (kp.k_dec + kp.k_decr * dec)).exp();
                let click = xorshift(&mut self.noise) * (-t * 700.0).exp() * kp.k_click;
                ((body + click) * kp.k_drive).tanh() * 0.9
            }
            1 => { // snare: two body sines + high-passed noise
                self.ph = (self.ph + kp.s_f1 * ratio * dt).fract(); self.ph2 = (self.ph2 + kp.s_f2 * ratio * dt).fract();
                let tone = ((self.ph * TAU).sin() * 0.6 + (self.ph2 * TAU).sin() * 0.3) * (-t / kp.s_tone).exp();
                let n = xorshift(&mut self.noise); let hp = n - self.lp; self.lp += kp.s_nlp * (n - self.lp);
                tone * kp.s_mix + hp * (-t / (kp.s_ndec + kp.s_ndecr * dec)).exp() * 0.8
            }
            2 => { // clap: fast noise bursts then a tail
                let n = xorshift(&mut self.noise); let bp = { let h = n - self.hp; self.hp += 0.3 * (n - self.hp); self.lp += kp.c_lp * (h - self.lp); self.lp };
                let mut burst = 0.0;
                for i in 0..kp.c_n { let o = i as f32 * kp.c_gap; if t >= o { burst += (-(t - o) * kp.c_rate).exp(); } }
                let t0 = kp.c_n as f32 * kp.c_gap;
                let tail = if t > t0 { (-(t - t0) / (kp.c_tail + kp.c_tailr * dec)).exp() } else { 0.0 };
                bp * (burst * 0.5 + tail) * 1.6
            }
            3 | 4 => { // hats: six detuned squares (metallic) + noise, high-passed
                let mut s = 0.0;
                for (i, f) in HAT_F.iter().enumerate() { let ph = (t * f * kp.h_mul * ratio + i as f32 * 0.17).fract(); s += if ph < 0.5 { 1.0 } else { -1.0 }; }
                let n = xorshift(&mut self.noise);
                let raw = s * kp.h_sq + n * kp.h_noise; let hp = raw - self.hp; self.hp += kp.h_hp * (raw - self.hp);
                let d = if k == 3 { kp.h_cd + kp.h_cdr * dec } else { kp.h_od + kp.h_odr * dec };
                if self.choke < 1.0 { self.choke *= 0.97; }
                hp * (-t / d).exp() * self.choke * 0.9
            }
            5 => { // tom
                let f = kp.t_f * ratio * (1.0 + kp.t_drop * (-t * kp.t_rate).exp());
                self.ph = (self.ph + f * dt).fract();
                ((self.ph * TAU).sin() + xorshift(&mut self.noise) * (-t * 500.0).exp() * kp.t_click) * (-t / (kp.t_dec + kp.t_decr * dec)).exp() * 0.9
            }
            6 => { // rim shot: short two-tone click
                self.ph = (self.ph + kp.r_f1 * ratio * dt).fract(); self.ph2 = (self.ph2 + kp.r_f2 * ratio * dt).fract();
                ((self.ph * TAU).sin() * 0.5 + (self.ph2 * TAU).sin() * 0.5 + xorshift(&mut self.noise) * kp.r_noise) * (-t / (kp.r_dec + kp.r_decr * dec)).exp()
            }
            _ => { // perc: cowbell (707/808) or ride bell (909): two squares through a low-pass
                self.ph = (self.ph + kp.p_f1 * ratio * dt).fract(); self.ph2 = (self.ph2 + kp.p_f2 * ratio * dt).fract();
                let raw = (if self.ph < 0.5 { 1.0 } else { -1.0 }) + (if self.ph2 < 0.5 { 1.0 } else { -1.0 });
                self.lp += kp.p_lp * (raw - self.lp);
                self.lp * 0.4 * ((-t / kp.p_d1).exp() * 0.6 + (-t / (kp.p_d2 + kp.p_dr * dec)).exp() * 0.4)
            }
        };
        self.t += dt;
        if self.t > 3.0 || (self.t > 0.02 && x.abs() < 1e-5 && self.choke < 0.01) { self.on = false; }
        if self.t > 0.5 && x.abs() < 1e-6 { self.on = false; }
        // 707 character: sample-and-hold decimation + 8-bit quantisation (post-processing only)
        let mut y = fin(x);
        if kp.hold > 1 { if self.hc % kp.hold == 0 { self.held = y; } self.hc = self.hc.wrapping_add(1); y = self.held; }
        if kp.bits > 0 { let q = (1u32 << (kp.bits - 1)) as f32; y = (y.clamp(-1.0, 1.0) * q).round() / q; }
        y * self.vel
    }
}

pub struct Drums {
    v: [Voice; TRACKS],
    step: u32, beat: f64, start: f64, running: bool, rnd: u32,
    pend: [(i32, u8, f32); 48], npend: usize,
    kit: u8,
    pub hits: [u32; TRACKS],
    pub cur_step: i32, // -1 when stopped (for the UI playhead)
}

impl Drums {
    pub fn new() -> Drums {
        Drums {
            v: std::array::from_fn(|i| Voice::new(0x9E3779B9u32.wrapping_mul(i as u32 + 7))),
            step: 0, beat: 0.0, start: 0.0, running: false, rnd: 0x1234567, pend: [(0, 0, 0.0); 48], npend: 0,
            kit: 1, hits: [0; TRACKS], cur_step: -1,
        }
    }

    /// Select the kit (0 = 707, 1 = 808, 2 = 909) used for hits triggered from now on.
    pub fn set_kit(&mut self, k: usize) { self.kit = k.min(2) as u8; }

    pub fn trigger(&mut self, track: usize, vel: f32) {
        let track = track.min(TRACKS - 1);
        if track == 3 { self.v[4].choke = 0.999; } // closed hat chokes the open hat
        self.v[track].trigger(vel.clamp(0.0, 1.5), self.kit);
        self.hits[track] = self.hits[track].wrapping_add(1);
    }

    #[inline] fn rnd01(&mut self) -> f32 { let mut x = self.rnd; x ^= x << 13; x ^= x >> 17; x ^= x << 5; self.rnd = x; (x >> 8) as f32 / 16777216.0 }

    /// Render one block of BLOCK samples (additive into `ol`/`or`). Advances the sequencer.
    pub fn process(&mut self, s: &[f32], bpm: f32, sr: f32, ol: &mut [f32; BLOCK], or: &mut [f32; BLOCK]) {
        let dt = (bpm.clamp(20.0, 300.0) as f64 / 60.0) * BLOCK as f64 / (sr.max(8000.0) as f64);
        self.set_kit(s[DR_KIT] as usize);
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
        for (kit, k) in (0..3).flat_map(|kit| (0..TRACKS).map(move |k| (kit, k))) {
            let mut d = Drums::new();
            d.set_kit(kit);
            d.trigger(k, 1.0);
            let mut s = default_patch(); s[DR_KIT] = kit as f32;
            let (mut energy, mut tail) = (0.0f32, 0.0f32);
            for b in 0..(48000 * 4 / 16) {
                let (mut l, mut r) = ([0.0f32; BLOCK], [0.0f32; BLOCK]);
                d.process(&s, 120.0, 48000.0, &mut l, &mut r);
                let e: f32 = l.iter().map(|x| x * x).sum();
                if b < 3000 { energy += e } else if b > 9600 { tail += e }
            }
            assert!(energy > 0.01, "{} ({}) is silent", NAMES[k], KIT_NAMES[kit]);
            assert!(tail < 1e-6, "{} ({}) never decays", NAMES[k], KIT_NAMES[kit]);
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

    #[test]
    fn kits_sound_different_and_stay_bounded() {
        let mut sums = [[0.0f32; 3]; 2];
        for kit in 0..3 {
            for (row, k) in [0usize, 3].iter().enumerate() {
                let mut d = Drums::new(); d.set_kit(kit); d.trigger(*k, 1.0);
                let mut s = default_patch(); s[DR_KIT] = kit as f32;
                let mut peak = 0.0f32;
                for _ in 0..(48000 / 16) {
                    let (mut l, mut r) = ([0.0f32; BLOCK], [0.0f32; BLOCK]);
                    d.process(&s, 120.0, 48000.0, &mut l, &mut r);
                    for x in l.iter() { assert!(x.is_finite()); peak = peak.max(x.abs()); sums[row][kit] += x * x; }
                }
                assert!(peak < 2.5, "kit {kit} voice {k} peak {peak}");
            }
        }
        // the kick of the 808 rings much longer than the 707's
        assert!(sums[0][1] > sums[0][0] * 1.5, "808 kick should carry more energy than 707: {:?}", sums[0]);
        for row in 0..2 { assert!((sums[row][0] - sums[row][2]).abs() > 1e-3, "707 and 909 identical"); }
    }
}
