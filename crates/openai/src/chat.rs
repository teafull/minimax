use crate::client::Client;
use crate::error::{Error, Result};
use crate::types::{ChatChunk, ChatResponse, Message, Tool};

pub struct ChatBuilder<'a> {
    client: &'a Client,
    model: String,
    messages: Vec<Message>,
    tools: Vec<Tool>,
    stream: bool,
    max_completion_tokens: Option<i32>,
    temperature: Option<f64>,
    top_p: Option<f64>,
}

impl<'a> ChatBuilder<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self {
            client,
            model: String::new(),
            messages: Vec::new(),
            tools: Vec::new(),
            stream: false,
            max_completion_tokens: None,
            temperature: None,
            top_p: None,
        }
    }

    pub fn model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }

    pub fn messages(mut self, messages: Vec<Message>) -> Self {
        self.messages = messages;
        self
    }

    pub fn tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = tools;
        self
    }

    pub fn stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    pub fn max_completion_tokens(mut self, tokens: i32) -> Self {
        self.max_completion_tokens = Some(tokens);
        self
    }

    pub fn temperature(mut self, temp: f64) -> Self {
        self.temperature = Some(temp);
        self
    }

    pub fn top_p(mut self, top_p: f64) -> Self {
        self.top_p = Some(top_p);
        self
    }

    pub fn send(self) -> Result<ChatResponse> {
        if self.model.is_empty() {
            return Err(Error::InvalidResponse("model is required".to_string()));
        }
        if self.messages.is_empty() {
            return Err(Error::InvalidResponse("messages cannot be empty".to_string()));
        }

        let url = format!("{}/chat/completions", self.client.base_url());
        let mut body = serde_json::json!({
            "model": self.model,
            "messages": self.messages,
            "stream": self.stream,
            "max_completion_tokens": self.max_completion_tokens,
            "temperature": self.temperature,
            "top_p": self.top_p,
        });

        if !self.tools.is_empty() {
            body["tools"] = serde_json::to_value(&self.tools).unwrap();
        }

        let request = self.client.http_client()
            .post(&url)
            .json(&body);

        self.client.request(request)
    }

    pub fn send_stream(self) -> Result<impl Iterator<Item = Result<ChatChunk>>> {
        if self.model.is_empty() {
            return Err(Error::InvalidResponse("model is required".to_string()));
        }
        if self.messages.is_empty() {
            return Err(Error::InvalidResponse("messages cannot be empty".to_string()));
        }

        let url = format!("{}/chat/completions", self.client.base_url());
        let mut body = serde_json::json!({
            "model": self.model,
            "messages": self.messages,
            "stream": true,
            "max_completion_tokens": self.max_completion_tokens,
            "temperature": self.temperature,
            "top_p": self.top_p,
        });

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

        let chunks: Vec<Result<ChatChunk>> = lines
            .into_iter()
            .filter(|l| l.trim() != "[DONE]")
            .map(|line| {
                serde_json::from_str(&line)
                    .map_err(|e| Error::InvalidResponse(format!("failed to parse chunk: {}", e)))
            })
            .collect();

        Ok(chunks.into_iter())
    }
}
