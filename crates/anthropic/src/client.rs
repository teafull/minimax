use crate::error::{Error, Result};
use crate::types::{MessageResponse, Model, ModelList, StreamEvent, Tool};

pub struct AnthropicClient {
    api_key: String,
    base_url: String,
}

impl AnthropicClient {
    pub fn new(api_key: &str) -> Result<Self> {
        Self::with_base_url(api_key, "https://api.minimaxi.com/anthropic/v1")
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

    pub fn anthropic(&self) -> ChatBuilder<'_> {
        ChatBuilder::new(self)
    }

    pub fn models(&self) -> Models<'_> {
        Models::new(self)
    }

    pub fn request<T>(&self, request: reqwest::blocking::RequestBuilder) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let response = request
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
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
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
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
    client: &'a AnthropicClient,
}

impl<'a> Models<'a> {
    fn new(client: &'a AnthropicClient) -> Self {
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

pub struct ChatBuilder<'a> {
    client: &'a AnthropicClient,
    model: String,
    messages: Vec<crate::types::Message>,
    system: Option<String>,
    tools: Vec<Tool>,
    max_tokens: i32,
    temperature: Option<f64>,
    top_p: Option<f64>,
}

impl<'a> ChatBuilder<'a> {
    pub fn new(client: &'a AnthropicClient) -> Self {
        Self {
            client,
            model: String::new(),
            messages: Vec::new(),
            system: None,
            tools: Vec::new(),
            max_tokens: 1024,
            temperature: None,
            top_p: None,
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

    pub fn system(mut self, system: &str) -> Self {
        self.system = Some(system.to_string());
        self
    }

    pub fn tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = tools;
        self
    }

    pub fn max_tokens(mut self, tokens: i32) -> Self {
        self.max_tokens = tokens;
        self
    }

    pub fn send(self) -> Result<MessageResponse> {
        if self.model.is_empty() {
            return Err(Error::InvalidResponse("model is required".to_string()));
        }
        if self.messages.is_empty() {
            return Err(Error::InvalidResponse("messages cannot be empty".to_string()));
        }

        let url = format!("{}/messages", self.client.base_url());
        let mut body = serde_json::json!({
            "model": self.model,
            "messages": self.messages,
            "max_tokens": self.max_tokens,
            "temperature": self.temperature,
            "top_p": self.top_p,
        });

        if self.system.is_some() {
            body["system"] = serde_json::json!(self.system);
        }
        if !self.tools.is_empty() {
            body["tools"] = serde_json::to_value(&self.tools).unwrap();
        }

        let request = self.client.http_client()
            .post(&url)
            .json(&body);

        self.client.request(request)
    }

    pub fn send_stream(self) -> Result<impl Iterator<Item = Result<StreamEvent>>> {
        if self.model.is_empty() {
            return Err(Error::InvalidResponse("model is required".to_string()));
        }
        if self.messages.is_empty() {
            return Err(Error::InvalidResponse("messages cannot be empty".to_string()));
        }

        let url = format!("{}/messages", self.client.base_url());
        let mut body = serde_json::json!({
            "model": self.model,
            "messages": self.messages,
            "stream": true,
            "max_tokens": self.max_tokens,
            "temperature": self.temperature,
            "top_p": self.top_p,
        });

        if self.system.is_some() {
            body["system"] = serde_json::json!(self.system);
        }
        if !self.tools.is_empty() {
            body["tools"] = serde_json::to_value(&self.tools).unwrap();
        }

        let request = self.client.http_client()
            .post(&url)
            .json(&body);

        let text = self.client.send_streaming_request(request)?;

        let lines: Vec<String> = text.lines()
            .filter(|l| !l.is_empty() && l.starts_with("data: "))
            .map(|l| l.strip_prefix("data: ").unwrap_or(l).to_string())
            .collect();

        let chunks: Vec<Result<StreamEvent>> = lines
            .into_iter()
            .filter(|l| l.trim() != "[DONE]")
            .map(|line| {
                serde_json::from_str::<StreamEvent>(&line)
                    .map_err(|e| Error::InvalidResponse(format!("failed to parse chunk: {} at line: {}", e, line)))
            })
            .collect();

        Ok(chunks.into_iter())
    }
}
