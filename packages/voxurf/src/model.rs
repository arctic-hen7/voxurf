/// A trait for AI models which Voxurf can work with. This allows being generic over
/// both local and remote models of varying quality.
pub trait Model {
    /// Errors that may occur when working with the model.
    type Error: std::error::Error + 'static;

    /// Prompts the model with the given prompt and returns its response
    /// as a string. This is expected to use chat-style prompting.
    async fn prompt(&self, prompt: &str) -> Result<String, Self::Error>;
}
