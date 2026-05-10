# Anthropic API 支持实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 MiniMax Rust SDK 添加 Anthropic API 兼容格式支持，包括独立客户端、流式响应和模型接口。

**Architecture:** 独立 AnthropicClient，与现有 Client 完全分离。复用 reqwest blocking client。

**Tech Stack:** Rust, reqwest (blocking), serde, thiserror

---

## File Structure

```
src/
├── lib.rs              # 更新模块导出
├── anthropic_types.rs  # Create: Anthropic 专用类型
└── anthropic.rs       # Create: AnthropicClient, AnthropicChatBuilder, AnthropicModels
```

---

## Task 1: Create anthropic_types.rs - Anthropic Types

**Files:**
- Create: `src/anthropic_types.rs`

- [ ] **Step 1: Create anthropic_types.rs with all Anthropic types**

```rust
use crate::types::Usage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: String,
}

impl AnthropicMessage {
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
pub struct MessageResponse {
    pub id: String,
    #[serde(rename = "type")]
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
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text(TextBlock),
    #[serde(rename = "thinking")]
    Thinking(ThinkingBlock),
}

#[derive(Debug, Clone, Deserialize)]
pub struct TextBlock {
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ThinkingBlock {
    pub thinking: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContentBlockDelta {
    pub index: i32,
    pub delta: Delta,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Delta {
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub thinking: Option<String>,
    #[serde(default)]
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContentBlockStop {
    pub index: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessageDelta {
    pub delta: MessageDeltaContent,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessageDeltaContent {
    #[serde(default)]
    pub stop_reason: Option<String>,
    #[serde(default)]
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessageStop {
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicModelList {
    pub data: Vec<AnthropicModel>,
    #[serde(default)]
    pub first_id: Option<String>,
    #[serde(default)]
    pub has_more: bool,
    #[serde(default)]
    pub last_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicModel {
    pub id: String,
    pub created_at: String,
    pub display_name: String,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicModelDetail {
    pub id: String,
    pub created_at: String,
    pub display_name: String,
    #[serde(rename = "type")]
    pub type_: String,
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add src/anthropic_types.rs
git commit -m "feat: add Anthropic API types (Message, Response, ContentBlock, Models)"
```

---

## Task 2: Create anthropic.rs - AnthropicClient

**Files:**
- Create: `src/anthropic.rs`

- [ ] **Step 1: Create anthropic.rs with AnthropicClient, AnthropicChatBuilder, AnthropicModels**

```rust
use crate::anthropic_types::{
    AnthropicMessage, AnthropicModelDetail, AnthropicModelList, ContentBlock, MessageResponse,
};
use crate::error::{Error, Result};

const ANTHROPIC_BASE_URL: &str = "https://api.minimaxi.com/anthropic/v1";

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

    pub fn anthropic(&self) -> AnthropicChatBuilder {
        AnthropicChatBuilder::new(self)
    }

    pub fn models(&self) -> AnthropicModels {
        AnthropicModels::new(self)
    }

    pub(crate) fn request<T>(&self, request: reqwest::blocking::RequestBuilder) -> Result<T>
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

pub struct AnthropicChatBuilder<'a> {
    client: &'a AnthropicClient,
    model: String,
    messages: Vec<AnthropicMessage>,
    system: Option<String>,
    stream: bool,
    max_tokens: i32,
    temperature: Option<f64>,
    top_p: Option<f64>,
}

impl<'a> AnthropicChatBuilder<'a> {
    pub fn new(client: &'a AnthropicClient) -> Self {
        Self {
            client,
            model: String::new(),
            messages: Vec::new(),
            system: None,
            stream: false,
            max_tokens: 1024,
            temperature: None,
            top_p: None,
        }
    }

    pub fn model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }

    pub fn messages<T: Into<Vec<AnthropicMessage>>>(mut self, messages: T) -> Self {
        self.messages = messages.into();
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

    pub fn temperature(mut self, temp: f64) -> Self {
        self.temperature = Some(temp);
        self
    }

    pub fn top_p(mut self, top_p: f64) -> Self {
        self.top_p = Some(top_p);
        self
    }

    pub fn send(self) -> Result<MessageResponse> {
        if self.model.is_empty() {
            return Err(Error::InvalidResponse("model is required".to_string()));
        }
        if self.messages.is_empty() {
            return Err(Error::InvalidResponse("messages cannot be empty".to_string()));
        }

        let request_body = serde_json::json!({
            "model": self.model,
            "messages": self.messages,
            "system": self.system,
            "stream": self.stream,
            "max_tokens": self.max_tokens,
            "temperature": self.temperature,
            "top_p": self.top_p,
        });

        let request = self.client.http_client
            .post(format!("{}/messages", ANTHROPIC_BASE_URL))
            .json(&request_body);

        self.client.request(request)
    }

    pub fn send_stream(self) -> Result<impl Iterator<Item = Result<ContentBlock>>> {
        if self.model.is_empty() {
            return Err(Error::InvalidResponse("model is required".to_string()));
        }
        if self.messages.is_empty() {
            return Err(Error::InvalidResponse("messages cannot be empty".to_string()));
        }

        let request_body = serde_json::json!({
            "model": self.model,
            "messages": self.messages,
            "system": self.system,
            "stream": true,
            "max_tokens": self.max_tokens,
            "temperature": self.temperature,
            "top_p": self.top_p,
        });

        let response = self.client.http_client
            .post(format!("{}/messages", ANTHROPIC_BASE_URL))
            .header("x-api-key", &self.client.api_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()?;

        if !response.status().is_success() {
            let code = response.status().as_u16() as i32;
            let message = response.text().unwrap_or_default();
            return Err(Error::Api { code, message });
        }

        let text = response.text()?;
        let lines: Vec<String> = text.lines()
            .filter(|l| !l.is_empty() && l.starts_with("data: "))
            .map(|l| l.strip_prefix("data: ").unwrap_or(l).to_string())
            .collect();

        let chunks: Vec<Result<ContentBlock>> = lines
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

pub struct AnthropicModels<'a> {
    client: &'a AnthropicClient,
}

impl<'a> AnthropicModels<'a> {
    fn new(client: &'a AnthropicClient) -> Self {
        Self { client }
    }

    pub fn list(&self) -> Result<AnthropicModelList> {
        let request = self.client.http_client
            .get(format!("{}/models", ANTHROPIC_BASE_URL));
        self.client.request(request)
    }

    pub fn get(&self, model_id: &str) -> Result<AnthropicModelDetail> {
        let request = self.client.http_client
            .get(format!("{}/models/{}", ANTHROPIC_BASE_URL, model_id));
        self.client.request(request)
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add src/anthropic.rs
git commit -m "feat: add AnthropicClient with ChatBuilder and Models"
```

---

## Task 3: Update lib.rs - Module Exports

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Add anthropic modules and exports**

Add these lines to src/lib.rs:

```rust
mod anthropic;
mod anthropic_types;
```

And add these to the pub use section:

```rust
pub use anthropic::AnthropicClient;
pub use anthropic_types::{
    AnthropicMessage, AnthropicModel, AnthropicModelDetail, AnthropicModelList,
    ContentBlock, MessageResponse, TextBlock, ThinkingBlock,
};
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add src/lib.rs
git commit -m "feat: export AnthropicClient and types"
```

---

## Task 4: Verify Full Build

**Files:**
- None (verification only)

- [ ] **Step 1: Run cargo build**

Run: `cargo build`
Expected: Successful build

- [ ] **Step 2: Run cargo test**

Run: `cargo test`
Expected: All tests pass

- [ ] **Step 3: Run with clippy**

Run: `cargo clippy --all-features`
Expected: No errors (warnings acceptable)

---

## Task 5: Add Anthropic Tests

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Add unit tests for AnthropicMessage**

Add to the tests module in lib.rs:

```rust
#[test]
fn test_anthropic_message_user() {
    let msg = AnthropicMessage::user("hello");
    assert_eq!(msg.role, "user");
    assert_eq!(msg.content, "hello");
}

#[test]
fn test_anthropic_message_assistant() {
    let msg = AnthropicMessage::assistant("hi there");
    assert_eq!(msg.role, "assistant");
    assert_eq!(msg.content, "hi there");
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test`
Expected: All tests pass

- [ ] **Step 3: Commit**

```bash
git add src/lib.rs
git commit -m "test: add AnthropicMessage unit tests"
```

---

## Spec Coverage Check

| Spec Section | Tasks |
|--------------|-------|
| AnthropicClient | Task 2 |
| AnthropicChatBuilder | Task 2 |
| AnthropicModels | Task 2 |
| AnthropicMessage | Task 1 |
| MessageResponse | Task 1 |
| ContentBlock (Text, Thinking) | Task 1 |
| Streaming support | Task 2 (send_stream) |
| Model types | Task 1 |
| Module exports | Task 3 |

All spec items are covered.

---

## Plan Complete

Two execution options:

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?
