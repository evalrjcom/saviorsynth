//! Style banks: original patches recreating the sound *archetypes* the three most popular
//! soft synths are known for (Sylenth1-style, Diva-style, Serum-style). These are NOT copies of
//! any vendor's factory presets (those are proprietary) — every patch is built from scratch on
//! this engine, so no names, samples or parameter dumps from those products are used.

use super::{Preset, B};

pub fn add(out: &mut Vec<Preset>) {
    macro_rules! p {
        ($cat:expr, $name:expr, |$b:ident| $body:block) => {{ let mut $b = B::new($name); $body; out.push($b.done($cat)); }};
    }

    // ===================================================================== Syl Style
    // Dual-saw/square analog-ish subtractive, fat unison, trance/EDM staples.
    const S: &str = "Syl Style";
    p!(S, "Trance Gate Lead", |b| {
        b.saw('A', 7.0, 0.4, 0.6).n("A_RAND", 0.9).saw('B', 5.0, 0.3, 0.45).n("B_OCT", 1.0).n("B_RAND", 0.9)
         .filt("LP 24", 8500.0, 0.12).env(1, 0.004, 1.0, 1.0, 0.2).chorus(0.3).delay("1/8.", 0.4, 0.25).reverb(0.55, 0.2);
    });
    p!(S, "Big Room Saw Lead", |b| {
        b.saw('A', 7.0, 0.5, 0.65).n("A_SPREAD", 1.0).n("A_RAND", 1.0).saw('B', 3.0, 0.25, 0.4).n("B_FINE", 7.0)
         .filt("LP 24", 10000.0, 0.08).env(1, 0.003, 1.0, 1.0, 0.3).vibrato(1, 1, 0.01).dist("Soft", 0.15, 0.4).chorus(0.35).reverb(0.6, 0.22);
    });
    p!(S, "Euphoric Pluck Arp", |b| {
        b.saw('A', 5.0, 0.3, 0.6).n("A_RAND", 0.8).osc('B', "Pulse Width", 0.4, 3.0, 0.2, 0.4)
         .filt("LP 24", 1400.0, 0.2).n("F_ENV", 0.65).env(2, 0.001, 0.28, 0.0, 0.2).env(1, 0.002, 0.32, 0.0, 0.3)
         .chorus(0.3).delay("1/8.", 0.45, 0.3).reverb(0.6, 0.25);
    });
    p!(S, "Progressive Pluck", |b| {
        b.saw('A', 3.0, 0.2, 0.65).osc('B', "Basic Shapes", 0.75, 1.0, 0.0, 0.4).n("B_OCT", 1.0)
         .filt("LP 24", 900.0, 0.3).n("F_ENV", 0.7).env(2, 0.001, 0.35, 0.05, 0.25).env(1, 0.002, 0.5, 0.0, 0.3)
         .chorus(0.25).delay("1/8", 0.35, 0.22).reverb(0.5, 0.2);
    });
    p!(S, "Fat Saw Bass", |b| {
        b.saw('A', 3.0, 0.22, 0.7).n("A_OCT", -1.0).saw('B', 3.0, 0.22, 0.6).n("B_OCT", -1.0).n("B_FINE", 6.0)
         .filt("LP 24", 800.0, 0.2).n("F_ENV", 0.25).env(2, 0.002, 0.3, 0.3, 0.1).env(1, 0.003, 1.0, 1.0, 0.1)
         .on("SUB_ON", true).n("SUB_LEVEL", 0.35).mono(0.0, true).dist("Soft", 0.2, 0.5);
    });
    p!(S, "Rolling Psy Bass", |b| {
        b.saw('A', 1.0, 0.0, 0.8).filt("LP 24", 500.0, 0.45).n("F_ENV", 0.55)
         .env(2, 0.001, 0.12, 0.0, 0.05).env(1, 0.001, 0.14, 0.0, 0.04).on("SUB_ON", true).n("SUB_LEVEL", 0.3)
         .mono(0.0, false).dist("Soft", 0.3, 0.6);
    });
    p!(S, "Classic Hoover", |b| {
        b.osc('A', "Pulse Width", 0.15, 7.0, 0.55, 0.6).n("A_RAND", 1.0).saw('B', 5.0, 0.5, 0.5).n("B_RAND", 1.0)
         .filt("LP 24", 3800.0, 0.3).env(1, 0.01, 1.0, 1.0, 0.25)
         .env(2, 0.001, 0.4, 0.0, 0.2).m(1, "Env 2", "A Pitch", -0.35).m(2, "Env 2", "B Pitch", -0.35)
         .mono(0.08, true).dist("Soft", 0.3, 0.5).chorus(0.5).reverb(0.5, 0.2);
    });
    p!(S, "Rave Stab", |b| {
        b.saw('A', 5.0, 0.35, 0.6).n("A_RAND", 0.8).osc('B', "Pulse Width", 0.5, 3.0, 0.2, 0.45)
         .filt("LP 24", 1800.0, 0.25).n("F_ENV", 0.6).env(2, 0.001, 0.22, 0.1, 0.15).env(1, 0.002, 0.3, 0.0, 0.12)
         .dist("Soft", 0.25, 0.5).chorus(0.3).reverb(0.45, 0.2);
    });
    p!(S, "House Chord Stab", |b| {
        b.osc('A', "Analog Saw>Sq", 0.5, 3.0, 0.2, 0.6).osc('B', "Pulse Width", 0.35, 1.0, 0.0, 0.4)
         .filt("LP 12", 2200.0, 0.2).n("F_ENV", 0.4).env(2, 0.002, 0.18, 0.1, 0.15).env(1, 0.002, 0.28, 0.0, 0.15)
         .chorus(0.3).reverb(0.4, 0.15);
    });
    p!(S, "Warm Analog Pad", |b| {
        b.saw('A', 5.0, 0.25, 0.5).n("A_RAND", 1.0).osc('B', "Pulse Width", 0.5, 3.0, 0.2, 0.4).n("B_RAND", 1.0)
         .filt("LP 24", 1500.0, 0.1).env(1, 0.9, 1.0, 1.0, 1.6).lfo(1, "Sine", 0.2, "Free").m(1, "LFO 1", "Cutoff", 0.15)
         .chorus(0.5).reverb(0.7, 0.3);
    });
    p!(S, "Trance Gated Pad", |b| {
        b.saw('A', 7.0, 0.3, 0.55).n("A_RAND", 1.0).osc('B', "Basic Shapes", 0.3, 3.0, 0.2, 0.4)
         .filt("LP 24", 3500.0, 0.1).env(1, 0.05, 1.0, 1.0, 0.3).lfo(1, "Square", 1.0, "1/16").n("L1_FADE", 0.0)
         .m(1, "LFO 1", "Amp Level", -0.6).chorus(0.35).delay("1/8.", 0.4, 0.2).reverb(0.6, 0.25);
    });
    p!(S, "Dreamy Bell Pluck", |b| {
        b.osc('A', "FM Bell", 0.3, 3.0, 0.1, 0.6).osc('B', "Basic Shapes", 0.0, 1.0, 0.0, 0.3).n("B_OCT", 1.0)
         .filt("LP 12", 6500.0, 0.1).env(1, 0.002, 1.3, 0.0, 0.8).chorus(0.35).delay("1/8.", 0.5, 0.35).reverb(0.8, 0.4);
    });
    p!(S, "Soft Sine Lead", |b| {
        b.osc('A', "Basic Shapes", 0.0, 2.0, 0.08, 0.7).osc('B', "Basic Shapes", 0.25, 1.0, 0.0, 0.35)
         .filt("LP 24", 5500.0, 0.05).env(1, 0.02, 1.0, 1.0, 0.3).mono(0.06, true).vibrato(1, 1, 0.012)
         .chorus(0.25).delay("1/4", 0.4, 0.25).reverb(0.6, 0.25);
    });
    p!(S, "Pan Flute", |b| {
        b.osc('A', "Hollow Glass", 0.1, 1.0, 0.0, 0.5).n("NOISE_LEVEL", 0.07).on("NOISE_ON", true)
         .filt("BP", 1800.0, 0.2).env(1, 0.05, 1.0, 0.9, 0.3).vibrato(1, 1, 0.01).reverb(0.7, 0.3);
    });
    p!(S, "Analog Brass", |b| {
        b.saw('A', 3.0, 0.18, 0.65).saw('B', 2.0, 0.12, 0.5).n("B_FINE", 6.0)
         .filt("LP 24", 700.0, 0.15).n("F_ENV", 0.55).env(2, 0.09, 0.4, 0.5, 0.25).env(1, 0.05, 1.0, 1.0, 0.2)
         .chorus(0.25).reverb(0.4, 0.15);
    });
    p!(S, "Poly Strings", |b| {
        b.osc('A', "Bowed Strings", 0.4, 5.0, 0.22, 0.55).n("A_RAND", 1.0).saw('B', 3.0, 0.18, 0.35).n("B_RAND", 1.0)
         .filt("LP 24", 3200.0, 0.05).env(1, 0.3, 1.0, 1.0, 0.7).vibrato(1, 1, 0.005).chorus(0.5).reverb(0.6, 0.25);
    });
    p!(S, "Acid Squelch", |b| {
        b.saw('A', 1.0, 0.0, 0.8).filt("LP 24", 330.0, 0.85).n("F_ENV", 0.6).n("F_DRIVE", 0.3)
         .env(2, 0.001, 0.25, 0.0, 0.1).env(1, 0.002, 0.7, 0.8, 0.08).mono(0.05, true).dist("Hard", 0.25, 0.6).delay("1/8.", 0.35, 0.2);
    });
    p!(S, "Wobble Foundation", |b| {
        b.saw('A', 3.0, 0.2, 0.7).osc('B', "Basic Shapes", 0.75, 1.0, 0.0, 0.5).n("B_OCT", -1.0)
         .filt("LP 24", 500.0, 0.55).lfo(1, "Sine", 2.0, "1/8").m(1, "LFO 1", "Cutoff", 0.6)
         .on("SUB_ON", true).n("SUB_LEVEL", 0.35).env(1, 0.003, 1.0, 1.0, 0.1).mono(0.0, true).dist("Soft", 0.3, 0.7);
    });
    p!(S, "Sweep Uplifter", |b| {
        b.saw('A', 7.0, 0.5, 0.55).n("A_RAND", 1.0).on("NOISE_ON", true).n("NOISE_LEVEL", 0.18).e("NOISE_TYPE", "White")
         .filt("BP", 300.0, 0.5).n("F_ENV", 0.95).env(2, 3.5, 1.0, 1.0, 0.8).env(1, 1.5, 1.0, 1.0, 1.0)
         .m(1, "Env 2", "A Pitch", 0.35).chorus(0.4).reverb(0.7, 0.35);
    });
    p!(S, "Downlifter Sweep", |b| {
        b.saw('A', 5.0, 0.4, 0.5).on("NOISE_ON", true).n("NOISE_LEVEL", 0.2)
         .filt("BP", 9000.0, 0.45).n("F_ENV", -0.9).env(2, 0.01, 2.8, 0.0, 0.6).env(1, 0.01, 2.8, 0.0, 0.8)
         .reverb(0.8, 0.4);
    });
    p!(S, "Sync Screamer", |b| {
        b.saw('A', 3.0, 0.15, 0.7).e("A_WMODE", "Sync").n("A_WARP", 0.5).filt("LP 24", 6500.0, 0.2)
         .env(2, 0.001, 0.7, 0.4, 0.3).m(1, "Env 2", "A Warp", 0.6).mono(0.03, true).vibrato(2, 1, 0.01).dist("Soft", 0.2, 0.4).delay("1/8", 0.3, 0.2);
    });
    p!(S, "80s Poly Brass", |b| {
        b.saw('A', 5.0, 0.2, 0.55).osc('B', "Pulse Width", 0.5, 1.0, 0.0, 0.4).n("B_FINE", -8.0)
         .filt("LP 24", 900.0, 0.1).n("F_ENV", 0.5).env(2, 0.12, 0.6, 0.55, 0.4).env(1, 0.06, 1.0, 1.0, 0.4)
         .chorus(0.5).reverb(0.55, 0.25);
    });
    p!(S, "Mellow Keys", |b| {
        b.osc('A', "E-Piano", 0.3, 1.0, 0.0, 0.7).osc('B', "Basic Shapes", 0.0, 1.0, 0.0, 0.25).n("B_OCT", 1.0)
         .filt("LP 12", 5500.0, 0.05).env(1, 0.002, 1.6, 0.0, 0.5).m(1, "Velocity", "Cutoff", 0.3).chorus(0.3).reverb(0.5, 0.2);
    });
    p!(S, "Wide Supersaw Chord", |b| {
        b.saw('A', 7.0, 0.45, 0.6).n("A_SPREAD", 1.0).n("A_RAND", 1.0).saw('B', 7.0, 0.35, 0.4).n("B_RAND", 1.0).n("B_OCT", -1.0)
         .filt("LP 24", 6500.0, 0.08).env(1, 0.01, 1.0, 1.0, 0.6).chorus(0.4).reverb(0.65, 0.25);
    });
    p!(S, "Reese Growl Sub", |b| {
        b.saw('A', 3.0, 0.35, 0.65).n("A_OCT", -1.0).n("A_RAND", 0.6).saw('B', 3.0, 0.35, 0.65).n("B_OCT", -1.0).n("B_FINE", 12.0).n("B_RAND", 0.6)
         .filt("LP 24", 700.0, 0.35).lfo(1, "Smooth Rnd", 2.5, "Free").m(1, "LFO 1", "Cutoff", 0.3)
         .on("SUB_ON", true).n("SUB_LEVEL", 0.4).env(1, 0.004, 1.0, 1.0, 0.2).mono(0.03, true).dist("Soft", 0.3, 0.6).chorus(0.3);
    });

    // ===================================================================== Diva Style
    // Warm, analog-modelled voices: Moog / Jupiter / Juno / Prophet / ARP / Oberheim character.
    const D: &str = "Diva Style";
    p!(D, "Ladder Moog Bass", |b| {
        b.saw('A', 1.0, 0.0, 0.75).osc('B', "Pulse Width", 0.5, 1.0, 0.0, 0.5).n("B_OCT", -1.0).n("B_FINE", 4.0)
         .filt("LP 24", 330.0, 0.35).n("F_ENV", 0.55).n("F_DRIVE", 0.3).env(2, 0.002, 0.3, 0.2, 0.15).env(1, 0.002, 0.5, 0.7, 0.1)
         .mono(0.02, true).dist("Tube", 0.2, 0.5);
    });
    p!(D, "Taurus Pedal", |b| {
        b.osc('A', "Basic Shapes", 0.0, 1.0, 0.0, 0.8).n("A_OCT", -1.0).osc('B', "Basic Shapes", 0.5, 1.0, 0.0, 0.5).n("B_OCT", -1.0).n("B_FINE", 5.0)
         .filt("LP 24", 600.0, 0.2).env(1, 0.01, 1.0, 1.0, 0.25).mono(0.0, true).dist("Tube", 0.2, 0.5);
    });
    p!(D, "Minimoog Lead", |b| {
        b.saw('A', 2.0, 0.08, 0.7).osc('B', "Pulse Width", 0.5, 1.0, 0.0, 0.5).n("B_FINE", 6.0)
         .filt("LP 24", 2600.0, 0.3).n("F_ENV", 0.35).n("F_DRIVE", 0.25).env(2, 0.005, 0.5, 0.5, 0.2).env(1, 0.005, 1.0, 1.0, 0.2)
         .mono(0.08, true).vibrato(2, 1, 0.01).dist("Tube", 0.25, 0.5).delay("1/8", 0.3, 0.18);
    });
    p!(D, "Jupiter Brass", |b| {
        b.saw('A', 3.0, 0.15, 0.65).osc('B', "Pulse Width", 0.5, 3.0, 0.12, 0.45)
         .filt("LP 24", 800.0, 0.12).n("F_ENV", 0.5).env(2, 0.1, 0.5, 0.55, 0.3).env(1, 0.06, 1.0, 1.0, 0.25).chorus(0.35).reverb(0.4, 0.15);
    });
    p!(D, "Jupiter Strings", |b| {
        b.saw('A', 5.0, 0.2, 0.55).n("A_RAND", 1.0).osc('B', "Pulse Width", 0.5, 3.0, 0.18, 0.4).n("B_RAND", 1.0)
         .filt("LP 12", 4500.0, 0.05).env(1, 0.55, 1.0, 1.0, 0.8).vibrato(1, 1, 0.005).chorus(0.6).reverb(0.55, 0.22);
    });
    p!(D, "Juno Chorus Pad", |b| {
        b.saw('A', 3.0, 0.15, 0.55).osc('B', "Pulse Width", 0.5, 3.0, 0.1, 0.45).n("B_RAND", 1.0)
         .filt("LP 24", 2400.0, 0.1).env(1, 0.5, 1.0, 1.0, 1.2).lfo(1, "Triangle", 0.4, "Free").m(1, "LFO 1", "B Position", 0.2)
         .chorus(0.7).reverb(0.55, 0.22);
    });
    p!(D, "Juno Sub Bass", |b| {
        b.osc('A', "Pulse Width", 0.5, 1.0, 0.0, 0.7).n("A_OCT", -1.0).on("SUB_ON", true).n("SUB_LEVEL", 0.5).e("SUB_SHAPE", "Square")
         .filt("LP 12", 700.0, 0.1).env(1, 0.005, 1.0, 1.0, 0.15).mono(0.0, true).chorus(0.3);
    });
    p!(D, "Prophet Poly Sweep", |b| {
        b.saw('A', 3.0, 0.2, 0.6).osc('B', "Analog Saw>Sq", 0.5, 3.0, 0.15, 0.45)
         .filt("LP 24", 500.0, 0.25).n("F_ENV", 0.7).env(2, 1.2, 1.5, 0.4, 1.0).env(1, 0.3, 1.0, 1.0, 1.2)
         .chorus(0.35).reverb(0.6, 0.25);
    });
    p!(D, "Oberheim Stack Pad", |b| {
        b.saw('A', 5.0, 0.28, 0.5).n("A_RAND", 1.0).saw('B', 5.0, 0.28, 0.45).n("B_FINE", 8.0).n("B_RAND", 1.0)
         .filt("LP 12", 1800.0, 0.1).env(1, 0.8, 1.0, 1.0, 1.4).lfo(1, "Sine", 0.17, "Free").m(1, "LFO 1", "Cutoff", 0.2)
         .chorus(0.45).reverb(0.65, 0.28);
    });
    p!(D, "ARP Screamer", |b| {
        b.saw('A', 2.0, 0.1, 0.7).osc('B', "Pulse Width", 0.5, 1.0, 0.0, 0.5).n("B_FINE", 12.0)
         .filt("LP 24", 3400.0, 0.5).n("F_DRIVE", 0.45).env(1, 0.005, 1.0, 1.0, 0.2).mono(0.1, true)
         .vibrato(1, 1, 0.012).dist("Hard", 0.3, 0.5).delay("1/8.", 0.35, 0.2);
    });
    p!(D, "CS Epic Brass", |b| {
        b.saw('A', 5.0, 0.2, 0.55).saw('B', 3.0, 0.14, 0.45).n("B_OCT", -1.0)
         .filt("LP 24", 600.0, 0.2).n("F_ENV", 0.65).env(2, 0.45, 1.0, 0.65, 0.5).env(1, 0.3, 1.0, 1.0, 0.45)
         .vibrato(1, 1, 0.01).chorus(0.4).reverb(0.65, 0.3);
    });
    p!(D, "Polysix Ensemble", |b| {
        b.osc('A', "Analog Saw>Sq", 0.3, 5.0, 0.18, 0.55).n("A_RAND", 1.0).osc('B', "Pulse Width", 0.5, 1.0, 0.0, 0.3)
         .filt("LP 24", 3000.0, 0.08).env(1, 0.2, 1.0, 1.0, 0.6).chorus(0.8).reverb(0.5, 0.2);
    });
    p!(D, "PWM Strings", |b| {
        b.osc('A', "Pulse Width", 0.5, 3.0, 0.14, 0.6).n("A_RAND", 1.0).lfo(1, "Triangle", 0.6, "Free").m(1, "LFO 1", "A Position", 0.35)
         .filt("LP 12", 3500.0, 0.05).env(1, 0.35, 1.0, 1.0, 0.7).chorus(0.6).reverb(0.5, 0.22);
    });
    p!(D, "Analog Pluck", |b| {
        b.saw('A', 2.0, 0.1, 0.7).osc('B', "Pulse Width", 0.4, 1.0, 0.0, 0.4)
         .filt("LP 24", 500.0, 0.3).n("F_ENV", 0.7).env(2, 0.001, 0.25, 0.0, 0.2).env(1, 0.002, 0.4, 0.0, 0.2)
         .chorus(0.25).delay("1/8", 0.3, 0.2).reverb(0.4, 0.15);
    });
    p!(D, "Hollow Organ", |b| {
        b.osc('A', "Organ", 0.2, 1.0, 0.0, 0.7).osc('B', "Organ", 0.6, 1.0, 0.0, 0.3).n("B_OCT", 1.0)
         .filt("LP 12", 6000.0, 0.05).env(1, 0.004, 1.0, 1.0, 0.1).lfo(1, "Sine", 6.0, "Free").m(1, "LFO 1", "A Pitch", 0.006).chorus(0.4).reverb(0.4, 0.15);
    });
    p!(D, "Vintage Clav Funk", |b| {
        b.osc('A', "Pulse Width", 0.2, 1.0, 0.0, 0.7).osc('B', "Pulse Width", 0.6, 1.0, 0.0, 0.4)
         .filt("LP 12", 700.0, 0.2).n("F_ENV", 0.55).n("F_KEY", 0.5).env(2, 0.001, 0.15, 0.0, 0.1).env(1, 0.001, 0.25, 0.0, 0.1);
    });
    p!(D, "Rhodes Tine", |b| {
        b.osc('A', "E-Piano", 0.5, 1.0, 0.0, 0.7).osc('B', "FM Bell", 0.1, 1.0, 0.0, 0.2).n("B_OCT", 2.0)
         .filt("LP 12", 4500.0, 0.05).env(1, 0.002, 2.0, 0.0, 0.5).m(1, "Velocity", "B Level", 0.3).chorus(0.4).reverb(0.4, 0.15);
    });
    p!(D, "Vangelis Swell", |b| {
        b.saw('A', 5.0, 0.2, 0.55).saw('B', 3.0, 0.15, 0.4).n("B_OCT", -1.0)
         .filt("LP 24", 450.0, 0.2).n("F_ENV", 0.7).env(2, 0.9, 1.2, 0.6, 1.2).env(1, 0.6, 1.0, 1.0, 1.4)
         .vibrato(1, 1, 0.008).chorus(0.4).delay("1/4", 0.45, 0.25).reverb(0.8, 0.4);
    });
    p!(D, "Wide Sync Lead", |b| {
        b.saw('A', 3.0, 0.12, 0.7).e("A_WMODE", "Sync").n("A_WARP", 0.4).filt("LP 24", 5200.0, 0.2)
         .lfo(1, "Triangle", 4.0, "Free").m(1, "LFO 1", "A Warp", 0.2).env(1, 0.005, 1.0, 1.0, 0.2)
         .mono(0.06, true).vibrato(2, 2, 0.01).chorus(0.3).delay("1/8.", 0.35, 0.2);
    });
    p!(D, "Glide Mono Lead", |b| {
        b.osc('A', "Analog Saw>Sq", 0.2, 2.0, 0.08, 0.7).osc('B', "Basic Shapes", 0.0, 1.0, 0.0, 0.4).n("B_OCT", -1.0)
         .filt("LP 24", 2800.0, 0.25).env(1, 0.01, 1.0, 1.0, 0.25).mono(0.15, true).vibrato(1, 1, 0.01).dist("Tube", 0.2, 0.5).reverb(0.4, 0.15);
    });
    p!(D, "Filter Sweep Pad", |b| {
        b.saw('A', 5.0, 0.22, 0.5).n("A_RAND", 1.0).osc('B', "Pulse Width", 0.5, 3.0, 0.16, 0.4)
         .filt("LP 24", 300.0, 0.4).n("F_ENV", 0.8).env(2, 2.5, 1.5, 0.5, 1.5).env(1, 0.8, 1.0, 1.0, 1.5)
         .chorus(0.5).reverb(0.7, 0.3);
    });
    p!(D, "Resonant Bell Pad", |b| {
        b.osc('A', "FM Bell", 0.4, 3.0, 0.12, 0.55).filt("LP 24", 2200.0, 0.5).n("F_ENV", 0.4)
         .env(2, 0.01, 1.5, 0.0, 0.8).env(1, 0.01, 1.8, 0.3, 1.5).chorus(0.4).delay("1/4", 0.45, 0.25).reverb(0.75, 0.35);
    });
    p!(D, "Sci-Fi Drone", |b| {
        b.saw('A', 5.0, 0.4, 0.5).n("A_OCT", -1.0).osc('B', "Spectral Comb", 0.5, 3.0, 0.3, 0.4)
         .filt("BP", 600.0, 0.5).env(1, 2.0, 1.0, 1.0, 3.0).lfo(1, "Smooth Rnd", 0.3, "Free").m(1, "LFO 1", "Cutoff", 0.5).m(2, "LFO 1", "B Position", 0.5)
         .chorus(0.4).reverb(0.85, 0.45);
    });
    p!(D, "Analog Kick Tom", |b| {
        b.osc('A', "Basic Shapes", 0.0, 1.0, 0.0, 0.9).env(1, 0.001, 0.35, 0.0, 0.1).env(2, 0.001, 0.09, 0.0, 0.1)
         .m(1, "Env 2", "A Pitch", 0.6).nofilt().dist("Soft", 0.2, 0.5);
    });

    // ===================================================================== Serum Style
    // Wavetable-first modern sound design: warp modes, position modulation, FM/ring, growl.
    const R: &str = "Serum Style";
    p!(R, "Neuro Growl", |b| {
        b.osc('A', "Bass Growl", 0.3, 3.0, 0.2, 0.7).osc('B', "Metal FM", 0.4, 1.0, 0.0, 0.5).e("B_WMODE", "FM from other").n("B_WARP", 0.35)
         .filt("LP 24", 1500.0, 0.45).lfo(1, "Sine", 2.0, "1/8").m(1, "LFO 1", "A Position", 0.7).lfo(2, "Saw Down", 2.0, "1/4").m(2, "LFO 2", "Cutoff", 0.4)
         .on("SUB_ON", true).n("SUB_LEVEL", 0.35).env(1, 0.003, 1.0, 1.0, 0.1).mono(0.0, true).dist("Fold", 0.5, 0.75);
    });
    p!(R, "Talking Bass", |b| {
        b.osc('A', "Formant Vowels", 0.0, 2.0, 0.1, 0.8).saw('B', 1.0, 0.0, 0.4).n("B_OCT", -1.0)
         .filt("LP 24", 2500.0, 0.3).lfo(1, "Triangle", 1.0, "1/4").m(1, "LFO 1", "A Position", 0.9)
         .on("SUB_ON", true).n("SUB_LEVEL", 0.3).env(1, 0.003, 1.0, 1.0, 0.1).mono(0.0, true).dist("Hard", 0.4, 0.6);
    });
    p!(R, "Riddim Square Bass", |b| {
        b.osc('A', "Pulse Width", 0.5, 2.0, 0.08, 0.7).e("A_WMODE", "PWM").n("A_WARP", 0.3).osc('B', "Metal FM", 0.2, 1.0, 0.0, 0.4).e("B_WMODE", "Ring other").n("B_WARP", 0.2)
         .filt("LP 24", 900.0, 0.5).lfo(1, "Square", 2.0, "1/8T").m(1, "LFO 1", "Cutoff", 0.55)
         .on("SUB_ON", true).n("SUB_LEVEL", 0.4).env(1, 0.003, 1.0, 1.0, 0.1).mono(0.0, true).dist("Hard", 0.5, 0.7);
    });
    p!(R, "Future Bass Chord", |b| {
        b.saw('A', 7.0, 0.5, 0.6).n("A_SPREAD", 1.0).n("A_RAND", 1.0).saw('B', 7.0, 0.4, 0.45).n("B_OCT", 1.0).n("B_RAND", 1.0)
         .filt("LP 24", 3800.0, 0.15).env(1, 0.005, 1.2, 0.8, 0.5)
         .lfo(1, "Saw Down", 1.0, "1/8").m(1, "LFO 1", "Cutoff", 0.25).dist("Soft", 0.15, 0.4).chorus(0.5).reverb(0.55, 0.25);
    });
    p!(R, "Pitched Vocal Chop", |b| {
        b.osc('A', "Choir", 0.3, 2.0, 0.08, 0.75).osc('B', "Formant Vowels", 0.4, 1.0, 0.0, 0.4)
         .filt("LP 12", 6500.0, 0.1).env(1, 0.005, 0.5, 0.3, 0.25).lfo(1, "Smooth Rnd", 3.0, "Free").m(1, "LFO 1", "B Position", 0.4)
         .chorus(0.4).delay("1/8.", 0.4, 0.3).reverb(0.6, 0.3);
    });
    p!(R, "Trap Bell Lead", |b| {
        b.osc('A', "FM Bell", 0.3, 1.0, 0.0, 0.7).osc('B', "Hollow Glass", 0.4, 1.0, 0.0, 0.3).n("B_OCT", 1.0)
         .filt("LP 12", 7000.0, 0.1).env(1, 0.002, 1.1, 0.0, 0.5).delay("1/8.", 0.4, 0.3).reverb(0.65, 0.3);
    });
    p!(R, "Glitch Hop Lead", |b| {
        b.osc('A', "Metal FM", 0.5, 2.0, 0.1, 0.65).e("A_WMODE", "Quantize").n("A_WARP", 0.45)
         .filt("LP 24", 5000.0, 0.25).env(1, 0.003, 0.5, 0.7, 0.2).env(2, 0.001, 0.4, 0.0, 0.2).m(1, "Env 2", "A Position", 0.5)
         .mono(0.02, true).dist("Decimate", 0.3, 0.3).delay("1/16.", 0.4, 0.25);
    });
    p!(R, "Hybrid Supersaw Lead", |b| {
        b.osc('A', "Analog Saw>Sq", 0.1, 7.0, 0.45, 0.65).n("A_SPREAD", 1.0).n("A_RAND", 1.0).osc('B', "Sync Sweep", 0.4, 3.0, 0.2, 0.4)
         .filt("LP 24", 9000.0, 0.1).env(1, 0.004, 1.0, 1.0, 0.3).lfo(1, "Triangle", 0.5, "Free").m(1, "LFO 1", "B Position", 0.5)
         .vibrato(2, 2, 0.01).chorus(0.35).delay("1/8.", 0.35, 0.22).reverb(0.55, 0.22);
    });
    p!(R, "Screech Lead", |b| {
        b.osc('A', "Sync Sweep", 0.6, 3.0, 0.12, 0.7).e("A_WMODE", "Bend +").n("A_WARP", 0.4).filt("HP 12", 400.0, 0.2)
         .env(1, 0.003, 1.0, 1.0, 0.2).env(2, 0.001, 0.8, 0.0, 0.3).m(1, "Env 2", "A Position", -0.4).mono(0.04, true).dist("Hard", 0.45, 0.6).delay("1/8", 0.3, 0.2);
    });
    p!(R, "Digital Choir Pad", |b| {
        b.osc('A', "Choir", 0.0, 5.0, 0.2, 0.6).n("A_RAND", 1.0).osc('B', "Formant Vowels", 0.5, 3.0, 0.15, 0.4).n("B_RAND", 1.0)
         .filt("LP 24", 8000.0, 0.05).env(1, 0.9, 1.0, 1.0, 1.8).lfo(1, "Sine", 0.12, "Free").m(1, "LFO 1", "A Position", 0.5).m(2, "LFO 1", "B Position", -0.5)
         .chorus(0.4).reverb(0.8, 0.4);
    });
    p!(R, "Evolving Spectral Pad", |b| {
        b.osc('A', "Spectral Comb", 0.1, 3.0, 0.25, 0.55).osc('B', "Harmonic Sweep", 0.2, 3.0, 0.2, 0.45)
         .filt("LP 24", 6000.0, 0.1).env(1, 1.5, 1.0, 1.0, 2.5)
         .lfo(1, "Triangle", 0.09, "Free").m(1, "LFO 1", "A Position", 0.9).lfo(2, "Smooth Rnd", 0.2, "Free").m(2, "LFO 2", "B Position", 0.7)
         .chorus(0.4).delay("1/4", 0.5, 0.3).reverb(0.85, 0.45);
    });
    p!(R, "Cinematic Metal Drone", |b| {
        b.osc('A', "Metal FM", 0.3, 5.0, 0.35, 0.5).n("A_OCT", -1.0).osc('B', "Noise Rough", 0.4, 1.0, 0.0, 0.3)
         .filt("LP 24", 1200.0, 0.3).env(1, 2.5, 1.0, 1.0, 3.5).lfo(1, "Smooth Rnd", 0.15, "Free").m(1, "LFO 1", "A Position", 0.8).m(2, "LFO 1", "Cutoff", 0.4)
         .reverb(0.9, 0.5);
    });
    p!(R, "Lo-Fi Keys", |b| {
        b.osc('A', "E-Piano", 0.4, 2.0, 0.05, 0.7).filt("LP 12", 3200.0, 0.1).env(1, 0.003, 1.4, 0.0, 0.4)
         .lfo(1, "Sine", 0.4, "Free").m(1, "LFO 1", "A Pitch", 0.01).dist("Bitcrush", 0.15, 0.25).chorus(0.4).reverb(0.5, 0.2);
    });
    p!(R, "Wavetable Pluck", |b| {
        b.osc('A', "Harmonic Sweep", 0.7, 2.0, 0.1, 0.7).filt("LP 24", 1800.0, 0.15).n("F_ENV", 0.5)
         .env(2, 0.001, 0.3, 0.0, 0.2).m(1, "Env 2", "A Position", -0.6).env(1, 0.002, 0.45, 0.0, 0.2).chorus(0.25).delay("1/8.", 0.4, 0.25).reverb(0.5, 0.2);
    });
    p!(R, "FM Glass Arp", |b| {
        b.osc('A', "Hollow Glass", 0.3, 1.0, 0.0, 0.6).osc('B', "FM Bell", 0.5, 1.0, 0.0, 0.5).e("A_WMODE", "FM from other").n("A_WARP", 0.3)
         .filt("LP 24", 7000.0, 0.1).env(1, 0.002, 0.4, 0.0, 0.3).env(2, 0.001, 0.25, 0.0, 0.1).m(1, "Env 2", "A Warp", 0.5)
         .chorus(0.3).delay("1/8.", 0.45, 0.3).reverb(0.6, 0.3);
    });
    p!(R, "Brass Wavetable Stab", |b| {
        b.osc('A', "Analog Saw>Sq", 0.2, 3.0, 0.18, 0.65).osc('B', "Bowed Strings", 0.6, 1.0, 0.0, 0.35)
         .filt("LP 24", 900.0, 0.2).n("F_ENV", 0.55).env(2, 0.05, 0.3, 0.2, 0.15).env(1, 0.02, 0.4, 0.2, 0.15).dist("Soft", 0.2, 0.4).reverb(0.4, 0.15);
    });
    p!(R, "808 Glide Sub", |b| {
        b.osc('A', "Basic Shapes", 0.0, 1.0, 0.0, 0.9).n("A_OCT", -1.0).on("SUB_ON", true).n("SUB_LEVEL", 0.45)
         .env(1, 0.001, 1.8, 0.0, 0.3).env(2, 0.001, 0.05, 0.0, 0.05).m(1, "Env 2", "A Pitch", 0.35)
         .nofilt().mono(0.12, false).dist("Soft", 0.3, 0.7);
    });
    p!(R, "Hoover Riser", |b| {
        b.osc('A', "Pulse Width", 0.2, 7.0, 0.6, 0.55).n("A_RAND", 1.0).saw('B', 5.0, 0.5, 0.4).n("B_RAND", 1.0)
         .filt("LP 24", 600.0, 0.3).n("F_ENV", 0.9).env(2, 4.0, 1.0, 1.0, 0.5).env(1, 0.5, 1.0, 1.0, 0.6)
         .m(1, "Env 2", "A Pitch", 0.4).m(2, "Env 2", "B Pitch", 0.4).dist("Soft", 0.35, 0.5).chorus(0.4).reverb(0.6, 0.3);
    });
    p!(R, "Noise Sweep FX", |b| {
        b.on("A_ON", false).on("NOISE_ON", true).n("NOISE_LEVEL", 0.5).e("NOISE_TYPE", "Pink")
         .filt("BP", 250.0, 0.55).n("F_ENV", 0.95).env(2, 3.0, 1.0, 1.0, 0.8).env(1, 1.0, 1.0, 1.0, 1.0).reverb(0.8, 0.4);
    });
    p!(R, "Reverse Vocal Swell", |b| {
        b.osc('A', "Choir", 0.5, 3.0, 0.15, 0.7).filt("LP 24", 500.0, 0.2).n("F_ENV", 0.8).env(2, 2.5, 1.0, 1.0, 0.4)
         .env(1, 2.2, 0.3, 1.0, 0.5).lfo(1, "Sine", 0.3, "Free").m(1, "LFO 1", "A Position", 0.4).chorus(0.3).delay("1/4", 0.45, 0.25).reverb(0.8, 0.4);
    });
    p!(R, "Ring Mod Robot", |b| {
        b.osc('A', "Metal FM", 0.3, 1.0, 0.0, 0.6).osc('B', "Basic Shapes", 0.0, 1.0, 0.0, 0.5).e("A_WMODE", "Ring other").n("A_WARP", 0.6).n("B_FINE", 37.0)
         .filt("LP 24", 4500.0, 0.2).env(1, 0.003, 0.7, 0.3, 0.2).mono(0.0, false).delay("1/16.", 0.4, 0.25).reverb(0.4, 0.15);
    });
    p!(R, "Tech House Stab", |b| {
        b.osc('A', "Formant Vowels", 0.2, 3.0, 0.18, 0.6).saw('B', 3.0, 0.2, 0.4)
         .filt("LP 24", 1500.0, 0.25).n("F_ENV", 0.5).env(2, 0.001, 0.2, 0.0, 0.15).env(1, 0.002, 0.3, 0.0, 0.12).m(1, "Env 2", "A Position", 0.4)
         .chorus(0.25).delay("1/8.", 0.35, 0.2).reverb(0.4, 0.15);
    });
    p!(R, "Dark Techno Pluck", |b| {
        b.osc('A', "Spectral Comb", 0.4, 2.0, 0.12, 0.65).filt("LP 24", 700.0, 0.35).n("F_ENV", 0.65)
         .env(2, 0.001, 0.2, 0.0, 0.15).env(1, 0.002, 0.3, 0.0, 0.15).on("SUB_ON", true).n("SUB_LEVEL", 0.25)
         .delay("1/8.", 0.5, 0.3).reverb(0.7, 0.3);
    });
}
