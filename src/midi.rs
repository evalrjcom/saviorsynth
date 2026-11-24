//! Optional MIDI input (feature "midi"). Messages are validated and pushed onto the lock-free queue.

use crate::shared::{Event, Shared};
use std::sync::Arc;

#[cfg(feature = "midi")]
mod imp {
    use super::*;
    pub struct MidiConn { _c: midir::MidiInputConnection<()> }
    pub fn list_ports() -> Vec<String> {
        match midir::MidiInput::new("list") {
            Ok(m) => m.ports().iter().map(|p| m.port_name(p).unwrap_or_else(|_| "?".into())).collect(),
            Err(_) => vec![],
        }
    }
    pub fn connect(index: usize, shared: Arc<Shared>) -> Result<MidiConn, String> {
        let mut m = midir::MidiInput::new("saviorsynth").map_err(|e| e.to_string())?;
        m.ignore(midir::Ignore::SysexAndTime);
        let ports = m.ports();
        let port = ports.get(index).ok_or("MIDI port vanished")?.clone();
        let c = m.connect(&port, "in", move |_, msg, _| handle(msg, &shared), ()).map_err(|e| e.to_string())?;
        Ok(MidiConn { _c: c })
    }
}
#[cfg(not(feature = "midi"))]
mod imp {
    use super::*;
    pub struct MidiConn;
    pub fn list_ports() -> Vec<String> { vec![] }
    pub fn connect(_i: usize, _s: Arc<Shared>) -> Result<MidiConn, String> { Err("built without MIDI support".into()) }
}
pub use imp::*;

#[allow(dead_code)]
fn handle(msg: &[u8], sh: &Shared) {
    if msg.is_empty() { return; }
    let st = msg[0] & 0xF0;
    let d1 = msg.get(1).copied().unwrap_or(0) & 0x7F;
    let d2 = msg.get(2).copied().unwrap_or(0) & 0x7F;
    match st {
        0x90 if msg.len() >= 3 => if d2 > 0 { sh.note_on(d1, d2) } else { sh.note_off(d1) },
        0x80 if msg.len() >= 3 => sh.note_off(d1),
        0xE0 if msg.len() >= 3 => sh.set_bend((((d2 as i32) << 7 | d1 as i32) - 8192) as f32 / 8192.0),
        0xB0 if msg.len() >= 3 => match d1 {
            1 => sh.set_mod(d2 as f32 / 127.0),
            64 => sh.push(Event::Sustain(d2 >= 64)),
            120 | 123 => sh.push(Event::AllOff),
            _ => {}
        },
        _ => {}
    }
}
