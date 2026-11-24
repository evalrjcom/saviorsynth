//! Real-time synthesis engine: 16-voice polyphony, two wavetable oscillators with unison + warp,
//! sub + noise, state-variable filter, 3 envelopes, 4 LFOs, 8-slot mod matrix and the FX chain.
//! The audio callback only touches pre-allocated memory, atomics and a lock-free queue.

use crate::fx::{flush, tanh_fast, Fx};
use crate::params::*;
use crate::drums::Drums;
use crate::looper::{Cmd as LoopCmd, Looper};
use crate::perform::Performer;
use crate::shared::{Event, Shared, SCOPE_LEN};
use crate::wavetable::{WaveBank, N_FRAMES, TABLE_SIZE};
use std::f32::consts::PI;
use std::sync::atomic::Ordering;
use std::sync::Arc;

pub const BLOCK: usize = 16;
pub const MAX_VOICES: usize = 16;
const MAX_UNI: usize = 7;
const SMOOTH: f32 = 0.18;

// ---------------------------------------------------------------- envelope

#[derive(Clone, Copy, PartialEq)]
enum Stage { Idle, Attack, Hold, Decay, Sustain, Release }

#[derive(Clone, Copy, Default)]
struct EnvCoef { a_inc: f32, h_samples: f32, d_coef: f32, sus: f32, r_coef: f32 }

impl EnvCoef {
    fn from(s: &[f32], e: &EnvP, sr: f32) -> EnvCoef {
        let a = s[e.a].max(0.0005);
        let h = s[e.h];
        let d = s[e.d].max(0.005);
        let r = s[e.r].max(0.005);
        EnvCoef {
            a_inc: 1.0 / (a * sr),
            h_samples: if h > 0.0006 { h * sr } else { 0.0 },
            d_coef: (-6.9078 / (d * sr)).exp(),
            sus: s[e.s],
            r_coef: (-6.9078 / (r * sr)).exp(),
        }
    }
}

#[derive(Clone, Copy)]
struct Env { stage: Stage, level: f32, hold: f32 }
impl Env {
    const fn new() -> Env { Env { stage: Stage::Idle, level: 0.0, hold: 0.0 } }
    fn gate_on(&mut self) { self.stage = Stage::Attack; self.hold = 0.0; }
    fn gate_off(&mut self) { if self.stage != Stage::Idle { self.stage = Stage::Release; } }
    #[inline]
    fn tick(&mut self, c: &EnvCoef) -> f32 {
        match self.stage {
            Stage::Idle => { self.level = 0.0; }
            Stage::Attack => {
                self.level += c.a_inc;
                if self.level >= 1.0 {
                    self.level = 1.0; self.hold = 0.0;
                    self.stage = if c.h_samples > 0.0 { Stage::Hold } else { Stage::Decay };
                }
            }
            Stage::Hold => { self.hold += 1.0; if self.hold >= c.h_samples { self.stage = Stage::Decay; } }
            Stage::Decay => {
                self.level = c.sus + (self.level - c.sus) * c.d_coef;
                if (self.level - c.sus).abs() < 1e-5 { self.level = c.sus; self.stage = Stage::Sustain; }
            }
            Stage::Sustain => { self.level = c.sus; }
            Stage::Release => {
                self.level *= c.r_coef;
                if self.level < 1e-5 { self.level = 0.0; self.stage = Stage::Idle; }
            }
        }
        self.level
    }
}

// ---------------------------------------------------------------- filter

#[derive(Clone, Copy, Default)]
struct Svf { ic1: f32, ic2: f32 }
#[derive(Clone, Copy, Default)]
struct SvfCoef { a1: f32, a2: f32, a3: f32, k: f32 }
impl SvfCoef {
    fn new(g: f32, k: f32) -> SvfCoef { let a1 = 1.0 / (1.0 + g * (g + k)); let a2 = g * a1; SvfCoef { a1, a2, a3: g * a2, k } }
}
#[derive(Clone, Copy, PartialEq)]
enum FMode { Low, High, Band, Notch }
impl Svf {
    #[inline]
    fn tick(&mut self, c: &SvfCoef, x: f32, m: FMode) -> f32 {
        let v3 = x - self.ic2;
        let v1 = c.a1 * self.ic1 + c.a2 * v3;
        let v2 = self.ic2 + c.a2 * self.ic1 + c.a3 * v3;
        self.ic1 = flush(2.0 * v1 - self.ic1);
        self.ic2 = flush(2.0 * v2 - self.ic2);
        match m {
            FMode::Low => v2,
            FMode::High => x - c.k * v1 - v2,
            FMode::Band => c.k * v1,
            FMode::Notch => x - c.k * v1,
        }
    }
}

// ---------------------------------------------------------------- helpers

#[inline] fn polyblep(t: f32, dt: f32) -> f32 {
    if t < dt { let x = t / dt; x + x - x * x - 1.0 }
    else if t > 1.0 - dt { let x = (t - 1.0) / dt; x * x + x + x + 1.0 }
    else { 0.0 }
}
#[inline] fn xorshift(s: &mut u32) -> u32 { let mut x = *s; x ^= x << 13; x ^= x >> 17; x ^= x << 5; *s = x; x }
#[inline] fn rnd_bipolar(s: &mut u32) -> f32 { (xorshift(s) >> 8) as f32 / 8388608.0 - 1.0 }
#[inline] fn rnd_unit(s: &mut u32) -> f32 { (xorshift(s) >> 8) as f32 / 16777216.0 }
#[inline] fn note_to_hz(n: f32) -> f32 { 440.0 * ((n - 69.0) * (1.0 / 12.0)).exp2() }

#[inline]
fn warp_phase(p: f32, mode: usize, amt: f32, other: f32) -> f32 {
    match mode {
        1 => (p * (1.0 + amt * 7.0)).fract(),
        2 => p.powf(1.0 + amt * 3.0),
        3 => 1.0 - (1.0 - p).powf(1.0 + amt * 3.0),
        4 => { let w = (0.5 - 0.47 * amt).max(0.03); if p < w { p / w * 0.5 } else { 0.5 + (p - w) / (1.0 - w) * 0.5 } }
        5 => { let m = if p < 0.5 { 2.0 * p } else { 2.0 - 2.0 * p }; p + (m - p) * amt }
        6 => { let n = 2.0 + (1.0 - amt) * (1.0 - amt) * 254.0; (p * n).floor() / n }
        7 => { let x = p + other * amt * 0.5; x - x.floor() }
        _ => p,
    }
}

#[inline]
fn lfo_shape(shape: usize, ph: f32, cur: f32, next: f32) -> f32 {
    match shape {
        0 => (2.0 * PI * ph).sin(),
        1 => 1.0 - 4.0 * (ph - 0.5).abs(),
        2 => 2.0 * ph - 1.0,
        3 => 1.0 - 2.0 * ph,
        4 => if ph < 0.5 { 1.0 } else { -1.0 },
        5 => { let t = 0.5 - 0.5 * (PI * ph).cos(); cur + (next - cur) * t }
        _ => cur,
    }
}

fn lfo_rate(s: &[f32], i: usize, bpm: f32) -> f32 {
    let sync = (s[LFO[i].sync] as usize).min(11);
    if sync > 0 { bpm / 60.0 / SYNC_BEATS[sync] } else { s[LFO[i].rate] }
}

// ---------------------------------------------------------------- per-block context

struct UniInfo { n: usize, inc_mul: [f32; MAX_UNI], wl: [f32; MAX_UNI], wr: [f32; MAX_UNI], wm: [f32; MAX_UNI] }

struct BlockCtx<'a> {
    snap: &'a [f32],
    bank: &'a WaveBank,
    sr: f32,
    inv_sr: f32,
    bpm: f32,
    glob_lfo: [f32; 4],
    ecoef: [EnvCoef; 3],
    bend: f32,
    modwheel: f32,
    glide_k: f32,
}

// ---------------------------------------------------------------- voice

#[derive(Clone)]
struct Voice {
    active: bool, gate: bool, sustained: bool,
    note: u8, vel: f32, age: u64,
    pitch: f32,
    env: [Env; 3],
    lfo_ph: [f32; 4], lfo_last: [f32; 4], lfo_cur: [f32; 4], lfo_next: [f32; 4],
    t_since: f32,
    rnd: f32,
    rng: u32,
    ph: [[f32; MAX_UNI]; 2],
    sub_ph: f32,
    other: [f32; 2],
    flt: [[Svf; 2]; 2], // [channel][stage]
    pink: [f32; 7], brown: f32,
}

impl Voice {
    fn new(seed: u32) -> Voice {
        Voice {
            active: false, gate: false, sustained: false, note: 60, vel: 1.0, age: 0, pitch: 60.0,
            env: [Env::new(); 3],
            lfo_ph: [0.0; 4], lfo_last: [0.0; 4], lfo_cur: [0.0; 4], lfo_next: [0.0; 4],
            t_since: 0.0, rnd: 0.0, rng: seed | 1,
            ph: [[0.0; MAX_UNI]; 2], sub_ph: 0.0, other: [0.0; 2],
            flt: [[Svf::default(); 2]; 2], pink: [0.0; 7], brown: 0.0,
        }
    }

    fn start(&mut self, note: u8, vel: u8, start_pitch: f32, s: &[f32], age: u64) {
        let was_idle = !self.active;
        self.active = true; self.gate = true; self.sustained = false;
        self.note = note; self.vel = vel as f32 / 127.0; self.age = age;
        self.pitch = start_pitch;
        for e in self.env.iter_mut() { e.gate_on(); if was_idle { e.level = 0.0; } }
        self.t_since = 0.0;
        self.rnd = rnd_bipolar(&mut self.rng);
        for i in 0..4 {
            self.lfo_ph[i] = 0.0; self.lfo_last[i] = 0.0;
            self.lfo_cur[i] = rnd_bipolar(&mut self.rng); self.lfo_next[i] = rnd_bipolar(&mut self.rng);
        }
        for o in 0..2 {
            let p = &OSC[o];
            let base = s[p.phase]; let rnd = s[p.rand];
            for u in 0..MAX_UNI {
                let r = rnd * rnd_unit(&mut self.rng);
                self.ph[o][u] = (base + r).fract();
            }
        }
        self.sub_ph = 0.0;
        if was_idle { self.flt = [[Svf::default(); 2]; 2]; self.other = [0.0; 2]; }
    }

    fn release(&mut self) { self.gate = false; for e in self.env.iter_mut() { e.gate_off(); } }

    /// Renders BLOCK samples and ADDS them into (ol, or_).
    fn render(&mut self, c: &BlockCtx, ol: &mut [f32; BLOCK], or_: &mut [f32; BLOCK]) {
        let s = c.snap;
        // glide + pitch
        self.pitch += (self.note as f32 - self.pitch) * c.glide_k;
        let block_t = BLOCK as f32 * c.inv_sr;
        self.t_since += block_t;

        // envelopes 2/3 advance a block; env1 per sample below
        let mut e2 = 0.0; let mut e3 = 0.0;
        for _ in 0..BLOCK { e2 = self.env[1].tick(&c.ecoef[1]); e3 = self.env[2].tick(&c.ecoef[2]); }

        // LFOs
        let mut lfo = [0.0f32; 4];
        for i in 0..4 {
            let rate = lfo_rate(s, i, c.bpm);
            let ph = if s[LFO[i].trig] >= 0.5 {
                let mut p = self.lfo_ph[i] + rate * block_t; p -= p.floor(); self.lfo_ph[i] = p; p
            } else { c.glob_lfo[i] };
            if ph < self.lfo_last[i] { self.lfo_cur[i] = self.lfo_next[i]; self.lfo_next[i] = rnd_bipolar(&mut self.rng); }
            self.lfo_last[i] = ph;
            let fade = s[LFO[i].fade];
            let g = if fade > 0.001 { (self.t_since / fade).min(1.0) } else { 1.0 };
            lfo[i] = lfo_shape(s[LFO[i].shape] as usize, ph, self.lfo_cur[i], self.lfo_next[i]) * g;
        }
        let key = ((self.note as f32 - 60.0) / 60.0).clamp(-1.0, 1.0);
        let srcs = [0.0, e2, e3, lfo[0], lfo[1], lfo[2], lfo[3], self.vel, key, c.modwheel, self.rnd];

        // modulation matrix
        let mut d = [0.0f32; MD_COUNT];
        for m in MODS.iter() {
            let src = s[m.src] as usize; let dst = s[m.dst] as usize;
            if src != 0 && dst != 0 { d[dst] += srcs[src.min(10)] * s[m.amt]; }
        }

        // ---------------- oscillator setup
        let mut runs: [OscRun; 2] = [OscRun::off(), OscRun::off()];
        let md_pos = [MD_A_POS, MD_B_POS]; let md_warp = [MD_A_WARP, MD_B_WARP];
        let md_pitch = [MD_A_PITCH, MD_B_PITCH]; let md_level = [MD_A_LEVEL, MD_B_LEVEL];
        for o in 0..2 {
            let p = &OSC[o];
            if s[p.on] < 0.5 { continue; }
            let semis = self.pitch + s[p.oct] * 12.0 + s[p.semi] + s[p.fine] * 0.01 + c.bend + d[md_pitch[o]] * 12.0;
            let base = note_to_hz(semis).clamp(1.0, 20000.0);
            let n = (s[p.uni] as usize).clamp(1, MAX_UNI);
            let det = (s[p.det] + d[MD_DETUNE]).clamp(0.0, 1.0);
            let det_cents = 100.0 * det * det.sqrt();
            let wmode = (s[p.wmode] as usize).min(9);
            let wamt = (s[p.warp] + d[md_warp[o]]).clamp(0.0, 1.0);
            let pos = (s[p.pos] + d[md_pos[o]]).clamp(0.0, 1.0);
            let fp = pos * (N_FRAMES - 1) as f32;
            let f0 = fp.floor() as usize; let f1 = (f0 + 1).min(N_FRAMES - 1); let ff = fp - f0 as f32;
            let top = base * (det_cents / 1200.0).exp2() * if wmode == 1 { 1.0 + wamt * 7.0 } else { 1.0 };
            let mip = WaveBank::mip_for(top, c.sr);
            let wt = s[p.wt] as usize;
            let level = (s[p.level] + d[md_level[o]]).clamp(0.0, 1.0);
            let blend = s[p.blend]; let spread = s[p.spread]; let pan = (s[p.pan] + d[MD_PAN]).clamp(-1.0, 1.0);
            let mut uni = UniInfo { n, inc_mul: [0.0; MAX_UNI], wl: [0.0; MAX_UNI], wr: [0.0; MAX_UNI], wm: [0.0; MAX_UNI] };
            let mut raw = [0.0f32; MAX_UNI]; let mut sum2 = 0.0;
            for u in 0..n {
                let off = if n == 1 { 0.0 } else { 2.0 * u as f32 / (n - 1) as f32 - 1.0 };
                uni.inc_mul[u] = (off * det_cents / 1200.0).exp2();
                raw[u] = if n == 1 { 1.0 } else { 0.08 + blend * off.abs() + (1.0 - blend) * (1.0 - off.abs()) };
                sum2 += raw[u] * raw[u];
            }
            let norm = 1.0 / sum2.sqrt().max(1e-6);
            for u in 0..n {
                let off = if n == 1 { 0.0 } else { 2.0 * u as f32 / (n - 1) as f32 - 1.0 };
                let w = raw[u] * norm;
                let pp = (off * spread + pan).clamp(-1.0, 1.0);
                let ang = (pp + 1.0) * PI * 0.25;
                uni.wm[u] = w;
                uni.wl[u] = w * level * ang.cos() * 1.4142;
                uni.wr[u] = w * level * ang.sin() * 1.4142;
            }
            runs[o] = OscRun {
                on: true,
                t0: c.bank.slice(wt, f0, mip), t1: c.bank.slice(wt, f1, mip), ff,
                inc: base * c.inv_sr, uni, mode: wmode, amt: wamt,
            };
        }

        // sub + noise setup
        let sub_on = s[SUB_ON] >= 0.5;
        let sub_inc = note_to_hz(self.pitch + c.bend + s[SUB_OCT] * 12.0).clamp(1.0, 5000.0) * c.inv_sr;
        let sub_level = if sub_on { (s[SUB_LEVEL] + d[MD_SUB]).clamp(0.0, 1.0) } else { 0.0 };
        let sub_shape = s[SUB_SHAPE] as usize;
        let noise_level = if s[NOISE_ON] >= 0.5 { (s[NOISE_LEVEL] + d[MD_NOISE]).clamp(0.0, 1.0) } else { 0.0 };
        let noise_type = s[NOISE_TYPE] as usize;

        // filter setup
        let f_on = s[F_ON] >= 0.5;
        let ftype = (s[F_TYPE] as usize).min(5);
        let octs = s[F_KEY] * (self.pitch - 60.0) / 12.0 + s[F_ENV] * 7.0 * e2 + d[MD_CUT] * 7.0;
        let fc = (s[F_CUT] * octs.clamp(-12.0, 12.0).exp2()).clamp(20.0, (0.45 * c.sr).min(20000.0));
        let res = (s[F_RES] + d[MD_RES]).clamp(0.0, 1.0);
        let drive = (s[F_DRIVE] + d[MD_DRIVE]).clamp(0.0, 1.0);
        let pre = 1.0 + drive * 8.0;
        let g = (PI * fc * c.inv_sr).tan();
        let q = 0.6 * (40.0f32 / 0.6).powf(res);
        let k1 = 1.0 / q;
        let co1 = SvfCoef::new(g, k1);
        let co2 = SvfCoef::new(g, 1.4142);
        let (fmode, two) = match ftype { 0 => (FMode::Low, false), 1 => (FMode::Low, true), 2 => (FMode::High, false), 3 => (FMode::High, true), 4 => (FMode::Band, false), _ => (FMode::Notch, false) };
        let fmix = s[F_MIX];
        let (ra, rb, rs, rn) = (s[F_RA] >= 0.5, s[F_RB] >= 0.5, s[F_RS] >= 0.5, s[F_RN] >= 0.5);

        // amp
        let amp_gain = (1.0 + d[MD_AMP]).clamp(0.0, 2.0) * (0.3 + 0.7 * self.vel) * 0.32;
        let pan_v = d[MD_PAN].clamp(-1.0, 1.0);
        let (pl, pr) = if pan_v >= 0.0 { (1.0 - pan_v, 1.0) } else { (1.0, 1.0 + pan_v) };

        // ---------------- sample loop
        for i in 0..BLOCK {
            let env1 = self.env[0].tick(&c.ecoef[0]);
            // oscillators (A uses B's previous output, B uses A's current)
            let (al, ar, am) = runs[0].sample(&mut self.ph[0], self.other[1]);
            let (bl, br, bm) = runs[1].sample(&mut self.ph[1], am);
            self.other = [am, bm];
            // sub
            let mut sub = 0.0;
            if sub_level > 0.0 {
                self.sub_ph += sub_inc; self.sub_ph -= self.sub_ph.floor();
                let t = self.sub_ph;
                sub = sub_level * match sub_shape {
                    0 => (2.0 * PI * t).sin(),
                    1 => 4.0 * (t - 0.5).abs() - 1.0,
                    2 => (if t < 0.5 { 1.0 } else { -1.0 }) + polyblep(t, sub_inc) - polyblep((t + 0.5).fract(), sub_inc),
                    _ => 2.0 * t - 1.0 - polyblep(t, sub_inc),
                };
            }
            // noise
            let mut nz = 0.0;
            if noise_level > 0.0 {
                let w = rnd_bipolar(&mut self.rng);
                nz = noise_level * match noise_type {
                    0 => w,
                    1 => {
                        let b = &mut self.pink;
                        b[0] = 0.99886 * b[0] + w * 0.0555179; b[1] = 0.99332 * b[1] + w * 0.0750759;
                        b[2] = 0.96900 * b[2] + w * 0.1538520; b[3] = 0.86650 * b[3] + w * 0.3104856;
                        b[4] = 0.55000 * b[4] + w * 0.5329522; b[5] = -0.7616 * b[5] - w * 0.0168980;
                        let out = b[0] + b[1] + b[2] + b[3] + b[4] + b[5] + b[6] + w * 0.5362;
                        b[6] = w * 0.115926;
                        out * 0.22
                    }
                    _ => { self.brown = (self.brown + w * 0.02) / 1.02; self.brown * 3.5 }
                };
            }
            // routing
            let mut fl = 0.0; let mut fr = 0.0; let mut dl = 0.0; let mut dr = 0.0;
            if ra { fl += al; fr += ar; } else { dl += al; dr += ar; }
            if rb { fl += bl; fr += br; } else { dl += bl; dr += br; }
            if rs { fl += sub; fr += sub; } else { dl += sub; dr += sub; }
            if rn { fl += nz; fr += nz; } else { dl += nz; dr += nz; }
            let (mut yl, mut yr) = (fl, fr);
            if f_on {
                let (mut xl, mut xr) = (fl, fr);
                if drive > 0.001 { let inv = 1.0 / pre.sqrt(); xl = tanh_fast(xl * pre) * inv; xr = tanh_fast(xr * pre) * inv; }
                let mut wl = self.flt[0][0].tick(&co1, xl, fmode);
                let mut wr = self.flt[1][0].tick(&co1, xr, fmode);
                if two { wl = self.flt[0][1].tick(&co2, wl, fmode); wr = self.flt[1][1].tick(&co2, wr, fmode); }
                yl = fl + (wl - fl) * fmix; yr = fr + (wr - fr) * fmix;
            }
            let gain = env1 * amp_gain;
            ol[i] += (yl + dl) * gain * pl;
            or_[i] += (yr + dr) * gain * pr;
        }
        if self.env[0].stage == Stage::Idle || (self.env[0].stage == Stage::Sustain && c.ecoef[0].sus < 1e-4) {
            self.active = false;
        }
    }
}

struct OscRun<'a> {
    on: bool, t0: &'a [f32], t1: &'a [f32], ff: f32, inc: f32, uni: UniInfo, mode: usize, amt: f32,
}
static EMPTY: [f32; 1] = [0.0];
impl<'a> OscRun<'a> {
    fn off() -> OscRun<'a> {
        OscRun { on: false, t0: &EMPTY, t1: &EMPTY, ff: 0.0, inc: 0.0,
            uni: UniInfo { n: 0, inc_mul: [0.0; MAX_UNI], wl: [0.0; MAX_UNI], wr: [0.0; MAX_UNI], wm: [0.0; MAX_UNI] }, mode: 0, amt: 0.0 }
    }
    /// Returns (left, right, mono-for-crossmod).
    #[inline]
    fn sample(&self, ph: &mut [f32; MAX_UNI], other: f32) -> (f32, f32, f32) {
        if !self.on { return (0.0, 0.0, 0.0); }
        let (mut l, mut r, mut m) = (0.0, 0.0, 0.0);
        let ff = self.ff;
        for u in 0..self.uni.n {
            let mut p = ph[u] + self.inc * self.uni.inc_mul[u];
            p -= p.floor();
            ph[u] = p;
            let wp = warp_phase(p, self.mode, self.amt, other);
            let x = wp * TABLE_SIZE as f32;
            let i = x as usize; let fr = x - i as f32;
            let i0 = i & (TABLE_SIZE - 1); let i1 = (i + 1) & (TABLE_SIZE - 1);
            let a = self.t0[i0] + (self.t0[i1] - self.t0[i0]) * fr;
            let b = self.t1[i0] + (self.t1[i1] - self.t1[i0]) * fr;
            let mut sv = a + (b - a) * ff;
            if self.mode == 8 { sv *= 1.0 - self.amt + self.amt * (0.5 + 0.5 * other); }
            else if self.mode == 9 { sv *= 1.0 - self.amt + self.amt * other; }
            l += sv * self.uni.wl[u]; r += sv * self.uni.wr[u]; m += sv * self.uni.wm[u];
        }
        (l, r, m)
    }
}

// ---------------------------------------------------------------- engine

pub struct Engine {
    sr: f32,
    shared: Arc<Shared>,
    bank: Arc<WaveBank>,
    snap: Vec<f32>,
    voices: Vec<Voice>,
    fx: Fx,
    glob_lfo: [f32; 4],
    stack: Vec<(u8, u8)>,
    perf: Performer,
    drums: Drums,
    looper: Looper,
    perf_act: bool,
    latch: bool,
    sustain: bool,
    last_pitch: f32,
    age: u64,
    bend_s: f32,
    stage_l: [f32; BLOCK], stage_r: [f32; BLOCK], pos: usize,
    pk_l: f32, pk_r: f32, scope_ctr: u32,
}

#[inline] fn soft_clip(x: f32) -> f32 {
    let a = x.abs();
    if a < 0.8 { x } else { x.signum() * (0.8 + 0.2 * tanh_fast((a - 0.8) * 5.0)) }
}

impl Engine {
    pub fn new(sr: f32, shared: Arc<Shared>, bank: Arc<WaveBank>) -> Engine {
        let snap = shared.params.snapshot();
        Engine {
            sr, shared, bank, snap,
            voices: (0..MAX_VOICES).map(|i| Voice::new(0x9E3779B9u32.wrapping_mul(i as u32 + 1))).collect(),
            fx: Fx::new(sr), glob_lfo: [0.0; 4],
            stack: Vec::with_capacity(64), perf: Performer::new(), drums: Drums::new(), looper: Looper::new(sr), perf_act: false, latch: false, sustain: false, last_pitch: 60.0, age: 0, bend_s: 0.0,
            stage_l: [0.0; BLOCK], stage_r: [0.0; BLOCK], pos: BLOCK, pk_l: 0.0, pk_r: 0.0, scope_ctr: 0,
        }
    }

    fn voice_limit(&self) -> usize { (self.snap[VOICES] as usize).clamp(1, MAX_VOICES) }

    fn pick_voice(&self, limit: usize, note: u8) -> usize {
        if let Some(i) = (0..limit).find(|&i| self.voices[i].active && self.voices[i].note == note) { return i; }
        if let Some(i) = (0..limit).find(|&i| !self.voices[i].active) { return i; }
        // steal: quietest releasing voice, else oldest
        let mut best = 0usize; let mut score = f32::MAX;
        for i in 0..limit {
            let v = &self.voices[i];
            let sc = if !v.gate { v.env[0].level } else { 10.0 + v.age as f32 };
            if sc < score { score = sc; best = i; }
        }
        best
    }

    fn note_on(&mut self, note: u8, vel: u8) {
        self.age += 1;
        let glide = self.snap[GLIDE] > 0.0015;
        let start = if glide { self.last_pitch } else { note as f32 };
        let mode = self.snap[MODE] as usize;
        if mode == 0 {
            let limit = self.voice_limit();
            let i = self.pick_voice(limit, note);
            self.voices[i].start(note, vel, start, &self.snap, self.age);
        } else {
            self.stack.retain(|&(n, _)| n != note);
            if self.stack.len() >= 60 { self.stack.remove(0); }
            self.stack.push((note, vel));
            let age = self.age;
            let v = &mut self.voices[0];
            if mode == 2 && v.active && v.gate {
                v.note = note; v.vel = vel as f32 / 127.0; v.age = age;
            } else {
                v.start(note, vel, start, &self.snap, age);
            }
            for j in 1..MAX_VOICES { if self.voices[j].active && self.voices[j].gate { self.voices[j].release(); } }
        }
        self.last_pitch = note as f32;
    }

    fn note_off(&mut self, note: u8) {
        let mode = self.snap[MODE] as usize;
        if mode == 0 {
            for v in self.voices.iter_mut() {
                if v.active && v.gate && v.note == note {
                    if self.sustain { v.sustained = true; } else { v.release(); }
                }
            }
            return;
        }
        self.stack.retain(|&(n, _)| n != note);
        // only react when the key that is actually sounding was released
        if self.voices[0].note != note { return; }
        match self.stack.last().copied() {
            Some((n, vel)) => {
                let age = self.age;
                let v = &mut self.voices[0];
                if mode == 1 { let p = v.pitch; v.start(n, vel, p, &self.snap, age); } else { v.note = n; }
            }
            None => {
                let v = &mut self.voices[0];
                if v.active && v.gate { if self.sustain { v.sustained = true; } else { v.release(); } }
            }
        }
    }

    fn set_sustain(&mut self, on: bool) {
        self.sustain = on;
        if !on {
            let hold_mono = !self.stack.is_empty() && (self.snap[MODE] as usize) != 0;
            for (i, v) in self.voices.iter_mut().enumerate() {
                if v.active && v.sustained { v.sustained = false; if !(hold_mono && i == 0) { v.release(); } }
            }
        }
    }

    fn all_off(&mut self) {
        self.stack.clear(); self.sustain = false;
        for v in self.voices.iter_mut() { if v.active { v.sustained = false; v.release(); } }
    }

    /// Release every sounding voice but keep the sustain-pedal state (used when the performance stage takes over).
    fn release_voices(&mut self) {
        self.stack.clear();
        for v in self.voices.iter_mut() { if v.active { v.sustained = false; v.release(); } }
    }

    fn apply_perf(&mut self, n: usize) {
        for i in 0..n {
            let e = self.perf.out[i];
            if e.on { self.note_on(e.note, e.vel); } else { self.note_off(e.note); }
        }
    }

    /// Chord / progression / arpeggiator stage. Keys are always tracked; they reach the voices
    /// directly only while the stage is inactive, so toggling it mid-note never leaves stuck notes.
    fn run_performance(&mut self) {
        let act = self.snap[CH_MODE] >= 0.5 || self.snap[AR_ON] >= 0.5;
        let latch = self.snap[AR_LATCH] >= 0.5;
        if self.latch && !latch { self.perf.unlatch(); }
        self.latch = latch;
        if act && !self.perf_act { self.release_voices(); }
        let was = self.perf_act;
        self.perf_act = act;
        if act || was || self.perf.info_root >= 0 {
            let n = self.perf.process(&self.snap, self.snap[BPM], self.sr);
            self.apply_perf(n);
            let (r, m) = (self.perf.info_root, self.perf.info_mask);
            self.shared.chord_root.store(if r >= 0 { r as u32 } else { u32::MAX }, Ordering::Relaxed);
            self.shared.chord_mask.store(m, Ordering::Relaxed);
        }
        if was && !act {
            // hand the physically held keys back to the voices as ordinary notes
            for i in 0..self.perf.held().len() { let (k, v) = self.perf.held()[i]; self.note_on(k, v); }
        }
    }

    fn drain_events(&mut self) {
        for _ in 0..128 {
            match self.shared.events.pop() {
                Some(Event::NoteOn(n, v)) => { self.perf.track_on(n, v, self.latch); if !self.perf_act { self.note_on(n, v); } }
                Some(Event::NoteOff(n)) => { self.perf.track_off(n, self.latch); if !self.perf_act { self.note_off(n); } }
                Some(Event::AllOff) => { self.perf.reset(); self.all_off(); }
                Some(Event::Sustain(b)) => self.set_sustain(b),
                Some(Event::Loop(c)) => if let Some(cmd) = LoopCmd::from_u8(c) { self.looper.command(cmd, self.snap[LP_BARS] as usize, self.snap[BPM], self.sr); },
                Some(Event::Drum(t, v)) => { let t = (t as usize).min(7); self.drums.trigger(t, v as f32 / 127.0 * self.snap[DR_V1 + t * 5]); }
                None => break,
            }
        }
    }

    fn render_block(&mut self) {
        for i in 0..PARAM_COUNT {
            let r = self.shared.params.get(i);
            if PARAMS[i].is_discrete() { self.snap[i] = r; } else { self.snap[i] += (r - self.snap[i]) * SMOOTH; }
        }
        self.drain_events();
        self.run_performance();
        let inv_sr = 1.0 / self.sr;
        let bpm = self.snap[BPM];
        let target_bend = self.shared.get_bend() * self.snap[BEND];
        self.bend_s += (target_bend - self.bend_s) * 0.3;
        for i in 0..4 {
            let rate = lfo_rate(&self.snap, i, bpm);
            let p = self.glob_lfo[i] + rate * BLOCK as f32 * inv_sr;
            self.glob_lfo[i] = p - p.floor();
        }
        let ecoef = [EnvCoef::from(&self.snap, &ENV[0], self.sr), EnvCoef::from(&self.snap, &ENV[1], self.sr), EnvCoef::from(&self.snap, &ENV[2], self.sr)];
        let g = self.snap[GLIDE];
        let glide_k = if g <= 0.0015 { 1.0 } else { 1.0 - (-(BLOCK as f32 * inv_sr) / (g * 0.25)).exp() };
        let mut bl = [0.0f32; BLOCK]; let mut br = [0.0f32; BLOCK];
        let mut nact = 0u32;
        {
            let ctx = BlockCtx {
                snap: &self.snap, bank: &self.bank, sr: self.sr, inv_sr, bpm,
                glob_lfo: self.glob_lfo, ecoef, bend: self.bend_s,
                modwheel: self.shared.get_mod(), glide_k,
            };
            for v in self.voices.iter_mut() {
                if v.active { v.render(&ctx, &mut bl, &mut br); nact += 1; }
            }
        }
        // safety net: never let a NaN/inf reach the FX state or the speakers
        if bl.iter().chain(br.iter()).any(|x| !x.is_finite()) {
            for v in self.voices.iter_mut() { v.active = false; }
            bl = [0.0; BLOCK]; br = [0.0; BLOCK];
        }
        self.shared.voices.store(nact, Ordering::Relaxed);
        self.shared.loop_state.store(self.looper.state as u32, Ordering::Relaxed);
        self.shared.loop_pos.store(self.looper.progress().to_bits(), Ordering::Relaxed);
        self.shared.drum_step.store(if self.drums.cur_step >= 0 { self.drums.cur_step as u32 } else { u32::MAX }, Ordering::Relaxed);
        self.fx.begin_block(&self.snap, bpm);
        let m = self.snap[MASTER];
        let mg = m * m * 2.0;
        // drum machine renders into its own block, mixed ahead of the master (it skips the synth FX chain)
        let (mut dl, mut dr) = ([0.0f32; BLOCK], [0.0f32; BLOCK]);
        self.drums.process(&self.snap, bpm, self.sr, &mut dl, &mut dr);
        self.looper.background();
        let lvol = self.snap[LP_VOL];
        for i in 0..BLOCK {
            let (l, r) = self.fx.process(bl[i], br[i]);
            let (l, r) = ((l + dl[i]) * mg, (r + dr[i]) * mg);
            // the looper records the live mix (never its own playback) and adds the loop back in
            let (ll, lr) = self.looper.process(l, r, lvol);
            let l = soft_clip(l + ll); let r = soft_clip(r + lr);
            self.stage_l[i] = l; self.stage_r[i] = r;
            self.pk_l = (self.pk_l * 0.9998).max(l.abs());
            self.pk_r = (self.pk_r * 0.9998).max(r.abs());
            self.scope_ctr += 1;
            if self.scope_ctr & 3 == 0 {
                let p = self.shared.scope_pos.load(Ordering::Relaxed);
                self.shared.scope[p % SCOPE_LEN].store((0.5 * (l + r)).to_bits(), Ordering::Relaxed);
                self.shared.scope_pos.store((p + 1) % SCOPE_LEN, Ordering::Relaxed);
            }
        }
    }

    /// Fill an interleaved stereo buffer (len must be even).
    pub fn fill(&mut self, out: &mut [f32]) {
        let mut idx = 0;
        while idx + 1 < out.len() {
            if self.pos >= BLOCK { self.render_block(); self.pos = 0; }
            let n = ((out.len() - idx) / 2).min(BLOCK - self.pos);
            for j in 0..n {
                out[idx + 2 * j] = self.stage_l[self.pos + j];
                out[idx + 2 * j + 1] = self.stage_r[self.pos + j];
            }
            idx += 2 * n; self.pos += n;
        }
        self.shared.peak_l.store(self.pk_l.to_bits(), Ordering::Relaxed);
        self.shared.peak_r.store(self.pk_r.to_bits(), Ordering::Relaxed);
    }
}
