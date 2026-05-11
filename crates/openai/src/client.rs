use crate::error::{Error, Result};
use crate::types::{Model, ModelList};

pub struct Client {
    api_key: String,
    base_url: String,
}

impl Client {
    pub fn new(api_key: &str) -> Result<Self> {
        Self::with_base_url(api_key, "https://api.minimaxi.com/v1")
    }

    pub fn with_base_url(api_key: &str, base_url: &str) -> Result<Self> {
        if api_key.is_empty() {
            return Err(Error::MissingApiKey);
        }
        Ok(Self {
            api_key: api_key.to_string(),
            base_url: base_url.to_string(),
        })
    }

    pub fn chat(&self) -> crate::chat::ChatBuilder<'_> {
        crate::chat::ChatBuilder::new(self)
    }

    pub fn models(&self) -> Models<'_> {
        Models::new(self)
    }

    pub fn request<T>(&self, request: reqwest::blocking::RequestBuilder) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let response = request
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()?;

        if !response.status().is_success() {
            let code = response.status().as_u16() as i32;
            let message = response.text().unwrap_or_default();
            return Err(Error::Api { code, message });
        }

        response.json().map_err(|e| Error::InvalidResponse(e.to_string()))
    }

    pub(crate) fn send_streaming_request(&self, request: reqwest::blocking::RequestBuilder) -> Result<String> {
        let response = request
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()?;

        if !response.status().is_success() {
            let code = response.status().as_u16() as i32;
            let message = response.text().unwrap_or_default();
            return Err(Error::Api { code, message });
        }

        response.text().map_err(|e| Error::InvalidResponse(e.to_string()))
    }

    pub fn http_client(&self) -> reqwest::blocking::Client {
        reqwest::blocking::Client::builder()
            .build()
            .expect("Failed to create HTTP client")
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

pub struct Models<'a> {
    client: &'a Client,
}

impl<'a> Models<'a> {
    fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub fn list(&self) -> Result<ModelList> {
        let url = format!("{}/models", self.client.base_url());
        let request = self.client.http_client().get(&url);
        self.client.request(request)
    }

    pub fn get(&self, model_id: &str) -> Result<Model> {
        let url = format!("{}/models/{}", self.client.base_url(), model_id);
        let request = self.client.http_client().get(&url);
        self.client.request(request)
    }
}

/// Trait for executing tool calls returned by the model.
/// Implement this trait and pass it to `ChatWithExecutor` to handle
/// tool calls automatically.
pub trait ToolExecutor: Send + Sync {
    /// Execute a tool call with the given name and arguments.
    /// Returns the tool result as a JSON string on success, or an error
    /// which will be passed to the model for handling.
    fn execute(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>>;
}

/// Builder for chat requests with automatic tool execution.
/// Use `Client::chat_with_executor()` to create an instance.
pub struct ChatWithExecutor<'a> {
    client: &'a Client,
    tools: Vec<crate::types::Tool>,
    executor: std::sync::Arc<dyn ToolExecutor>,
    model: String,
    messages: Vec<crate::types::Message>,
    temperature: Option<f64>,
    top_p: Option<f64>,
    max_completion_tokens: Option<i32>,
}

impl<'a> ChatWithExecutor<'a> {
    pub fn new(
        client: &'a Client,
        tools: Vec<crate::types::Tool>,
        executor: std::sync::Arc<dyn ToolExecutor>,
    ) -> Self {
        Self {
            client,
            tools,
            executor,
            model: String::new(),
            messages: Vec::new(),
            temperature: None,
            top_p: None,
            max_completion_tokens: None,
        }
    }

    pub fn model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }

    pub fn messages(mut self, messages: Vec<crate::types::Message>) -> Self {
        self.messages = messages;
        self
    }

    pub fn temperature(mut self, temperature: f64) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn top_p(mut self, top_p: f64) -> Self {
        self.top_p = Some(top_p);
        self
    }

    pub fn max_completion_tokens(mut self, tokens: i32) -> Self {
        self.max_completion_tokens = Some(tokens);
        self
    }

    pub fn send(self) -> crate::error::Result<crate::types::ChatResponse> {
        let mut messages = self.messages;
        let model = self.model;

        loop {
            // Build request body
            let mut body = serde_json::json!({
                "model": model,
                "messages": messages,
                "max_completion_tokens": self.max_completion_tokens,
                "temperature": self.temperature,
                "top_p": self.top_p,
            });

            if !self.tools.is_empty() {
                body["tools"] = serde_json::to_value(&self.tools).unwrap();
            }

            let url = format!("{}/chat/completions", self.client.base_url());
            let request = self.client.http_client().post(&url).json(&body);

            // Send request and parse response
            let response: crate::types::ChatResponse = self.client.request(request)?;

            // Check if response has tool calls
            let tool_calls = response.choices.first()
                .and_then(|c| c.tool_calls.clone());

            let Some(tool_calls) = tool_calls else {
                // No tool calls - this is the final answer
                return Ok(response);
            };

            // Process each tool call
            for call in &tool_calls {
                let arguments: serde_json::Value = match serde_json::from_str(&call.function.arguments) {
                    Ok(v) => v,
                    Err(e) => {
                        // On parse error, create error result and continue
                        let error_msg = serde_json::json!({
                            "error": format!("failed to parse arguments: {}", e)
                        });
                        messages.push(crate::types::Message::tool(&call.id, &error_msg.to_string()));
                        continue;
                    }
                };

                // Execute the tool
                let result = self.executor.execute(&call.function.name, arguments);

                // Create tool result message
                let tool_message = match result {
                    Ok(result_str) => crate::types::Message::tool(&call.id, &result_str),
                    Err(e) => {
                        let error_obj = serde_json::json!({
                            "error": e.to_string()
                        });
                        crate::types::Message::tool(&call.id, &error_obj.to_string())
                    }
                };

                messages.push(tool_message);
            }

            // Add assistant message with tool_calls to the conversation
            let assistant_message = crate::types::Message {
                role: "assistant".to_string(),
                content: response.choices.first()
                    .map(|c| c.message.content.clone())
                    .unwrap_or_default(),
                name: None,
                tool_calls: Some(tool_calls),
                tool_call_id: None,
            };
            messages.push(assistant_message);
        }
    }
}
