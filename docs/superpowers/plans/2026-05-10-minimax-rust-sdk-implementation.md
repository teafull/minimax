# MiniMax Rust SDK Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a minimal but functional MiniMax Rust SDK with sync HTTP, chainable chat API, streaming support, and model management.

**Architecture:** Single crate with feature flags. Core uses blocking reqwest for HTTP. Chat builder pattern for progressive request construction. SSE streaming via futures when `stream` feature is enabled.

**Tech Stack:** Rust, reqwest (blocking), serde, thiserror, futures (optional)

---

## File Structure

```
src/
├── lib.rs           # Module exports, re-exports
├── error.rs         # Error enum with thiserror
├── types.rs         # Message, ChatRequest, ChatResponse, Usage, Choice, Model
├── client.rs        # Client struct, Models struct
├── chat.rs          # ChatBuilder, ChatResponse, StreamChunk, send methods
└── models.rs        # ModelList, Model structs
```

```
Cargo.toml           # Update with proper deps and features
```

---

## Task 1: Update Cargo.toml with Dependencies

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Update Cargo.toml**

```toml
[package]
name = "MiniMax"
version = "0.1.0"
edition = "2021"

[features]
default = ["sync"]
sync = ["reqwest/blocking", "dep:futures"]
stream = ["futures"]

[dependencies]
reqwest = { version = "0.12", features = ["json", "stream"], optional = true }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
anyhow = "1"
futures = { version = "0.3", optional = true }
tokio = { version = "1", features = ["rt-multi-thread", "macros"], optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

- [ ] **Step 2: Verify cargo check passes**

Run: `cargo check`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "chore: add dependencies and features"
```

---

## Task 2: Create types.rs - Core Data Types

**Files:**
- Create: `src/types.rs`

- [ ] **Step 1: Create types.rs with all core types**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub name: Option<String>,
}

impl Message {
    pub fn user(content: &str) -> Self {
        Self {
            role: "user".to_string(),
            content: content.to_string(),
            name: None,
        }
    }

    pub fn system(content: &str) -> Self {
        Self {
            role: "system".to_string(),
            content: content.to_string(),
            name: None,
        }
    }

    pub fn assistant(content: &str) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.to_string(),
            name: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    #[serde(default)]
    pub prompt_tokens: i32,
    #[serde(default)]
    pub completion_tokens: i32,
    #[serde(default)]
    pub total_tokens: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub index: i32,
    pub message: Message,
    #[serde(default)]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
    #[serde(default)]
    pub usage: Usage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatChunkChoice {
    pub index: i32,
    pub delta: Message,
    #[serde(default)]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatChunk {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChatChunkChoice>,
    #[serde(default)]
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Model {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub owned_by: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelList {
    pub object: String,
    pub data: Vec<Model>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelDetail {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub owned_by: String,
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add src/types.rs
git commit -m "feat: add core types (Message, ChatResponse, Usage, etc)"
```

---

## Task 3: Create error.rs - Error Handling

**Files:**
- Create: `src/error.rs`

- [ ] **Step 1: Create error.rs with Error enum**

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("API error: {code} - {message}")]
    Api { code: i32, message: String },

    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Missing API key")]
    MissingApiKey,
}

pub type Result<T> = std::result::Result<T, Error>;
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add src/error.rs
git commit -m "feat: add error types with thiserror"
```

---

## Task 4: Create client.rs - Client Core

**Files:**
- Create: `src/client.rs`

- [ ] **Step 1: Create client.rs with Client and Models structs**

```rust
use crate::error::{Error, Result};
use crate::types::{Model, ModelDetail, ModelList};

const BASE_URL: &str = "https://api.minimaxi.com/v1";

pub struct Client {
    api_key: String,
    http_client: reqwest::blocking::Client,
}

impl Client {
    pub fn new(api_key: &str) -> Result<Self> {
        if api_key.is_empty() {
            return Err(Error::MissingApiKey);
        }
        let http_client = reqwest::blocking::Client::builder()
            .base_url(BASE_URL)
            .build()?;
        Ok(Self {
            api_key: api_key.to_string(),
            http_client,
        })
    }

    pub fn chat(&self) -> super::chat::ChatBuilder {
        super::chat::ChatBuilder::new(self)
    }

    pub fn models(&self) -> Models {
        Models::new(self)
    }

    pub(crate) fn request<T>(&self, request: reqwest::blocking::RequestBuilder) -> Result<T>
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

        response.json().map_err(Error::InvalidResponse)
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
        let request = self.client.http_client.get("/models");
        self.client.request(request)
    }

    pub fn get(&self, model_id: &str) -> Result<ModelDetail> {
        let request = self.client.http_client.get(&format!("/models/{model_id}"));
        self.client.request(request)
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add src/client.rs
git commit -m "feat: add Client and Models structs"
```

---

## Task 5: Create chat.rs - Chat Builder and Responses

**Files:**
- Create: `src/chat.rs`

- [ ] **Step 1: Create chat.rs with ChatBuilder**

```rust
use crate::client::Client;
use crate::error::{Error, Result};
use crate::types::{ChatChunk, ChatResponse, Message};
use std::borrow::Cow;

pub struct ChatBuilder<'a> {
    client: &'a Client,
    model: String,
    messages: Vec<Message>,
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

    pub fn messages<T: Into<Cow<'a, [Message]>>>(mut self, messages: T) -> Self {
        self.messages = messages.into().into_owned();
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

        let request = self.client.http_client
            .post("/chat/completions")
            .json(&serde_json::json!({
                "model": self.model,
                "messages": self.messages,
                "stream": self.stream,
                "max_completion_tokens": self.max_completion_tokens,
                "temperature": self.temperature,
                "top_p": self.top_p,
            }));

        self.client.request(request)
    }

    #[cfg(feature = "stream")]
    pub fn send_stream(self) -> Result<impl Iterator<Item = Result<ChatChunk>>> {
        if self.model.is_empty() {
            return Err(Error::InvalidResponse("model is required".to_string()));
        }
        if self.messages.is_empty() {
            return Err(Error::InvalidResponse("messages cannot be empty".to_string()));
        }

        let request = self.client.http_client
            .post("/chat/completions")
            .json(&serde_json::json!({
                "model": self.model,
                "messages": self.messages,
                "stream": true,
                "max_completion_tokens": self.max_completion_tokens,
                "temperature": self.temperature,
                "top_p": self.top_p,
            }));

        let response = request
            .header("Authorization", format!("Bearer {}", self.client.api_key))
            .send()?;

        if !response.status().is_success() {
            let code = response.status().as_u16() as i32;
            let message = response.text().unwrap_or_default();
            return Err(Error::Api { code, message });
        }

        let stream = response.bytes_stream().map(|chunk| {
            let bytes = chunk.map_err(Error::Request)?;
            let text = String::from_utf8_lossy(&bytes);
            let line = text.strip_prefix("data: ").unwrap_or(&text);
            if line.trim() == "[DONE]" {
                return Err(Error::InvalidResponse("stream ended".to_string()));
            }
            serde_json::from_str(line)
                .map_err(|e| Error::InvalidResponse(format!("failed to parse chunk: {}", e)))
        });

        Ok(stream)
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add src/chat.rs
git commit -m "feat: add ChatBuilder with send and send_stream methods"
```

---

## Task 6: Create lib.rs - Module Exports

**Files:**
- Create: `src/lib.rs`

- [ ] **Step 1: Create lib.rs with module exports and send_content helper**

```rust
mod chat;
mod client;
mod error;
mod types;

pub use client::Client;
pub use error::{Error, Result};
pub use types::{ChatChunk, ChatResponse, Choice, Message, Model, ModelDetail, ModelList, Usage};

pub trait ChatSend {
    fn send_content(self) -> error::Result<String>;
}

impl ChatSend for chat::ChatBuilder<'_, '_> {
    fn send_content(self) -> error::Result<String> {
        let response = self.send()?;
        let content = response
            .choices
            .first()
            .ok_or_else(|| error::Error::InvalidResponse("no choices in response".to_string()))?
            .message
            .content
            .clone();
        Ok(content)
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add src/lib.rs
git commit -m "feat: add lib.rs with module exports and ChatSend trait"
```

---

## Task 7: Verify Full Build

**Files:**
- None (verification only)

- [ ] **Step 1: Run cargo build**

Run: `cargo build`
Expected: Successful build

- [ ] **Step 2: Run cargo test**

Run: `cargo test`
Expected: Tests pass (may have placeholder test still)

- [ ] **Step 3: Run with clippy**

Run: `cargo clippy --all-features`
Expected: No warnings or errors

---

## Task 8: Update Existing Placeholder Test

**Files:**
- Modify: `src/lib.rs` (remove placeholder, add real test structure)

- [ ] **Step 1: Remove placeholder test from lib.rs**

The existing placeholder test should be replaced with documentation tests or integration tests.

---

## Spec Coverage Check

| Spec Section | Tasks |
|--------------|-------|
| 同步阻塞设计 | Task 1, 4 (reqwest blocking) |
| 组合器链式调用 | Task 5 (ChatBuilder) |
| thiserror 错误处理 | Task 3 |
| 流式 SSE | Task 5 (send_stream) |
| 双响应类型 (完整+简化) | Task 6 (send_content) |
| 统一客户端接口 | Task 4 (models()) |
| API Key 构造函数 | Task 4 (Client::new) |
| Feature flags | Task 1 |

All spec items are covered.

---

## Plan Complete

Two execution options:

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?
