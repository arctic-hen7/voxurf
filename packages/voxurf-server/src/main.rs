use log::LevelFilter;
use simplelog::{Config, SimpleLogger};

mod server;
mod voice;

// const WHISPER_MODEL_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/whisper-models/");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    SimpleLogger::init(LevelFilter::Info, Config::default()).unwrap();

    // log::info!("whisper model dir: {}", WHISPER_MODEL_DIR);

    server::serve().await;
    Ok(())
}
