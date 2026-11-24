//! Factory presets (classic wavetable-synth archetypes) + text preset file I/O.
//!
//! Presets are built from the default patch with a tiny DSL (`B`), addressing parameters by key
//! name and enum entries by label. Unknown names are counted (see `bad_count`) and caught by
//! `--selftest`, so a typo can never silently produce a wrong sound in release.

use crate::params::*;
mod banks;
use std::sync::atomic::{AtomicUsize, Ordering};

static BAD: AtomicUsize = AtomicUsize::new(0);
pub fn bad_count() -> usize { BAD.load(Ordering::Relaxed) }

#[derive(Clone)]
pub struct Preset { pub name: String, pub cat: String, pub vals: Vec<f32> }

struct B { v: Vec<f32>, name: String }

impl B {
    fn new(name: &str) -> B { B { v: default_patch(), name: name.to_string() } }
    fn bad(&self, what: &str) { BAD.fetch_add(1, Ordering::Relaxed); eprintln!("preset '{}': unknown {}", self.name, what); }
    fn n(&mut self, key: &str, val: f32) -> &mut B {
        match index_of(key) { Some(i) => self.v[i] = PARAMS[i].clamp(val), None => self.bad(key) }
        self
    }
    fn e(&mut self, key: &str, label: &str) -> &mut B {
        let Some(i) = index_of(key) else { self.bad(key); return self; };
        if let Kind::Enum(names) = PARAMS[i].kind {
            let l = label.to_lowercase();
            let pos = names.iter().position(|n| n.to_lowercase() == l).or_else(|| names.iter().position(|n| n.to_lowercase().starts_with(&l)));
            match pos { Some(p) => self.v[i] = p as f32, None => self.bad(&format!("{}={}", key, label)) }
        } else { self.bad(key); }
        self
    }
    fn on(&mut self, key: &str, b: bool) -> &mut B { self.n(key, if b { 1.0 } else { 0.0 }) }
    /// Oscillator shortcut: (osc 'A'|'B', wavetable, position, unison voices, detune, level)
    fn osc(&mut self, x: char, wt: &str, pos: f32, uni: f32, det: f32, level: f32) -> &mut B {
        let p = |s: &str| format!("{}_{}", x, s);
        self.on(&p("ON"), true).e(&p("WT"), wt).n(&p("POS"), pos).n(&p("UNI"), uni).n(&p("DET"), det).n(&p("LEVEL"), level)
    }
    fn saw(&mut self, x: char, uni: f32, det: f32, level: f32) -> &mut B { self.osc(x, "Basic Shapes", 0.5, uni, det, level) }
    fn off(&mut self, x: char) -> &mut B { self.on(&format!("{}_ON", x), false) }
    fn filt(&mut self, ty: &str, cut: f32, res: f32) -> &mut B { self.on("F_ON", true).e("F_TYPE", ty).n("F_CUT", cut).n("F_RES", res) }
    fn nofilt(&mut self) -> &mut B { self.on("F_ON", false) }
    fn env(&mut self, i: usize, a: f32, d: f32, s: f32, r: f32) -> &mut B {
        self.n(&format!("E{}_A", i), a).n(&format!("E{}_D", i), d).n(&format!("E{}_S", i), s).n(&format!("E{}_R", i), r)
    }
    fn lfo(&mut self, i: usize, shape: &str, rate: f32, sync: &str) -> &mut B {
        self.e(&format!("L{}_SHAPE", i), shape).n(&format!("L{}_RATE", i), rate).e(&format!("L{}_SYNC", i), sync)
    }
    fn m(&mut self, slot: usize, src: &str, dst: &str, amt: f32) -> &mut B {
        self.e(&format!("M{}_SRC", slot), src).e(&format!("M{}_DST", slot), dst).n(&format!("M{}_AMT", slot), amt)
    }
    fn dist(&mut self, ty: &str, drive: f32, mix: f32) -> &mut B { self.on("D_ON", true).e("D_TYPE", ty).n("D_DRIVE", drive).n("D_MIX", mix) }
    fn chorus(&mut self, mix: f32) -> &mut B { self.on("C_ON", true).n("C_MIX", mix) }
    fn delay(&mut self, sync: &str, fb: f32, mix: f32) -> &mut B { self.on("DL_ON", true).e("DL_SYNC", sync).n("DL_FB", fb).n("DL_MIX", mix) }
    fn reverb(&mut self, size: f32, mix: f32) -> &mut B { self.on("R_ON", true).n("R_SIZE", size).n("R_MIX", mix) }
    /// Gentle delayed vibrato on both oscillators.
    fn vibrato(&mut self, slot: usize, lfo: usize, depth: f32) -> &mut B {
        let l = format!("LFO {}", lfo);
        self.lfo(lfo, "Sine", 5.5, "Free").n(&format!("L{}_FADE", lfo), 0.6)
            .m(slot, &l, "A Pitch", depth).m(slot + 1, &l, "B Pitch", depth)
    }
    fn mono(&mut self, glide: f32, legato: bool) -> &mut B { self.e("MODE", if legato { "Legato" } else { "Mono" }).n("GLIDE", glide) }
    /// Chord stage: mode = "One-Key" | "Degree" | "Progression"; ext = variation ("Triad", "7th", "Sus4", ...).
    fn chord(&mut self, mode: &str, ext: &str) -> &mut B { self.e("CH_MODE", mode).e("CH_EXT", ext) }
    /// One-key progression: pattern label (e.g. "I-V-VI-IV") and step length ("1 bar", "1/2 bar", ...).
    fn prog(&mut self, pat: &str, len: &str) -> &mut B { self.e("CH_MODE", "Progression").e("PR_PAT", pat).e("PR_LEN", len) }
    /// Arpeggiator: pattern, rate, octave span ("1 Oct".."4 Oct"), gate length.
    fn arp(&mut self, pat: &str, rate: &str, oct: &str, gate: f32) -> &mut B { self.on("AR_ON", true).e("AR_PAT", pat).e("AR_RATE", rate).e("AR_OCT", oct).n("AR_GATE", gate) }
    /// Built-in drum machine: start running with a named pattern at `bpm` and `swing` (0..0.6).
    fn drums(&mut self, pattern: &str, bpm: f32, swing: f32) -> &mut B {
        let Some(pi) = crate::drums::PATTERNS.iter().position(|p| p.name == pattern) else { self.bad(pattern); return self; };
        let m = crate::drums::pattern_masks(pi);
        for (t, mk) in m.iter().enumerate() { self.n(&format!("DR_T{}", t + 1), *mk as f32); }
        self.on("DR_RUN", true).n("BPM", bpm).n("DR_SWING", swing)
    }
    fn done(&self, cat: &str) -> Preset { Preset { name: self.name.clone(), cat: cat.to_string(), vals: self.v.clone() } }
}

pub fn factory() -> Vec<Preset> {
    let mut out: Vec<Preset> = Vec::new();
    macro_rules! p {
        ($cat:expr, $name:expr, |$b:ident| $body:block) => {{ let mut $b = B::new($name); $body; out.push($b.done($cat)); }};
    }

    // ------------------------------------------------------------ Init
    p!("Init", "Init Sine", |_b| {});
    p!("Init", "Init Saw", |b| { b.saw('A', 1.0, 0.25, 0.75); });
    p!("Init", "Init Supersaw", |b| { b.saw('A', 7.0, 0.35, 0.75).n("A_RAND", 0.8).n("A_SPREAD", 0.9); });

    // ------------------------------------------------------------ Bass
    p!("Bass", "Reese Bass", |b| {
        b.saw('A', 3.0, 0.3, 0.7).saw('B', 3.0, 0.3, 0.7).n("B_FINE", 9.0).n("A_RAND", 0.5).n("B_RAND", 0.5)
         .filt("LP 24", 650.0, 0.2).env(1, 0.004, 1.0, 1.0, 0.15)
         .on("SUB_ON", true).n("SUB_LEVEL", 0.45).mono(0.04, true)
         .dist("Soft", 0.2, 0.6).chorus(0.35);
    });
    p!("Bass", "Wobble Bass", |b| {
        b.saw('A', 2.0, 0.2, 0.7).osc('B', "Basic Shapes", 0.75, 1.0, 0.0, 0.5)
         .filt("LP 24", 450.0, 0.55).on("SUB_ON", true).n("SUB_LEVEL", 0.4)
         .lfo(1, "Sine", 2.0, "1/8").m(1, "LFO 1", "Cutoff", 0.65)
         .env(1, 0.003, 1.0, 1.0, 0.1).mono(0.0, true).dist("Soft", 0.35, 0.8);
    });
    p!("Bass", "Sub Bass", |b| {
        b.osc('A', "Basic Shapes", 0.0, 1.0, 0.0, 0.8).on("SUB_ON", true).n("SUB_LEVEL", 0.4).e("SUB_SHAPE", "Triangle")
         .filt("LP 24", 900.0, 0.1).env(1, 0.004, 1.0, 1.0, 0.1).mono(0.0, true).dist("Soft", 0.12, 0.5);
    });
    p!("Bass", "Acid 303", |b| {
        b.saw('A', 1.0, 0.0, 0.8).filt("LP 24", 380.0, 0.82).n("F_ENV", 0.55).n("F_DRIVE", 0.25)
         .env(2, 0.001, 0.22, 0.0, 0.1).env(1, 0.002, 0.6, 0.75, 0.07).mono(0.055, true).dist("Soft", 0.25, 0.7);
    });
    p!("Bass", "FM Growl Bass", |b| {
        b.osc('A', "Bass Growl", 0.3, 2.0, 0.15, 0.8).on("SUB_ON", true).n("SUB_LEVEL", 0.35)
         .filt("LP 12", 3200.0, 0.2).lfo(1, "Sine", 3.0, "1/8T").m(1, "LFO 1", "A Position", 0.55)
         .env(1, 0.003, 1.0, 1.0, 0.1).mono(0.0, true).dist("Fold", 0.35, 0.7);
    });
    p!("Bass", "Pluck Bass", |b| {
        b.osc('A', "Analog Saw>Sq", 0.25, 2.0, 0.12, 0.8).osc('B', "Pulse Width", 0.3, 1.0, 0.0, 0.5)
         .filt("LP 24", 260.0, 0.3).n("F_ENV", 0.7).env(2, 0.001, 0.25, 0.0, 0.2).env(1, 0.002, 0.4, 0.0, 0.1)
         .on("SUB_ON", true).n("SUB_LEVEL", 0.4);
    });
    p!("Bass", "Rubber Bass", |b| {
        b.saw('A', 1.0, 0.0, 0.8).osc('B', "Basic Shapes", 0.25, 1.0, 0.0, 0.6).n("B_OCT", -1.0)
         .filt("LP 24", 220.0, 0.5).n("F_ENV", 0.75).env(2, 0.001, 0.2, 0.05, 0.15).env(1, 0.002, 0.3, 0.6, 0.12).mono(0.0, false);
    });
    p!("Bass", "Dubstep Growl", |b| {
        b.osc('A', "Metal FM", 0.3, 2.0, 0.1, 0.7).osc('B', "Bass Growl", 0.5, 1.0, 0.0, 0.6)
         .filt("LP 24", 1000.0, 0.5).lfo(1, "Saw Down", 2.0, "1/8").m(1, "LFO 1", "A Position", 0.6)
         .lfo(2, "Sine", 2.0, "1/4T").m(3, "LFO 2", "Cutoff", 0.45)
         .on("SUB_ON", true).n("SUB_LEVEL", 0.4).env(1, 0.003, 1.0, 1.0, 0.1).mono(0.0, true).dist("Hard", 0.5, 0.7);
    });
    p!("Bass", "808 Sub Boom", |b| {
        b.osc('A', "Basic Shapes", 0.0, 1.0, 0.0, 0.9).n("A_OCT", -1.0).on("SUB_ON", true).n("SUB_LEVEL", 0.5)
         .env(1, 0.001, 1.4, 0.0, 0.25).env(2, 0.001, 0.07, 0.0, 0.1).m(1, "Env 2", "A Pitch", 0.4)
         .nofilt().dist("Soft", 0.18, 0.6);
    });
    p!("Bass", "Wide Detuned Bass", |b| {
        b.saw('A', 5.0, 0.3, 0.7).n("A_OCT", -1.0).n("A_RAND", 0.7).filt("LP 24", 1100.0, 0.15)
         .on("SUB_ON", true).n("SUB_LEVEL", 0.4).env(1, 0.005, 1.0, 1.0, 0.18).chorus(0.3);
    });

    // ------------------------------------------------------------ Lead
    p!("Lead", "Supersaw Lead", |b| {
        b.saw('A', 7.0, 0.45, 0.65).n("A_SPREAD", 0.95).n("A_RAND", 0.9).saw('B', 5.0, 0.3, 0.45).n("B_RAND", 0.9)
         .filt("LP 24", 9500.0, 0.1).env(1, 0.006, 1.0, 1.0, 0.25).vibrato(1, 1, 0.012)
         .chorus(0.3).delay("1/8", 0.35, 0.22).reverb(0.5, 0.18);
    });
    p!("Lead", "Sync Lead", |b| {
        b.saw('A', 2.0, 0.12, 0.8).e("A_WMODE", "Sync").n("A_WARP", 0.35).filt("LP 24", 7500.0, 0.15)
         .env(2, 0.001, 0.6, 0.35, 0.3).m(1, "Env 2", "A Warp", 0.7).mono(0.03, true)
         .vibrato(2, 1, 0.01).delay("1/8.", 0.3, 0.2).reverb(0.4, 0.12);
    });
    p!("Lead", "Screamer", |b| {
        b.osc('A', "Sync Sweep", 0.6, 3.0, 0.2, 0.7).filt("LP 12", 6500.0, 0.3).mono(0.05, true)
         .env(1, 0.003, 1.0, 1.0, 0.2).vibrato(1, 1, 0.012).dist("Hard", 0.4, 0.7).delay("1/8", 0.3, 0.2);
    });
    p!("Lead", "Square Lead", |b| {
        b.osc('A', "Basic Shapes", 0.75, 2.0, 0.08, 0.7).osc('B', "Pulse Width", 0.5, 1.0, 0.0, 0.4).n("B_OCT", 1.0)
         .filt("LP 12", 5000.0, 0.15).env(1, 0.004, 1.0, 0.9, 0.2).vibrato(1, 1, 0.014).delay("1/8.", 0.3, 0.2).reverb(0.4, 0.12);
    });
    p!("Lead", "Hoover", |b| {
        b.osc('A', "Sync Sweep", 0.0, 7.0, 0.7, 0.6).osc('B', "Basic Shapes", 0.5, 5.0, 0.6, 0.4).n("A_RAND", 1.0).n("B_RAND", 1.0)
         .filt("LP 24", 3500.0, 0.2).env(2, 0.001, 0.5, 0.0, 0.2).m(1, "Env 2", "A Pitch", -0.12).m(2, "Env 2", "B Pitch", -0.12)
         .env(1, 0.003, 1.0, 1.0, 0.2).mono(0.04, true).dist("Soft", 0.5, 0.8).chorus(0.4);
    });
    p!("Lead", "Portamento Mono Lead", |b| {
        b.osc('A', "Analog Saw>Sq", 0.3, 2.0, 0.1, 0.7).on("SUB_ON", true).n("SUB_LEVEL", 0.25)
         .filt("LP 24", 3200.0, 0.28).n("F_ENV", 0.3).env(2, 0.002, 0.5, 0.3, 0.2).mono(0.09, true)
         .vibrato(1, 1, 0.012).delay("1/8", 0.3, 0.18);
    });
    p!("Lead", "Trance Lead", |b| {
        b.saw('A', 7.0, 0.38, 0.6).n("A_RAND", 0.9).osc('B', "Basic Shapes", 0.75, 3.0, 0.2, 0.4)
         .filt("LP 24", 6000.0, 0.2).env(1, 0.004, 1.0, 1.0, 0.3).delay("1/8.", 0.4, 0.28).reverb(0.6, 0.25);
    });
    p!("Lead", "EDM Brass Lead", |b| {
        b.osc('A', "Analog Saw>Sq", 0.05, 3.0, 0.15, 0.75).filt("LP 24", 700.0, 0.2).n("F_ENV", 0.55)
         .env(2, 0.05, 0.5, 0.5, 0.25).env(1, 0.02, 0.5, 0.8, 0.2).vibrato(1, 1, 0.01).reverb(0.4, 0.12);
    });
    p!("Lead", "Chiptune Lead", |b| {
        b.osc('A', "Pulse Width", 0.3, 1.0, 0.0, 0.7).osc('B', "Basic Shapes", 0.75, 1.0, 0.0, 0.3).n("B_OCT", 1.0)
         .nofilt().env(1, 0.001, 0.4, 0.8, 0.05).vibrato(1, 1, 0.01);
    });
    p!("Lead", "Vocal Lead", |b| {
        b.osc('A', "Formant Vowels", 0.0, 2.0, 0.1, 0.8).filt("LP 24", 9000.0, 0.05).env(1, 0.03, 1.0, 1.0, 0.3)
         .lfo(2, "Triangle", 0.35, "Free").m(3, "LFO 2", "A Position", 0.4).vibrato(1, 1, 0.01).reverb(0.5, 0.2);
    });

    // ------------------------------------------------------------ Pad
    p!("Pad", "Warm Pad", |b| {
        b.saw('A', 5.0, 0.25, 0.55).n("A_RAND", 1.0).osc('B', "Basic Shapes", 0.25, 3.0, 0.2, 0.45).n("B_RAND", 1.0)
         .filt("LP 24", 1700.0, 0.1).env(1, 1.2, 1.0, 1.0, 1.8).lfo(1, "Sine", 0.15, "Free").m(1, "LFO 1", "Cutoff", 0.15)
         .chorus(0.4).reverb(0.65, 0.3);
    });
    p!("Pad", "Strings Ensemble", |b| {
        b.osc('A', "Bowed Strings", 0.3, 5.0, 0.28, 0.55).n("A_RAND", 1.0).saw('B', 3.0, 0.2, 0.35).n("B_RAND", 1.0)
         .filt("LP 24", 3500.0, 0.05).env(1, 0.45, 1.0, 1.0, 0.9).vibrato(1, 1, 0.006).chorus(0.5).reverb(0.6, 0.25);
    });
    p!("Pad", "Glass Pad", |b| {
        b.osc('A', "Hollow Glass", 0.3, 3.0, 0.2, 0.55).osc('B', "FM Bell", 0.2, 1.0, 0.0, 0.3).n("B_OCT", 1.0)
         .filt("LP 24", 8000.0, 0.1).env(1, 0.8, 1.0, 1.0, 2.0).lfo(1, "Triangle", 0.2, "Free").m(1, "LFO 1", "A Position", 0.5)
         .chorus(0.35).delay("1/4", 0.45, 0.25).reverb(0.8, 0.4);
    });
    p!("Pad", "Choir Aah", |b| {
        b.osc('A', "Choir", 0.0, 3.0, 0.15, 0.8).n("A_RAND", 1.0).filt("LP 24", 7000.0, 0.05).env(1, 0.6, 1.0, 1.0, 1.2)
         .lfo(1, "Sine", 0.28, "Free").m(1, "LFO 1", "A Position", 0.3).chorus(0.3).reverb(0.75, 0.35);
    });
    p!("Pad", "Dark Pad", |b| {
        b.saw('A', 3.0, 0.2, 0.55).n("A_OCT", -1.0).osc('B', "Spectral Comb", 0.4, 1.0, 0.0, 0.4)
         .filt("LP 24", 700.0, 0.25).env(1, 1.5, 1.0, 1.0, 2.2).lfo(1, "Sine", 0.12, "Free").m(1, "LFO 1", "Cutoff", 0.3)
         .m(2, "LFO 1", "B Position", 0.5).chorus(0.3).reverb(0.8, 0.4);
    });
    p!("Pad", "Evolving Wavetable Pad", |b| {
        b.osc('A', "Harmonic Sweep", 0.2, 3.0, 0.2, 0.5).osc('B', "Hollow Glass", 0.5, 3.0, 0.2, 0.4)
         .filt("LP 24", 5000.0, 0.15).env(1, 1.5, 1.0, 1.0, 2.5)
         .lfo(1, "Sine", 0.08, "Free").m(1, "LFO 1", "A Position", 0.8).lfo(2, "Sine", 0.13, "Free").m(2, "LFO 2", "B Position", 0.8)
         .chorus(0.3).reverb(0.8, 0.4);
    });
    p!("Pad", "Ambient Swell", |b| {
        b.osc('A', "Basic Shapes", 0.25, 4.0, 0.25, 0.6).filt("LP 24", 1800.0, 0.1).env(1, 3.0, 1.0, 1.0, 4.0)
         .lfo(1, "Sine", 0.1, "Free").m(1, "LFO 1", "Cutoff", 0.2).delay("1/4", 0.5, 0.3).reverb(0.95, 0.5);
    });
    p!("Pad", "Analog Poly Pad", |b| {
        b.saw('A', 2.0, 0.15, 0.55).osc('B', "Basic Shapes", 0.75, 2.0, 0.15, 0.4).n("B_FINE", 6.0)
         .filt("LP 24", 2200.0, 0.15).env(1, 0.15, 1.0, 0.9, 0.55).chorus(0.5).reverb(0.5, 0.2);
    });
    p!("Pad", "Sweep Pad", |b| {
        b.saw('A', 5.0, 0.3, 0.6).n("A_RAND", 1.0).filt("LP 24", 450.0, 0.3).n("F_ENV", 0.65)
         .env(2, 1.6, 2.0, 0.55, 1.5).env(1, 0.9, 1.0, 1.0, 1.4).chorus(0.4).reverb(0.7, 0.3);
    });

    // ------------------------------------------------------------ Pluck
    p!("Pluck", "EDM Pluck", |b| {
        b.saw('A', 3.0, 0.25, 0.65).osc('B', "Basic Shapes", 0.75, 1.0, 0.0, 0.35).filt("LP 24", 500.0, 0.2).n("F_ENV", 0.6)
         .env(2, 0.001, 0.22, 0.0, 0.2).env(1, 0.002, 0.38, 0.0, 0.15).delay("1/8", 0.3, 0.25).reverb(0.4, 0.15);
    });
    p!("Pluck", "Future Bass Pluck", |b| {
        b.saw('A', 5.0, 0.35, 0.6).n("A_RAND", 0.8).filt("LP 12", 2200.0, 0.15).n("F_ENV", 0.35)
         .env(2, 0.001, 0.3, 0.0, 0.2).env(1, 0.003, 0.55, 0.0, 0.35).chorus(0.35).delay("1/8.", 0.35, 0.2).reverb(0.55, 0.28);
    });
    p!("Pluck", "Pizzicato", |b| {
        b.osc('A', "Bowed Strings", 0.1, 1.0, 0.0, 0.8).filt("LP 24", 1400.0, 0.15).n("F_ENV", 0.4)
         .env(2, 0.001, 0.12, 0.0, 0.1).env(1, 0.001, 0.2, 0.0, 0.12).reverb(0.4, 0.15);
    });
    p!("Pluck", "Kalimba", |b| {
        b.osc('A', "FM Bell", 0.12, 1.0, 0.0, 0.7).n("A_OCT", 1.0).osc('B', "Basic Shapes", 0.0, 1.0, 0.0, 0.2).n("B_OCT", 2.0)
         .nofilt().env(1, 0.001, 0.55, 0.0, 0.35).reverb(0.5, 0.2);
    });
    p!("Pluck", "Marimba", |b| {
        b.osc('A', "FM Bell", 0.05, 1.0, 0.0, 0.8).osc('B', "Basic Shapes", 0.0, 1.0, 0.0, 0.18).n("B_OCT", 2.0)
         .filt("LP 12", 5000.0, 0.1).env(1, 0.001, 0.32, 0.0, 0.2).reverb(0.35, 0.15);
    });
    p!("Pluck", "Harp Pluck", |b| {
        b.osc('A', "Basic Shapes", 0.25, 2.0, 0.05, 0.75).filt("LP 12", 3800.0, 0.1).n("F_ENV", 0.3)
         .env(2, 0.001, 0.5, 0.0, 0.3).env(1, 0.001, 1.3, 0.0, 0.5).chorus(0.2).reverb(0.6, 0.3);
    });
    p!("Pluck", "Trance Pluck", |b| {
        b.saw('A', 5.0, 0.3, 0.55).osc('B', "Basic Shapes", 0.75, 2.0, 0.15, 0.35).filt("LP 24", 800.0, 0.2).n("F_ENV", 0.7)
         .env(2, 0.001, 0.25, 0.0, 0.2).env(1, 0.002, 0.35, 0.0, 0.2).delay("1/8.", 0.4, 0.3).reverb(0.5, 0.2);
    });
    p!("Pluck", "Arp Pluck", |b| {
        b.osc('A', "Analog Saw>Sq", 0.4, 2.0, 0.1, 0.7).filt("LP 24", 1800.0, 0.3).n("F_ENV", 0.4)
         .env(2, 0.001, 0.2, 0.0, 0.1).env(1, 0.001, 0.22, 0.0, 0.1).delay("1/8", 0.35, 0.3).reverb(0.4, 0.15);
    });

    // ------------------------------------------------------------ Keys
    p!("Keys", "Electric Piano", |b| {
        b.osc('A', "E-Piano", 0.35, 1.0, 0.0, 0.75).osc('B', "Basic Shapes", 0.0, 1.0, 0.0, 0.18).n("B_OCT", 1.0)
         .nofilt().env(1, 0.002, 1.6, 0.0, 0.45).m(1, "Velocity", "A Position", 0.4).chorus(0.3).reverb(0.4, 0.15);
    });
    p!("Keys", "Drawbar Organ", |b| {
        b.osc('A', "Organ", 0.3, 1.0, 0.0, 0.7).osc('B', "Organ", 0.7, 1.0, 0.0, 0.35).n("B_OCT", 1.0)
         .nofilt().env(1, 0.004, 0.1, 1.0, 0.06).dist("Tube", 0.15, 0.6).chorus(0.5).n("C_RATE", 0.9).reverb(0.3, 0.12);
    });
    p!("Keys", "Clav", |b| {
        b.osc('A', "Pulse Width", 0.5, 1.0, 0.0, 0.8).filt("LP 24", 1100.0, 0.2).n("F_ENV", 0.5)
         .env(2, 0.001, 0.2, 0.0, 0.1).env(1, 0.001, 0.4, 0.15, 0.08);
    });
    p!("Keys", "Tubular Bell", |b| {
        b.osc('A', "FM Bell", 0.5, 1.0, 0.0, 0.7).osc('B', "Metal FM", 0.3, 1.0, 0.0, 0.3).n("B_OCT", 1.0)
         .nofilt().env(1, 0.001, 3.0, 0.0, 2.0).reverb(0.7, 0.3);
    });
    p!("Keys", "Vibraphone", |b| {
        b.osc('A', "FM Bell", 0.1, 1.0, 0.0, 0.8).nofilt().env(1, 0.002, 2.2, 0.0, 1.0)
         .lfo(1, "Sine", 5.0, "Free").m(1, "LFO 1", "Amp Level", 0.22).reverb(0.5, 0.2);
    });
    p!("Keys", "Glass Keys", |b| {
        b.osc('A', "Hollow Glass", 0.3, 1.0, 0.0, 0.6).osc('B', "FM Bell", 0.2, 1.0, 0.0, 0.3).n("B_OCT", 1.0)
         .nofilt().env(1, 0.002, 1.6, 0.0, 1.0).reverb(0.7, 0.35);
    });
    p!("Keys", "Soft Keys", |b| {
        b.osc('A', "Basic Shapes", 0.25, 1.0, 0.0, 0.8).filt("LP 12", 3000.0, 0.05).env(1, 0.004, 1.0, 0.3, 0.35)
         .m(1, "Velocity", "Cutoff", 0.3).chorus(0.25).reverb(0.4, 0.15);
    });
    p!("Keys", "Harpsichord", |b| {
        b.osc('A', "Pulse Width", 0.7, 2.0, 0.08, 0.6).saw('B', 1.0, 0.0, 0.3).n("B_OCT", 1.0)
         .filt("LP 12", 7000.0, 0.1).env(1, 0.001, 0.8, 0.0, 0.08).reverb(0.35, 0.12);
    });

    // ------------------------------------------------------------ Chord / Stab
    p!("Stab", "Trance Stab", |b| {
        b.saw('A', 7.0, 0.4, 0.6).n("A_RAND", 0.8).osc('B', "Basic Shapes", 0.75, 3.0, 0.2, 0.35)
         .filt("LP 24", 1500.0, 0.2).n("F_ENV", 0.5).env(2, 0.001, 0.25, 0.0, 0.2).env(1, 0.002, 0.3, 0.2, 0.15)
         .delay("1/8.", 0.4, 0.3).reverb(0.5, 0.25);
    });
    p!("Stab", "Detuned Chord Stab", |b| {
        b.saw('A', 6.0, 0.5, 0.65).n("A_RAND", 1.0).filt("LP 24", 900.0, 0.15).n("F_ENV", 0.6)
         .env(2, 0.001, 0.3, 0.0, 0.2).env(1, 0.002, 0.45, 0.1, 0.2).chorus(0.3).reverb(0.4, 0.2);
    });
    p!("Stab", "Brass Stab", |b| {
        b.osc('A', "Analog Saw>Sq", 0.1, 3.0, 0.15, 0.7).filt("LP 24", 600.0, 0.15).n("F_ENV", 0.6)
         .env(2, 0.03, 0.4, 0.5, 0.2).env(1, 0.015, 0.4, 0.7, 0.15).reverb(0.35, 0.12);
    });
    p!("Stab", "Future Chord", |b| {
        b.saw('A', 7.0, 0.5, 0.6).n("A_RAND", 1.0).n("A_SPREAD", 1.0).filt("LP 12", 4500.0, 0.1)
         .env(1, 0.004, 1.0, 0.6, 0.35).chorus(0.35).delay("1/8", 0.3, 0.2).reverb(0.6, 0.3);
    });
    p!("Stab", "Synthwave Poly", |b| {
        b.saw('A', 3.0, 0.2, 0.55).osc('B', "Basic Shapes", 0.75, 2.0, 0.15, 0.4).filt("LP 24", 2600.0, 0.15)
         .env(1, 0.02, 1.0, 1.0, 0.45).chorus(0.6).delay("1/8", 0.3, 0.2).reverb(0.55, 0.3);
    });

    // ------------------------------------------------------------ Brass
    p!("Brass", "Brass Section", |b| {
        b.osc('A', "Analog Saw>Sq", 0.1, 4.0, 0.15, 0.7).filt("LP 24", 520.0, 0.1).n("F_ENV", 0.6)
         .env(2, 0.09, 0.5, 0.55, 0.3).env(1, 0.05, 1.0, 1.0, 0.25).vibrato(1, 1, 0.01).n("L1_FADE", 1.2).reverb(0.4, 0.15);
    });
    p!("Brass", "Synth Brass", |b| {
        b.saw('A', 3.0, 0.2, 0.6).saw('B', 2.0, 0.15, 0.4).n("B_FINE", 8.0).filt("LP 24", 450.0, 0.2).n("F_ENV", 0.65)
         .env(2, 0.06, 0.6, 0.5, 0.3).env(1, 0.03, 1.0, 0.9, 0.2).chorus(0.3).reverb(0.4, 0.15);
    });

    // ------------------------------------------------------------ Sequence / Gate
    p!("Gate", "Trance Gate Pad", |b| {
        b.saw('A', 5.0, 0.3, 0.6).n("A_RAND", 1.0).filt("LP 24", 3000.0, 0.15).env(1, 0.01, 1.0, 1.0, 0.3)
         .lfo(1, "Square", 2.0, "1/16").n("L1_TRIG", 0.0).m(1, "LFO 1", "Amp Level", -0.9).delay("1/8.", 0.4, 0.25).reverb(0.5, 0.2);
    });
    p!("Gate", "Pulse Sequencer", |b| {
        b.osc('A', "Pulse Width", 0.5, 2.0, 0.1, 0.7).filt("LP 24", 700.0, 0.4).env(1, 0.002, 1.0, 1.0, 0.1)
         .lfo(1, "Saw Down", 2.0, "1/8").n("L1_TRIG", 0.0).m(1, "LFO 1", "Cutoff", 0.6).delay("1/8", 0.4, 0.2);
    });

    // ------------------------------------------------------------ FX
    p!("FX", "Noise Riser", |b| {
        b.osc('A', "Noise Rough", 0.2, 5.0, 0.6, 0.4).on("NOISE_ON", true).n("NOISE_LEVEL", 0.6).e("NOISE_TYPE", "White")
         .filt("LP 24", 200.0, 0.4).n("F_ENV", 1.0).env(2, 7.0, 0.1, 1.0, 1.0).env(1, 3.0, 1.0, 1.0, 0.6)
         .m(1, "Env 2", "A Pitch", 0.15).reverb(0.7, 0.3);
    });
    p!("FX", "Downlifter", |b| {
        b.saw('A', 5.0, 0.5, 0.45).on("NOISE_ON", true).n("NOISE_LEVEL", 0.5).e("NOISE_TYPE", "White")
         .filt("LP 24", 150.0, 0.35).n("F_ENV", 1.0).env(2, 0.001, 4.5, 0.0, 1.0).env(1, 0.002, 5.0, 0.0, 1.0)
         .m(1, "Env 2", "A Pitch", 0.2).reverb(0.7, 0.3);
    });
    p!("FX", "Laser Zap", |b| {
        b.saw('A', 1.0, 0.0, 0.8).nofilt().env(2, 0.001, 0.22, 0.0, 0.1).env(1, 0.001, 0.3, 0.0, 0.1)
         .m(1, "Env 2", "A Pitch", 1.0).m(2, "Env 2", "A Pitch", 1.0).m(3, "Env 2", "A Pitch", 1.0).delay("1/8", 0.4, 0.2);
    });
    p!("FX", "Alien Whale", |b| {
        b.osc('A', "Formant Vowels", 0.0, 3.0, 0.15, 0.8).filt("LP 24", 6000.0, 0.1).env(1, 0.8, 1.0, 1.0, 1.5)
         .lfo(1, "Sine", 0.3, "Free").m(1, "LFO 1", "A Position", 0.8).lfo(2, "Sine", 0.21, "Free").m(2, "LFO 2", "A Pitch", 0.3)
         .delay("1/4", 0.5, 0.3).reverb(0.85, 0.45);
    });
    p!("FX", "Wind", |b| {
        b.off('A').on("NOISE_ON", true).n("NOISE_LEVEL", 1.0).e("NOISE_TYPE", "White").filt("LP 24", 900.0, 0.45)
         .env(1, 2.0, 1.0, 1.0, 2.5).lfo(1, "Sine", 0.17, "Free").m(1, "LFO 1", "Cutoff", 0.55)
         .lfo(2, "Smooth Rnd", 0.6, "Free").m(2, "LFO 2", "Cutoff", 0.25).reverb(0.8, 0.4);
    });
    p!("FX", "Impact Boom", |b| {
        b.osc('A', "Basic Shapes", 0.0, 1.0, 0.0, 0.9).n("A_OCT", -2.0).on("NOISE_ON", true).n("NOISE_LEVEL", 0.45).e("NOISE_TYPE", "Brown")
         .filt("LP 24", 300.0, 0.2).n("F_ENV", 0.6).env(2, 0.001, 0.8, 0.0, 0.5).env(1, 0.001, 2.5, 0.0, 1.0)
         .m(1, "Env 2", "A Pitch", 0.35).dist("Soft", 0.2, 0.5).reverb(0.8, 0.4);
    });
    p!("FX", "Sci-Fi Sweep", |b| {
        b.osc('A', "Metal FM", 0.2, 3.0, 0.4, 0.6).filt("BP", 500.0, 0.6).env(1, 0.5, 1.0, 1.0, 1.5).env(2, 2.5, 1.0, 1.0, 1.0)
         .n("F_ENV", 0.9).lfo(1, "Smooth Rnd", 3.0, "Free").m(1, "LFO 1", "A Position", 0.5).delay("1/8T", 0.5, 0.3).reverb(0.7, 0.35);
    });

    // ------------------------------------------------------------ Wavetable showcase (one per table, slow scan)
    for (i, name) in WT_NAMES.iter().enumerate() {
        let nm = format!("WT {}", name);
        let mut b = B::new(&nm);
        b.osc('A', name, 0.0, 3.0, 0.15, 0.65).n("A_RAND", 0.6).filt("LP 24", 9000.0, 0.1).env(1, 0.01, 1.0, 1.0, 0.35)
         .lfo(1, "Triangle", 0.25, "Free").m(1, "LFO 1", "A Position", 0.5).n("A_POS", 0.5).chorus(0.25).reverb(0.4, 0.15);
        let _ = i;
        out.push(b.done("Wavetables"));
    }

    // Style banks (Sylenth1-/Diva-/Serum-style archetypes; original patches)
    banks::add(&mut out);
    banks::vintage::add(&mut out);
    banks::hits::add(&mut out);
    banks::playable::add(&mut out);
    banks::beats::add(&mut out);
    out
}

// -------------------------------------------------------------------- preset files

pub fn sanitize_name(s: &str) -> String {
    let t: String = s.chars().filter(|c| !c.is_control()).take(48).collect();
    let t = t.trim().to_string();
    if t.is_empty() { "Untitled".to_string() } else { t }
}

pub fn to_text(p: &Preset, app: &str) -> String {
    let mut s = format!("# {} preset\nname={}\ncategory={}\n", app, sanitize_name(&p.name), sanitize_name(&p.cat));
    for (i, def) in PARAMS.iter().enumerate() { s.push_str(&format!("{}={}\n", def.key, p.vals[i])); }
    s
}

/// Defensive parser: size/line limits, unknown keys ignored, every value clamped.
pub fn from_text(text: &str, fallback_name: &str) -> Result<Preset, String> {
    if text.len() > 200_000 { return Err("preset file too large".into()); }
    let mut vals = default_patch();
    let mut name = fallback_name.to_string();
    let mut cat = "User".to_string();
    let mut seen = 0usize;
    for (ln, line) in text.lines().enumerate() {
        if ln > 2000 { return Err("too many lines".into()); }
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        let Some((k, v)) = line.split_once('=') else { continue };
        let (k, v) = (k.trim(), v.trim());
        match k {
            "name" => name = sanitize_name(v),
            "category" => cat = sanitize_name(v),
            _ => if let Some(i) = index_of(k) {
                if let Ok(x) = v.parse::<f32>() { if x.is_finite() { vals[i] = PARAMS[i].clamp(x); seen += 1; } }
            }
        }
    }
    if seen == 0 { return Err("no recognised parameters found".into()); }
    Ok(Preset { name, cat, vals })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_presets_are_valid() {
        let f = factory();
        assert!(f.len() >= 295);
        assert_eq!(bad_count(), 0, "unknown key/label used in a factory preset");
        for p in &f {
            assert_eq!(p.vals.len(), PARAM_COUNT);
            for (i, v) in p.vals.iter().enumerate() { assert!(v.is_finite() && *v >= PARAMS[i].min && *v <= PARAMS[i].max, "{} {}", p.name, PARAMS[i].key); }
        }
    }

    #[test]
    fn file_roundtrip() {
        let p = factory().remove(5);
        let t = to_text(&p, "test");
        let q = from_text(&t, "x").unwrap();
        assert_eq!(q.name, p.name);
        for i in 0..PARAM_COUNT { assert!((q.vals[i] - p.vals[i]).abs() < 1e-4, "{}", PARAMS[i].key); }
    }

    #[test]
    fn hostile_files_are_contained() {
        assert!(from_text("", "x").is_err());
        assert!(from_text("garbage\nno equals here", "x").is_err());
        assert!(from_text(&"a=1\n".repeat(100_000), "x").is_err());
        let q = from_text("name=\u{7}\u{0}evil\nF_CUT=NaN\nA_UNI=999999\nA_POS=-5\nM1_DST=1e30\n", "x").unwrap();
        assert!(!q.name.chars().any(|c| c.is_control()));
        assert_eq!(q.vals[A_UNI], 7.0);
        assert_eq!(q.vals[A_POS], 0.0);
        assert_eq!(q.vals[F_CUT], PARAMS[F_CUT].def);
        assert!(q.vals[M1_DST] <= 16.0);
    }
}
