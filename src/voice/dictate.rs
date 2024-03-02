use super::{Audio, Transcriptor};
use anyhow::bail;
use std::sync::mpsc::Sender;
use std::{sync::mpsc::channel, thread::JoinHandle};
use tempfile::NamedTempFile;

pub struct Dictation {
    recording: Option<Recording>,
    transcriptor: Transcriptor,
}

impl Dictation {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            recording: None,
            transcriptor: Transcriptor::new()?,
        })
    }

    /// Start a dictation.
    pub fn start(&mut self) -> anyhow::Result<()> {
        self.recording = Some(Recording::start()?);
        Ok(())
    }

    /// End a dictation.
    pub fn end(&mut self) -> anyhow::Result<String> {
        match self.recording.take() {
            Some(recording) => {
                let audio_file = recording.end();

                // Now, the audio file should contain the recorded audio, so we can transcribe the result.
                Ok(self.transcriptor.transcribe(audio_file).unwrap())
            }
            None => bail!("cannot end a recording if none was started"),
        }
    }
}

struct Recording {
    audio_file: NamedTempFile,
    end_recording_tx: Sender<()>,
    recording_thread_join_handle: JoinHandle<()>,
}

impl Recording {
    /// Start a recording. This will spawn a thread in the background that can
    /// be notified to stop recording via `end_recording_rx`.
    pub fn start() -> anyhow::Result<Self> {
        // The tmp audio file we record to.
        let audio_file = NamedTempFile::new().unwrap();

        // Create a channel used to notify the recording thread to stop recording.
        let (end_recording_tx, end_recording_rx) = channel();

        let audio_file_path = audio_file.path().to_owned();
        let recording_thread_join_handle = std::thread::spawn(move || {
            Audio::record_to_file(audio_file_path, end_recording_rx);
        });

        Ok(Self {
            audio_file,
            end_recording_tx,
            recording_thread_join_handle,
        })
    }

    /// Ends the recording, transcribes the audio, and returns the transcribed audio.
    pub fn end(self) -> NamedTempFile {
        // Notify the recording thread that it should stop recording now.
        let _ = self.end_recording_tx.send(());

        // We told the recording thread to stop recording, so it just terminate soon.
        let _ = self.recording_thread_join_handle.join();

        self.audio_file
    }
}
