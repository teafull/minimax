# Workspace 重构实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 MiniMax Rust SDK 重构为 workspace 模式，创建 openai、anthropic 独立库和 minimax 测试程序。

**Architecture:** Rust workspace 模式，三个 crate：openai（OpenAI 兼容）、anthropic（Anthropic 兼容）、minimax（测试主程序）。

**Tech Stack:** Rust, reqwest, serde, thiserror

---

## File Structure

```
minimax-workspace/
├── Cargo.toml (workspace)
├── Cargo.lock
└── crates/
    ├── openai/
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── error.rs
    │       ├── types.rs
    │       ├── chat.rs
    │       └── client.rs
    ├── anthropic/
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── types.rs
    │       ├── client.rs
    └── minimax/
        ├── Cargo.toml
        └── src/
            └── main.rs
```

---

## Task 1: Create Workspace Root

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `crates/` directory structure

- [ ] **Step 1: Create workspace Cargo.toml**

```toml
[workspace]
resolver = "2"
members = [
    "crates/openai",
    "crates/anthropic",
    "crates/minimax",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
```

- [ ] **Step 2: Create directory structure**

```bash
mkdir -p crates/openai/src
mkdir -p crates/anthropic/src
mkdir -p crates/minimax/src
```

- [ ] **Step 3: Verify workspace Cargo.toml**

Run: `cargo metadata --format-version 1 2>&1 | head -20`
Expected: Shows 3 workspace members

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml && git commit -m "feat: create workspace root"
```

---

## Task 2: Create openai Crate

**Files:**
- Create: `crates/openai/Cargo.toml`
- Create: `crates/openai/src/error.rs`
- Create: `crates/openai/src/types.rs`
- Create: `crates/openai/src/chat.rs`
- Create: `crates/openai/src/client.rs`
- Create: `crates/openai/src/lib.rs`

- [ ] **Step 1: Create crates/openai/Cargo.toml**

```toml
[package]
name = "openai"
version.workspace = true
edition.workspace = true

[dependencies]
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
```

- [ ] **Step 2: Create crates/openai/src/error.rs**

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

- [ ] **Step 3: Create crates/openai/src/types.rs**

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
    pub prompt_tokens: u32,
    #[serde(default)]
    pub completion_tokens: u32,
    #[serde(default)]
    pub total_tokens: u32,
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
```

- [ ] **Step 4: Create crates/openai/src/chat.rs**

```rust
use crate::client::Client;
use crate::error::{Error, Result};
use crate::types::{ChatResponse, Message};

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

    pub fn messages(mut self, messages: Vec<Message>) -> Self {
        self.messages = messages;
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
            .post("/v1/chat/completions")
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
}
```

- [ ] **Step 5: Create crates/openai/src/client.rs**

```rust
use crate::error::{Error, Result};
use crate::types::{ChatResponse, ModelList};

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
            .base_url("https://api.minimaxi.com")
            .build()?;
        Ok(Self {
            api_key: api_key.to_string(),
            http_client,
        })
    }

    pub fn chat(&self) -> super::ChatBuilder {
        super::ChatBuilder::new(self)
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
}
```

- [ ] **Step 6: Create crates/openai/src/lib.rs**

```rust
mod chat;
mod client;
mod error;
mod types;

pub use chat::ChatBuilder;
pub use client::Client;
pub use error::{Error, Result};
pub use types::{ChatResponse, Choice, Message, Model, ModelList, Usage};
```

- [ ] **Step 7: Verify openai crate compiles**

Run: `cd crates/openai && cargo check`
Expected: No errors

- [ ] **Step 8: Commit**

```bash
git add crates/openai/ && git commit -m "feat: create openai crate"
```

---

## Task 3: Create anthropic Crate

**Files:**
- Create: `crates/anthropic/Cargo.toml`
- Create: `crates/anthropic/src/types.rs`
- Create: `crates/anthropic/src/client.rs`
- Create: `crates/anthropic/src/lib.rs`

- [ ] **Step 1: Create crates/anthropic/Cargo.toml**

```toml
[package]
name = "anthropic"
version.workspace = true
edition.workspace = true

[dependencies]
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
```

- [ ] **Step 2: Create crates/anthropic/src/types.rs**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Message {
    pub fn user(content: &str) -> Self {
        Self {
            role: "user".to_string(),
            content: content.to_string(),
        }
    }

    pub fn assistant(content: &str) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    #[serde(default)]
    pub input_tokens: u32,
    #[serde(default)]
    pub output_tokens: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TextBlock {
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ThinkingBlock {
    pub thinking: String,
    #[serde(default)]
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text(TextBlock),
    #[serde(rename = "thinking")]
    Thinking(ThinkingBlock),
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessageResponse {
    pub id: String,
    pub type_: String,
    pub role: String,
    pub model: String,
    pub content: Vec<ContentBlock>,
    #[serde(default)]
    pub stop_reason: Option<String>,
    #[serde(default)]
    pub usage: Usage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Model {
    pub id: String,
    pub created_at: String,
    pub display_name: String,
    pub type_: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelList {
    pub data: Vec<Model>,
    #[serde(default)]
    pub first_id: Option<String>,
    #[serde(default)]
    pub has_more: bool,
    #[serde(default)]
    pub last_id: Option<String>,
}
```

- [ ] **Step 3: Create crates/anthropic/src/client.rs**

```rust
use crate::error::{Error, Result};
use crate::types::{MessageResponse, ModelList};

pub struct AnthropicClient {
    api_key: String,
    http_client: reqwest::blocking::Client,
}

impl AnthropicClient {
    pub fn new(api_key: &str) -> Result<Self> {
        if api_key.is_empty() {
            return Err(Error::MissingApiKey);
        }
        let http_client = reqwest::blocking::Client::builder()
            .build()?;
        Ok(Self {
            api_key: api_key.to_string(),
            http_client,
        })
    }

    pub fn anthropic(&self) -> ChatBuilder {
        ChatBuilder::new(self)
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
}

pub struct ChatBuilder<'a> {
    client: &'a AnthropicClient,
    model: String,
    messages: Vec<super::types::Message>,
    system: Option<String>,
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
            max_tokens: 1024,
            temperature: None,
            top_p: None,
        }
    }

    pub fn model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }

    pub fn messages(mut self, messages: Vec<super::types::Message>) -> Self {
        self.messages = messages;
        self
    }

    pub fn system(mut self, system: &str) -> Self {
        self.system = Some(system.to_string());
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

        let request = self.client.http_client
            .post("https://api.minimaxi.com/anthropic/v1/messages")
            .json(&serde_json::json!({
                "model": self.model,
                "messages": self.messages,
                "system": self.system,
                "max_tokens": self.max_tokens,
                "temperature": self.temperature,
                "top_p": self.top_p,
            }));

        self.client.request(request)
    }
}
```

- [ ] **Step 4: Create crates/anthropic/src/lib.rs**

```rust
mod client;
mod error;
mod types;

pub use client::{AnthropicClient, ChatBuilder};
pub use error::{Error, Result};
pub use types::{Message, MessageResponse, Model, ModelList, Usage};
```

- [ ] **Step 5: Verify anthropic crate compiles**

Run: `cd crates/anthropic && cargo check`
Expected: No errors

- [ ] **Step 6: Commit**

```bash
git add crates/anthropic/ && git commit -m "feat: create anthropic crate"
```

---

## Task 4: Create minimax Test Program

**Files:**
- Create: `crates/minimax/Cargo.toml`
- Create: `crates/minimax/src/main.rs`

- [ ] **Step 1: Create crates/minimax/Cargo.toml**

```toml
[package]
name = "minimax"
version.workspace = true
edition.workspace = true

[[bin]]
name = "minimax"
path = "src/main.rs"

[dependencies]
openai = { path = "../openai" }
anthropic = { path = "../anthropic" }
```

- [ ] **Step 2: Create crates/minimax/src/main.rs**

```rust
fn main() {
    let api_key = std::env::var("MINIMAX_API_KEY")
        .expect("MINIMAX_API_KEY must be set");

    println!("Testing MiniMax API with OpenAI format...");
    test_openai(&api_key);

    println!("\nTesting MiniMax API with Anthropic format...");
    test_anthropic(&api_key);
}

fn test_openai(api_key: &str) {
    let client = openai::Client::new(api_key).expect("Failed to create OpenAI client");
    let response = client.chat()
        .model("MiniMax-M2.7")
        .messages(vec![openai::Message::user("Hello")])
        .send()
        .expect("OpenAI request failed");

    println!("OpenAI Response:");
    println!("  ID: {}", response.id);
    println!("  Model: {}", response.model);
    if let Some(choice) = response.choices.first() {
        println!("  Content: {}", choice.message.content);
    }
    println!("  Usage: {:?}", response.usage);
}

fn test_anthropic(api_key: &str) {
    let client = anthropic::AnthropicClient::new(api_key).expect("Failed to create Anthropic client");
    let response = client.anthropic()
        .model("MiniMax-M2.7")
        .messages(vec![anthropic::Message::user("Hello")])
        .max_tokens(1024)
        .send()
        .expect("Anthropic request failed");

    println!("Anthropic Response:");
    println!("  ID: {}", response.id);
    println!("  Model: {}", response.model);
    println!("  Content blocks: {:?}", response.content.len());
    println!("  Usage: {:?}", response.usage);
}
```

- [ ] **Step 3: Verify minimax compiles**

Run: `cd crates/minimax && cargo check`
Expected: No errors

- [ ] **Step 4: Commit**

```bash
git add crates/minimax/ && git commit -m "feat: create minimax test program"
```

---

## Task 5: Verify Workspace Build

**Files:**
- None (verification only)

- [ ] **Step 1: Build entire workspace**

Run: `cargo build --workspace`
Expected: Successful build

- [ ] **Step 2: Run tests**

Run: `cargo test --workspace`
Expected: Tests pass (if any)

- [ ] **Step 3: Verify all crates compile**

Run: `cargo check --workspace --all-targets`
Expected: No errors

---

## Task 6: Update README

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Update README.md**

Update to reflect new workspace structure and usage.

- [ ] **Step 2: Commit**

```bash
git add README.md && git commit -m "docs: update README for workspace structure"
```

---

## Spec Coverage Check

| Spec Section | Tasks |
|--------------|-------|
| Workspace 结构 | Task 1 |
| openai crate | Task 2 |
| anthropic crate | Task 3 |
| minimax 测试程序 | Task 4 |
| 验证构建 | Task 5 |
| 文档更新 | Task 6 |

All spec items are covered.

---

## Plan Complete

Two execution options:

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?
