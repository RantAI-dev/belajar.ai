//! The language model: an OpenAI-compatible llama-server. Thinking is off; answers stream.

use std::time::Duration;

use eventsource_stream::Eventsource;
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Message {
    pub role: &'static str,
    pub content: String,
}

#[derive(Clone)]
pub struct Llm {
    http: reqwest::Client,
    url: String,
    model: String,
    first_byte: Duration,
}

#[derive(Deserialize)]
struct ChunkOut {
    #[serde(default)]
    choices: Vec<ChoiceOut>,
}

#[derive(Deserialize)]
struct ChoiceOut {
    #[serde(default)]
    delta: Option<ContentOut>,
    #[serde(default)]
    message: Option<ContentOut>,
}

#[derive(Deserialize)]
struct ContentOut {
    #[serde(default)]
    content: Option<String>,
}

#[derive(Deserialize)]
struct CompletionOut {
    #[serde(default)]
    choices: Vec<ChoiceOut>,
}

impl Llm {
    pub fn new(url: &str, model: &str, first_byte: Duration) -> Result<Self, String> {
        // No overall timeout: an answer streams for as long as it takes; the first byte is
        // bounded instead.
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Self {
            http,
            url: url.to_owned(),
            model: model.to_owned(),
            first_byte,
        })
    }

    fn body(
        &self,
        messages: &[Message],
        max_tokens: u32,
        stream: bool,
        schema: Option<&Value>,
    ) -> Value {
        let mut body = json!({
            "model": self.model,
            "messages": messages,
            "stream": stream,
            "max_tokens": max_tokens,
            "temperature": 0.3,
            "chat_template_kwargs": { "enable_thinking": false },
        });
        if let (Some(schema), Some(map)) = (schema, body.as_object_mut()) {
            map.insert(
                "response_format".into(),
                json!({ "type": "json_schema", "json_schema": { "name": "out", "schema": schema } }),
            );
        }
        body
    }

    async fn send(&self, body: &Value) -> Result<reqwest::Response, String> {
        let request = self
            .http
            .post(format!("{}/v1/chat/completions", self.url))
            .json(body)
            .send();
        let response = tokio::time::timeout(self.first_byte, request)
            .await
            .map_err(|_| "the model did not answer in time".to_owned())?
            .map_err(|e| format!("the model is unreachable: {e}"))?;
        if !response.status().is_success() {
            let status = response.status();
            let text: String = response
                .text()
                .await
                .unwrap_or_default()
                .chars()
                .take(300)
                .collect();
            return Err(format!("the model answered {status}: {text}"));
        }
        Ok(response)
    }

    /// The answer as it is written, piece by piece.
    pub async fn stream(
        &self,
        messages: &[Message],
        max_tokens: u32,
    ) -> Result<impl Stream<Item = Result<String, String>> + Send + use<>, String> {
        let response = self
            .send(&self.body(messages, max_tokens, true, None))
            .await?;
        let events = response.bytes_stream().eventsource();
        Ok(events.filter_map(|event| async move {
            match event {
                Err(e) => Some(Err(format!("the model stream broke: {e}"))),
                Ok(event) if event.data.trim() == "[DONE]" => None,
                Ok(event) => {
                    let chunk: ChunkOut = serde_json::from_str(&event.data).ok()?;
                    let text = chunk
                        .choices
                        .into_iter()
                        .find_map(|c| c.delta.and_then(|d| d.content))?;
                    (!text.is_empty()).then_some(Ok(text))
                }
            }
        }))
    }

    /// One whole answer, constrained to a JSON schema when one is given.
    pub async fn complete(
        &self,
        messages: &[Message],
        max_tokens: u32,
        schema: Option<&Value>,
    ) -> Result<String, String> {
        let response = self
            .send(&self.body(messages, max_tokens, false, schema))
            .await?;
        let out: CompletionOut = response
            .json()
            .await
            .map_err(|e| format!("the model's answer is not JSON: {e}"))?;
        out.choices
            .into_iter()
            .find_map(|c| c.message.and_then(|m| m.content))
            .ok_or_else(|| "the model returned no text".to_owned())
    }
}
