# MiniMax Rust SDK - Anthropic API 兼容设计

## 概述

为 MiniMax Rust SDK 添加 Anthropic API 兼容格式支持。Anthropic 客户端与 OpenAI 客户端完全独立。

## 设计决策

| 决策项 | 选择 | 理由 |
|--------|------|------|
| 客户端结构 | 独立客户端 | AnthropicClient 与 Client 完全分离 |
| 类型体系 | 独立类型 | Anthropic 响应格式 (content blocks) 与 OpenAI 不同 |
| 流式支持 | 是 | 支持 SSE 格式的 content blocks 流式响应 |
| 模型接口 | AnthropicClient 自带 | anthropic().models() 获取 |

## 文件结构

```
src/
├── lib.rs              # 模块导出
├── client.rs          # Client (OpenAI)
├── chat.rs           # ChatBuilder (OpenAI)
├── anthropic.rs       # AnthropicClient, AnthropicChatBuilder, AnthropicModels
└── anthropic_types.rs # Anthropic 专用类型
```

## API 设计

### AnthropicClient

```rust
pub struct AnthropicClient {
    api_key: String,
    http_client: reqwest::blocking::Client,
}

impl AnthropicClient {
    pub fn new(api_key: &str) -> Result<Self>;
    pub fn anthropic(&self) -> AnthropicChatBuilder;
    pub fn models(&self) -> AnthropicModels;
}
```

### AnthropicChatBuilder

```rust
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

impl AnthropicChatBuilder {
    pub fn model(mut self, model: &str) -> Self;
    pub fn messages<T: Into<Vec<AnthropicMessage>>>(mut self, messages: T) -> Self;
    pub fn system(mut self, system: &str) -> Self;
    pub fn max_tokens(mut self, tokens: i32) -> Self;
    pub fn temperature(mut self, temp: f64) -> Self;
    pub fn top_p(mut self, top_p: f64) -> Self;
    pub fn send(self) -> Result<MessageResponse>;
    pub fn send_stream(self) -> Result<impl Iterator<Item = Result<ContentBlock>>>;
}
```

### AnthropicModels

```rust
pub struct AnthropicModels<'a> {
    client: &'a AnthropicClient,
}

impl AnthropicModels {
    pub fn list(&self) -> Result<AnthropicModelList>;
    pub fn get(&self, model_id: &str) -> Result<AnthropicModelDetail>;
}
```

## Anthropic 类型

### AnthropicMessage

```rust
pub struct AnthropicMessage {
    pub role: String,  // "user" | "assistant"
    pub content: String,
}

impl AnthropicMessage {
    pub fn user(content: &str) -> Self;
    pub fn assistant(content: &str) -> Self;
}
```

### MessageResponse

```rust
pub struct MessageResponse {
    pub id: String,
    pub type_: String,  // "message"
    pub role: String,
    pub model: String,
    pub content: Vec<ContentBlock>,
    pub stop_reason: Option<String>,
    pub usage: Usage,
}
```

### ContentBlock

```rust
pub enum ContentBlock {
    Thinking(ThinkingBlock),
    Text(TextBlock),
    ContentBlockDelta(ContentBlockDelta),
    ContentBlockStop(ContentBlockStop),
    MessageDelta(MessageDelta),
    MessageStop(MessageStop),
}

pub struct TextBlock {
    pub type_: String,  // "text"
    pub text: String,
}

pub struct ThinkingBlock {
    pub type_: String,  // "thinking"
    pub thinking: String,
}

pub struct ContentBlockDelta {
    pub type_: String,
    pub index: i32,
    pub delta: Delta,
}

pub struct Delta {
    pub type_: String,
    pub text: Option<String>,
    pub thinking: Option<String>,
    pub signature: Option<String>,
}

pub struct ContentBlockStop {
    pub type_: String,  // "content_block_stop"
    pub index: i32,
}

pub struct MessageDelta {
    pub type_: String,
    pub delta: MessageDeltaContent,
}

pub struct MessageDeltaContent {
    pub stop_reason: Option<String>,
    pub usage: Usage,
}

pub struct MessageStop {
    pub type_: String,  // "message_stop"
}
```

### AnthropicModels 类型

```rust
pub struct AnthropicModelList {
    pub data: Vec<AnthropicModel>,
    pub first_id: Option<String>,
    pub has_more: bool,
    pub last_id: Option<String>,
}

pub struct AnthropicModel {
    pub id: String,
    pub created_at: String,
    pub display_name: String,
    pub type_: String,
}

pub struct AnthropicModelDetail {
    pub id: String,
    pub created_at: String,
    pub display_name: String,
    pub type_: String,
}
```

## 实现顺序

1. `anthropic_types.rs` - Anthropic 专用类型
2. `anthropic.rs` - AnthropicClient, AnthropicChatBuilder, AnthropicModels
3. `lib.rs` - 更新模块导出
4. 测试验证

## 依赖

无需新增依赖，复用现有 reqwest blocking client。
