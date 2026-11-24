//! 70s and 80s hardware-synth archetypes (Minimoog, ARP, Prophet, CS-80, Juno, Jupiter, DX7, PPG, D-50 ...).
//! Original approximations of each instrument's *character* built on this engine; no samples or
//! vendor presets are used, and the engine is not a model of any specific circuit.

use super::super::{Preset, B};

pub fn add(out: &mut Vec<Preset>) {
    macro_rules! p {
        ($cat:expr, $name:expr, |$b:ident| $body:block) => {{ let mut $b = B::new($name); $body; out.push($b.done($cat)); }};
    }

    // ===================================================================== 70s
    const S: &str = "70s Synths";
    p!(S, "Minimoog Model D Lead", |b| {
        b.saw('A', 1.0, 0.0, 0.7).osc('B', "Pulse Width", 0.5, 1.0, 0.0, 0.5).n("B_FINE", 5.0).osc('A', "Analog Saw>Sq", 0.0, 2.0, 0.06, 0.7)
         .filt("LP 24", 2400.0, 0.3).n("F_DRIVE", 0.3).n("F_ENV", 0.35).env(2, 0.004, 0.45, 0.5, 0.2).env(1, 0.004, 1.0, 1.0, 0.2)
         .mono(0.07, true).vibrato(2, 1, 0.01).dist("Tube", 0.2, 0.5).delay("1/8", 0.3, 0.15);
    });
    p!(S, "Minimoog Fat Bass", |b| {
        b.saw('A', 1.0, 0.0, 0.75).osc('B', "Pulse Width", 0.5, 1.0, 0.0, 0.55).n("B_OCT", -1.0).n("B_FINE", 4.0)
         .filt("LP 24", 300.0, 0.35).n("F_DRIVE", 0.35).n("F_ENV", 0.6).env(2, 0.002, 0.25, 0.25, 0.15).env(1, 0.002, 0.5, 0.75, 0.1)
         .mono(0.02, true).dist("Tube", 0.25, 0.5);
    });
    p!(S, "Minimoog Unison Brass", |b| {
        b.saw('A', 3.0, 0.1, 0.7).saw('B', 3.0, 0.1, 0.55).n("B_FINE", 8.0).filt("LP 24", 800.0, 0.2).n("F_ENV", 0.5)
         .env(2, 0.08, 0.5, 0.55, 0.3).env(1, 0.05, 1.0, 1.0, 0.25).mono(0.05, true).dist("Tube", 0.2, 0.5).reverb(0.4, 0.15);
    });
    p!(S, "ARP Odyssey Sync Lead", |b| {
        b.saw('A', 1.0, 0.0, 0.7).e("A_WMODE", "Sync").n("A_WARP", 0.45).osc('B', "Pulse Width", 0.5, 1.0, 0.0, 0.3)
         .filt("HP 12", 250.0, 0.25).env(2, 0.001, 0.9, 0.3, 0.3).m(1, "Env 2", "A Warp", 0.6).env(1, 0.004, 1.0, 1.0, 0.2)
         .mono(0.05, true).vibrato(2, 1, 0.012).delay("1/8", 0.3, 0.15);
    });
    p!(S, "ARP Odyssey Ring Mod", |b| {
        b.osc('A', "Basic Shapes", 0.0, 1.0, 0.0, 0.6).osc('B', "Basic Shapes", 0.5, 1.0, 0.0, 0.5).e("A_WMODE", "Ring other").n("A_WARP", 0.7).n("B_SEMI", 7.0).n("B_FINE", 13.0)
         .filt("LP 24", 3500.0, 0.35).env(1, 0.005, 0.8, 0.4, 0.3).mono(0.0, false).delay("1/8.", 0.35, 0.2).reverb(0.4, 0.15);
    });
    p!(S, "ARP 2600 Drone Bass", |b| {
        b.saw('A', 1.0, 0.0, 0.7).n("A_OCT", -1.0).osc('B', "Pulse Width", 0.35, 1.0, 0.0, 0.5).n("B_OCT", -1.0).n("B_FINE", 3.0)
         .filt("LP 24", 450.0, 0.55).lfo(1, "Smooth Rnd", 0.8, "Free").m(1, "LFO 1", "Cutoff", 0.25).env(1, 0.01, 1.0, 1.0, 0.3).mono(0.03, true).dist("Soft", 0.25, 0.5);
    });
    p!(S, "ARP Pro Soloist Brass", |b| {
        b.osc('A', "Pulse Width", 0.5, 1.0, 0.0, 0.6).saw('B', 1.0, 0.0, 0.5).filt("LP 12", 1200.0, 0.2).n("F_ENV", 0.4)
         .env(2, 0.07, 0.4, 0.6, 0.2).env(1, 0.04, 1.0, 1.0, 0.2).mono(0.02, true).vibrato(2, 1, 0.01);
    });
    p!(S, "ARP String Ensemble", |b| {
        b.saw('A', 5.0, 0.2, 0.55).n("A_RAND", 1.0).saw('B', 5.0, 0.2, 0.4).n("B_OCT", 1.0).n("B_RAND", 1.0)
         .filt("LP 12", 3800.0, 0.05).env(1, 0.4, 1.0, 1.0, 0.6).chorus(0.8).reverb(0.5, 0.2);
    });
    p!(S, "Moog Modular Sweep", |b| {
        b.saw('A', 3.0, 0.15, 0.6).osc('B', "Pulse Width", 0.5, 1.0, 0.0, 0.4).filt("LP 24", 250.0, 0.55).n("F_ENV", 0.85)
         .env(2, 1.8, 1.5, 0.3, 1.0).env(1, 0.3, 1.0, 1.0, 1.0).reverb(0.6, 0.25);
    });
    p!(S, "Prophet-5 Poly Brass", |b| {
        b.saw('A', 3.0, 0.14, 0.6).osc('B', "Pulse Width", 0.5, 1.0, 0.0, 0.45).n("B_FINE", -6.0)
         .filt("LP 24", 700.0, 0.2).n("F_ENV", 0.55).env(2, 0.1, 0.6, 0.5, 0.3).env(1, 0.06, 1.0, 1.0, 0.25).chorus(0.3).reverb(0.4, 0.15);
    });
    p!(S, "Prophet-5 Warm Pad", |b| {
        b.saw('A', 3.0, 0.2, 0.5).n("A_RAND", 1.0).osc('B', "Pulse Width", 0.5, 3.0, 0.15, 0.4).n("B_RAND", 1.0)
         .filt("LP 24", 1400.0, 0.1).env(1, 1.0, 1.0, 1.0, 1.5).lfo(1, "Triangle", 0.3, "Free").m(1, "LFO 1", "B Position", 0.3).chorus(0.5).reverb(0.65, 0.3);
    });
    p!(S, "Oberheim SEM Pad", |b| {
        b.saw('A', 3.0, 0.22, 0.55).saw('B', 3.0, 0.22, 0.4).n("B_FINE", 9.0).filt("BP", 1500.0, 0.3)
         .env(1, 0.8, 1.0, 1.0, 1.2).lfo(1, "Sine", 0.2, "Free").m(1, "LFO 1", "Cutoff", 0.3).chorus(0.5).reverb(0.65, 0.3);
    });
    p!(S, "CS-80 Brass", |b| {
        b.saw('A', 5.0, 0.2, 0.55).saw('B', 3.0, 0.14, 0.45).n("B_OCT", -1.0).filt("LP 24", 520.0, 0.2).n("F_ENV", 0.7)
         .env(2, 0.3, 1.2, 0.7, 0.5).env(1, 0.2, 1.0, 1.0, 0.5).vibrato(1, 1, 0.01).chorus(0.35).delay("1/4", 0.4, 0.2).reverb(0.7, 0.3);
    });
    p!(S, "CS-80 Poly Pad", |b| {
        b.osc('A', "Analog Saw>Sq", 0.3, 5.0, 0.2, 0.5).n("A_RAND", 1.0).saw('B', 3.0, 0.15, 0.4).n("B_RAND", 1.0)
         .filt("LP 24", 1100.0, 0.15).n("F_ENV", 0.3).env(2, 1.0, 1.0, 0.6, 1.2).env(1, 0.9, 1.0, 1.0, 1.5).chorus(0.45).reverb(0.7, 0.3);
    });
    p!(S, "Polymoog Vox Humana", |b| {
        b.osc('A', "Formant Vowels", 0.2, 3.0, 0.1, 0.65).osc('B', "Choir", 0.5, 1.0, 0.0, 0.35)
         .filt("LP 12", 5000.0, 0.1).env(1, 0.25, 1.0, 1.0, 0.7).lfo(1, "Sine", 5.0, "Free").n("L1_FADE", 0.8).m(1, "LFO 1", "A Pitch", 0.006).chorus(0.5).reverb(0.6, 0.25);
    });
    p!(S, "Mellotron Strings", |b| {
        b.osc('A', "Bowed Strings", 0.5, 3.0, 0.22, 0.6).n("A_RAND", 1.0).on("NOISE_ON", true).n("NOISE_LEVEL", 0.03).e("NOISE_TYPE", "Pink")
         .filt("LP 12", 2800.0, 0.05).env(1, 0.12, 1.0, 1.0, 0.5).lfo(1, "Smooth Rnd", 5.5, "Free").m(1, "LFO 1", "A Pitch", 0.01).m(2, "LFO 1", "Cutoff", 0.1).dist("Bitcrush", 0.08, 0.2).reverb(0.5, 0.2);
    });
    p!(S, "Mellotron Flute", |b| {
        b.osc('A', "Hollow Glass", 0.1, 1.0, 0.0, 0.6).on("NOISE_ON", true).n("NOISE_LEVEL", 0.04).e("NOISE_TYPE", "Pink")
         .filt("LP 12", 3500.0, 0.05).env(1, 0.06, 1.0, 1.0, 0.35).lfo(1, "Smooth Rnd", 5.0, "Free").m(1, "LFO 1", "A Pitch", 0.012).dist("Bitcrush", 0.08, 0.2).reverb(0.5, 0.25);
    });
    p!(S, "Mellotron Choir", |b| {
        b.osc('A', "Choir", 0.3, 3.0, 0.15, 0.65).on("NOISE_ON", true).n("NOISE_LEVEL", 0.03).e("NOISE_TYPE", "Pink")
         .filt("LP 12", 3200.0, 0.05).env(1, 0.15, 1.0, 1.0, 0.6).lfo(1, "Smooth Rnd", 4.5, "Free").m(1, "LFO 1", "A Pitch", 0.01).dist("Bitcrush", 0.08, 0.2).reverb(0.6, 0.3);
    });
    p!(S, "Korg MS-20 Screamer", |b| {
        b.saw('A', 1.0, 0.0, 0.7).osc('B', "Pulse Width", 0.3, 1.0, 0.0, 0.4).n("B_OCT", -1.0).filt("LP 12", 1800.0, 0.8).n("F_DRIVE", 0.5)
         .env(2, 0.002, 0.5, 0.2, 0.2).n("F_ENV", 0.4).env(1, 0.003, 1.0, 0.9, 0.15).mono(0.03, true).dist("Hard", 0.4, 0.5);
    });
    p!(S, "Korg MS-20 Pulse Bass", |b| {
        b.osc('A', "Pulse Width", 0.2, 1.0, 0.0, 0.8).n("A_OCT", -1.0).filt("LP 12", 700.0, 0.6).n("F_DRIVE", 0.4).n("F_ENV", 0.45)
         .env(2, 0.001, 0.18, 0.1, 0.1).env(1, 0.002, 0.3, 0.6, 0.1).mono(0.0, false).dist("Hard", 0.3, 0.5);
    });
    p!(S, "Jupiter-4 Strings", |b| {
        b.saw('A', 5.0, 0.18, 0.5).n("A_RAND", 1.0).osc('B', "Pulse Width", 0.5, 3.0, 0.12, 0.4).n("B_RAND", 1.0)
         .filt("LP 12", 3500.0, 0.05).env(1, 0.45, 1.0, 1.0, 0.8).chorus(0.65).reverb(0.55, 0.25);
    });
    p!(S, "EMS VCS3 Blips", |b| {
        b.osc('A', "Basic Shapes", 0.0, 1.0, 0.0, 0.6).osc('B', "Metal FM", 0.2, 1.0, 0.0, 0.4).e("A_WMODE", "FM from other").n("A_WARP", 0.5)
         .filt("LP 24", 2000.0, 0.5).lfo(1, "Sample&Hold", 6.0, "1/8").m(1, "LFO 1", "A Pitch", 0.15).m(2, "LFO 1", "Cutoff", 0.35)
         .env(1, 0.002, 0.25, 0.0, 0.15).delay("1/8.", 0.5, 0.3).reverb(0.7, 0.35);
    });
    p!(S, "Clavinet D6", |b| {
        b.osc('A', "Pulse Width", 0.15, 1.0, 0.0, 0.7).osc('B', "Pulse Width", 0.6, 1.0, 0.0, 0.4)
         .filt("LP 12", 900.0, 0.2).n("F_ENV", 0.6).n("F_KEY", 0.5).env(2, 0.001, 0.14, 0.0, 0.1).env(1, 0.001, 0.22, 0.0, 0.08);
    });
    p!(S, "Fender Rhodes Mk I", |b| {
        b.osc('A', "E-Piano", 0.3, 1.0, 0.0, 0.7).osc('B', "FM Bell", 0.1, 1.0, 0.0, 0.2).n("B_OCT", 2.0)
         .filt("LP 12", 4800.0, 0.05).env(1, 0.002, 2.0, 0.0, 0.5).m(1, "Velocity", "B Level", 0.4).chorus(0.45).reverb(0.4, 0.15);
    });
    p!(S, "Wurlitzer 200A", |b| {
        b.osc('A', "E-Piano", 0.7, 1.0, 0.0, 0.7).osc('B', "Pulse Width", 0.5, 1.0, 0.0, 0.2).n("B_OCT", 1.0)
         .filt("LP 12", 3600.0, 0.1).env(1, 0.002, 1.2, 0.0, 0.3).m(1, "Velocity", "Cutoff", 0.3).dist("Tube", 0.15, 0.3).chorus(0.3);
    });
    p!(S, "Hammond B3 Drawbar", |b| {
        b.osc('A', "Organ", 0.15, 1.0, 0.0, 0.7).osc('B', "Organ", 0.7, 1.0, 0.0, 0.35).n("B_OCT", 1.0)
         .on("SUB_ON", true).n("SUB_LEVEL", 0.3).e("SUB_SHAPE", "Sine").filt("LP 12", 7000.0, 0.05).env(1, 0.003, 1.0, 1.0, 0.06)
         .lfo(1, "Sine", 6.8, "Free").m(1, "LFO 1", "A Pitch", 0.007).dist("Tube", 0.3, 0.5).chorus(0.35).reverb(0.35, 0.12);
    });
    p!(S, "Pro-One Lead", |b| {
        b.saw('A', 1.0, 0.0, 0.7).osc('B', "Pulse Width", 0.4, 1.0, 0.0, 0.45).n("B_FINE", 8.0).filt("LP 24", 3000.0, 0.3).n("F_ENV", 0.3)
         .env(2, 0.003, 0.4, 0.5, 0.2).env(1, 0.003, 1.0, 1.0, 0.18).mono(0.06, true).vibrato(2, 1, 0.01).delay("1/8", 0.3, 0.15);
    });
    p!(S, "Moog Taurus Bass Pedal", |b| {
        b.osc('A', "Basic Shapes", 0.5, 1.0, 0.0, 0.8).n("A_OCT", -1.0).osc('B', "Basic Shapes", 0.0, 1.0, 0.0, 0.5).n("B_OCT", -2.0)
         .filt("LP 24", 400.0, 0.25).env(1, 0.01, 1.0, 1.0, 0.3).mono(0.0, true).dist("Tube", 0.25, 0.5);
    });
    p!(S, "Roland SH-5 Space Lead", |b| {
        b.saw('A', 2.0, 0.1, 0.7).osc('B', "Pulse Width", 0.5, 1.0, 0.0, 0.4).filt("LP 24", 3800.0, 0.4)
         .env(1, 0.01, 1.0, 1.0, 0.35).mono(0.12, true).vibrato(1, 1, 0.012).delay("1/4", 0.45, 0.3).reverb(0.7, 0.3);
    });

    // ===================================================================== 80s
    const E: &str = "80s Synths";
    p!(E, "DX7 Tine E-Piano", |b| {
        b.osc('A', "E-Piano", 0.5, 1.0, 0.0, 0.65).osc('B', "FM Bell", 0.3, 1.0, 0.0, 0.3).n("B_OCT", 2.0)
         .filt("LP 12", 8000.0, 0.05).env(1, 0.002, 1.8, 0.0, 0.5).m(1, "Velocity", "B Level", 0.5).chorus(0.4).reverb(0.4, 0.15);
    });
    p!(E, "DX7 Slap Bass", |b| {
        b.osc('A', "FM Bell", 0.1, 1.0, 0.0, 0.7).n("A_OCT", -1.0).osc('B', "Basic Shapes", 0.0, 1.0, 0.0, 0.5).n("B_OCT", -1.0)
         .filt("LP 24", 1800.0, 0.1).n("F_ENV", 0.4).env(2, 0.001, 0.12, 0.0, 0.1).env(1, 0.001, 0.4, 0.5, 0.08).m(1, "Env 2", "A Position", 0.4).mono(0.0, false);
    });
    p!(E, "DX7 Marimba", |b| {
        b.osc('A', "FM Bell", 0.15, 1.0, 0.0, 0.65).osc('B', "Basic Shapes", 0.0, 1.0, 0.0, 0.3).n("B_OCT", 2.0)
         .env(1, 0.001, 0.4, 0.0, 0.2).env(2, 0.001, 0.08, 0.0, 0.1).m(1, "Env 2", "A Position", 0.3).filt("LP 12", 5000.0, 0.05).reverb(0.35, 0.12);
    });
    p!(E, "DX7 Tubular Bell", |b| {
        b.osc('A', "FM Bell", 0.6, 1.0, 0.0, 0.65).osc('B', "Metal FM", 0.3, 1.0, 0.0, 0.25).n("B_OCT", 1.0)
         .env(1, 0.002, 2.5, 0.0, 1.2).filt("LP 12", 9000.0, 0.05).chorus(0.3).reverb(0.7, 0.3);
    });
    p!(E, "DX7 Brass", |b| {
        b.osc('A', "Metal FM", 0.2, 1.0, 0.0, 0.6).osc('B', "Analog Saw>Sq", 0.4, 1.0, 0.0, 0.4)
         .filt("LP 24", 2200.0, 0.1).n("F_ENV", 0.4).env(2, 0.08, 0.5, 0.6, 0.3).env(1, 0.05, 1.0, 1.0, 0.25).m(1, "Env 2", "A Position", 0.3).chorus(0.3).reverb(0.4, 0.15);
    });
    p!(E, "Juno-60 Chorus Pad", |b| {
        b.saw('A', 3.0, 0.14, 0.55).osc('B', "Pulse Width", 0.5, 3.0, 0.1, 0.45).n("B_RAND", 1.0)
         .filt("LP 24", 2300.0, 0.1).env(1, 0.6, 1.0, 1.0, 1.2).lfo(1, "Triangle", 0.5, "Free").m(1, "LFO 1", "B Position", 0.2).chorus(0.8).reverb(0.5, 0.2);
    });
    p!(E, "Juno-106 Strings", |b| {
        b.saw('A', 5.0, 0.2, 0.55).n("A_RAND", 1.0).osc('B', "Pulse Width", 0.5, 3.0, 0.15, 0.4).n("B_RAND", 1.0)
         .filt("LP 12", 4000.0, 0.05).env(1, 0.4, 1.0, 1.0, 0.7).chorus(0.85).reverb(0.55, 0.22);
    });
    p!(E, "Juno Sub Pluck", |b| {
        b.osc('A', "Pulse Width", 0.5, 1.0, 0.0, 0.65).on("SUB_ON", true).n("SUB_LEVEL", 0.4).e("SUB_SHAPE", "Square")
         .filt("LP 24", 600.0, 0.3).n("F_ENV", 0.55).env(2, 0.001, 0.22, 0.0, 0.15).env(1, 0.002, 0.35, 0.0, 0.15).chorus(0.4).delay("1/8", 0.3, 0.15);
    });
    p!(E, "Jupiter-8 Brass", |b| {
        b.saw('A', 5.0, 0.16, 0.6).saw('B', 3.0, 0.12, 0.45).n("B_OCT", -1.0).filt("LP 24", 750.0, 0.2).n("F_ENV", 0.6)
         .env(2, 0.1, 0.7, 0.55, 0.3).env(1, 0.06, 1.0, 1.0, 0.25).chorus(0.35).reverb(0.45, 0.2);
    });
    p!(E, "Jupiter-8 Strings", |b| {
        b.osc('A', "Analog Saw>Sq", 0.2, 7.0, 0.22, 0.5).n("A_RAND", 1.0).saw('B', 5.0, 0.2, 0.4).n("B_RAND", 1.0).n("B_OCT", 1.0)
         .filt("LP 24", 4200.0, 0.05).env(1, 0.5, 1.0, 1.0, 0.9).chorus(0.6).reverb(0.6, 0.25);
    });
    p!(E, "Jupiter-8 Sweep Pad", |b| {
        b.saw('A', 5.0, 0.22, 0.5).n("A_RAND", 1.0).osc('B', "Pulse Width", 0.5, 3.0, 0.15, 0.4)
         .filt("LP 24", 350.0, 0.35).n("F_ENV", 0.75).env(2, 2.0, 1.5, 0.5, 1.5).env(1, 0.9, 1.0, 1.0, 1.5).chorus(0.5).reverb(0.7, 0.3);
    });
    p!(E, "Jupiter-8 Arp Pluck", |b| {
        b.saw('A', 3.0, 0.14, 0.65).osc('B', "Pulse Width", 0.4, 1.0, 0.0, 0.4).filt("LP 24", 600.0, 0.3).n("F_ENV", 0.7)
         .env(2, 0.001, 0.22, 0.0, 0.15).env(1, 0.002, 0.3, 0.0, 0.15).chorus(0.35).delay("1/8.", 0.35, 0.2).reverb(0.4, 0.15);
    });
    p!(E, "OB-Xa Stab", |b| {
        b.saw('A', 5.0, 0.2, 0.6).saw('B', 3.0, 0.15, 0.5).n("B_FINE", 7.0).filt("LP 24", 1300.0, 0.2).n("F_ENV", 0.5)
         .env(2, 0.001, 0.25, 0.1, 0.15).env(1, 0.002, 0.3, 0.0, 0.15).chorus(0.3).reverb(0.4, 0.15);
    });
    p!(E, "OB-8 Brass Pad", |b| {
        b.saw('A', 5.0, 0.2, 0.55).n("A_RAND", 1.0).osc('B', "Pulse Width", 0.5, 3.0, 0.15, 0.4)
         .filt("LP 24", 900.0, 0.15).n("F_ENV", 0.4).env(2, 0.5, 1.0, 0.7, 0.8).env(1, 0.4, 1.0, 1.0, 0.8).chorus(0.4).reverb(0.6, 0.25);
    });
    p!(E, "Prophet-600 Pluck", |b| {
        b.osc('A', "Pulse Width", 0.3, 2.0, 0.08, 0.65).osc('B', "Analog Saw>Sq", 0.5, 1.0, 0.0, 0.4)
         .filt("LP 24", 800.0, 0.3).n("F_ENV", 0.6).env(2, 0.001, 0.3, 0.0, 0.2).env(1, 0.002, 0.45, 0.0, 0.2).chorus(0.25).reverb(0.4, 0.15);
    });
    p!(E, "Prophet VS Vector Pad", |b| {
        b.osc('A', "Harmonic Sweep", 0.2, 3.0, 0.2, 0.5).osc('B', "Hollow Glass", 0.5, 3.0, 0.2, 0.45).n("B_RAND", 1.0).filt("LP 24", 7000.0, 0.1)
         .env(1, 1.0, 1.0, 1.0, 1.8).lfo(1, "Triangle", 0.12, "Free").m(1, "LFO 1", "A Position", 0.8).m(2, "LFO 1", "A Level", -0.4).m(3, "LFO 1", "B Level", 0.4)
         .chorus(0.4).reverb(0.75, 0.35);
    });
    p!(E, "PPG Wave Glass Pad", |b| {
        b.osc('A', "Hollow Glass", 0.3, 3.0, 0.2, 0.55).osc('B', "Spectral Comb", 0.3, 3.0, 0.15, 0.4).filt("LP 24", 6500.0, 0.15)
         .env(1, 0.9, 1.0, 1.0, 1.8).lfo(1, "Sine", 0.17, "Free").m(1, "LFO 1", "A Position", 0.6).m(2, "LFO 1", "B Position", -0.6)
         .dist("Bitcrush", 0.08, 0.2).chorus(0.35).reverb(0.8, 0.4);
    });
    p!(E, "PPG Digital Bell", |b| {
        b.osc('A', "FM Bell", 0.5, 1.0, 0.0, 0.6).osc('B', "Spectral Comb", 0.7, 1.0, 0.0, 0.3).n("B_OCT", 1.0)
         .env(1, 0.002, 2.0, 0.0, 1.0).dist("Bitcrush", 0.1, 0.25).filt("LP 12", 9000.0, 0.05).chorus(0.3).reverb(0.7, 0.3);
    });
    p!(E, "Fairlight Orchestra Stab", |b| {
        b.osc('A', "Bowed Strings", 0.3, 3.0, 0.15, 0.55).osc('B', "Metal FM", 0.3, 1.0, 0.0, 0.4)
         .filt("LP 24", 3500.0, 0.1).env(1, 0.002, 0.5, 0.0, 0.3).dist("Bitcrush", 0.15, 0.3).reverb(0.5, 0.2);
    });
    p!(E, "Fairlight ArcoPad", |b| {
        b.osc('A', "Bowed Strings", 0.7, 3.0, 0.2, 0.55).n("A_RAND", 1.0).osc('B', "Choir", 0.2, 1.0, 0.0, 0.3)
         .filt("LP 12", 3500.0, 0.05).env(1, 0.8, 1.0, 1.0, 1.2).dist("Bitcrush", 0.1, 0.25).chorus(0.4).reverb(0.7, 0.3);
    });
    p!(E, "Synclavier Bright Pluck", |b| {
        b.osc('A', "Harmonic Sweep", 0.8, 2.0, 0.1, 0.65).osc('B', "FM Bell", 0.3, 1.0, 0.0, 0.3)
         .filt("LP 24", 7000.0, 0.1).env(1, 0.001, 0.5, 0.0, 0.3).env(2, 0.001, 0.2, 0.0, 0.1).m(1, "Env 2", "A Position", -0.5).chorus(0.3).reverb(0.5, 0.2);
    });
    p!(E, "D-50 Digital Native Dance", |b| {
        b.osc('A', "Analog Saw>Sq", 0.4, 3.0, 0.15, 0.55).osc('B', "Choir", 0.6, 1.0, 0.0, 0.4).filt("LP 12", 8500.0, 0.1)
         .env(1, 0.3, 1.0, 1.0, 1.2).lfo(1, "Sine", 0.2, "Free").m(1, "LFO 1", "B Position", 0.4).chorus(0.5).delay("1/4", 0.4, 0.2).reverb(0.7, 0.35);
    });
    p!(E, "D-50 Fantasia", |b| {
        b.osc('A', "Hollow Glass", 0.5, 3.0, 0.2, 0.5).osc('B', "FM Bell", 0.4, 1.0, 0.0, 0.35).n("B_OCT", 1.0)
         .filt("LP 24", 9000.0, 0.1).env(1, 0.6, 1.0, 1.0, 2.0).env(2, 0.001, 1.5, 0.0, 0.5).m(1, "Env 2", "B Level", 0.6)
         .chorus(0.45).delay("1/4", 0.5, 0.3).reverb(0.85, 0.45);
    });
    p!(E, "Korg M1 Organ", |b| {
        b.osc('A', "Organ", 0.5, 1.0, 0.0, 0.6).osc('B', "Organ", 0.2, 1.0, 0.0, 0.4).n("B_OCT", 1.0).filt("LP 12", 6500.0, 0.05)
         .env(1, 0.004, 1.0, 1.0, 0.08).chorus(0.5).reverb(0.4, 0.15);
    });
    p!(E, "Casio CZ Bass", |b| {
        b.osc('A', "Sync Sweep", 0.2, 1.0, 0.0, 0.75).n("A_OCT", -1.0).filt("LP 24", 1500.0, 0.2).n("F_ENV", 0.45)
         .env(2, 0.001, 0.3, 0.1, 0.1).m(1, "Env 2", "A Position", 0.4).env(1, 0.002, 0.5, 0.6, 0.1).mono(0.0, false);
    });
    p!(E, "Casio CZ Glass Bell", |b| {
        b.osc('A', "Sync Sweep", 0.6, 1.0, 0.0, 0.6).osc('B', "FM Bell", 0.4, 1.0, 0.0, 0.35).n("B_OCT", 1.0)
         .env(1, 0.002, 1.5, 0.0, 0.8).env(2, 0.001, 0.4, 0.0, 0.2).m(1, "Env 2", "A Position", -0.4).filt("LP 12", 9000.0, 0.05).chorus(0.3).reverb(0.6, 0.3);
    });
    p!(E, "SH-101 Sub Lead", |b| {
        b.saw('A', 1.0, 0.0, 0.7).osc('B', "Pulse Width", 0.5, 1.0, 0.0, 0.4).on("SUB_ON", true).n("SUB_LEVEL", 0.4).e("SUB_SHAPE", "Square")
         .filt("LP 24", 2200.0, 0.35).n("F_ENV", 0.35).env(2, 0.003, 0.4, 0.4, 0.15).env(1, 0.003, 1.0, 1.0, 0.12).mono(0.06, true).vibrato(2, 1, 0.01).delay("1/8", 0.3, 0.15);
    });
    p!(E, "SH-101 Acid Bass", |b| {
        b.saw('A', 1.0, 0.0, 0.75).on("SUB_ON", true).n("SUB_LEVEL", 0.3).e("SUB_SHAPE", "Square").filt("LP 24", 360.0, 0.7).n("F_ENV", 0.55)
         .env(2, 0.001, 0.2, 0.0, 0.1).env(1, 0.002, 0.5, 0.7, 0.08).mono(0.04, true).dist("Soft", 0.25, 0.5);
    });
    p!(E, "TB-303 Squelch", |b| {
        b.saw('A', 1.0, 0.0, 0.8).filt("LP 24", 300.0, 0.88).n("F_ENV", 0.65).n("F_DRIVE", 0.3)
         .env(2, 0.001, 0.2, 0.0, 0.08).env(1, 0.001, 0.6, 0.8, 0.06).mono(0.05, true).dist("Hard", 0.3, 0.55).delay("1/8.", 0.35, 0.18);
    });
    p!(E, "Memorymoog Lead", |b| {
        b.saw('A', 3.0, 0.1, 0.65).osc('B', "Pulse Width", 0.4, 1.0, 0.0, 0.4).filt("LP 24", 2800.0, 0.3).n("F_ENV", 0.35)
         .env(2, 0.005, 0.5, 0.5, 0.2).env(1, 0.005, 1.0, 1.0, 0.2).mono(0.07, true).vibrato(2, 1, 0.01).chorus(0.3).delay("1/8", 0.3, 0.18).reverb(0.4, 0.15);
    });
    p!(E, "Emulator II Strings", |b| {
        b.osc('A', "Bowed Strings", 0.4, 3.0, 0.18, 0.55).n("A_RAND", 1.0).filt("LP 12", 3500.0, 0.05).env(1, 0.3, 1.0, 1.0, 0.7)
         .dist("Bitcrush", 0.08, 0.2).chorus(0.6).reverb(0.55, 0.25);
    });
}
