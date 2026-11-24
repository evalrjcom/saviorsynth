#!/usr/bin/env python3
"""Generates src/params.rs (parameter table + index constants). Run: python3 tools/gen_params.py"""
import os, sys
ENUMS = {
 "WT_NAMES": ["Basic Shapes","Analog Saw>Sq","Pulse Width","Harmonic Sweep","Formant Vowels","FM Bell","Organ","Hollow Glass","Choir","Spectral Comb","Metal FM","Noise Rough","E-Piano","Bowed Strings","Sync Sweep","Bass Growl"],
 "WARP_NAMES": ["Off","Sync","Bend +","Bend -","PWM","Mirror","Quantize","FM from other","AM from other","Ring other"],
 "SUB_SHAPES": ["Sine","Triangle","Square","Saw"],
 "NOISE_TYPES": ["White","Pink","Brown"],
 "FILTER_TYPES": ["LP 12","LP 24","HP 12","HP 24","BP","Notch"],
 "LFO_SHAPES": ["Sine","Triangle","Saw Up","Saw Down","Square","Smooth Rnd","Sample&Hold"],
 "SYNC_NAMES": ["Free","1/1","1/2","1/4","1/8","1/16","1/4T","1/8T","1/16T","1/4.","1/8.","1/16."],
 "VOICE_MODES": ["Poly","Mono","Legato"],
 "MOD_SRCS": ["None","Env 2","Env 3","LFO 1","LFO 2","LFO 3","LFO 4","Velocity","Key","Mod Wheel","Random"],
 "MOD_DSTS": ["None","A Position","B Position","A Warp","B Warp","A Pitch","B Pitch","A Level","B Level","Sub Level","Noise Level","Cutoff","Resonance","Drive","Detune","Pan","Amp Level"],
 "CHORD_MODES": ["Off","One-Key","Degree","Progression"],
 "SCALES": ["Major","Minor","Dorian","Mixolydian","Harm Minor"],
 "DEGREES": ["I","II","III","IV","V","VI","VII"],
 "CHORD_QUALS": ["Major","Minor","Dim","Aug"],
 "CHORD_EXTS": ["Triad","Sus2","Sus4","6th","7th","Dom 7","9th","Add9","Power","Octave"],
 "PROG_PATS": ["I-IV-V-I","I-V-VI-IV","II-V-I-I","I-VI-IV-V","VI-IV-I-V","I-II-IV-I","I-IV-II-V","I-III-IV-V","I-II-IV-V","I-VII-VI-V","Custom"],
 "PROG_LENS": ["2 bars","1 bar","1/2 bar","1 beat"],
 "ARP_PATS": ["Up","Down","Up-Down","Down-Up","Up+Down","Played","Random","Converge","Diverge","Thumb Up","Pinky Up","Stairs","Alberti","Chord Pulse"],
 "ARP_RATES": ["1/4","1/8","1/16","1/32","1/8T","1/16T","1/8.","1/16."],
 "INVERSIONS": ["Root","1st","2nd","3rd"],
 "ARP_OCTS": ["1 Oct","2 Oct","3 Oct","4 Oct"],
 "ARP_INVS": ["Fixed","Cycle Up","Cycle Down","Ping-Pong"],
 "DIST_TYPES": ["Soft","Hard","Fold","Tube","Bitcrush","Decimate"],
}
P = []  # (const, display, min, max, def, kind, unit)
def add(c, n, mn, mx, df, kind="Lin", unit="None"):
    P.append((c, n, mn, mx, df, kind, unit))
def en(names): return "Kind::Enum(&%s)" % names
add("MASTER","Master",0,1,0.7,"Lin","Pct")
add("VOICES","Voices",1,16,10,"Int","Int")
add("MODE","Mode",0,2,0,en("VOICE_MODES"))
add("GLIDE","Glide",0.001,2,0.001,"Exp","Sec")
add("BEND","Bend Range",0,24,2,"Int","Semi")
add("BPM","BPM",40,240,120,"Lin","Bpm")
for X,on in (("A",1),("B",0)):
    add(X+"_ON","On",0,1,on,"Bool")
    add(X+"_WT","Wavetable",0,15,0,en("WT_NAMES"))
    add(X+"_POS","WT Pos",0,1,0,"Lin","Pct")
    add(X+"_WMODE","Warp",0,9,0,en("WARP_NAMES"))
    add(X+"_WARP","Warp Amt",0,1,0,"Lin","Pct")
    add(X+"_OCT","Octave",-3,3,0,"Int","Oct")
    add(X+"_SEMI","Semi",-12,12,0,"Int","Semi")
    add(X+"_FINE","Fine",-100,100,0,"Lin","Cents")
    add(X+"_UNI","Unison",1,7,1,"Int","Int")
    add(X+"_DET","Detune",0,1,0.25,"Lin","Pct")
    add(X+"_BLEND","Blend",0,1,0.5,"Lin","Pct")
    add(X+"_PHASE","Phase",0,1,0,"Lin","Pct")
    add(X+"_RAND","Rand",0,1,0,"Lin","Pct")
    add(X+"_LEVEL","Level",0,1,0.75,"Lin","Pct")
    add(X+"_PAN","Pan",-1,1,0,"Lin","Signed")
    add(X+"_SPREAD","Spread",0,1,0.6,"Lin","Pct")
add("SUB_ON","On",0,1,0,"Bool")
add("SUB_SHAPE","Shape",0,3,0,en("SUB_SHAPES"))
add("SUB_OCT","Octave",-3,0,-1,"Int","Oct")
add("SUB_LEVEL","Level",0,1,0.5,"Lin","Pct")
add("NOISE_ON","On",0,1,0,"Bool")
add("NOISE_TYPE","Type",0,2,0,en("NOISE_TYPES"))
add("NOISE_LEVEL","Level",0,1,0.25,"Lin","Pct")
add("F_ON","On",0,1,1,"Bool")
add("F_TYPE","Type",0,5,1,en("FILTER_TYPES"))
add("F_CUT","Cutoff",20,20000,20000,"Exp","Hz")
add("F_RES","Reso",0,1,0.1,"Lin","Pct")
add("F_DRIVE","Drive",0,1,0,"Lin","Pct")
add("F_KEY","Key Trk",0,1,0,"Lin","Pct")
add("F_ENV","Env2 Amt",-1,1,0,"Lin","Signed")
add("F_MIX","Mix",0,1,1,"Lin","Pct")
add("F_RA","A",0,1,1,"Bool"); add("F_RB","B",0,1,1,"Bool")
add("F_RS","Sub",0,1,1,"Bool"); add("F_RN","Noise",0,1,1,"Bool")
envdef = {1:(0.001,0.0005,1.0,1.0,0.12), 2:(0.001,0.0005,0.4,0.0,0.3), 3:(0.001,0.0005,0.4,0.0,0.3)}
for i in (1,2,3):
    a,h,d,s,r = envdef[i]
    add("E%d_A"%i,"Attack",0.0005,10,a,"Exp","Sec")
    add("E%d_H"%i,"Hold",0.0005,5,h,"Exp","Sec")
    add("E%d_D"%i,"Decay",0.005,10,d,"Exp","Sec")
    add("E%d_S"%i,"Sustain",0,1,s,"Lin","Pct")
    add("E%d_R"%i,"Release",0.005,10,r,"Exp","Sec")
for i in (1,2,3,4):
    add("L%d_SHAPE"%i,"Shape",0,6,0,en("LFO_SHAPES"))
    add("L%d_RATE"%i,"Rate",0.02,40,2,"Exp","Hz")
    add("L%d_SYNC"%i,"Sync",0,11,0,en("SYNC_NAMES"))
    add("L%d_TRIG"%i,"Trig",0,1,1,"Bool")
    add("L%d_FADE"%i,"Fade",0,4,0,"Lin","Sec")
for i in range(1,9):
    add("M%d_SRC"%i,"Source",0,10,0,en("MOD_SRCS"))
    add("M%d_DST"%i,"Dest",0,16,0,en("MOD_DSTS"))
    add("M%d_AMT"%i,"Amount",-1,1,0,"Lin","Signed")
add("D_ON","On",0,1,0,"Bool")
add("D_TYPE","Type",0,5,0,en("DIST_TYPES"))
add("D_DRIVE","Drive",0,1,0.3,"Lin","Pct")
add("D_MIX","Mix",0,1,1,"Lin","Pct")
add("C_ON","On",0,1,0,"Bool")
add("C_RATE","Rate",0.05,8,0.6,"Exp","Hz")
add("C_DEPTH","Depth",0,1,0.5,"Lin","Pct")
add("C_DELAY","Delay",1,30,12,"Lin","Ms")
add("C_MIX","Mix",0,1,0.4,"Lin","Pct")
add("DL_ON","On",0,1,0,"Bool")
add("DL_TIME","Time",0.02,2,0.375,"Exp","Sec")
add("DL_SYNC","Sync",0,11,0,en("SYNC_NAMES"))
add("DL_FB","Feedback",0,0.95,0.4,"Lin","Pct")
add("DL_TONE","Tone",300,16000,7000,"Exp","Hz")
add("DL_MIX","Mix",0,1,0.3,"Lin","Pct")
add("DL_PP","Ping-Pong",0,1,1,"Bool")
add("R_ON","On",0,1,0,"Bool")
add("R_SIZE","Size",0,1,0.6,"Lin","Pct")
add("R_DAMP","Damp",0,1,0.5,"Lin","Pct")
add("R_MIX","Mix",0,1,0.25,"Lin","Pct")

# ---- performance: chords / progressions / arpeggiator (processed in src/perform.rs)
add("CH_MODE","Chord Mode",0,3,0,en("CHORD_MODES"))
add("CH_SCALE","Scale",0,4,0,en("SCALES"))
add("CH_DEG","Degree",0,6,0,en("DEGREES"))
add("CH_QUAL","Quality",0,3,0,en("CHORD_QUALS"))
add("CH_EXT","Variation",0,9,0,en("CHORD_EXTS"))
add("CH_INV","Inversion",0,3,0,en("INVERSIONS"))
add("CH_OPEN","Open",0,1,0,"Bool")
add("CH_BASS","Bass",0,1,0,"Bool")
add("PR_PAT","Progression",0,10,0,en("PROG_PATS"))
add("PR_LEN","Step Len",0,3,1,en("PROG_LENS"))
for i in range(1,5):
    add("PR_S%d"%i,"Step %d"%i,0,6,[0,3,4,0][i-1],en("DEGREES"))
add("AR_ON","On",0,1,0,"Bool")
add("AR_PAT","Pattern",0,13,0,en("ARP_PATS"))
add("AR_RATE","Rate",0,7,2,en("ARP_RATES"))
add("AR_OCT","Octaves",0,3,0,en("ARP_OCTS"))
add("AR_GATE","Gate",0.05,1,0.6,"Lin","Pct")
add("AR_SWING","Swing",0,0.6,0,"Lin","Pct")
add("AR_INVM","Inversions",0,3,0,en("ARP_INVS"))
add("AR_LATCH","Latch",0,1,0,"Bool")
add("AR_ACC","Accent",0,1,0.3,"Lin","Pct")

idx = {p[0]: i for i, p in enumerate(P)}
out = []
w = out.append
w("// AUTO-GENERATED by tools/gen_params.py -- do not edit by hand.\n")
w("""#![allow(dead_code)]
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Clone, Copy, PartialEq)]
pub enum Kind { Lin, Exp, Int, Enum(&'static [&'static str]), Bool }
#[derive(Clone, Copy, PartialEq)]
pub enum Unit { None, Pct, Signed, Hz, Sec, Cents, Semi, Oct, Ms, Bpm, Int }

pub struct ParamDef {
    pub key: &'static str, pub name: &'static str,
    pub min: f32, pub max: f32, pub def: f32,
    pub kind: Kind, pub unit: Unit,
}

impl ParamDef {
    pub fn is_discrete(&self) -> bool { matches!(self.kind, Kind::Int | Kind::Enum(_) | Kind::Bool) }
    /// Clamp + sanitise (NaN/inf -> default, discrete -> rounded).
    pub fn clamp(&self, v: f32) -> f32 {
        let v = if v.is_finite() { v } else { self.def };
        let v = v.clamp(self.min, self.max);
        if self.is_discrete() { v.round() } else { v }
    }
    pub fn to_norm(&self, v: f32) -> f32 {
        let n = match self.kind {
            Kind::Exp => (v.max(self.min) / self.min).ln() / (self.max / self.min).ln(),
            _ => (v - self.min) / (self.max - self.min),
        };
        if n.is_finite() { n.clamp(0.0, 1.0) } else { 0.0 }
    }
    pub fn from_norm(&self, n: f32) -> f32 {
        let n = n.clamp(0.0, 1.0);
        let v = match self.kind {
            Kind::Exp => self.min * (self.max / self.min).powf(n),
            _ => self.min + n * (self.max - self.min),
        };
        self.clamp(v)
    }
    pub fn format(&self, v: f32) -> String {
        match self.kind {
            Kind::Enum(names) => names[(v.round().max(0.0) as usize).min(names.len() - 1)].to_string(),
            Kind::Bool => if v >= 0.5 { "On".into() } else { "Off".into() },
            _ => match self.unit {
                Unit::Pct => format!("{:.0}%", v * 100.0),
                Unit::Signed => format!("{:+.0}%", v * 100.0),
                Unit::Hz => if v >= 1000.0 { format!("{:.2} kHz", v / 1000.0) } else if v >= 100.0 { format!("{:.0} Hz", v) } else { format!("{:.2} Hz", v) },
                Unit::Sec => if v < 0.1 { format!("{:.1} ms", v * 1000.0) } else if v < 1.0 { format!("{:.0} ms", v * 1000.0) } else { format!("{:.2} s", v) },
                Unit::Cents => format!("{:+.0} ct", v),
                Unit::Semi => format!("{:+.0} st", v),
                Unit::Oct => format!("{:+.0}", v),
                Unit::Ms => format!("{:.1} ms", v),
                Unit::Bpm => format!("{:.0} bpm", v),
                Unit::Int => format!("{:.0}", v),
                Unit::None => format!("{:.2}", v),
            },
        }
    }
}

""")
for k, names in ENUMS.items():
    w("pub static %s: [&str; %d] = [%s];\n" % (k, len(names), ", ".join('"%s"' % n for n in names)))
w("\n")
for i, p in enumerate(P):
    w("pub const %s: usize = %d;\n" % (p[0], i))
w("pub const PARAM_COUNT: usize = %d;\n\n" % len(P))
w("pub static PARAMS: [ParamDef; PARAM_COUNT] = [\n")
def f(x): return repr(float(x))
for p in P:
    w('    ParamDef { key: "%s", name: "%s", min: %s, max: %s, def: %s, kind: %s, unit: Unit::%s },\n' % (p[0], p[1], f(p[2]), f(p[3]), f(p[4]),
      p[5] if p[5].startswith("Kind::") else "Kind::"+p[5], p[6]))
w("];\n\n")
w("pub struct OscP { pub on: usize, pub wt: usize, pub pos: usize, pub wmode: usize, pub warp: usize, pub oct: usize, pub semi: usize, pub fine: usize, pub uni: usize, pub det: usize, pub blend: usize, pub phase: usize, pub rand: usize, pub level: usize, pub pan: usize, pub spread: usize }\n")
w("pub const OSC: [OscP; 2] = [\n")
for X in "AB":
    fs = ["on","wt","pos","wmode","warp","oct","semi","fine","uni","det","blend","phase","rand","level","pan","spread"]
    w("    OscP { %s },\n" % ", ".join("%s: %s_%s" % (a, X, a.upper()) for a in fs))
w("];\n")
w("pub struct EnvP { pub a: usize, pub h: usize, pub d: usize, pub s: usize, pub r: usize }\n")
w("pub const ENV: [EnvP; 3] = [\n")
for i in (1,2,3):
    w("    EnvP { a: E%d_A, h: E%d_H, d: E%d_D, s: E%d_S, r: E%d_R },\n" % ((i,)*5))
w("];\n")
w("pub struct LfoP { pub shape: usize, pub rate: usize, pub sync: usize, pub trig: usize, pub fade: usize }\n")
w("pub const LFO: [LfoP; 4] = [\n")
for i in (1,2,3,4):
    w("    LfoP { shape: L%d_SHAPE, rate: L%d_RATE, sync: L%d_SYNC, trig: L%d_TRIG, fade: L%d_FADE },\n" % ((i,)*5))
w("];\n")
w("pub struct ModP { pub src: usize, pub dst: usize, pub amt: usize }\n")
w("pub const MODS: [ModP; 8] = [\n")
for i in range(1,9):
    w("    ModP { src: M%d_SRC, dst: M%d_DST, amt: M%d_AMT },\n" % ((i,)*3))
w("];\n")
w("""
// Modulation destination ids (index into MOD_DSTS)
pub const MD_NONE: usize = 0; pub const MD_A_POS: usize = 1; pub const MD_B_POS: usize = 2;
pub const MD_A_WARP: usize = 3; pub const MD_B_WARP: usize = 4; pub const MD_A_PITCH: usize = 5;
pub const MD_B_PITCH: usize = 6; pub const MD_A_LEVEL: usize = 7; pub const MD_B_LEVEL: usize = 8;
pub const MD_SUB: usize = 9; pub const MD_NOISE: usize = 10; pub const MD_CUT: usize = 11;
pub const MD_RES: usize = 12; pub const MD_DRIVE: usize = 13; pub const MD_DETUNE: usize = 14;
pub const MD_PAN: usize = 15; pub const MD_AMP: usize = 16; pub const MD_COUNT: usize = 17;

/// Beats per LFO/delay cycle for each SYNC_NAMES entry (0 = free running).
pub const SYNC_BEATS: [f32; 12] = [0.0, 4.0, 2.0, 1.0, 0.5, 0.25, 2.0 / 3.0, 1.0 / 3.0, 1.0 / 6.0, 1.5, 0.75, 0.375];

/// Parameter store shared between UI / MIDI threads and the audio thread.
/// Values are f32 bit patterns in atomics: lock-free and wait-free on every platform we ship.
pub struct ParamStore { v: Vec<AtomicU32> }
impl ParamStore {
    pub fn new() -> Self { ParamStore { v: PARAMS.iter().map(|p| AtomicU32::new(p.def.to_bits())).collect() } }
    #[inline] pub fn get(&self, i: usize) -> f32 { f32::from_bits(self.v[i].load(Ordering::Relaxed)) }
    #[inline] pub fn set(&self, i: usize, x: f32) { self.v[i].store(PARAMS[i].clamp(x).to_bits(), Ordering::Relaxed) }
    pub fn snapshot(&self) -> Vec<f32> { (0..PARAM_COUNT).map(|i| self.get(i)).collect() }
    pub fn load(&self, vals: &[f32]) { for (i, v) in vals.iter().enumerate().take(PARAM_COUNT) { self.set(i, *v); } }
    pub fn reset(&self) { for i in 0..PARAM_COUNT { self.set(i, PARAMS[i].def); } }
}
pub fn default_patch() -> Vec<f32> { PARAMS.iter().map(|p| p.def).collect() }
pub fn index_of(key: &str) -> Option<usize> { PARAMS.iter().position(|p| p.key == key) }
""")
dst = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "src", "params.rs")
open(dst, "w").write("".join(out))
print("params:", len(P))
