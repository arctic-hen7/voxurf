use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    StreamConfig,
};
use hound::{WavReader, WavSpec};
use std::{path::Path, sync::mpsc::Receiver};

// Audio format requirements set by Whisper.
const REQUIRED_CHANNELS: u16 = 1;
const REQUIRED_SAMPLE_RATE: u32 = 16_000;

/// WAV audio in its floating point representation. If parsing from the
/// input was successful, the format will be in 16kHZ mono f32.
pub struct Audio {
    pub data: Vec<f32>,
}

impl Audio {
    pub fn from_file<P: AsRef<Path>>(audio_file: P) -> Self {
        let mut reader = WavReader::open(audio_file).expect("failed to open file");

        let WavSpec {
            sample_rate,
            channels,
            ..
        } = reader.spec();

        // Guarantee that audio is in the format required by whisper, which is mono f32 audio in 16kHz.
        assert!(
            sample_rate == REQUIRED_SAMPLE_RATE,
            "sample rate must be 16KHz, but was: {}",
            sample_rate
        );

        assert!(
            channels == REQUIRED_CHANNELS,
            "audio needs to be mono (1 channel), but number of channels was: {}",
            channels
        );

        // Convert the audio to floating point samples.
        let audio = reader
            .samples::<f32>()
            .map(|sample| sample.expect("invalid sample"))
            .collect();

        Self { data: audio }
    }

    /// Record audio to a file, and end the recording once notified via the provided channel.
    pub fn record_to_file<P: AsRef<Path>>(audio_file_path: P, end_recording_rx: Receiver<()>) {
        let host = cpal::default_host();
        let input_device = host.default_input_device().unwrap();
        let dflt_config = input_device.default_input_config().unwrap();

        // Initialize the WAV writer.
        let spec = hound::WavSpec {
            channels: REQUIRED_CHANNELS,
            sample_rate: REQUIRED_SAMPLE_RATE,
            bits_per_sample: (dflt_config.sample_format().sample_size() * 8) as u16,
            sample_format: hound::SampleFormat::Float,
        };
        let config = StreamConfig {
            channels: REQUIRED_CHANNELS,
            sample_rate: cpal::SampleRate(REQUIRED_SAMPLE_RATE),
            buffer_size: cpal::BufferSize::Default,
        };

        let mut writer = hound::WavWriter::create(&audio_file_path, spec).unwrap();

        // Initialize the CPAL audio input stream.
        let input_stream = input_device
            .build_input_stream(
                &config,
                move |data: &[f32], _| {
                    // Callback function to receive audio data
                    for sample in data {
                        writer
                            .write_sample(*sample)
                            .expect("error writing audio data to WAV file");
                    }
                },
                |err| {
                    // Error callback
                    panic!("Error in audio stream: {:?}", err);
                },
                None,
            )
            .unwrap();

        // Start the audio stream.
        input_stream
            .play()
            .expect("failed to start recording audio");

        // Wait for a signal from the receiver to stop recording.
        let _ = end_recording_rx.recv();

        // Stop and close the audio stream.
        drop(input_stream);
    }
}
