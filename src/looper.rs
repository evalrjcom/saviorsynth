//! Audio looper: records the live output (synth + drums) for a fixed number of bars (tempo-derived) or
//! free length, then loops it with unlimited overdub passes and a one-level overdub undo.
//!
//! Real-time rules: both layers are allocated once in `new`, nothing allocates afterwards, and the
//! (potentially multi-megabyte) clears for Clear / Undo are spread over many render blocks instead of
//! one big memset, so a button press can never cause an audio dropout.

pub const MAX_SECS: usize = 32;
const FADE: usize = 128; // frames of click-free fade at the loop seam
const CLEAR_CHUNK: usize = 8192; // frames cleared per block

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LState { Empty = 0, Rec = 1, Play = 2, Dub = 3, Stop = 4 }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cmd { RecDub = 0, PlayStop = 1, Undo = 2, Clear = 3 }
impl Cmd { pub fn from_u8(x: u8) -> Option<Cmd> { match x { 0 => Some(Cmd::RecDub), 1 => Some(Cmd::PlayStop), 2 => Some(Cmd::Undo), 3 => Some(Cmd::Clear), _ => None } } }

const BARS: [usize; 5] = [0, 1, 2, 4, 8]; // index 0 = free length

pub struct Looper {
    a: Vec<f32>, b: Vec<f32>, // base take / overdub layer (interleaved stereo)
    cap: usize, len: usize, pos: usize, target: usize,
    pub state: LState,
    clr_a: usize, clr_b: usize, // frames still to clear (counting down)
}

impl Looper {
    pub fn new(sr: f32) -> Looper {
        let cap = (sr.max(8000.0) as usize) * MAX_SECS;
        Looper { a: vec![0.0; cap * 2], b: vec![0.0; cap * 2], cap, len: 0, pos: 0, target: 0, state: LState::Empty, clr_a: 0, clr_b: 0 }
    }
    fn clearing(&self) -> bool { self.clr_a > 0 || self.clr_b > 0 }
    pub fn progress(&self) -> f32 {
        match self.state {
            LState::Rec => if self.target > 0 { self.pos as f32 / self.target as f32 } else { (self.pos as f32 / self.cap as f32).min(1.0) },
            LState::Play | LState::Dub | LState::Stop => if self.len > 0 { self.pos as f32 / self.len as f32 } else { 0.0 },
            _ => 0.0,
        }
    }
    #[allow(dead_code)]
    pub fn len_frames(&self) -> usize { self.len }
    #[allow(dead_code)]
    pub fn is_clearing(&self) -> bool { self.clearing() }

    fn finish_rec(&mut self) {
        let n = self.pos;
        if n < 256 { self.state = LState::Empty; self.pos = 0; self.clr_a = n; return; }
        // fade the seam so the loop point never clicks
        let f = FADE.min(n / 2);
        for i in 0..f {
            let g = i as f32 / f as f32;
            for c in 0..2 { self.a[i * 2 + c] *= g; self.a[(n - 1 - i) * 2 + c] *= g; }
        }
        self.len = n; self.pos = 0; self.state = LState::Play;
    }

    pub fn command(&mut self, c: Cmd, bars_idx: usize, bpm: f32, sr: f32) {
        match c {
            Cmd::RecDub => match self.state {
                LState::Empty => if !self.clearing() {
                    let bars = BARS[bars_idx.min(4)];
                    self.target = if bars > 0 { ((bars as f32) * 4.0 * 60.0 / bpm.clamp(20.0, 300.0) * sr) as usize } else { 0 };
                    self.target = if self.target == 0 { 0 } else { self.target.min(self.cap) };
                    self.pos = 0; self.len = 0; self.state = LState::Rec;
                },
                LState::Rec => self.finish_rec(),
                LState::Play => if self.clr_b == 0 { self.state = LState::Dub },
                LState::Dub => self.state = LState::Play,
                LState::Stop => if self.clr_b == 0 && self.len > 0 { self.pos = 0; self.state = LState::Dub },
            },
            Cmd::PlayStop => match self.state {
                LState::Play | LState::Dub => self.state = LState::Stop,
                LState::Stop => if self.len > 0 { self.pos = 0; self.state = LState::Play },
                LState::Rec => { self.finish_rec(); }
                LState::Empty => {}
            },
            Cmd::Undo => if matches!(self.state, LState::Play | LState::Dub | LState::Stop) {
                self.clr_b = self.len; // overdub layer is muted until the progressive clear finishes
                if self.state == LState::Dub { self.state = LState::Play; }
            },
            Cmd::Clear => {
                let used = self.len.max(if self.state == LState::Rec { self.pos } else { 0 });
                self.clr_a = used; self.clr_b = self.len;
                self.state = LState::Empty; self.len = 0; self.pos = 0; self.target = 0;
            }
        }
    }

    /// Call once per render block: advances the background clears.
    pub fn background(&mut self) {
        if self.clr_a > 0 { let n = self.clr_a.min(CLEAR_CHUNK); let lo = self.clr_a - n; self.a[lo * 2..self.clr_a * 2].fill(0.0); self.clr_a = lo; }
        if self.clr_b > 0 { let n = self.clr_b.min(CLEAR_CHUNK); let lo = self.clr_b - n; self.b[lo * 2..self.clr_b * 2].fill(0.0); self.clr_b = lo; }
    }

    /// Process one stereo frame. `l`/`r` = live signal (what gets recorded). Returns the loop playback.
    #[inline] pub fn process(&mut self, l: f32, r: f32, vol: f32) -> (f32, f32) {
        match self.state {
            LState::Empty | LState::Stop => (0.0, 0.0),
            LState::Rec => {
                let i = self.pos * 2;
                self.a[i] = l; self.a[i + 1] = r;
                self.pos += 1;
                let limit = if self.target > 0 { self.target } else { self.cap };
                if self.pos >= limit { self.finish_rec(); }
                (0.0, 0.0)
            }
            LState::Play | LState::Dub => {
                let i = self.pos * 2;
                let muted = self.clr_b > 0;
                let (bl, br) = if muted { (0.0, 0.0) } else { (self.b[i], self.b[i + 1]) };
                let out = ((self.a[i] + bl) * vol, (self.a[i + 1] + br) * vol);
                if self.state == LState::Dub && !muted {
                    self.b[i] = (self.b[i] + l).clamp(-2.0, 2.0); self.b[i + 1] = (self.b[i + 1] + r).clamp(-2.0, 2.0);
                }
                self.pos += 1; if self.pos >= self.len { self.pos = 0; }
                out
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const SR: f32 = 8000.0;

    fn run(lp: &mut Looper, frames: usize, f: impl Fn(usize) -> f32) -> Vec<f32> {
        (0..frames).map(|n| { if n % 16 == 0 { lp.background(); } lp.process(f(n), f(n), 1.0).0 }).collect()
    }

    #[test]
    fn records_exact_bars_then_loops_seamlessly() {
        let mut lp = Looper::new(SR);
        lp.command(Cmd::RecDub, 1, 120.0, SR); // 1 bar @120 = 2 s
        assert_eq!(lp.state, LState::Rec);
        run(&mut lp, 16000, |n| (n as f32 * 0.05).sin());
        assert_eq!(lp.state, LState::Play);
        assert_eq!(lp.len_frames(), 16000);
        // loop plays back the recording (after the seam fade)
        let out = run(&mut lp, 16000, |_| 0.0);
        for n in [500usize, 4000, 9000, 15000] { assert!((out[n] - (n as f32 * 0.05).sin()).abs() < 1e-5, "frame {n}"); }
        assert!(out[0].abs() < 1e-6 && out[15999].abs() < 0.2, "seam fade");
        // second lap identical
        let out2 = run(&mut lp, 16000, |_| 0.0);
        assert_eq!(out[7777], out2[7777]);
    }

    #[test]
    fn overdub_adds_and_undo_removes_without_blocking() {
        let mut lp = Looper::new(SR);
        lp.command(Cmd::RecDub, 1, 120.0, SR);
        run(&mut lp, 16000, |_| 0.25);
        lp.command(Cmd::RecDub, 1, 120.0, SR); // -> Dub
        assert_eq!(lp.state, LState::Dub);
        run(&mut lp, 16000, |_| 0.25); // one full pass of overdub
        lp.command(Cmd::RecDub, 1, 120.0, SR); // -> Play
        let out = run(&mut lp, 16000, |_| 0.0);
        assert!((out[8000] - 0.5).abs() < 1e-5, "overdub summed: {}", out[8000]);
        lp.command(Cmd::Undo, 1, 120.0, SR);
        let out = run(&mut lp, 16000, |_| 0.0);
        assert!((out[8000] - 0.25).abs() < 1e-5, "undo removed overdub: {}", out[8000]);
        // clear -> empty, silent, and a fresh recording is not polluted by old data
        lp.command(Cmd::Clear, 1, 120.0, SR);
        assert_eq!(lp.state, LState::Empty);
        run(&mut lp, 4000, |_| 0.0);
        assert!(!lp.is_clearing());
        lp.command(Cmd::RecDub, 1, 120.0, SR);
        run(&mut lp, 16000, |_| 0.1);
        lp.command(Cmd::RecDub, 1, 120.0, SR); lp.command(Cmd::RecDub, 1, 120.0, SR); // Dub then Play, silent pass
        let out = run(&mut lp, 16000, |_| 0.0);
        assert!((out[8000] - 0.1).abs() < 1e-5, "fresh take only: {}", out[8000]);
    }

    #[test]
    fn free_length_and_stop_start() {
        let mut lp = Looper::new(SR);
        lp.command(Cmd::RecDub, 0, 120.0, SR); // free
        run(&mut lp, 5000, |_| 0.3);
        lp.command(Cmd::RecDub, 0, 120.0, SR); // finish
        assert_eq!((lp.state, lp.len_frames()), (LState::Play, 5000));
        lp.command(Cmd::PlayStop, 0, 120.0, SR);
        assert_eq!(lp.state, LState::Stop);
        assert!(run(&mut lp, 100, |_| 0.0).iter().all(|&x| x == 0.0));
        lp.command(Cmd::PlayStop, 0, 120.0, SR);
        assert_eq!(lp.state, LState::Play);
        // too-short takes are discarded instead of creating a broken loop
        let mut lp = Looper::new(SR);
        lp.command(Cmd::RecDub, 0, 120.0, SR); run(&mut lp, 10, |_| 0.3); lp.command(Cmd::RecDub, 0, 120.0, SR);
        assert_eq!(lp.state, LState::Empty);
    }
}
