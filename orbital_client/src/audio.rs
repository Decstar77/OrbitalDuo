use alto::{Alto, Source};
use std::{collections::HashMap, sync::Arc};

use crate::{types::{normalized_f64_to_i16}, config};

pub struct AudioState {
    alto: Alto,
    device: alto::OutputDevice,
    context: alto::Context,
    sources: Vec<alto::StaticSource>,
    buffers: HashMap<String, Arc<alto::Buffer>>,
    base_path: String,
}

impl AudioState {
    pub fn new() -> Self {
        let alto = Alto::load_default().expect("Failed to load OpenAL");
        let device = alto.open(None).expect("Failed to open device");
        let context = device.new_context(None).expect("Failed to create context");

        let mut sources = Vec::new();
        for _ in 0..32 {
            let source = context
                .new_static_source()
                .expect("Failed to create source");
            sources.push(source);
        }

        let cfg = config::get_config();

        Self {
            alto,
            device,
            context,
            sources,
            buffers: HashMap::new(),
            base_path: cfg.assets_path.clone(),
        }
    }

    fn load_sound_wav(&mut self, name: &str) {
        let path = format!("{}{}.wav", self.base_path, name);
        match hound::WavReader::open(path) {
            Ok(mut reader) => {
                let spec = reader.spec();

                if spec.channels != 2 {
                    println!("Only stereo sounds are supported");
                    return;
                }

                if spec.bits_per_sample == 16 {
                    let samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
                    let frequency = spec.sample_rate as i32;

                    let buffer = self
                        .context
                        .new_buffer::<alto::Stereo<i16>, _>(samples, frequency)
                        .expect("Failed to create buffer");

                    self.buffers.insert(name.to_string(), Arc::new(buffer));
                } else if spec.bits_per_sample == 24 {
                    let max_24bit_value : f64 = (i32::pow(2, 23) - 1) as f64;
                    let samples: Vec<i16> = reader
                        .samples::<i32>()
                        .map(|s| normalized_f64_to_i16((s.unwrap() as f64) / max_24bit_value))
                        .collect();

                    let frequency = spec.sample_rate as i32;
                    let buffer = self
                        .context
                        .new_buffer::<alto::Stereo<i16>, _>(samples, frequency)
                        .expect("Failed to create buffer");

                    self.buffers.insert(name.to_string(), Arc::new(buffer));
                }
            }
            Err(err) => {
                println!("Failed to load sound: {}", err);
            }
        }
    }

    pub fn play_sound(&mut self, name: &str) {
        if !self.buffers.contains_key(name) {
            self.load_sound_wav(name);
        }

        let buffer = self.buffers.get(name).expect("Failed to get buffer");
        for source in self.sources.iter_mut() {
            if source.state() == alto::SourceState::Stopped
                || source.state() == alto::SourceState::Initial
            {
                source
                    .set_buffer(buffer.clone())
                    .expect("Failed to set buffer");
                source.play();
                break;
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::{thread, time::Duration};

    use super::*;

    #[test]
    pub fn basic_sound() {
        let alto = Alto::load_default().expect("Failed to load OpenAL");
        let device = alto.open(None).expect("Failed to open device");
        let context = device.new_context(None).expect("Failed to create context");
        let mut source = context
            .new_static_source()
            .expect("Failed to create source");

        let mut wav_reader = hound::WavReader::open("test.wav").unwrap();
        let spec = wav_reader.spec();
        println!("{:?}", spec);
        let samples: Vec<i16> = wav_reader.samples::<i16>().map(|s| s.unwrap()).collect();
        let frequency = spec.sample_rate as i32;

        let buffer = context
            .new_buffer::<alto::Stereo<i16>, _>(samples, frequency)
            .expect("Failed to create buffer");
        let arc_buffer = Arc::new(buffer);

        source.set_buffer(arc_buffer).expect("Failed to set buffer");
        source.play();

        while source.state() == alto::SourceState::Playing {
            thread::sleep(Duration::from_millis(100));
        }
    }
}
