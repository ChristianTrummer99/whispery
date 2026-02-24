use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SampleRate, StreamConfig};
use std::sync::{Arc, Mutex};

pub struct AudioRecorder {
    samples: Arc<Mutex<Vec<f32>>>,
    current_level: Arc<Mutex<f32>>,
    stream: Option<cpal::Stream>,
    capture_rate: u32,
}

unsafe impl Send for AudioRecorder {}
unsafe impl Sync for AudioRecorder {}

const TARGET_RATE: u32 = 16000;

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            samples: Arc::new(Mutex::new(Vec::new())),
            current_level: Arc::new(Mutex::new(0.0)),
            stream: None,
            capture_rate: TARGET_RATE,
        }
    }

    pub fn list_devices(&self) -> Vec<String> {
        let host = cpal::default_host();
        host.input_devices()
            .map(|devices| devices.filter_map(|d| d.name().ok()).collect())
            .unwrap_or_default()
    }

    pub fn capture_rate(&self) -> u32 {
        self.capture_rate
    }

    pub fn get_audio_level(&self) -> f32 {
        *self.current_level.lock().unwrap()
    }

    pub fn start(&mut self, device_name: Option<&str>) -> Result<(), String> {
        // Drop any existing stream — stops the old callback
        self.stream = None;

        // Create a FRESH buffer for this session so any lingering old
        // callback writes to the previous (now-orphaned) Arc, not this one.
        self.samples = Arc::new(Mutex::new(Vec::new()));

        let host = cpal::default_host();
        let device = match device_name {
            Some(name) => host
                .input_devices()
                .ok()
                .and_then(|mut devs| devs.find(|d| d.name().map(|n| n == name).unwrap_or(false))),
            None => host.default_input_device(),
        }
        .ok_or("No input device found")?;

        let supported = device
            .supported_input_configs()
            .map_err(|e| format!("Failed to query configs: {e}"))?;

        let config = supported
            .filter(|c| c.sample_format() == SampleFormat::F32)
            .find(|c| {
                c.channels() == 1
                    && c.min_sample_rate() <= SampleRate(TARGET_RATE)
                    && c.max_sample_rate() >= SampleRate(TARGET_RATE)
            })
            .map(|c| c.with_sample_rate(SampleRate(TARGET_RATE)))
            .or_else(|| device.default_input_config().ok())
            .ok_or("No suitable audio config found")?;

        self.capture_rate = config.sample_rate().0;
        let channels = config.channels() as usize;
        eprintln!(
            "[whispery] Audio config: {}Hz, {} channel(s), format={:?}",
            self.capture_rate,
            channels,
            config.sample_format()
        );

        let stream_config: StreamConfig = config.into();
        let samples = self.samples.clone();
        let level = self.current_level.clone();

        let stream = device
            .build_input_stream(
                &stream_config,
                move |data: &[f32], _| {
                    let mut buf = samples.lock().unwrap();
                    if channels == 1 {
                        buf.extend_from_slice(data);
                    } else {
                        for chunk in data.chunks(channels) {
                            buf.push(chunk[0]);
                        }
                    }

                    if !data.is_empty() {
                        let sum: f32 = data.iter().map(|s| s * s).sum();
                        let rms = (sum / data.len() as f32).sqrt();
                        let normalized = (rms * 8.0).min(1.0);
                        *level.lock().unwrap() = normalized;
                    }
                },
                |err| eprintln!("[whispery] Audio stream error: {err}"),
                None,
            )
            .map_err(|e| format!("Failed to build stream: {e}"))?;

        stream
            .play()
            .map_err(|e| format!("Failed to play stream: {e}"))?;
        self.stream = Some(stream);
        Ok(())
    }

    pub fn stop(&mut self) -> Vec<f32> {
        self.stream = None;
        *self.current_level.lock().unwrap() = 0.0;
        let mut buf = self.samples.lock().unwrap();
        std::mem::take(&mut *buf)
    }

    pub fn cancel(&mut self) {
        self.stream = None;
        *self.current_level.lock().unwrap() = 0.0;
        self.samples.lock().unwrap().clear();
    }

    fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
        if from_rate == to_rate {
            return samples.to_vec();
        }
        let ratio = from_rate as f64 / to_rate as f64;
        let out_len = (samples.len() as f64 / ratio) as usize;
        let mut out = Vec::with_capacity(out_len);
        for i in 0..out_len {
            let src_idx = i as f64 * ratio;
            let idx = src_idx as usize;
            let frac = src_idx - idx as f64;
            let s0 = samples[idx.min(samples.len() - 1)];
            let s1 = samples[(idx + 1).min(samples.len() - 1)];
            out.push(s0 + (s1 - s0) * frac as f32);
        }
        out
    }

    pub fn encode_wav(&self, samples: &[f32]) -> Vec<u8> {
        let resampled = Self::resample(samples, self.capture_rate, TARGET_RATE);
        eprintln!(
            "[whispery] Resample: {}Hz -> {}Hz ({} -> {} samples)",
            self.capture_rate,
            TARGET_RATE,
            samples.len(),
            resampled.len()
        );

        let mut cursor = std::io::Cursor::new(Vec::new());
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: TARGET_RATE,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::new(&mut cursor, spec).unwrap();
        for &s in &resampled {
            let val = (s * 32767.0).clamp(-32768.0, 32767.0) as i16;
            writer.write_sample(val).unwrap();
        }
        writer.finalize().unwrap();
        cursor.into_inner()
    }
}
