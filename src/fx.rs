//! Effects chain: distortion -> chorus -> delay -> reverb -> DC blocker. All buffers are
//! pre-allocated in `Fx::new`; nothing here allocates on the audio thread.

use crate::params::*;

#[inline] pub fn tanh_fast(x: f32) -> f32 {
    let x = x.clamp(-3.0, 3.0);
    x * (27.0 + x * x) / (27.0 + 9.0 * x * x)
}
#[inline] pub fn flush(x: f32) -> f32 { if x.abs() < 1e-18 { 0.0 } else { x } }

struct Comb { buf: Vec<f32>, pos: usize, store: f32 }
impl Comb {
    fn new(n: usize) -> Comb { Comb { buf: vec![0.0; n.max(1)], pos: 0, store: 0.0 } }
    #[inline] fn tick(&mut self, x: f32, fb: f32, damp: f32) -> f32 {
        let out = self.buf[self.pos];
        self.store = flush(out * (1.0 - damp) + self.store * damp);
        self.buf[self.pos] = flush(x + self.store * fb);
        self.pos += 1; if self.pos >= self.buf.len() { self.pos = 0; }
        out
    }
}
struct Allpass { buf: Vec<f32>, pos: usize }
impl Allpass {
    fn new(n: usize) -> Allpass { Allpass { buf: vec![0.0; n.max(1)], pos: 0 } }
    #[inline] fn tick(&mut self, x: f32) -> f32 {
        let b = self.buf[self.pos];
        let out = -x + b;
        self.buf[self.pos] = flush(x + b * 0.5);
        self.pos += 1; if self.pos >= self.buf.len() { self.pos = 0; }
        out
    }
}

pub struct Fx {
    sr: f32,
    // distortion
    d_on: bool, d_type: usize, d_gain: f32, d_drive: f32, d_mix: f32, d_comp: f32,
    dec_cnt: [f32; 2], dec_hold: [f32; 2],
    // chorus
    c_on: bool, c_rate: f32, c_depth: f32, c_delay: f32, c_mix: f32,
    ch_buf: [Vec<f32>; 2], ch_mask: usize, ch_pos: usize, ch_ph: f32,
    // delay
    dl_on: bool, dl_target: f32, dl_cur: f32, dl_fb: f32, dl_lp_a: f32, dl_mix: f32, dl_pp: bool,
    dl_buf: [Vec<f32>; 2], dl_pos: usize, dl_lp: [f32; 2],
    // reverb
    r_on: bool, r_fb: f32, r_damp: f32, r_mix: f32,
    combs: [Vec<Comb>; 2], allp: [Vec<Allpass>; 2],
    // dc blocker
    dc_x: [f32; 2], dc_y: [f32; 2],
}

const COMB_LEN: [usize; 8] = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
const ALLP_LEN: [usize; 4] = [556, 441, 341, 225];

impl Fx {
    pub fn new(sr: f32) -> Fx {
        let ch_n = ((0.06 * sr) as usize).next_power_of_two();
        let dl_n = (2.1 * sr) as usize + 8;
        let sc = sr / 44100.0;
        let mk_c = |extra: usize| COMB_LEN.iter().map(|&n| Comb::new(((n + extra) as f32 * sc) as usize)).collect::<Vec<_>>();
        let mk_a = |extra: usize| ALLP_LEN.iter().map(|&n| Allpass::new(((n + extra) as f32 * sc) as usize)).collect::<Vec<_>>();
        Fx {
            sr,
            d_on: false, d_type: 0, d_gain: 1.0, d_drive: 0.0, d_mix: 1.0, d_comp: 1.0,
            dec_cnt: [0.0; 2], dec_hold: [0.0; 2],
            c_on: false, c_rate: 0.6, c_depth: 0.5, c_delay: 12.0, c_mix: 0.4,
            ch_buf: [vec![0.0; ch_n], vec![0.0; ch_n]], ch_mask: ch_n - 1, ch_pos: 0, ch_ph: 0.0,
            dl_on: false, dl_target: 0.3 * sr, dl_cur: 0.3 * sr, dl_fb: 0.3, dl_lp_a: 0.5, dl_mix: 0.3, dl_pp: true,
            dl_buf: [vec![0.0; dl_n], vec![0.0; dl_n]], dl_pos: 0, dl_lp: [0.0; 2],
            r_on: false, r_fb: 0.8, r_damp: 0.4, r_mix: 0.2,
            combs: [mk_c(0), mk_c(23)], allp: [mk_a(0), mk_a(23)],
            dc_x: [0.0; 2], dc_y: [0.0; 2],
        }
    }

    /// Cache per-block constants from the (smoothed) parameter snapshot.
    pub fn begin_block(&mut self, s: &[f32], bpm: f32) {
        self.d_on = s[D_ON] >= 0.5;
        self.d_type = s[D_TYPE] as usize;
        self.d_drive = s[D_DRIVE];
        self.d_gain = 1.0 + self.d_drive * self.d_drive * 60.0 + self.d_drive * 2.0;
        self.d_mix = s[D_MIX];
        self.d_comp = 1.0 / (1.0 + self.d_drive * 1.3);
        self.c_on = s[C_ON] >= 0.5;
        self.c_rate = s[C_RATE]; self.c_depth = s[C_DEPTH]; self.c_delay = s[C_DELAY]; self.c_mix = s[C_MIX];
        self.dl_on = s[DL_ON] >= 0.5;
        let sync = (s[DL_SYNC] as usize).min(11);
        let secs = if sync > 0 { SYNC_BEATS[sync] * 60.0 / bpm.max(1.0) } else { s[DL_TIME] };
        self.dl_target = (secs.clamp(0.01, 1.95) * self.sr).max(2.0);
        self.dl_fb = s[DL_FB];
        self.dl_lp_a = 1.0 - (-2.0 * std::f32::consts::PI * s[DL_TONE] / self.sr).exp();
        self.dl_mix = s[DL_MIX];
        self.dl_pp = s[DL_PP] >= 0.5;
        self.r_on = s[R_ON] >= 0.5;
        self.r_fb = 0.7 + 0.28 * s[R_SIZE];
        self.r_damp = 0.05 + 0.85 * s[R_DAMP];
        self.r_mix = s[R_MIX];
    }

    #[inline]
    fn shape(&mut self, ch: usize, x: f32) -> f32 {
        match self.d_type {
            0 => tanh_fast(x * self.d_gain),
            1 => (x * self.d_gain).clamp(-1.0, 1.0),
            2 => { let y = x * self.d_gain; let t = (y + 1.0).rem_euclid(4.0); if t < 2.0 { t - 1.0 } else { 3.0 - t } }
            3 => tanh_fast(x * self.d_gain * 0.8 + 0.3) - tanh_fast(0.3),
            4 => { let bits = 16.0 - self.d_drive * 14.0; let lv = (2.0f32).powf(bits.max(2.0)) * 0.5; (x * lv).round() / lv }
            _ => {
                let n = 1.0 + self.d_drive * self.d_drive * 40.0;
                self.dec_cnt[ch] += 1.0;
                if self.dec_cnt[ch] >= n { self.dec_cnt[ch] -= n; self.dec_hold[ch] = x; }
                self.dec_hold[ch]
            }
        }
    }

    #[inline]
    pub fn process(&mut self, mut l: f32, mut r: f32) -> (f32, f32) {
        if self.d_on {
            let (dl, dr) = (l, r);
            let wl = self.shape(0, l) * self.d_comp;
            let wr = self.shape(1, r) * self.d_comp;
            l = dl + (wl - dl) * self.d_mix;
            r = dr + (wr - dr) * self.d_mix;
        }
        // chorus (also keeps writing when off so switching on never clicks on stale data)
        {
            let p = self.ch_pos;
            self.ch_buf[0][p] = l; self.ch_buf[1][p] = r;
            if self.c_on {
                self.ch_ph += self.c_rate / self.sr; if self.ch_ph >= 1.0 { self.ch_ph -= 1.0; }
                let mut wet = [0.0f32; 2];
                for ch in 0..2 {
                    let mut acc = 0.0;
                    for tap in 0..2 {
                        let off = ch as f32 * 0.25 + tap as f32 * 0.5;
                        let lfo = (2.0 * std::f32::consts::PI * (self.ch_ph + off)).sin();
                        let d_ms = (self.c_delay + self.c_depth * 8.0 * lfo).max(0.5);
                        let d = d_ms * 0.001 * self.sr;
                        let rp = p as f32 - d;
                        let ri = rp.floor(); let fr = rp - ri;
                        let i0 = (ri as isize).rem_euclid(self.ch_mask as isize + 1) as usize;
                        let i1 = (i0 + 1) & self.ch_mask;
                        acc += self.ch_buf[ch][i0] * (1.0 - fr) + self.ch_buf[ch][i1] * fr;
                    }
                    wet[ch] = acc * 0.5;
                }
                l = l * (1.0 - self.c_mix * 0.5) + wet[0] * self.c_mix;
                r = r * (1.0 - self.c_mix * 0.5) + wet[1] * self.c_mix;
            }
            self.ch_pos = (p + 1) & self.ch_mask;
        }
        // delay
        if self.dl_on {
            self.dl_cur += (self.dl_target - self.dl_cur) * 0.0004;
            let n = self.dl_buf[0].len();
            let rp = self.dl_pos as f32 - self.dl_cur;
            let ri = rp.floor(); let fr = rp - ri;
            let i0 = (ri as isize).rem_euclid(n as isize) as usize;
            let i1 = if i0 + 1 >= n { 0 } else { i0 + 1 };
            let dl = self.dl_buf[0][i0] * (1.0 - fr) + self.dl_buf[0][i1] * fr;
            let dr = self.dl_buf[1][i0] * (1.0 - fr) + self.dl_buf[1][i1] * fr;
            self.dl_lp[0] = flush(self.dl_lp[0] + (dl - self.dl_lp[0]) * self.dl_lp_a);
            self.dl_lp[1] = flush(self.dl_lp[1] + (dr - self.dl_lp[1]) * self.dl_lp_a);
            let (wl, wr) = if self.dl_pp {
                ((l + r) * 0.5 + self.dl_lp[1] * self.dl_fb, self.dl_lp[0] * self.dl_fb)
            } else {
                (l + self.dl_lp[0] * self.dl_fb, r + self.dl_lp[1] * self.dl_fb)
            };
            self.dl_buf[0][self.dl_pos] = flush(tanh_fast(wl));
            self.dl_buf[1][self.dl_pos] = flush(tanh_fast(wr));
            self.dl_pos += 1; if self.dl_pos >= n { self.dl_pos = 0; }
            l += dl * self.dl_mix; r += dr * self.dl_mix;
        }
        // reverb
        if self.r_on {
            let inp = (l + r) * 0.5 * 0.015;
            let mut wl = 0.0; let mut wr = 0.0;
            for c in self.combs[0].iter_mut() { wl += c.tick(inp, self.r_fb, self.r_damp); }
            for c in self.combs[1].iter_mut() { wr += c.tick(inp, self.r_fb, self.r_damp); }
            for a in self.allp[0].iter_mut() { wl = a.tick(wl); }
            for a in self.allp[1].iter_mut() { wr = a.tick(wr); }
            l = l * (1.0 - self.r_mix * 0.4) + wl * 3.0 * self.r_mix;
            r = r * (1.0 - self.r_mix * 0.4) + wr * 3.0 * self.r_mix;
        }
        // DC blocker
        let ol = l - self.dc_x[0] + 0.9995 * self.dc_y[0]; self.dc_x[0] = l; self.dc_y[0] = flush(ol);
        let or = r - self.dc_x[1] + 0.9995 * self.dc_y[1]; self.dc_x[1] = r; self.dc_y[1] = flush(or);
        (ol, or)
    }
}
