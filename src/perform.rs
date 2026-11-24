//! Performance stage: one-key chords, scale-degree chords, chord progressions and the arpeggiator.
//!
//! Sits between incoming note events and the voice engine. Runs on the audio thread once per
//! render block (sample-quantised to 16 samples ~ 0.3 ms), uses only fixed-size arrays (no
//! allocation, no locks) and bounds every loop, so hostile parameter values or key floods cannot
//! stall or overflow it.
//!
//! Pipeline per block:  held keys --> (chord builder) --> shape --> (inversion cycling, octave spread)
//!                      --> (arp pattern | sustained chord) --> note on/off events for the engine.

use crate::params::*;

pub const BLOCK_SAMPLES: f64 = 16.0;
const MAXN: usize = 32; // notes in a chord shape
const MAXS: usize = 96; // notes in an octave-expanded arp sequence
const MAXKEYS: usize = 16; // physically tracked keys
const MAXOUT: usize = 160;

/// Scale intervals (semitones from tonic) for each SCALES entry.
pub const SCALE_IV: [[i32; 7]; 5] = [
    [0, 2, 4, 5, 7, 9, 11], // Major
    [0, 2, 3, 5, 7, 8, 10], // Natural minor
    [0, 2, 3, 5, 7, 9, 10], // Dorian
    [0, 2, 4, 5, 7, 9, 10], // Mixolydian
    [0, 2, 3, 5, 7, 8, 11], // Harmonic minor
];
/// Progression presets as 0-based scale degrees (index 10 = Custom, read from PR_S1..4).
pub const PROG: [[usize; 4]; 10] = [
    [0, 3, 4, 0], // I-IV-V-I
    [0, 4, 5, 3], // I-V-VI-IV
    [1, 4, 0, 0], // II-V-I-I
    [0, 5, 3, 4], // I-VI-IV-V
    [5, 3, 0, 4], // VI-IV-I-V
    [0, 1, 3, 0], // I-II-IV-I
    [0, 3, 1, 4], // I-IV-II-V
    [0, 2, 3, 4], // I-III-IV-V
    [0, 1, 3, 4], // I-II-IV-V
    [0, 6, 5, 4], // I-VII-VI-V
];
const PROG_BEATS: [f64; 4] = [8.0, 4.0, 2.0, 1.0];
const ARP_BEATS: [f64; 8] = [1.0, 0.5, 0.25, 0.125, 1.0 / 3.0, 1.0 / 6.0, 0.75, 0.375];

#[derive(Clone, Copy, Default)]
pub struct PEv { pub on: bool, pub note: u8, pub vel: u8 }

/// Sorted, de-duplicated note list with fixed capacity.
#[derive(Clone, Copy)]
pub struct Notes { pub n: usize, pub v: [i32; MAXS] }
impl Notes {
    pub const fn new() -> Notes { Notes { n: 0, v: [0; MAXS] } }
    fn push(&mut self, x: i32, cap: usize) { if self.n < cap.min(MAXS) { self.v[self.n] = x; self.n += 1; } }
    fn sort_dedup(&mut self) {
        // insertion sort: n is tiny, no allocation
        for i in 1..self.n { let x = self.v[i]; let mut j = i; while j > 0 && self.v[j - 1] > x { self.v[j] = self.v[j - 1]; j -= 1; } self.v[j] = x; }
        let mut w = 0;
        for r in 0..self.n { if w == 0 || self.v[w - 1] != self.v[r] { self.v[w] = self.v[r]; w += 1; } }
        self.n = w;
    }
    /// Raise the lowest `k` notes by an octave (k-th inversion). Wraps for k >= n.
    pub fn invert(&mut self, k: usize) {
        if self.n < 2 { return; }
        for _ in 0..(k % self.n) { self.v[0] += 12; self.sort_dedup(); if self.n < 2 { break; } }
    }
}

#[inline] fn pos(sc: &[i32; 7], k: usize) -> i32 { sc[k % 7] + 12 * (k / 7) as i32 }

/// Build one chord. Returns notes as semitone offsets from `tonic` plus the chord-root offset.
/// `mode`: 1 = One-Key (played note is the root, quality chosen), 2/3 = diatonic degree of the played tonic.
pub fn build_chord(s: &[f32], mode: usize, deg: usize) -> (Notes, i32) {
    let sc = &SCALE_IV[(s[CH_SCALE] as usize).min(4)];
    let ext = (s[CH_EXT] as usize).min(9);
    let (t0, t3, t5, t7);
    if mode == 1 {
        let q = (s[CH_QUAL] as usize).min(3);
        let (a, b, c) = [(4, 7, 11), (3, 7, 10), (3, 6, 9), (4, 8, 10)][q];
        t0 = 0; t3 = a; t5 = b; t7 = c;
    } else {
        let d = deg.min(6);
        t0 = pos(sc, d); t3 = pos(sc, d + 2); t5 = pos(sc, d + 4); t7 = pos(sc, d + 6);
    }
    let mut n = Notes::new();
    let cap = MAXN;
    match ext {
        0 => { n.push(t0, cap); n.push(t3, cap); n.push(t5, cap); }
        1 => { n.push(t0, cap); n.push(t0 + 2, cap); n.push(t5, cap); }
        2 => { n.push(t0, cap); n.push(t0 + 5, cap); n.push(t5, cap); }
        3 => { n.push(t0, cap); n.push(t3, cap); n.push(t5, cap); n.push(t0 + 9, cap); }
        4 => { n.push(t0, cap); n.push(t3, cap); n.push(t5, cap); n.push(t7, cap); }
        5 => { n.push(t0, cap); n.push(t3, cap); n.push(t5, cap); n.push(t0 + 10, cap); }
        6 => { n.push(t0, cap); n.push(t3, cap); n.push(t5, cap); n.push(t7, cap); n.push(t0 + 14, cap); }
        7 => { n.push(t0, cap); n.push(t3, cap); n.push(t5, cap); n.push(t0 + 14, cap); }
        8 => { n.push(t0, cap); n.push(t0 + 7, cap); n.push(t0 + 12, cap); }
        _ => { n.push(t0, cap); n.push(t0 + 12, cap); }
    }
    n.sort_dedup();
    n.invert((s[CH_INV] as usize).min(3));
    if s[CH_OPEN] >= 0.5 && n.n >= 3 { n.v[n.n - 2] -= 12; n.sort_dedup(); } // "drop 2" open voicing
    if s[CH_BASS] >= 0.5 { n.push(t0 - 12, cap); n.sort_dedup(); }
    (n, t0)
}

/// Pattern step -> index into an ascending sequence of `m` notes. Returns (index, cycle length).
fn pat_index(pat: usize, m: usize, step: u32, rnd: &mut u32) -> (usize, u32) {
    let m = m.max(1);
    let s = step as usize;
    let conv = |k: usize| if k % 2 == 0 { k / 2 } else { m - 1 - k / 2 };
    let (i, l) = match pat {
        0 => (s % m, m),
        1 => (m - 1 - s % m, m),
        2 => { let l = (2 * m).saturating_sub(2).max(1); let k = s % l; (if k < m { k } else { 2 * m - 2 - k }, l) }
        3 => { let l = (2 * m).saturating_sub(2).max(1); let k = s % l; (m - 1 - (if k < m { k } else { 2 * m - 2 - k }), l) }
        4 => { let l = 2 * m; let k = s % l; (if k < m { k } else { 2 * m - 1 - k }, l) }
        5 => (s % m, m), // "Played" order handled by the caller (unsorted source)
        6 => { let mut x = *rnd; x ^= x << 13; x ^= x >> 17; x ^= x << 5; *rnd = x; ((x >> 8) as usize % m, m) }
        7 => (conv(s % m), m),
        8 => (conv(m - 1 - s % m), m),
        9 => if m < 2 { (0, 1) } else { let l = 2 * (m - 1); let k = s % l; (if k % 2 == 0 { 0 } else { 1 + (k / 2) % (m - 1) }, l) },
        10 => if m < 2 { (0, 1) } else { let l = 2 * (m - 1); let k = s % l; (if k % 2 == 0 { m - 1 } else { (k / 2) % (m - 1) }, l) },
        11 => { let l = 3 * m.saturating_sub(1).max(1); let k = s % l; ((k / 3 + [0, 1, 0][k % 3]).min(m - 1), l) }
        12 => { let k = s % 4; ([0, m - 1, 1.min(m - 1), m - 1][k], 4) }
        _ => (0, 1),
    };
    (i.min(m - 1), l.max(1) as u32)
}

pub struct Performer {
    held: [(u8, u8); MAXKEYS], nheld: usize, // physical keys in press order
    src: [(u8, u8); MAXKEYS], nsrc: usize,   // effective source (differs from held while latched)
    owned: [u8; 64], nowned: usize,          // notes this stage currently holds on in the engine
    ovel: u8,
    pend: [(i32, u8, u8); 64], npend: usize, // humanized note-ons waiting (samples left, note, vel)
    pub out: [PEv; MAXOUT], pub nout: usize,
    beat: f64, start: f64, step: u32, cycle: u32, gate_off: f64, running: bool, rnd: u32,
    pub info_root: i32, pub info_mask: u32,  // for the UI chord display (-1 = none)
}

impl Performer {
    pub fn new() -> Performer {
        Performer {
            held: [(0, 0); MAXKEYS], nheld: 0, src: [(0, 0); MAXKEYS], nsrc: 0,
            owned: [0; 64], nowned: 0, ovel: 100, pend: [(0, 0, 0); 64], npend: 0, out: [PEv::default(); MAXOUT], nout: 0,
            beat: 0.0, start: 0.0, step: 0, cycle: 0, gate_off: -1.0, running: false, rnd: 0x2545F491,
            info_root: -1, info_mask: 0,
        }
    }
    pub fn held(&self) -> &[(u8, u8)] { &self.held[..self.nheld] }

    /// The voices were released from outside (instrument change): forget which notes we think are sounding so the
    /// next block re-triggers them, but keep held/latched keys, the progression position and the arp clock.
    pub fn resync(&mut self) { self.nowned = 0; self.npend = 0; self.gate_off = -1.0; }

    pub fn reset(&mut self) { self.nheld = 0; self.nsrc = 0; self.running = false; }

    pub fn track_on(&mut self, n: u8, v: u8, latch: bool) {
        if latch && self.nheld == 0 { self.nsrc = 0; self.running = false; } // new gesture replaces the latched set
        if let Some(i) = self.held[..self.nheld].iter().position(|k| k.0 == n) { self.held.copy_within(i + 1..self.nheld, i); self.nheld -= 1; }
        if self.nheld == MAXKEYS { self.held.copy_within(1..MAXKEYS, 0); self.nheld -= 1; }
        self.held[self.nheld] = (n, v); self.nheld += 1;
        if let Some(i) = self.src[..self.nsrc].iter().position(|k| k.0 == n) { self.src.copy_within(i + 1..self.nsrc, i); self.nsrc -= 1; }
        if self.nsrc == MAXKEYS { self.src.copy_within(1..MAXKEYS, 0); self.nsrc -= 1; }
        if self.nsrc == 0 { self.running = false; }
        self.src[self.nsrc] = (n, v); self.nsrc += 1;
    }

    pub fn track_off(&mut self, n: u8, latch: bool) {
        if let Some(i) = self.held[..self.nheld].iter().position(|k| k.0 == n) { self.held.copy_within(i + 1..self.nheld, i); self.nheld -= 1; }
        if !latch {
            if let Some(i) = self.src[..self.nsrc].iter().position(|k| k.0 == n) { self.src.copy_within(i + 1..self.nsrc, i); self.nsrc -= 1; }
        }
    }

    /// Called when the latch switch is turned off: drop the latched set unless keys are really held.
    pub fn unlatch(&mut self) { self.nsrc = self.nheld; self.src = self.held; }

    fn emit(&mut self, on: bool, note: i32, vel: u8) {
        if !(0..=127).contains(&note) || self.nout >= MAXOUT { return; }
        let note = note as u8;
        if on {
            if self.nowned < self.owned.len() && !self.owned[..self.nowned].contains(&note) { self.owned[self.nowned] = note; self.nowned += 1; }
        } else if let Some(i) = self.owned[..self.nowned].iter().position(|&x| x == note) {
            self.owned.copy_within(i + 1..self.nowned, i); self.nowned -= 1;
        }
        self.out[self.nout] = PEv { on, note, vel }; self.nout += 1;
    }
    fn release_owned(&mut self) {
        self.npend = 0; // pending (humanized) note-ons are cancelled together with the sounding notes
        while self.nowned > 0 && self.nout < MAXOUT { let n = self.owned[self.nowned - 1] as i32; self.emit(false, n, 0); }
    }

    #[inline] fn rnd01(&mut self) -> f32 { let mut x = self.rnd; x ^= x << 13; x ^= x >> 17; x ^= x << 5; self.rnd = x; (x >> 8) as f32 / 16777216.0 }

    fn is_pending(&self, note: i32) -> bool { self.pend[..self.npend].iter().any(|p| p.1 as i32 == note) }

    /// Note-on with optional humanization: random late timing (strum / laid-back feel) and velocity spread.
    /// `max_delay` (samples) keeps the delay inside the note's own step so notes never land after their gate.
    fn emit_on_h(&mut self, note: i32, vel: u8, s: &[f32], sr: f32, max_delay: f32) {
        let (ht, hv) = (s[HUM_TIME], s[HUM_VEL]);
        let v = if hv > 0.0 { ((vel as f32) * (1.0 + (self.rnd01() - 0.5) * 2.0 * hv * 0.3)).clamp(1.0, 127.0) as u8 } else { vel };
        let delay = if ht > 0.0 { (self.rnd01() * ht * 0.035 * sr).min(max_delay.max(0.0)) } else { 0.0 };
        if delay < BLOCK_SAMPLES as f32 || self.npend >= self.pend.len() || !(0..=127).contains(&note) {
            self.emit(true, note, v);
        } else {
            self.pend[self.npend] = (delay as i32, note as u8, v); self.npend += 1;
        }
    }

    /// Chord shape (absolute MIDI notes, ascending) for the current source keys. Also reports the chord root.
    fn shape(&self, s: &[f32], deg: usize) -> (Notes, i32, u32) {
        let mode = (s[CH_MODE] as usize).min(3);
        let mut all = Notes::new();
        let (mut root, mut mask) = (-1i32, 0u32);
        match mode {
            0 => for k in &self.src[..self.nsrc] { all.push(k.0 as i32, MAXN); },
            1 => for (i, k) in self.src[..self.nsrc].iter().take(8).enumerate() {
                let (c, t0) = build_chord(s, 1, 0);
                for j in 0..c.n { all.push(k.0 as i32 + c.v[j], MAXN); }
                if i == 0 { root = k.0 as i32 + t0; mask = chord_mask(&c, t0); }
            },
            _ => if self.nsrc > 0 {
                let tonic = self.src[self.nsrc - 1].0 as i32;
                let (c, t0) = build_chord(s, mode, deg);
                for j in 0..c.n { all.push(tonic + c.v[j], MAXN); }
                root = tonic + t0; mask = chord_mask(&c, t0);
            },
        }
        all.sort_dedup();
        (all, root, mask)
    }

    /// Advance one render block. Returns the number of events in `self.out`.
    pub fn process(&mut self, s: &[f32], bpm: f32, sr: f32) -> usize {
        self.nout = 0;
        // humanized notes whose delay has elapsed
        let mut i = 0;
        while i < self.npend {
            self.pend[i].0 -= BLOCK_SAMPLES as i32;
            if self.pend[i].0 <= 0 { let (_, n, v) = self.pend[i]; self.pend.copy_within(i + 1..self.npend, i); self.npend -= 1; self.emit(true, n as i32, v); } else { i += 1; }
        }
        let mode = (s[CH_MODE] as usize).min(3);
        let arp = s[AR_ON] >= 0.5;
        let dt = (bpm.clamp(20.0, 300.0) as f64 / 60.0) * BLOCK_SAMPLES / (sr.max(8000.0) as f64);
        if (mode == 0 && !arp) || self.nsrc == 0 {
            self.release_owned(); self.running = false; self.beat += dt; self.info_root = -1; return self.nout;
        }
        if !self.running { self.running = true; self.start = self.beat; self.step = 0; self.cycle = 0; self.gate_off = -1.0; }
        let end = self.beat + dt;

        // progression: chord degree follows elapsed beats since the first key went down
        let mut deg = (s[CH_DEG] as usize).min(6);
        if mode == 3 {
            let len = PROG_BEATS[(s[PR_LEN] as usize).min(3)];
            let ps = (((self.beat - self.start).max(0.0) / len) as usize) % 4;
            let pat = (s[PR_PAT] as usize).min(10);
            deg = if pat >= 10 { (s[PR_S1 + ps] as usize).min(6) } else { PROG[pat][ps] };
        }
        let (base, root, mask) = self.shape(s, deg);
        self.info_root = root; self.info_mask = mask;
        self.ovel = self.src[self.nsrc - 1].1.max(1);
        let vel = self.ovel;

        if !arp {
            // sustained chord: diff the desired note set against what is sounding (or about to sound)
            let mut i = 0;
            while i < self.nowned && self.nout < MAXOUT {
                let n = self.owned[i] as i32;
                if !base.v[..base.n].contains(&n) { self.emit(false, n, 0); } else { i += 1; }
            }
            let mut j = 0;
            while j < self.npend { if base.v[..base.n].contains(&(self.pend[j].1 as i32)) { j += 1; } else { self.pend.copy_within(j + 1..self.npend, j); self.npend -= 1; } }
            for j in 0..base.n {
                let n = base.v[j];
                if !self.owned[..self.nowned].contains(&(n.clamp(0, 127) as u8)) && !self.is_pending(n) { self.emit_on_h(n, vel, s, sr, sr * 0.1); }
            }
            self.beat = end;
            return self.nout;
        }

        // ---- arpeggiator
        let l = ARP_BEATS[(s[AR_RATE] as usize).min(7)];
        let swing = s[AR_SWING] as f64;
        let st = self.start;
        let t_of = move |n: u32| -> f64 { st + n as f64 * l + if n & 1 == 1 { swing * l * 0.5 } else { 0.0 } };
        if self.gate_off >= 0.0 && self.gate_off < self.beat { self.release_owned(); self.gate_off = -1.0; }
        let octs = (s[AR_OCT] as usize).min(3) + 1;
        let pat = (s[AR_PAT] as usize).min(13);
        let invm = (s[AR_INVM] as usize).min(3);
        let mut fired = 0;
        while t_of(self.step) < end && fired < 4 {
            fired += 1;
            self.release_owned();
            let mut shape = base;
            if pat == 5 && mode == 0 { // "Played": keep key-press order
                shape.n = 0; for k in &self.src[..self.nsrc] { shape.push(k.0 as i32, MAXN); }
            } else if shape.n > 1 && invm != 0 {
                let n = shape.n;
                let k = match invm { 1 => self.cycle as usize % n, 2 => (n - self.cycle as usize % n) % n, _ => { let p = self.cycle as usize % (2 * n - 2).max(1); if p < n { p } else { 2 * n - 2 - p } } };
                shape.invert(k);
            }
            let mut seq = Notes::new();
            for o in 0..octs { for j in 0..shape.n { let x = shape.v[j] + 12 * o as i32; if x <= 127 { seq.push(x, MAXS); } } }
            if seq.n == 0 { self.step += 1; continue; }
            let acc = s[AR_ACC];
            let accent = if self.step % 4 == 0 { 1.0 + 0.3 * acc } else { 1.0 - 0.15 * acc };
            let v = ((vel as f32) * accent).clamp(1.0, 127.0) as u8;
            let (idx, cyc_len) = pat_index(pat, seq.n, self.step, &mut self.rnd);
            let step_samples = (l * 60.0 / bpm.clamp(20.0, 300.0) as f64) as f32 * sr;
            let maxd = step_samples * 0.45;
            if pat == 13 {
                for j in 0..shape.n { self.emit_on_h(shape.v[j], v, s, sr, maxd); }
            } else {
                self.emit_on_h(seq.v[idx], v, s, sr, maxd);
            }
            let hg = s[HUM_GATE] as f64;
            let gate = if hg > 0.0 { ((s[AR_GATE] as f64) * (1.0 + (self.rnd01() as f64 - 0.5) * hg * 0.8)).clamp(0.02, 1.0) } else { s[AR_GATE] as f64 };
            self.gate_off = t_of(self.step) + gate * l;
            self.step = self.step.wrapping_add(1);
            if self.step % cyc_len == 0 { self.cycle = self.cycle.wrapping_add(1); }
        }
        if self.gate_off >= 0.0 && self.gate_off < end { self.release_owned(); self.gate_off = -1.0; }
        self.beat = end;
        self.nout
    }
}

fn chord_mask(c: &Notes, t0: i32) -> u32 {
    let mut m = 0u32;
    for j in 0..c.n { let d = c.v[j] - t0; if (0..24).contains(&d) { m |= 1 << d; } }
    m
}

/// Human-readable chord name from a root note and interval mask (UI only; may allocate).
pub fn chord_name(root: i32, mask: u32) -> String {
    if !(0..128).contains(&root) { return String::new(); }
    const N: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let has = |d: u32| mask & (1 << d) != 0;
    let mut s = String::from(N[(root % 12) as usize]);
    if has(7) && has(12) && !has(3) && !has(4) { s.push('5'); return s; }
    if has(2) && !has(3) && !has(4) { s.push_str("sus2"); if has(10) { s.push('7'); } return s; }
    if has(5) && !has(3) && !has(4) { s.push_str("sus4"); if has(10) { s.push('7'); } return s; }
    let minor = has(3);
    if minor && has(6) && !has(7) { s.push_str(if has(9) { "dim7" } else { "dim" }); return s; }
    if has(4) && has(8) && !has(7) { s.push_str("aug"); return s; }
    if minor { s.push('m'); }
    if has(14) { s.push_str(if has(10) { "9" } else if has(11) { "maj9" } else { "add9" }); }
    else if has(11) { s.push_str("maj7"); }
    else if has(10) { s.push('7'); }
    else if has(9) { s.push('6'); }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap() -> Vec<f32> { default_patch() }
    fn set(s: &mut Vec<f32>, i: usize, v: f32) { s[i] = PARAMS[i].clamp(v); }

    #[test]
    fn degree_chords_are_diatonic() {
        let mut s = snap();
        set(&mut s, CH_MODE, 2.0);
        // C major: I = C E G, IV = F A C, II = D F A
        let (c, t0) = build_chord(&s, 2, 0); assert_eq!((t0, &c.v[..c.n]), (0, &[0, 4, 7][..]));
        let (c, t0) = build_chord(&s, 2, 3); assert_eq!((t0, &c.v[..c.n]), (5, &[5, 9, 12][..]));
        let (c, t0) = build_chord(&s, 2, 1); assert_eq!((t0, &c.v[..c.n]), (2, &[2, 5, 9][..])); // D minor
        set(&mut s, CH_EXT, 4.0); // 7th
        let (c, _) = build_chord(&s, 2, 4); assert_eq!(&c.v[..c.n], &[7, 11, 14, 17]); // G7 (G B D F)
    }

    #[test]
    fn inversions_open_and_bass() {
        let mut s = snap();
        set(&mut s, CH_INV, 1.0);
        let (c, _) = build_chord(&s, 1, 0); assert_eq!(&c.v[..c.n], &[4, 7, 12]);
        set(&mut s, CH_INV, 2.0);
        let (c, _) = build_chord(&s, 1, 0); assert_eq!(&c.v[..c.n], &[7, 12, 16]);
        set(&mut s, CH_INV, 0.0); set(&mut s, CH_BASS, 1.0);
        let (c, _) = build_chord(&s, 1, 0); assert_eq!(&c.v[..c.n], &[-12, 0, 4, 7]);
        set(&mut s, CH_BASS, 0.0); set(&mut s, CH_OPEN, 1.0);
        let (c, _) = build_chord(&s, 1, 0); assert_eq!(&c.v[..c.n], &[-8, 0, 7]); // drop-2
    }

    #[test]
    fn arp_patterns_visit_expected_order() {
        let mut r = 1u32;
        let mut seq = |pat: usize, m: usize, len: usize| -> Vec<usize> { (0..len as u32).map(|s| pat_index(pat, m, s, &mut r).0).collect() };
        assert_eq!(seq(0, 3, 6), [0, 1, 2, 0, 1, 2]);
        assert_eq!(seq(1, 3, 4), [2, 1, 0, 2]);
        assert_eq!(seq(2, 3, 8), [0, 1, 2, 1, 0, 1, 2, 1]);
        assert_eq!(seq(4, 3, 6), [0, 1, 2, 2, 1, 0]);
        assert_eq!(seq(7, 4, 4), [0, 3, 1, 2]);
        assert_eq!(seq(9, 3, 4), [0, 1, 0, 2]);
        assert_eq!(seq(10, 3, 4), [2, 0, 2, 1]);
        assert_eq!(seq(12, 3, 4), [0, 2, 1, 2]);
        for p in 0..14 { for m in 1..9 { for st in 0..40 { let mut r2 = 7u32; assert!(pat_index(p, m, st, &mut r2).0 < m); } } }
    }

    #[test]
    fn chord_names() {
        let mk = |iv: &[i32]| iv.iter().fold(0u32, |m, d| m | 1 << d);
        assert_eq!(chord_name(60, mk(&[0, 4, 7])), "C");
        assert_eq!(chord_name(65, mk(&[0, 4, 7, 11])), "Fmaj7");
        assert_eq!(chord_name(62, mk(&[0, 3, 7, 10])), "Dm7");
        assert_eq!(chord_name(67, mk(&[0, 7, 12])), "G5");
        assert_eq!(chord_name(59, mk(&[0, 3, 6])), "Bdim");
    }

    #[test]
    fn progression_steps_through_degrees_and_arp_emits_balanced_events() {
        let mut s = snap();
        set(&mut s, CH_MODE, 3.0); set(&mut s, PR_PAT, 5.0); set(&mut s, PR_LEN, 3.0); // I-II-IV-I, 1 beat each
        let mut p = Performer::new();
        p.track_on(60, 100, false);
        let mut roots = vec![];
        let blocks_per_beat = (48000.0f64 / 16.0 / 2.0) as usize; // 120 bpm
        for b in 0..blocks_per_beat * 4 {
            p.process(&s, 120.0, 48000.0);
            if b % blocks_per_beat == blocks_per_beat / 2 { roots.push(p.info_root); }
        }
        assert_eq!(roots, [60, 62, 65, 60]);
        p.track_off(60, false);
        p.process(&s, 120.0, 48000.0);
        assert_eq!(p.nowned, 0, "all notes released when keys go up");

        // arpeggio: every note-on is eventually matched by a note-off, nothing hangs
        set(&mut s, CH_MODE, 1.0); set(&mut s, AR_ON, 1.0); set(&mut s, AR_PAT, 6.0); set(&mut s, AR_OCT, 3.0);
        let mut p = Performer::new();
        p.track_on(48, 90, false); p.track_on(55, 90, false);
        let (mut on, mut off) = (0, 0);
        for _ in 0..20000 { let n = p.process(&s, 140.0, 44100.0); for e in &p.out[..n] { if e.on { on += 1 } else { off += 1 } } }
        p.track_off(48, false); p.track_off(55, false);
        let n = p.process(&s, 140.0, 44100.0); for e in &p.out[..n] { if e.on { on += 1 } else { off += 1 } }
        assert!(on > 50 && on == off, "on={on} off={off}");
    }

    #[test]
    fn latch_holds_until_new_gesture() {
        let mut p = Performer::new();
        p.track_on(60, 100, true); p.track_off(60, true);
        assert_eq!(p.nsrc, 1); // latched
        p.track_on(64, 100, true);
        assert_eq!((p.nsrc, p.src[0].0), (1, 64)); // replaced by new gesture
        p.track_off(64, false); p.unlatch();
        assert_eq!(p.nsrc, 0);
    }

    #[test]
    fn humanize_varies_velocity_and_timing_but_never_hangs_notes() {
        for chord_only in [false, true] {
            let mut s = snap();
            set(&mut s, CH_MODE, 1.0); set(&mut s, HUM_TIME, 1.0); set(&mut s, HUM_VEL, 1.0); set(&mut s, HUM_GATE, 1.0);
            if !chord_only { set(&mut s, AR_ON, 1.0); set(&mut s, AR_PAT, 6.0); set(&mut s, AR_RATE, 3.0); }
            let mut p = Performer::new();
            p.track_on(50, 100, false);
            let (mut on, mut off, mut vels) = (0, 0, std::collections::BTreeSet::new());
            for blk in 0..6000 {
                if chord_only && blk % 1500 == 0 { p.track_off(50, false); p.track_on(50 + (blk / 1500) as u8, 100, false); }
                let n = p.process(&s, 150.0, 48000.0);
                for e in &p.out[..n] { if e.on { on += 1; vels.insert(e.vel); } else { off += 1; } }
            }
            p.track_off(50, false); p.track_off(51, false); p.track_off(52, false); p.track_off(53, false);
            for _ in 0..4 { let n = p.process(&s, 150.0, 48000.0); for e in &p.out[..n] { if e.on { on += 1 } else { off += 1 } } }
            assert!(on > 3 && on == off, "chord_only={chord_only} on={on} off={off}");
            assert!(vels.len() > 2, "velocities should vary: {vels:?}");
            assert_eq!(p.nowned, 0);
        }
    }
}
