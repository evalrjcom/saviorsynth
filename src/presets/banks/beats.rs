//! "Beats" demos: a sound + one-key progression/arp + the built-in drum machine, all set up in
//! one click. Press a single key, hear a groove. Loading a Beats preset replaces the session
//! (BPM, drum pattern); other categories leave the drum machine and tempo untouched.

use super::super::{Preset, B};

pub fn add(out: &mut Vec<Preset>) {
    macro_rules! p {
        ($name:expr, |$b:ident| $body:block) => {{ let mut $b = B::new($name); $body; out.push($b.done("Beats")); }};
    }
    p!("Beat House Chords", |b| {
        b.saw('A', 5.0, 0.2, 0.5).osc('B', "Pulse Width", 0.5, 3.0, 0.15, 0.4).filt("LP 24", 3000.0, 0.1).env(1, 0.01, 0.5, 0.0, 0.25).chorus(0.4).reverb(0.4, 0.2);
        b.prog("I-V-VI-IV", "1 bar").e("CH_EXT", "7th").drums("House", 124.0, 0.0);
    });
    p!("Beat Techno Arp", |b| {
        b.saw('A', 3.0, 0.15, 0.65).filt("LP 24", 1100.0, 0.3).n("F_ENV", 0.6).env(2, 0.001, 0.2, 0.0, 0.2).env(1, 0.002, 0.25, 0.0, 0.2).delay("1/8.", 0.4, 0.25);
        b.arp("Up", "1/16", "2 Oct", 0.5).drums("Techno", 130.0, 0.0);
    });
    p!("Beat Boom Bap Keys", |b| {
        b.osc('A', "E-Piano", 0.4, 1.0, 0.0, 0.7).filt("LP 12", 3500.0, 0.05).env(1, 0.002, 1.2, 0.0, 0.4).chorus(0.3).reverb(0.35, 0.3);
        b.prog("II-V-I-I", "1 bar").e("CH_EXT", "7th").drums("Hip-Hop Boom Bap", 88.0, 0.3).n("HUM_TIME", 0.4).n("HUM_VEL", 0.4);
    });
    p!("Beat Trap Pluck Arp", |b| {
        b.saw('A', 3.0, 0.15, 0.6).filt("LP 24", 1400.0, 0.2).n("F_ENV", 0.5).env(2, 0.001, 0.2, 0.0, 0.2).env(1, 0.002, 0.3, 0.0, 0.2).reverb(0.4, 0.2);
        b.arp("Up-Down", "1/16", "1 Oct", 0.4).drums("Trap", 140.0, 0.0);
    });
    p!("Beat Disco Stabs", |b| {
        b.saw('A', 5.0, 0.2, 0.55).filt("LP 24", 4000.0, 0.1).env(1, 0.003, 0.25, 0.0, 0.2).chorus(0.4).reverb(0.3, 0.2);
        b.prog("I-VI-IV-V", "1/2 bar").e("CH_EXT", "7th").drums("Disco", 118.0, 0.0);
    });
    p!("Beat Funk Groove", |b| {
        b.osc('A', "E-Piano", 0.5, 1.0, 0.0, 0.7).filt("LP 12", 4500.0, 0.1).env(1, 0.002, 0.5, 0.0, 0.2);
        b.prog("I-IV-II-V", "1 bar").e("CH_EXT", "Dom 7").drums("Funk", 104.0, 0.2).n("HUM_TIME", 0.3).n("HUM_VEL", 0.5);
    });
    p!("Beat Lo-Fi Chill", |b| {
        b.osc('A', "E-Piano", 0.35, 1.0, 0.0, 0.7).filt("LP 12", 2200.0, 0.05).env(1, 0.005, 1.5, 0.0, 0.5).chorus(0.5).reverb(0.5, 0.4);
        b.prog("I-VI-IV-V", "1 bar").e("CH_EXT", "9th").drums("Lo-Fi", 78.0, 0.4).n("HUM_TIME", 0.6).n("HUM_VEL", 0.5).n("HUM_GATE", 0.3);
    });
    p!("Beat 80s Pop Anthem", |b| {
        b.saw('A', 5.0, 0.3, 0.55).osc('B', "Pulse Width", 0.5, 3.0, 0.2, 0.4).filt("LP 24", 5000.0, 0.1).env(1, 0.02, 1.0, 1.0, 0.5).chorus(0.5).reverb(0.5, 0.25);
        b.prog("I-V-VI-IV", "1 bar").drums("80s Pop", 112.0, 0.0);
    });
    p!("Beat Drum & Bass Arp", |b| {
        b.saw('A', 3.0, 0.2, 0.6).filt("LP 24", 1800.0, 0.2).n("F_ENV", 0.5).env(2, 0.001, 0.15, 0.0, 0.1).env(1, 0.002, 0.2, 0.0, 0.1).delay("1/8.", 0.3, 0.2);
        b.arp("Up", "1/16", "2 Oct", 0.4).drums("Drum & Bass", 172.0, 0.0);
    });
    p!("Beat Reggaeton Plucks", |b| {
        b.saw('A', 3.0, 0.15, 0.6).filt("LP 24", 1500.0, 0.2).n("F_ENV", 0.5).env(2, 0.001, 0.25, 0.0, 0.2).env(1, 0.002, 0.3, 0.0, 0.2).reverb(0.35, 0.2);
        b.prog("VI-IV-I-V", "1 bar").e("CH_EXT", "Triad").drums("Reggaeton", 96.0, 0.0);
    });
    p!("Beat Afro Chords", |b| {
        b.osc('A', "E-Piano", 0.45, 1.0, 0.0, 0.7).filt("LP 12", 4000.0, 0.05).env(1, 0.002, 0.8, 0.0, 0.3).chorus(0.3).reverb(0.4, 0.2);
        b.prog("I-VI-IV-V", "1 bar").e("CH_EXT", "7th").drums("Afrobeat", 108.0, 0.15).n("HUM_TIME", 0.3);
    });
    p!("Beat Rock Power", |b| {
        b.saw('A', 3.0, 0.15, 0.7).filt("LP 24", 2400.0, 0.2).env(1, 0.004, 1.0, 1.0, 0.15).dist("Hard", 0.35, 0.6);
        b.chord("One-Key", "Power").drums("Rock", 120.0, 0.0);
    });
}
