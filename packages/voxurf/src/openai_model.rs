use std::collections::HashMap;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use crate::model::Model;

/// The URL to post requests to.
const MODEL_URL: &str = "https://api.openai.com/v1/chat/completions";

/// The function the model can use to click an element.
const CLICK_FUNCTION: Tool = Tool::function(
    "click",
    "Use this function to click on an element by its ID.",
    &[
        ("id", "integer", "The element's unique ID"),
    ],
);
/// The function the model can use to type text into an element.
const TYPE_FUNCTION: Tool = Tool::function(
    "type",
    "Use this function to type some text into an element (it will be clicked and selected automatically).",
    &[
        ("id", "integer", "The element's unique ID"),
        ("text", "string", "The text to type into the element"),
    ],
);
/// The function the model can use to wait for the layout of the page to change.
const WAIT_FUNCTION: Tool = Tool::function(
    "wait",
    &r#"Use this function if you need to wait for the layout of the page to change so you can continue executing the user's command.
    Any other function calls after this will be ignored. Don't use this function if you don't need it!"#.replace("\n", " "),
    &[
        ("reason", "string", "A brief reason for why you need to wait for the page layout to change"),
    ],
);

/// An implementations of OpenAI's chat completion system for Voxurf. This uses the
/// function calling API under the hood to improve accuracy.
pub struct OpenAiModel {
    /// The model to use.
    model: String,
    /// The temperature to use for the model.
    temperature: f32,
    /// The API key to authenticate with.
    api_key: String,
}
impl OpenAiModel {
    /// Constructs a new OpenAI model with the given API key.
    pub fn new(api_key: String) -> Self {
        Self {
            model: "gpt-3.5-turbo".to_string(),
            temperature: 0.3,
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
                MessageInput {
                    role: "user",
                    content: prompt,
                },
            ],
            tools: vec![ CLICK_FUNCTION, TYPE_FUNCTION, WAIT_FUNCTION ],
        };

        let client = Client::new();
        let raw_response = client
            .post(MODEL_URL)
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .json(&request)
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
struct ChatRequest<'a, 'b> {
    model: &'a str,
    temperature: f32,
    messages: Vec<MessageInput<'b>>,
    tools: Vec<Tool>,
}
#[derive(Serialize, Deserialize)]
struct MessageInput<'a> {
    role: &'static str,
    content: &'a str,
}
#[derive(Serialize)]
struct Tool {
    #[serde(rename = "type")]
    ty: &'static str,
    function: OpenAiFunction,
}
impl Tool {
    /// Creates a new function for the model to call according to OpenAI's tool API.
    /// Parameters are provided in the form `(name, type, description)`.
    const fn function(name: &'static str, description: &'static str, params: &[(&'static str, &'static str, &'static str)]) -> Self {
        Self {
            ty: "function",
            function: OpenAiFunction {
                name,
                description,
                parameters: FunctionParameters {
                    ty: "object",
                    properties: params
                        .iter()
                        .map(|(name, ty, description)| (*name, FunctionParameter { ty, description }))
                        .collect()
                }
            }
        }
    }
}
#[derive(Serialize)]
struct OpenAiFunction {
    description: &'static str,
    name: &'static str,
    parameters: FunctionParameters,
}
#[derive(Serialize)]
struct FunctionParameters {
    #[serde(rename = "type")]
    ty: &'static str,
    properties: HashMap<&'static str, FunctionParameter>,
}
#[derive(Serialize)]
struct FunctionParameter {
    #[serde(rename = "type")]
    ty: &'static str,
    description: &'static str,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ResponseChoice>,
}
#[derive(Deserialize)]
struct ResponseChoice {
    message: MessageOutput,
}
#[derive(Deserialize)]
struct MessageOutput {
    role: String,
    // This will be `None` if the model called one or more functions
    content: Option<String>,
    tool_calls: Vec<ToolCall>
}
#[derive(Deserialize)]
struct ToolCall {
    function: FunctionCall,
}
#[derive(Deserialize)]
struct FunctionCall {
    name: String,
    arguments: HashMap<String, Value>,
}

/// Errors that can occur while using the OpenAI model.
#[derive(Error, Debug)]
pub enum OpenAiModelError {
    #[error("failed to send request to openai api")]
    RequestError {
        #[source]
        source: reqwest::Error,
    },
    #[error("failed to parse response from openai api")]
    ResponseParseFailed {
        #[source]
        source: serde_json::Error,
    },
}
