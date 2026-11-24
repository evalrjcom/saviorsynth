//! saviorsynth — wavetable synthesizer (Rust + FLTK).
//! All naming/version strings come from Cargo.toml so a version bump is a one-line change.

mod audio;
mod drums;
mod engine;
mod fx;
mod looper;
mod midi;
mod offline;
mod params;
mod perform;
mod presets;
mod shared;
mod ui;
mod wavetable;

use std::sync::Arc;

pub const APP_ID: &str = env!("CARGO_PKG_NAME");
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

fn usage() {
    println!("{APP_ID} {APP_VERSION} - wavetable synthesizer (Rust + FLTK)\n");
    println!("usage: {APP_ID} [options]\n");
    println!("  (no options)                     start the synthesizer GUI");
    println!("  --selftest                       render every preset headlessly and verify (NaN/silence/clipping/CPU)");
    println!("  --render <preset|all> <out>      render preset(s) to WAV (out = file, or directory for 'all')");
    println!("        [--note N] [--seconds S]   MIDI note (default 48) and length (default 4 s)");
    println!("  --list                           list factory presets");
    println!("  --version, --help");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |name: &str| args.iter().any(|a| a == name);
    let value = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    if flag("--help") || flag("-h") { usage(); return; }
    if flag("--version") { println!("{APP_ID} {APP_VERSION}"); return; }
    let list = presets::factory();
    if flag("--list") { for p in &list { println!("{:<10} {}", p.cat, p.name); } return; }

    eprintln!("generating wavetables...");
    let bank = Arc::new(wavetable::WaveBank::generate());

    if flag("--selftest") {
        std::process::exit(if offline::selftest(&bank, &list) { 0 } else { 1 });
    }
    if let Some(i) = args.iter().position(|a| a == "--render") {
        let (Some(which), Some(out)) = (args.get(i + 1), args.get(i + 2)) else { usage(); std::process::exit(2) };
        let note = value("--note").and_then(|s| s.parse::<u8>().ok()).unwrap_or(48).min(127);
        let secs = value("--seconds").and_then(|s| s.parse::<f32>().ok()).unwrap_or(4.0).clamp(0.1, 60.0);
        let sel: Vec<&presets::Preset> = if which == "all" { list.iter().collect() }
            else { list.iter().filter(|p| p.name.to_lowercase().contains(&which.to_lowercase())).take(1).collect() };
        if sel.is_empty() { eprintln!("no preset matches '{which}' (see --list)"); std::process::exit(2); }
        for p in sel {
            let buf = offline::render(&bank, &p.vals, &[note], secs, secs * 0.65, 48000.0);
            let path = if which == "all" {
                let dir = std::path::Path::new(out);
                if let Err(e) = std::fs::create_dir_all(dir) { eprintln!("cannot create {out}: {e}"); std::process::exit(1); }
                dir.join(format!("{}.wav", offline::file_stem(&p.name)))
            } else { std::path::PathBuf::from(out) };
            match offline::write_wav(&path, &buf, 48000) {
                Ok(()) => println!("wrote {}", path.display()),
                Err(e) => { eprintln!("write failed: {e}"); std::process::exit(1); }
            }
        }
        return;
    }
    // Windows: we are a console-subsystem exe so the CLI modes print normally; drop the console for the GUI.
    #[cfg(windows)]
    unsafe { extern "system" { fn FreeConsole() -> i32; } FreeConsole(); }
    ui::run(bank, list);
}
