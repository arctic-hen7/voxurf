use serde::Deserialize;
use std::path::PathBuf;

const WHISPER_MODEL_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/whisper-models/");
const WHISPER_MODEL_URLS: &str = include_str!("../../assets/whisper-models-urls.json");

/// The different kinds of Whisper models that can be downloaded.
#[derive(Clone, Copy, Default)]
pub enum WhisperModel {
    WhisperTiny,
    #[default]
    WhisperBase,
    WhisperSmall,
    WhisperMedium,
    WhisperLarge,
}

#[derive(Deserialize)]
struct ModelURLs {
    whisper_tiny: String,
    whisper_base: String,
    whisper_small: String,
    whisper_medium: String,
    whisper_large: String,
}

impl WhisperModel {
    fn to_identifier(&self) -> &str {
        match self {
            Self::WhisperTiny => "whisper_tiny",
            Self::WhisperBase => "whisper_base",
            Self::WhisperSmall => "whisper_small",
            Self::WhisperMedium => "whisper_medium",
            Self::WhisperLarge => "whisper_large",
        }
    }

    /// Gets the path to this model. If it doesn't exist, it will be downloaded.
    pub fn get_or_download(&self) -> anyhow::Result<PathBuf> {
        if let Some(path) = self.get()? {
            Ok(path)
        } else {
            self.download()
        }
    }

    /// Gets the path to this model, or returns `Ok(None)` if it hasn't been downloaded yet.
    pub fn get(&self) -> anyhow::Result<Option<PathBuf>> {
        let whisper_model_dir_path = PathBuf::from(WHISPER_MODEL_DIR);

        std::fs::create_dir_all(WHISPER_MODEL_DIR)?;

        let model_key = self.to_identifier();
        let download_path = whisper_model_dir_path.join(format!("{model_key}.bin"));

        if download_path.exists() {
            Ok(Some(download_path))
        } else {
            Ok(None)
        }
    }

    /// Downloads this model, without checking whether it exists in the filesystem already.
    pub fn download(&self) -> anyhow::Result<PathBuf> {
        let model_urls_json: ModelURLs = serde_json::from_str(WHISPER_MODEL_URLS)?;

        let model_url = match self {
            WhisperModel::WhisperTiny => model_urls_json.whisper_tiny,
            WhisperModel::WhisperBase => model_urls_json.whisper_base,
            WhisperModel::WhisperSmall => model_urls_json.whisper_small,
            WhisperModel::WhisperMedium => model_urls_json.whisper_medium,
            WhisperModel::WhisperLarge => model_urls_json.whisper_large,
        };

        let client = reqwest::blocking::Client::new();

        // Get the model index first and resolve the URL for the model
        let response = client.get(model_url).send()?;
        if !response.status().is_success() {
            panic!("failed to download model");
        }

        let whisper_model_dir_path = PathBuf::from(WHISPER_MODEL_DIR);
        let download_path = whisper_model_dir_path.join(format!("{}.bin", self.to_identifier()));

        let content = response.text()?;

        std::fs::write(&download_path, content)?;

        Ok(download_path)
    }
}
