mod error;
mod executor;
mod interface;
mod model;
mod tree;
#[cfg(feature = "openai-model")]
mod openai_model;

pub use executor::{Executor, ExecutorOpts};
pub use interface::Interface;
pub use model::Model;
pub use tree::{Node, Tree};
#[cfg(feature = "openai-model")]
pub use openai_model::{OpenAiModel, OpenAiModelError};

#[cfg(target_arch = "wasm32")]
async fn sleep(ms: u64) {
    use gloo_timers::future::sleep;
    use std::time::Duration;

    sleep(Duration::from_millis(ms)).await;
}
#[cfg(not(target_arch = "wasm32"))]
async fn sleep(ms: u64) {
    use std::time::Duration;
    use tokio::time::sleep;

    sleep(Duration::from_millis(ms)).await;
}
