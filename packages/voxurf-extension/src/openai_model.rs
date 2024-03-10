use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use voxurf::Model;

/// An implementations of OpenAI's chat completion system for Voxurf.
pub struct OpenAiModel {
    model: String,
    temperature: f32,
    api_key: String,
}
impl OpenAiModel {
    pub fn new(api_key: String) -> Self {
        Self {
            model: "gpt-3.5-turbo".to_string(),
            temperature: 0.7,
            api_key,
        }
    }
    /// Changes the model's temperature. The default value has been manually tuned, so use
    /// this with some caution.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }
    /// Changes the model we'll use. The default is "gpt-3.5-turbo", and changing this to a more
    /// capable model like GPT-4 may improve accuracy, but also increase costs.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }
}
impl Model for OpenAiModel {
    type Error = OpenAiModelError;

    async fn prompt(&self, prompt: &str) -> Result<String, Self::Error> {
        let request = ChatRequest {
            model: &self.model,
            temperature: self.temperature,
            messages: vec![
                // TODO System message?
                Message {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
        };

        let raw_response = Request::post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .json(&request)
            // Serialization to JSON can't fail
            .unwrap()
            .send()
            .await
            .map_err(|err| OpenAiModelError::RequestError { source: err })?
            .text()
            .await
            .map_err(|err| OpenAiModelError::RequestError { source: err })?;
        let mut api_response: ChatResponse = serde_json::from_str(&raw_response)
            .map_err(|err| OpenAiModelError::ResponseParseFailed { source: err })?;
        // There is guaranteed to be exactly one choice
        let model_response = api_response.choices.remove(0).message.content;

        Ok(model_response)
    }
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    temperature: f32,
    messages: Vec<Message>,
}
#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}
#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ResponseChoice>,
}
#[derive(Deserialize)]
struct ResponseChoice {
    message: Message,
}

/// Errors that can occur while using the OpenAI model.
#[derive(Error, Debug)]
pub enum OpenAiModelError {
    #[error("failed to send request to openai api")]
    RequestError {
        #[source]
        source: gloo_net::Error,
    },
    #[error("failed to parse response from openai api")]
    ResponseParseFailed {
        #[source]
        source: serde_json::Error,
    },
}
