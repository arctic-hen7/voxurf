mod voice;

use crate::voice::Dictation;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    println!("starting dictation");

    let mut dictation = Dictation::start().unwrap();

    println!("dictation has started");

    println!("sleeping for a few seconds");
    std::thread::sleep(Duration::from_secs(5));

    println!("ending dictation");

    let transcribed = dictation.end();

    println!("{}", transcribed);

    Ok(())
}
