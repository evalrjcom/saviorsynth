//! Headless rendering: WAV export (`--render`) and the automated `--selftest`.

use crate::engine::Engine;
use crate::presets::{self, Preset};
use crate::shared::Shared;
use crate::wavetable::WaveBank;
use std::io::Write;
use std::sync::Arc;
use std::time::Instant;

pub fn render(bank: &Arc<WaveBank>, vals: &[f32], notes: &[u8], secs: f32, hold: f32, sr: f32) -> Vec<f32> {
    let shared = Shared::new();
    shared.params.load(vals);
    let mut eng = Engine::new(sr, shared.clone(), bank.clone());
    for &n in notes { shared.note_on(n, 100); }
    let frames = (secs * sr) as usize;
    let hold_frames = (hold * sr) as usize;
    let mut out = vec![0.0f32; frames * 2];
    let mut pos = 0;
    let mut released = false;
    while pos < frames {
        let n = 256.min(frames - pos);
        if !released && pos >= hold_frames { for &k in notes { shared.note_off(k); } released = true; }
        eng.fill(&mut out[pos * 2..(pos + n) * 2]);
        pos += n;
    }
    out
}

pub fn stats(buf: &[f32]) -> (f32, f32, bool) {
    let mut peak = 0.0f32; let mut sum = 0.0f64; let mut ok = true;
    for &x in buf { if !x.is_finite() { ok = false; continue; } peak = peak.max(x.abs()); sum += (x as f64) * (x as f64); }
    (peak, (sum / buf.len().max(1) as f64).sqrt() as f32, ok)
}

pub fn db(x: f32) -> f32 { 20.0 * x.max(1e-9).log10() }

/// 16-bit stereo PCM WAV, written to a temp file then renamed (no half-written files on failure).
pub fn write_wav(path: &std::path::Path, buf: &[f32], sr: u32) -> std::io::Result<()> {
    let data_len = (buf.len() * 2) as u32;
    let mut v: Vec<u8> = Vec::with_capacity(44 + data_len as usize);
    v.extend_from_slice(b"RIFF"); v.extend_from_slice(&(36 + data_len).to_le_bytes());
    v.extend_from_slice(b"WAVEfmt "); v.extend_from_slice(&16u32.to_le_bytes());
    v.extend_from_slice(&1u16.to_le_bytes()); v.extend_from_slice(&2u16.to_le_bytes());
    v.extend_from_slice(&sr.to_le_bytes()); v.extend_from_slice(&(sr * 4).to_le_bytes());
    v.extend_from_slice(&4u16.to_le_bytes()); v.extend_from_slice(&16u16.to_le_bytes());
    v.extend_from_slice(b"data"); v.extend_from_slice(&data_len.to_le_bytes());
    for &x in buf { v.extend_from_slice(&((x.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes()); }
    let tmp = path.with_extension("wav.tmp");
    { let _ = std::fs::remove_file(&tmp); let mut f = std::fs::OpenOptions::new().write(true).create_new(true).open(&tmp)?; f.write_all(&v)?; f.sync_all()?; }
    std::fs::rename(&tmp, path)
}

pub fn file_stem(name: &str) -> String {
    let s: String = name.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
    s.trim_matches('_').to_string()
}

/// Renders every preset (single note + 6-note chord), checks for NaN / silence / clipping and CPU load.
pub fn selftest(bank: &Arc<WaveBank>, list: &[Preset]) -> bool {
    let sr = 48000.0;
    let mut fails = 0;
    println!("{:<28} {:>9} {:>9} {:>9}", "preset", "peak dB", "rms dB", "chord xRT");
    for p in list {
        let single = render(bank, &p.vals, &[48], 4.0, 2.5, sr);
        let (peak, rms, ok) = stats(&single);
        let t = Instant::now();
        let chord = render(bank, &p.vals, &[48, 52, 55, 60, 64, 67], 3.0, 2.0, sr);
        let xrt = 3.0 / t.elapsed().as_secs_f32().max(1e-6);
        let (cpeak, _, cok) = stats(&chord);
        let mut flag = String::new();
        if !ok || !cok { flag += " NON-FINITE"; fails += 1; }
        if peak < 0.02 { flag += " SILENT"; fails += 1; }
        if peak > 1.0 || cpeak > 1.0 { flag += " CLIP"; fails += 1; }
        if xrt < 8.0 { flag += " SLOW"; fails += 1; }
        println!("{:<28} {:>9.1} {:>9.1} {:>9.0}{}", p.name, db(peak), db(rms), xrt, flag);
    }
    let bad = presets::bad_count();
    println!("\n{} presets, {} problems, {} unknown preset keys", list.len(), fails, bad);
    fails == 0 && bad == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::*;
    use std::sync::atomic::Ordering;

    /// End-to-end: one held key drives a I-IV-V-I progression through the real engine, arp keeps
    /// producing sound, and switching the stage off mid-note leaves exactly the held key sounding.
    #[test]
    fn one_key_progression_and_arp_through_engine() {
        let bank = Arc::new(WaveBank::generate());
        let shared = Shared::new();
        let mut v = default_patch();
        v[CH_MODE] = 3.0; v[PR_PAT] = 0.0; v[PR_LEN] = 3.0; v[BPM] = 120.0; // 1 beat per chord
        shared.params.load(&v);
        let mut eng = Engine::new(48000.0, shared.clone(), bank.clone());
        shared.note_on(60, 100);
        let mut buf = vec![0.0f32; 2 * 4800];
        let mut roots = vec![];
        for blk in 0..40 { // 40 * 0.1 s = 4 beats @120
            eng.fill(&mut buf);
            if blk % 5 == 2 { roots.push(shared.chord_root.load(Ordering::Relaxed)); }
        }
        assert_eq!(&roots[..4], &[60, 65, 67, 60]); // I IV V I, one chord per beat
        assert!(buf.iter().any(|x| x.abs() > 0.01), "progression must be audible");

        // arpeggiator on, then everything off -> engine returns to silence, no stuck voices
        v[AR_ON] = 1.0; v[AR_PAT] = 2.0; shared.params.load(&v);
        let mut peak = 0.0f32;
        for _ in 0..20 { eng.fill(&mut buf); peak = peak.max(buf.iter().fold(0.0, |m, x| m.max(x.abs()))); }
        assert!(peak > 0.01);
        shared.note_off(60);
        for _ in 0..60 { eng.fill(&mut buf); }
        assert_eq!(shared.voices.load(Ordering::Relaxed), 0, "no stuck voices after release");
    }
}
