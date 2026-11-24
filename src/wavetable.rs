//! Band-limited wavetable bank.
//!
//! Every table has N_FRAMES single-cycle frames. Each frame is stored as a harmonic spectrum and
//! rendered into N_MIPS mip levels (each halving the number of harmonics) so the oscillator can
//! pick an alias-free level for the note being played. Generation is done once at start-up with
//! a small in-house radix-2 FFT (no external DSP dependency).

use std::f64::consts::PI;

pub const TABLE_SIZE: usize = 2048;
pub const N_FRAMES: usize = 32;
pub const N_MIPS: usize = 10;
pub const N_TABLES: usize = 16;
const HALF: usize = TABLE_SIZE / 2; // 1024 usable harmonics (k = 1..1023)

#[derive(Clone, Copy, Default)]
struct C { re: f64, im: f64 }
impl C {
    fn new(re: f64, im: f64) -> C { C { re, im } }
    fn mul(self, o: C) -> C { C::new(self.re * o.re - self.im * o.im, self.re * o.im + self.im * o.re) }
    fn add(self, o: C) -> C { C::new(self.re + o.re, self.im + o.im) }
    fn sub(self, o: C) -> C { C::new(self.re - o.re, self.im - o.im) }
    fn scale(self, s: f64) -> C { C::new(self.re * s, self.im * s) }
}

/// In-place iterative radix-2 FFT. `sign` = -1 forward, +1 inverse (unnormalised).
fn fft(buf: &mut [C], sign: f64) {
    let n = buf.len();
    debug_assert!(n.is_power_of_two());
    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 { j ^= bit; bit >>= 1; }
        j |= bit;
        if i < j { buf.swap(i, j); }
    }
    let mut len = 2;
    while len <= n {
        let ang = sign * 2.0 * PI / len as f64;
        let wl = C::new(ang.cos(), ang.sin());
        for start in (0..n).step_by(len) {
            let mut w = C::new(1.0, 0.0);
            for k in 0..len / 2 {
                let u = buf[start + k];
                let v = buf[start + k + len / 2].mul(w);
                buf[start + k] = u.add(v);
                buf[start + k + len / 2] = u.sub(v);
                w = w.mul(wl);
            }
        }
        len <<= 1;
    }
}

/// Spectrum coefficient for a * sin(k*theta + phi).  Waveform = Re( sum Z[k] e^{+i k theta} ).
fn z(a: f64, phi: f64) -> C { C::new(a * phi.sin(), -a * phi.cos()) }

type Spec = Vec<C>; // length HALF, index 0 unused

fn zero() -> Spec { vec![C::default(); HALF] }
fn lerp_spec(a: &Spec, b: &Spec, t: f64) -> Spec {
    a.iter().zip(b.iter()).map(|(x, y)| x.scale(1.0 - t).add(y.scale(t))).collect()
}
fn shape(mut f: impl FnMut(usize) -> C) -> Spec { (0..HALF).map(|k| if k == 0 { C::default() } else { f(k) }).collect() }

fn sine() -> Spec { shape(|k| if k == 1 { z(1.0, 0.0) } else { C::default() }) }
fn saw() -> Spec { shape(|k| z(2.0 / (PI * k as f64) * if k % 2 == 1 { 1.0 } else { -1.0 }, 0.0)) }
fn square() -> Spec { shape(|k| if k % 2 == 1 { z(4.0 / (PI * k as f64), 0.0) } else { C::default() }) }
fn tri() -> Spec {
    shape(|k| if k % 2 == 1 {
        let s = if (k / 2) % 2 == 0 { 1.0 } else { -1.0 };
        z(8.0 / (PI * PI * (k * k) as f64) * s, 0.0)
    } else { C::default() })
}
/// Pulse = saw(theta) - saw(theta - 2*pi*duty)
fn pulse(duty: f64) -> Spec {
    let s = saw();
    s.iter().enumerate().map(|(k, c)| {
        let a = -2.0 * PI * k as f64 * duty;
        c.mul(C::new(1.0, 0.0).sub(C::new(a.cos(), a.sin())))
    }).collect()
}
fn from_time(x: &[f64]) -> Spec {
    let mut buf: Vec<C> = x.iter().map(|&v| C::new(v, 0.0)).collect();
    fft(&mut buf, -1.0);
    (0..HALF).map(|k| if k == 0 { C::default() } else { buf[k].scale(2.0 / TABLE_SIZE as f64) }).collect()
}
fn time_gen(f: impl Fn(f64) -> f64) -> Spec {
    let x: Vec<f64> = (0..TABLE_SIZE).map(|n| f(2.0 * PI * n as f64 / TABLE_SIZE as f64)).collect();
    from_time(&x)
}
fn apply_env(s: &Spec, env: impl Fn(usize) -> f64) -> Spec {
    s.iter().enumerate().map(|(k, c)| c.scale(env(k))).collect()
}
fn rotate_phase(s: &Spec, phases: &[f64]) -> Spec {
    s.iter().enumerate().map(|(k, c)| { let a = phases[k]; c.mul(C::new(a.cos(), a.sin())) }).collect()
}

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.0 >> 33) as f64) / ((1u64 << 31) as f64)
    }
}

fn formants(k: usize, f0: f64, set: &[(f64, f64, f64)]) -> f64 {
    let fk = k as f64 * f0;
    let mut e = 0.015;
    for &(fc, bw, g) in set { let d = (fk - fc) / bw; e += g * (-0.5 * d * d).exp(); }
    e
}
fn lerp_set(a: &[(f64, f64, f64); 3], b: &[(f64, f64, f64); 3], t: f64) -> [(f64, f64, f64); 3] {
    let mut o = [(0.0, 0.0, 0.0); 3];
    for i in 0..3 { o[i] = (a[i].0 + (b[i].0 - a[i].0) * t, a[i].1 + (b[i].1 - a[i].1) * t, a[i].2 + (b[i].2 - a[i].2) * t); }
    o
}

const VOWELS: [[(f64, f64, f64); 3]; 5] = [
    [(800.0, 90.0, 1.0), (1150.0, 110.0, 0.6), (2900.0, 160.0, 0.25)], // a
    [(400.0, 70.0, 1.0), (1600.0, 110.0, 0.5), (2700.0, 160.0, 0.25)], // e
    [(350.0, 60.0, 1.0), (1700.0, 110.0, 0.5), (2700.0, 160.0, 0.3)],  // i
    [(450.0, 70.0, 1.0), (800.0, 90.0, 0.6), (2830.0, 160.0, 0.2)],    // o
    [(325.0, 60.0, 1.0), (700.0, 90.0, 0.5), (2530.0, 160.0, 0.15)],   // u
];

/// Build the harmonic spectrum of frame `fi` (0..N_FRAMES) of table `t`.
fn frame_spectrum(t: usize, fi: usize) -> Spec {
    let f = fi as f64 / (N_FRAMES - 1) as f64;
    match t {
        0 => { // Basic shapes: sine -> tri -> saw -> square -> narrow pulse
            let set = [sine(), tri(), saw(), square(), pulse(0.2)];
            let p = f * 4.0;
            let i = (p.floor() as usize).min(3);
            lerp_spec(&set[i], &set[i + 1], p - i as f64)
        }
        1 => lerp_spec(&saw(), &square(), f),
        2 => pulse(0.5 - 0.46 * f),
        3 => { // additive build-up of a saw: 1..80 harmonics
            let nh = 1.0 + 79.0 * f;
            apply_env(&saw(), |k| (nh - k as f64 + 1.0).clamp(0.0, 1.0))
        }
        4 => { // vowels a-e-i-o-u (formants fixed relative to a 130 Hz reference)
            let p = f * 4.0; let i = (p.floor() as usize).min(3);
            let set = lerp_set(&VOWELS[i], &VOWELS[i + 1], p - i as f64);
            apply_env(&saw(), |k| formants(k, 130.0, &set) * 3.0)
        }
        5 => time_gen(|th| (th + (0.1 + 7.0 * f) * (3.0 * th).sin()).sin()),
        6 => { // organ drawbars sweeping through registrations
            let mut s = zero();
            for (j, &h) in [1usize, 2, 3, 4, 5, 6, 8].iter().enumerate() {
                let w = if h == 1 { 1.0 } else { 0.5 + 0.5 * (2.0 * PI * (f * 1.5 + j as f64 * 0.37)).sin() };
                s[h] = z(w * 0.8, 0.0);
            }
            s
        }
        7 => { // hollow glass: odd harmonics with a travelling notch
            let kn = 3.0 + 30.0 * f;
            apply_env(&square(), |k| 1.0 - 0.95 * (-((k as f64 - kn) / 2.5).powi(2)).exp())
        }
        8 => { // choir: female vowel formants, slightly randomised phases
            let sets: [[(f64, f64, f64); 3]; 5] = [
                [(850.0, 110.0, 1.0), (1220.0, 120.0, 0.6), (2810.0, 200.0, 0.3)],
                [(610.0, 100.0, 1.0), (1900.0, 140.0, 0.5), (2850.0, 200.0, 0.3)],
                [(500.0, 90.0, 1.0), (1000.0, 120.0, 0.6), (2800.0, 200.0, 0.2)],
                [(350.0, 80.0, 1.0), (800.0, 100.0, 0.5), (2900.0, 200.0, 0.2)],
                [(310.0, 70.0, 1.0), (2200.0, 150.0, 0.45), (3000.0, 220.0, 0.25)],
            ];
            let p = f * 4.0; let i = (p.floor() as usize).min(3);
            let set = lerp_set(&sets[i], &sets[i + 1], p - i as f64);
            let mut rng = Lcg(777);
            let phases: Vec<f64> = (0..HALF).map(|_| (rng.next() - 0.5) * 0.9).collect();
            rotate_phase(&apply_env(&saw(), |k| formants(k, 250.0, &set) * 3.0), &phases)
        }
        9 => { // moving comb on a saw
            let d = f * 0.25 + 0.01;
            apply_env(&saw(), |k| 0.5 + 0.5 * (2.0 * PI * k as f64 * d).cos())
        }
        10 => time_gen(|th| (th + (0.2 + 9.0 * f) * (5.0 * th + 0.6 * (th).sin()).sin()).sin()),
        11 => { // rough noise: morphing between random-phase spectra
            let anchors = 4usize;
            let p = f * (anchors - 1) as f64; let i = (p.floor() as usize).min(anchors - 2);
            let mk = |seed: u64| { let mut r = Lcg(seed); shape(|k| z(1.0 / (k as f64).powf(0.55), r.next() * 2.0 * PI)) };
            lerp_spec(&mk(11 + i as u64 * 7), &mk(11 + (i as u64 + 1) * 7), p - i as f64)
        }
        12 => time_gen(|th| { // e-piano tine: FM 1:1 + bark partial
            let i = 0.2 + 2.8 * f;
            (th + i * th.sin()).sin() + 0.25 * i * (14.0 * th).sin() * (-0.0 * th).exp()
        }),
        13 => { // bowed strings: saw with a body resonance moving up the harmonic series
            let kc = 2.0 + 12.0 * f;
            apply_env(&saw(), |k| 1.0 + 3.0 * (-0.5 * ((k as f64 - kc) / 1.6).powi(2)).exp())
        }
        14 => { // hard-sync sweep, ratio 1..8
            let r = 1.0 + 7.0 * f;
            time_gen(|th| { let p = (th / (2.0 * PI) * r).fract(); 2.0 * p - 1.0 })
        }
        15 => { // bass growl: low-reference formants over a pulse, sweeping
            let set = [(250.0 + 650.0 * f, 70.0, 1.0), (900.0 + 1300.0 * f, 130.0, 0.7), (2400.0, 250.0, 0.2)];
            apply_env(&pulse(0.38), |k| formants(k, 55.0, &set) * 2.5)
        }
        _ => sine(),
    }
}

pub struct WaveBank {
    /// tables[t][((frame * N_MIPS) + mip) * TABLE_SIZE + n]
    tables: Vec<Vec<f32>>,
}

impl WaveBank {
    pub fn generate() -> WaveBank {
        let mut tables = Vec::with_capacity(N_TABLES);
        for t in 0..N_TABLES {
            let mut data = vec![0.0f32; N_FRAMES * N_MIPS * TABLE_SIZE];
            for fi in 0..N_FRAMES {
                let spec = frame_spectrum(t, fi);
                let mut scale = 1.0f64;
                for mip in 0..N_MIPS {
                    let maxh = ((HALF >> mip).max(2)).min(HALF - 1) as f64;
                    let mut buf = vec![C::default(); TABLE_SIZE];
                    for k in 1..HALF {
                        let kf = k as f64;
                        if kf > maxh { break; }
                        // gentle taper over the top quarter of the band to tame Gibbs ringing
                        let lo = maxh * 0.75;
                        let w = if kf <= lo { 1.0 } else { 0.5 + 0.5 * (PI * (kf - lo) / (maxh - lo)).cos() };
                        buf[k] = spec[k].scale(w);
                    }
                    fft(&mut buf, 1.0);
                    if mip == 0 {
                        let peak = buf.iter().map(|c| c.re.abs()).fold(0.0, f64::max);
                        scale = if peak > 1e-9 { 0.95 / peak } else { 1.0 };
                    }
                    let base = (fi * N_MIPS + mip) * TABLE_SIZE;
                    for n in 0..TABLE_SIZE { data[base + n] = (buf[n].re * scale) as f32; }
                }
            }
            tables.push(data);
        }
        WaveBank { tables }
    }

    #[inline]
    pub fn slice(&self, table: usize, frame: usize, mip: usize) -> &[f32] {
        let t = &self.tables[table.min(N_TABLES - 1)];
        let o = (frame.min(N_FRAMES - 1) * N_MIPS + mip.min(N_MIPS - 1)) * TABLE_SIZE;
        &t[o..o + TABLE_SIZE]
    }

    /// Mip level that keeps every harmonic below Nyquist for the given frequency.
    #[inline]
    pub fn mip_for(freq: f32, sample_rate: f32) -> usize {
        let maxh = 0.5 * sample_rate / freq.max(1.0);
        if maxh >= HALF as f32 { 0 } else {
            ((HALF as f32 / maxh).log2().ceil().max(0.0) as usize).min(N_MIPS - 1)
        }
    }

    /// Interpolated mip-0 frame for display purposes (`out.len()` points).
    pub fn preview(&self, table: usize, pos: f32, out: &mut [f32]) {
        let fp = pos.clamp(0.0, 1.0) * (N_FRAMES - 1) as f32;
        let f0 = fp.floor() as usize; let f1 = (f0 + 1).min(N_FRAMES - 1); let ff = fp - f0 as f32;
        let a = self.slice(table, f0, 0); let b = self.slice(table, f1, 0);
        let n = out.len();
        for (i, o) in out.iter_mut().enumerate() {
            let idx = (i * TABLE_SIZE / n.max(1)).min(TABLE_SIZE - 1);
            *o = a[idx] * (1.0 - ff) + b[idx] * ff;
        }
    }
}
