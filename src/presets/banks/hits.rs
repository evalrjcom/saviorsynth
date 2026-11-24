//! "Hit Sounds": original patches that approximate the signature *synth timbre* of famous chart-topping
//! records. Names say "-style" on purpose: only the sound character is imitated. No melodies, lyrics,
//! samples or recordings are reproduced, and nothing here is endorsed by or affiliated with the artists.

use super::super::{Preset, B};

pub fn add(out: &mut Vec<Preset>) {
    macro_rules! p {
        ($name:expr, |$b:ident| $body:block) => {{ let mut $b = B::new($name); $body; out.push($b.done("Hit Sounds")); }};
    }
    p!("Jump-style Brass", |b| {
        b.saw('A', 5.0, 0.18, 0.62).saw('B', 3.0, 0.14, 0.5).n("B_FINE", 7.0).filt("LP 24", 1000.0, 0.2).n("F_ENV", 0.55)
         .env(2, 0.03, 0.5, 0.55, 0.25).env(1, 0.02, 1.0, 1.0, 0.2).dist("Soft", 0.15, 0.4).chorus(0.3).reverb(0.45, 0.2);
    });
    p!("Billie Jean-style Bass", |b| {
        b.saw('A', 1.0, 0.0, 0.7).osc('B', "Pulse Width", 0.5, 1.0, 0.0, 0.55).n("B_OCT", -1.0).filt("LP 24", 420.0, 0.3).n("F_ENV", 0.5)
         .env(2, 0.001, 0.2, 0.2, 0.1).env(1, 0.002, 0.35, 0.6, 0.08).mono(0.0, false).dist("Tube", 0.15, 0.4);
    });
    p!("Sweet Dreams-style Pulse Bass", |b| {
        b.osc('A', "Pulse Width", 0.5, 1.0, 0.0, 0.7).n("A_OCT", -1.0).saw('B', 1.0, 0.0, 0.4).n("B_OCT", -1.0)
         .filt("LP 24", 500.0, 0.55).n("F_ENV", 0.5).env(2, 0.001, 0.15, 0.0, 0.1).env(1, 0.001, 0.2, 0.0, 0.06).mono(0.0, false).dist("Soft", 0.2, 0.4);
    });
    p!("Take On Me-style Synth Lead", |b| {
        b.saw('A', 3.0, 0.16, 0.65).osc('B', "Pulse Width", 0.5, 3.0, 0.1, 0.4).filt("LP 24", 5200.0, 0.15)
         .env(1, 0.005, 1.0, 1.0, 0.2).chorus(0.7).delay("1/8.", 0.3, 0.15).reverb(0.4, 0.15);
    });
    p!("I Feel Love-style Sequence", |b| {
        b.saw('A', 1.0, 0.0, 0.7).filt("LP 24", 600.0, 0.55).n("F_ENV", 0.55).env(2, 0.001, 0.16, 0.0, 0.08).env(1, 0.001, 0.2, 0.0, 0.06)
         .on("SUB_ON", true).n("SUB_LEVEL", 0.25).delay("1/8.", 0.4, 0.22).reverb(0.4, 0.15);
    });
    p!("Final Countdown-style Brass", |b| {
        b.saw('A', 5.0, 0.2, 0.6).osc('B', "Analog Saw>Sq", 0.5, 3.0, 0.14, 0.45).filt("LP 24", 1500.0, 0.2).n("F_ENV", 0.4)
         .env(2, 0.04, 0.6, 0.65, 0.3).env(1, 0.03, 1.0, 1.0, 0.3).chorus(0.4).reverb(0.6, 0.3);
    });
    p!("Tainted Love-style Pulse Bass", |b| {
        b.osc('A', "Pulse Width", 0.5, 1.0, 0.0, 0.75).n("A_OCT", -1.0).filt("LP 12", 900.0, 0.2).n("F_ENV", 0.35)
         .env(2, 0.001, 0.12, 0.0, 0.1).env(1, 0.001, 0.18, 0.0, 0.05).mono(0.0, false).dist("Soft", 0.2, 0.4);
    });
    p!("Dont You Want Me-style Hook", |b| {
        b.saw('A', 3.0, 0.14, 0.6).osc('B', "Pulse Width", 0.5, 3.0, 0.1, 0.4).filt("LP 24", 3500.0, 0.2).n("F_ENV", 0.2)
         .env(2, 0.01, 0.5, 0.5, 0.2).env(1, 0.01, 1.0, 1.0, 0.25).chorus(0.5).reverb(0.5, 0.2);
    });
    p!("Never Gonna-style DX Bass", |b| {
        b.osc('A', "FM Bell", 0.1, 1.0, 0.0, 0.75).n("A_OCT", -1.0).osc('B', "Basic Shapes", 0.0, 1.0, 0.0, 0.4).n("B_OCT", -1.0)
         .filt("LP 24", 1400.0, 0.1).n("F_ENV", 0.3).env(2, 0.001, 0.15, 0.0, 0.1).env(1, 0.001, 0.35, 0.5, 0.08).mono(0.0, false);
    });
    p!("Rule the World-style Pluck", |b| {
        b.osc('A', "Analog Saw>Sq", 0.3, 2.0, 0.1, 0.6).osc('B', "FM Bell", 0.2, 1.0, 0.0, 0.3).filt("LP 24", 1100.0, 0.2).n("F_ENV", 0.55)
         .env(2, 0.001, 0.3, 0.0, 0.2).env(1, 0.002, 0.5, 0.0, 0.3).chorus(0.35).delay("1/8.", 0.4, 0.22).reverb(0.6, 0.3);
    });
    p!("Reflex-style Synth Stab", |b| {
        b.saw('A', 3.0, 0.15, 0.6).osc('B', "Pulse Width", 0.4, 3.0, 0.1, 0.45).filt("LP 24", 2500.0, 0.2).n("F_ENV", 0.4)
         .env(2, 0.001, 0.2, 0.1, 0.15).env(1, 0.002, 0.3, 0.0, 0.15).chorus(0.3).delay("1/8", 0.3, 0.15).reverb(0.4, 0.15);
    });
    p!("Superstition-style Clav", |b| {
        b.osc('A', "Pulse Width", 0.12, 1.0, 0.0, 0.7).osc('B', "Pulse Width", 0.55, 1.0, 0.0, 0.4).filt("LP 12", 1000.0, 0.25).n("F_ENV", 0.6).n("F_KEY", 0.5)
         .env(2, 0.001, 0.12, 0.0, 0.08).env(1, 0.001, 0.2, 0.0, 0.06).dist("Soft", 0.15, 0.3);
    });
    p!("Cars-style Saw Lead", |b| {
        b.saw('A', 2.0, 0.12, 0.7).osc('B', "Pulse Width", 0.5, 2.0, 0.1, 0.45).filt("LP 12", 3800.0, 0.2).env(1, 0.01, 1.0, 1.0, 0.2)
         .mono(0.04, true).chorus(0.5).dist("Soft", 0.2, 0.4).reverb(0.4, 0.15);
    });
    p!("Heart of Glass-style Disco Synth", |b| {
        b.saw('A', 3.0, 0.14, 0.6).osc('B', "Pulse Width", 0.4, 1.0, 0.0, 0.4).filt("LP 24", 2000.0, 0.3).n("F_ENV", 0.35)
         .env(2, 0.001, 0.2, 0.1, 0.15).env(1, 0.002, 0.3, 0.2, 0.15).chorus(0.4).delay("1/8.", 0.3, 0.15).reverb(0.4, 0.15);
    });
    p!("Funkytown-style Synth Hook", |b| {
        b.osc('A', "Pulse Width", 0.5, 2.0, 0.08, 0.65).saw('B', 1.0, 0.0, 0.4).filt("LP 24", 2800.0, 0.35).n("F_ENV", 0.4)
         .env(2, 0.001, 0.25, 0.1, 0.1).env(1, 0.002, 0.3, 0.3, 0.1).dist("Soft", 0.15, 0.3).chorus(0.3);
    });
    p!("Relax-style Pulse Sequence", |b| {
        b.osc('A', "Pulse Width", 0.3, 1.0, 0.0, 0.7).saw('B', 1.0, 0.0, 0.4).n("B_OCT", -1.0).filt("LP 24", 650.0, 0.5).n("F_ENV", 0.55)
         .env(2, 0.001, 0.18, 0.0, 0.08).env(1, 0.001, 0.2, 0.0, 0.05).mono(0.0, false).dist("Hard", 0.3, 0.5).delay("1/8.", 0.4, 0.2);
    });
    p!("Blinding Lights-style Synth", |b| {
        b.saw('A', 5.0, 0.2, 0.6).osc('B', "Pulse Width", 0.5, 3.0, 0.12, 0.4).filt("LP 24", 4200.0, 0.2).n("F_ENV", 0.2)
         .env(2, 0.001, 0.3, 0.3, 0.2).env(1, 0.003, 0.5, 0.7, 0.2).chorus(0.6).delay("1/8.", 0.35, 0.2).reverb(0.5, 0.2);
    });
    p!("Take My Breath Away-style Pad", |b| {
        b.saw('A', 5.0, 0.2, 0.5).n("A_RAND", 1.0).osc('B', "Hollow Glass", 0.4, 3.0, 0.15, 0.4).filt("LP 24", 3000.0, 0.1)
         .env(1, 0.7, 1.0, 1.0, 1.5).chorus(0.6).delay("1/4", 0.5, 0.25).reverb(0.85, 0.5);
    });
    p!("Sussudio-style Brass Riff", |b| {
        b.saw('A', 3.0, 0.14, 0.6).saw('B', 3.0, 0.12, 0.5).n("B_FINE", 6.0).filt("LP 24", 1100.0, 0.2).n("F_ENV", 0.5)
         .env(2, 0.02, 0.4, 0.5, 0.2).env(1, 0.01, 0.8, 0.9, 0.15).chorus(0.3).reverb(0.4, 0.15);
    });
    p!("Chariots of Fire-style Theme Lead", |b| {
        b.osc('A', "Analog Saw>Sq", 0.2, 3.0, 0.1, 0.6).osc('B', "Basic Shapes", 0.0, 1.0, 0.0, 0.4).filt("LP 24", 2600.0, 0.15).n("F_ENV", 0.3)
         .env(2, 0.08, 0.6, 0.6, 0.4).env(1, 0.06, 1.0, 1.0, 0.4).vibrato(1, 1, 0.01).chorus(0.4).delay("1/4", 0.35, 0.2).reverb(0.7, 0.3);
    });
}
