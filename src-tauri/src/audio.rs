use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SampleRate, StreamConfig};
use std::sync::{Arc, Mutex};

pub struct AudioRecorder {
    samples: Arc<Mutex<Vec<f32>>>,
    stream: Option<cpal::Stream>,
    sample_rate: u32,
}

// SAFETY: AudioRecorder is always accessed through a Mutex<AudioRecorder>
// in AppState, ensuring exclusive access. The cpal::Host and cpal::Stream
// types are only non-Send due to raw pointers in CoreAudio bindings, but
// our usage pattern (create on main, access exclusively via mutex) is safe.
unsafe impl Send for AudioRecorder {}
unsafe impl Sync for AudioRecorder {}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            samples: Arc::new(Mutex::new(Vec::new())),
            stream: None,
            sample_rate: 16000,
        }
    }

    pub fn list_devices(&self) -> Vec<String> {
        let host = cpal::default_host();
        host.input_devices()
            .map(|devices| devices.filter_map(|d| d.name().ok()).collect())
            .unwrap_or_default()
    }

    pub fn start(&mut self, device_name: Option<&str>) -> Result<(), String> {
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
            .filter(|c| c.channels() == 1 && c.sample_format() == SampleFormat::F32)
            .find(|c| {
                c.min_sample_rate() <= SampleRate(16000)
                    && c.max_sample_rate() >= SampleRate(16000)
            })
            .map(|c| c.with_sample_rate(SampleRate(16000)))
            .or_else(|| device.default_input_config().ok())
            .ok_or("No suitable audio config found")?;

        self.sample_rate = config.sample_rate().0;
        let channels = config.channels() as usize;

        let stream_config: StreamConfig = config.into();
        let samples = self.samples.clone();

        {
            let mut s = samples.lock().unwrap();
            s.clear();
        }

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
                },
                |err| eprintln!("Audio stream error: {err}"),
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
        let samples = self.samples.lock().unwrap().clone();
        samples
    }

    pub fn encode_wav(&self, samples: &[f32]) -> Vec<u8> {
        let mut cursor = std::io::Cursor::new(Vec::new());
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: self.sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::new(&mut cursor, spec).unwrap();
        for &s in samples {
            let val = (s * 32767.0).clamp(-32768.0, 32767.0) as i16;
            writer.write_sample(val).unwrap();
        }
        writer.finalize().unwrap();
        cursor.into_inner()
    }
}
