use log::LevelFilter;
use simplelog::{Config, SimpleLogger};

mod server;
mod voice;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    SimpleLogger::init(LevelFilter::Info, Config::default()).unwrap();

    server::serve().await;
    Ok(())
}
