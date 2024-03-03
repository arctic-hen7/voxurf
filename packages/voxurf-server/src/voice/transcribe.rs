use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext};

use super::WhisperModel;
use crate::voice::Audio;

pub struct Transcriptor {
    whisper_ctx: WhisperContext,
}

impl Transcriptor {
    pub async fn new() -> anyhow::Result<Self> {
        let model = WhisperModel::default();
        let model_path = model.get_or_download().await?;

        assert!(model_path.exists(), "expected whisper model file to exist");
        log::info!(
            "initializing whisper with model at path: {}",
            model_path.display()
        );

        let whisper_ctx = WhisperContext::new(&model_path.to_string_lossy())?;

        log::info!("whisper setup succeeded");

        Ok(Self { whisper_ctx })
    }

    /// Transcribes the audio in the given file to a string of text.
    pub fn transcribe<P: AsRef<Path>>(&self, audio_file: P) -> anyhow::Result<String> {
        assert!(
            audio_file.as_ref().exists(),
            "expected input audio file to exist"
        );

        let mut state = self.whisper_ctx.create_state()?;

        // Sampling parameters for the model.
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 0 });
        params.set_n_threads(num_cpus::get_physical() as i32);
        params.set_translate(false);
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        let audio = Audio::from_file(audio_file);

        // Run the inference.
        state
            .full(params, &audio.data[..])
            .expect("running inference failed");

        // Iterate through the segments of the transcript to extract the actual text
        let num_segments = state.full_n_segments()?;
        let mut segments = Vec::new();
        for i in 0..num_segments {
            let segment = state.full_get_segment_text(i)?;
            segments.push(segment);
        }
        let full_text = segments.join("");

        Ok(full_text)
    }
}
