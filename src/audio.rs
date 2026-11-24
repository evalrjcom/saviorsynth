//! cpal output stream. The callback owns the engine (no locks); it renders interleaved stereo
//! into a pre-allocated scratch buffer and converts to whatever sample format the device wants.

use crate::engine::Engine;
use crate::shared::Shared;
use crate::wavetable::WaveBank;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample};
use std::sync::atomic::Ordering;
use std::sync::Arc;

pub struct AudioHandle { _stream: cpal::Stream, pub info: String }

pub fn start(shared: Arc<Shared>, bank: Arc<WaveBank>) -> Result<AudioHandle, String> {
    let host = cpal::default_host();
    let device = host.default_output_device().ok_or("no audio output device found")?;
    let name = device.name().unwrap_or_else(|_| "unknown device".into());
    let supported = device.default_output_config().map_err(|e| format!("output config: {e}"))?;
    let sr = supported.sample_rate().0;
    let channels = supported.channels() as usize;
    if channels == 0 || sr < 8000 { return Err("unsupported audio device configuration".into()); }
    shared.sample_rate.store(sr, Ordering::Relaxed);
    let cfg: cpal::StreamConfig = supported.config();
    let engine = Engine::new(sr as f32, shared, bank);
    let stream = match supported.sample_format() {
        SampleFormat::F32 => build::<f32>(&device, &cfg, engine, channels),
        SampleFormat::I16 => build::<i16>(&device, &cfg, engine, channels),
        SampleFormat::U16 => build::<u16>(&device, &cfg, engine, channels),
        SampleFormat::I32 => build::<i32>(&device, &cfg, engine, channels),
        f => Err(format!("unsupported sample format {f:?}")),
    }?;
    stream.play().map_err(|e| format!("cannot start stream: {e}"))?;
    Ok(AudioHandle { _stream: stream, info: format!("{} @ {} Hz", name, sr) })
}

fn build<T>(device: &cpal::Device, cfg: &cpal::StreamConfig, mut engine: Engine, channels: usize) -> Result<cpal::Stream, String>
where T: SizedSample + FromSample<f32> {
    const CHUNK: usize = 4096;
    let mut scratch = vec![0.0f32; CHUNK * 2];
    device.build_output_stream(
        cfg,
        move |data: &mut [T], _| {
            let frames = data.len() / channels;
            let mut done = 0;
            while done < frames {
                let n = (frames - done).min(CHUNK);
                engine.fill(&mut scratch[..n * 2]);
                for i in 0..n {
                    let (l, r) = (scratch[2 * i], scratch[2 * i + 1]);
                    let o = &mut data[(done + i) * channels..(done + i + 1) * channels];
                    if channels == 1 { o[0] = T::from_sample(0.5 * (l + r)); }
                    else { for (c, s) in o.iter_mut().enumerate() { *s = T::from_sample(match c { 0 => l, 1 => r, _ => 0.0 }); } }
                }
                done += n;
            }
        },
        |e| eprintln!("audio stream error: {e}"),
        None,
    ).map_err(|e| format!("cannot open audio stream: {e}"))
}
