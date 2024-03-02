use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use std::env;

const OPENAI_API: &str = "https://api.openai.com/v1/chat/completions";
const OPENAI_API_KEY: &str = env!("OPENAI_API_KEY");

pub struct OpenAiApi;

impl OpenAiApi {
    pub async fn call(prompt: &str) -> Result<String, gloo_net::Error> {
        let body = ApiRequestBody {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            // TODO: adjust for best results?
            temperature: 0.7,
        };

        let response_text = Request::post(OPENAI_API)
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", OPENAI_API_KEY))
            .json(&body)?
            .send()
            .await?
            .text()
            .await?;

        let response: ApiResponse = serde_json::from_str(&response_text).unwrap();

        let total_content: String = response
            .choices
            .into_iter()
            .map(|choice| choice.message.content)
            .collect::<Vec<_>>()
            .join("\n");

        Ok(total_content)
    }
}

#[derive(Serialize)]
struct ApiRequestBody {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f64,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: Message,
}

#[derive(Deserialize, Debug)]
struct Message {
    role: String,
    content: String,
}
