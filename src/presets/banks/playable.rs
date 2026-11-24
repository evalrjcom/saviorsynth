//! Playable presets for the performance stage: one-key chords, degree chords (I / II / IV ...),
//! one-key chord progressions and arpeggiators with inversion cycling.
//! Play a single key: it becomes the tonic (progressions / degrees) or the chord root (One-Key).

use super::super::{Preset, B};

pub fn add(out: &mut Vec<Preset>) {
    macro_rules! p {
        ($cat:expr, $name:expr, |$b:ident| $body:block) => {{ let mut $b = B::new($name); $body; out.push($b.done($cat)); }};
    }
    // shared sound recipes
    fn pad(b: &mut B) { b.saw('A', 5.0, 0.22, 0.5).n("A_RAND", 1.0).osc('B', "Pulse Width", 0.5, 3.0, 0.15, 0.4).filt("LP 24", 2600.0, 0.1).env(1, 0.25, 1.0, 1.0, 0.9).chorus(0.5).reverb(0.65, 0.3); }
    fn keys(b: &mut B) { b.osc('A', "E-Piano", 0.4, 1.0, 0.0, 0.7).filt("LP 12", 5000.0, 0.05).env(1, 0.002, 1.6, 0.0, 0.5).chorus(0.35).reverb(0.45, 0.2); }
    fn pluck(b: &mut B) { b.saw('A', 3.0, 0.15, 0.65).osc('B', "Pulse Width", 0.4, 1.0, 0.0, 0.4).filt("LP 24", 1000.0, 0.25).n("F_ENV", 0.65).env(2, 0.001, 0.25, 0.0, 0.2).env(1, 0.002, 0.4, 0.0, 0.25).chorus(0.3).delay("1/8.", 0.35, 0.2).reverb(0.45, 0.2); }
    fn lead(b: &mut B) { b.saw('A', 5.0, 0.3, 0.6).n("A_RAND", 0.9).osc('B', "Pulse Width", 0.5, 3.0, 0.15, 0.4).filt("LP 24", 7000.0, 0.12).env(1, 0.004, 1.0, 1.0, 0.25).chorus(0.3).delay("1/8.", 0.35, 0.2).reverb(0.5, 0.2); }

    // ===================================================================== Chords (one key -> chord)
    const C: &str = "Chords";
    p!(C, "One-Key Major Triad", |b| { pad(&mut b); b.chord("One-Key", "Triad").e("CH_QUAL", "Major"); });
    p!(C, "One-Key Minor Triad", |b| { pad(&mut b); b.chord("One-Key", "Triad").e("CH_QUAL", "Minor"); });
    p!(C, "One-Key Major 7th Keys", |b| { keys(&mut b); b.chord("One-Key", "7th").e("CH_QUAL", "Major"); });
    p!(C, "One-Key Minor 9th Keys", |b| { keys(&mut b); b.chord("One-Key", "9th").e("CH_QUAL", "Minor"); });
    p!(C, "One-Key Sus4 Pad", |b| { pad(&mut b); b.chord("One-Key", "Sus4").e("CH_QUAL", "Major"); });
    p!(C, "One-Key Power Fifth", |b| { b.saw('A', 3.0, 0.15, 0.7).filt("LP 24", 2200.0, 0.2).env(1, 0.004, 1.0, 1.0, 0.15).dist("Hard", 0.35, 0.6).chord("One-Key", "Power"); });
    p!(C, "One-Key Open Voicing Pad", |b| { pad(&mut b); b.chord("One-Key", "7th").e("CH_QUAL", "Minor").on("CH_OPEN", true).on("CH_BASS", true); });
    p!(C, "Degree I  (Scale Chord)", |b| { pad(&mut b); b.chord("Degree", "Triad").e("CH_DEG", "I"); });
    p!(C, "Degree II  (Scale Chord)", |b| { pad(&mut b); b.chord("Degree", "Triad").e("CH_DEG", "II"); });
    p!(C, "Degree IV  (Scale Chord)", |b| { pad(&mut b); b.chord("Degree", "Triad").e("CH_DEG", "IV"); });
    p!(C, "Degree V 7th  (Scale Chord)", |b| { keys(&mut b); b.chord("Degree", "7th").e("CH_DEG", "V"); });
    p!(C, "Degree VI Minor Keys", |b| { keys(&mut b); b.chord("Degree", "7th").e("CH_DEG", "VI"); });
    p!(C, "Dorian IV Chord", |b| { pad(&mut b); b.chord("Degree", "7th").e("CH_SCALE", "Dorian").e("CH_DEG", "IV"); });
    p!(C, "Mixolydian V Chord", |b| { pad(&mut b); b.chord("Degree", "Triad").e("CH_SCALE", "Mixolydian").e("CH_DEG", "V"); });
    p!(C, "1st Inversion Triad", |b| { pad(&mut b); b.chord("One-Key", "Triad").e("CH_INV", "1st"); });
    p!(C, "2nd Inversion Triad", |b| { pad(&mut b); b.chord("One-Key", "Triad").e("CH_INV", "2nd"); });
    p!(C, "3rd Inversion 7th", |b| { keys(&mut b); b.chord("One-Key", "7th").e("CH_INV", "3rd"); });

    // ===================================================================== Progressions (one key -> sequence)
    const P: &str = "Progressions";
    p!(P, "Prog I-IV-V-I Pad", |b| { pad(&mut b); b.prog("I-IV-V-I", "1 bar"); });
    p!(P, "Prog I-V-VI-IV Pop Keys", |b| { keys(&mut b); b.prog("I-V-VI-IV", "1 bar").e("CH_EXT", "Add9"); });
    p!(P, "Prog II-V-I Jazz 7ths", |b| { keys(&mut b); b.prog("II-V-I-I", "1 bar").e("CH_EXT", "7th"); });
    p!(P, "Prog I-VI-IV-V Fifties", |b| { keys(&mut b); b.prog("I-VI-IV-V", "1 bar"); });
    p!(P, "Prog VI-IV-I-V Minor Pop", |b| { pad(&mut b); b.prog("VI-IV-I-V", "1 bar"); });
    p!(P, "Prog I-II-IV-I Climb", |b| { pad(&mut b); b.prog("I-II-IV-I", "1 bar"); });
    p!(P, "Prog I-IV-II-V Turnaround", |b| { keys(&mut b); b.prog("I-IV-II-V", "1 bar").e("CH_EXT", "7th"); });
    p!(P, "Prog I-II-IV-V Anthem", |b| { b.saw('A', 5.0, 0.3, 0.6).n("A_RAND", 1.0).filt("LP 24", 5000.0, 0.1).env(1, 0.01, 1.0, 1.0, 0.4).chorus(0.4).reverb(0.6, 0.25); b.prog("I-II-IV-V", "1 bar"); });
    p!(P, "Prog Andalusian Minor", |b| { pad(&mut b); b.prog("I-VII-VI-V", "1 bar").e("CH_SCALE", "Minor"); });
    p!(P, "Prog Custom (I-II-IV-V editable)", |b| { pad(&mut b); b.prog("Custom", "1 bar").e("PR_S1", "I").e("PR_S2", "II").e("PR_S3", "IV").e("PR_S4", "V"); });
    p!(P, "Prog Half-Bar Stabs", |b| { pluck(&mut b); b.prog("I-V-VI-IV", "1/2 bar").e("CH_EXT", "Triad"); });
    p!(P, "Prog Two-Bar Slow Pad", |b| { pad(&mut b); b.prog("I-V-VI-IV", "2 bars").e("CH_EXT", "Sus2"); });
    p!(P, "Prog Dorian Groove", |b| { keys(&mut b); b.prog("I-IV-II-V", "1 bar").e("CH_SCALE", "Dorian").e("CH_EXT", "7th"); });
    p!(P, "Prog Trance Minor Pluck+Arp", |b| { pluck(&mut b); b.prog("VI-IV-I-V", "1 bar").e("CH_SCALE", "Major").arp("Up", "1/16", "2 Oct", 0.55); });

    // ===================================================================== Arpeggios
    const A: &str = "Arps";
    p!(A, "Arp Up Pluck (held keys)", |b| { pluck(&mut b); b.arp("Up", "1/16", "2 Oct", 0.55); });
    p!(A, "Arp Down Bell", |b| { b.osc('A', "FM Bell", 0.4, 1.0, 0.0, 0.65).env(1, 0.002, 0.6, 0.0, 0.4).filt("LP 12", 8000.0, 0.05).delay("1/8.", 0.45, 0.3).reverb(0.6, 0.3); b.arp("Down", "1/8", "2 Oct", 0.5); });
    p!(A, "Arp Up-Down Saw", |b| { lead(&mut b); b.arp("Up-Down", "1/16", "2 Oct", 0.6); });
    p!(A, "Arp Converge Glass", |b| { b.osc('A', "Hollow Glass", 0.3, 2.0, 0.1, 0.65).env(1, 0.002, 0.5, 0.0, 0.4).filt("LP 24", 7000.0, 0.1).delay("1/8.", 0.4, 0.25).reverb(0.7, 0.35); b.arp("Converge", "1/16", "2 Oct", 0.5); });
    p!(A, "Arp Diverge Pluck", |b| { pluck(&mut b); b.arp("Diverge", "1/16", "2 Oct", 0.5); });
    p!(A, "Arp Thumb Up Funk", |b| { pluck(&mut b); b.arp("Thumb Up", "1/16", "1 Oct", 0.4); });
    p!(A, "Arp Pinky Up Rise", |b| { pluck(&mut b); b.arp("Pinky Up", "1/16", "2 Oct", 0.45); });
    p!(A, "Arp Stairs Trance", |b| { lead(&mut b); b.arp("Stairs", "1/16", "2 Oct", 0.55); });
    p!(A, "Arp Alberti Keys", |b| { keys(&mut b); b.arp("Alberti", "1/8", "1 Oct", 0.6); });
    p!(A, "Arp Random Sparkle", |b| { b.osc('A', "FM Bell", 0.5, 1.0, 0.0, 0.6).env(1, 0.002, 0.4, 0.0, 0.3).filt("LP 12", 9000.0, 0.05).delay("1/8.", 0.5, 0.35).reverb(0.8, 0.4); b.arp("Random", "1/16", "3 Oct", 0.4); });
    p!(A, "Arp Chord Pulse Gate", |b| { pad(&mut b); b.env(1, 0.002, 1.0, 1.0, 0.1).arp("Chord Pulse", "1/8", "1 Oct", 0.4); });
    p!(A, "Arp Triplet Roll", |b| { pluck(&mut b); b.arp("Up-Down", "1/8T", "2 Oct", 0.6); });
    p!(A, "Arp Swing 16ths", |b| { pluck(&mut b); b.arp("Up", "1/16", "1 Oct", 0.5).n("AR_SWING", 0.35); });
    p!(A, "Arp Latch Ambient", |b| { b.osc('A', "Hollow Glass", 0.4, 3.0, 0.15, 0.6).env(1, 0.01, 1.0, 1.0, 1.0).filt("LP 24", 5000.0, 0.1).chorus(0.4).delay("1/4", 0.5, 0.3).reverb(0.85, 0.45); b.arp("Up-Down", "1/8", "3 Oct", 0.9).on("AR_LATCH", true); });

    // ---- arps over generated chords, with inversion cycling ("different inverse chord shapes")
    p!(A, "Inversion Arp Up (Cycle Up)", |b| { pluck(&mut b); b.chord("One-Key", "7th").e("CH_QUAL", "Minor").arp("Up", "1/16", "1 Oct", 0.55).e("AR_INVM", "Cycle Up"); });
    p!(A, "Inversion Arp Down (Cycle Down)", |b| { pluck(&mut b); b.chord("One-Key", "Triad").e("CH_QUAL", "Major").arp("Down", "1/16", "2 Oct", 0.55).e("AR_INVM", "Cycle Down"); });
    p!(A, "Inversion Arp Ping-Pong", |b| { lead(&mut b); b.chord("One-Key", "9th").e("CH_QUAL", "Minor").arp("Up", "1/16", "1 Oct", 0.5).e("AR_INVM", "Ping-Pong"); });
    p!(A, "Open Voicing Arp", |b| { keys(&mut b); b.chord("One-Key", "7th").e("CH_QUAL", "Major").on("CH_OPEN", true).on("CH_BASS", true).arp("Up", "1/8", "2 Oct", 0.7); });
    p!(A, "Progression + Arp I-V-VI-IV", |b| { pluck(&mut b); b.prog("I-V-VI-IV", "1 bar").arp("Up-Down", "1/16", "2 Oct", 0.55).e("AR_INVM", "Cycle Up"); });
    p!(A, "Progression + Arp II-V-I Jazz", |b| { keys(&mut b); b.prog("II-V-I-I", "1 bar").e("CH_EXT", "7th").arp("Alberti", "1/8", "2 Oct", 0.7); });
    p!(A, "Progression + Arp Epic Trance", |b| { lead(&mut b); b.prog("VI-IV-I-V", "1 bar").arp("Stairs", "1/16", "2 Oct", 0.6).e("AR_INVM", "Ping-Pong"); });
    p!(A, "Progression + Chord Pulse I-II-IV-I", |b| { pad(&mut b); b.env(1, 0.002, 1.0, 1.0, 0.1).prog("I-II-IV-I", "1 bar").arp("Chord Pulse", "1/8", "1 Oct", 0.4); });
}
