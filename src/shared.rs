//! State shared between the UI / MIDI threads and the real-time audio thread.
//! Everything here is lock-free: atomics and a bounded MPMC queue that never allocates after creation.

use crate::params::ParamStore;
use crossbeam_queue::ArrayQueue;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub enum Event {
    NoteOn(u8, u8),
    NoteOff(u8),
    AllOff,
    Sustain(bool),
    Loop(u8),      // looper command (see looper::Cmd)
    Drum(u8, u8),  // audition a drum voice (track, velocity)
}

pub const SCOPE_LEN: usize = 1024;

pub struct Shared {
    pub params: ParamStore,
    pub events: ArrayQueue<Event>,
    pub bend: AtomicU32,      // f32 -1..1
    pub modwheel: AtomicU32,  // f32 0..1
    pub peak_l: AtomicU32,
    pub peak_r: AtomicU32,
    pub voices: AtomicU32,
    pub scope: Vec<AtomicU32>,
    pub scope_pos: AtomicUsize,
    pub sample_rate: AtomicU32,
    pub chord_root: AtomicU32, // MIDI note of the sounding chord root, or u32::MAX when none
    pub chord_mask: AtomicU32, // bit d set = chord contains root+d semitones
    pub loop_state: AtomicU32, // looper::LState as u32
    pub loop_pos: AtomicU32,   // f32 0..1 position/progress
    pub drum_step: AtomicU32,  // current drum step 0..15, or u32::MAX when stopped
}

impl Shared {
    pub fn new() -> Arc<Shared> {
        Arc::new(Shared {
            params: ParamStore::new(),
            events: ArrayQueue::new(1024),
            bend: AtomicU32::new(0f32.to_bits()),
            modwheel: AtomicU32::new(0f32.to_bits()),
            peak_l: AtomicU32::new(0),
            peak_r: AtomicU32::new(0),
            voices: AtomicU32::new(0),
            scope: (0..SCOPE_LEN).map(|_| AtomicU32::new(0)).collect(),
            scope_pos: AtomicUsize::new(0),
            sample_rate: AtomicU32::new(48000),
            chord_root: AtomicU32::new(u32::MAX),
            chord_mask: AtomicU32::new(0),
            loop_state: AtomicU32::new(0),
            loop_pos: AtomicU32::new(0),
            drum_step: AtomicU32::new(u32::MAX),
        })
    }
    /// Never blocks; silently drops the event if the queue is full (only possible under a flood).
    pub fn push(&self, e: Event) { let _ = self.events.push(e); }
    pub fn note_on(&self, n: u8, v: u8) { if n < 128 { self.push(Event::NoteOn(n, v.clamp(1, 127))); } }
    pub fn note_off(&self, n: u8) { if n < 128 { self.push(Event::NoteOff(n)); } }
    pub fn set_bend(&self, v: f32) { self.bend.store(v.clamp(-1.0, 1.0).to_bits(), Ordering::Relaxed); }
    pub fn set_mod(&self, v: f32) { self.modwheel.store(v.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed); }
    pub fn get_bend(&self) -> f32 { f32::from_bits(self.bend.load(Ordering::Relaxed)) }
    pub fn get_mod(&self) -> f32 { f32::from_bits(self.modwheel.load(Ordering::Relaxed)) }
    pub fn get_peaks(&self) -> (f32, f32) {
        (f32::from_bits(self.peak_l.load(Ordering::Relaxed)), f32::from_bits(self.peak_r.load(Ordering::Relaxed)))
    }
}
