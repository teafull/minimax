# MiniMax Rust SDK Workspace 重构设计

## 概述

将 MiniMax Rust SDK 重构为 Rust workspace 模式，三个独立 crate：
- `openai` - OpenAI 兼容接口独立库
- `anthropic` - Anthropic 兼容接口独立库
- `minimax` - 测试主程序，使用 openai 或 anthropic 接口测试 MiniMax API

## 目标

minimax 是测试程序，用于验证 MiniMax API 支持 OpenAI 和 Anthropic 两种接口格式。

## 项目结构

```
minimax-workspace/
├── Cargo.toml (workspace)
└── crates/
    ├── openai/        # 独立库 - OpenAI 兼容接口
    ├── anthropic/     # 独立库 - Anthropic 兼容接口
    └── minimax/       # 测试主程序
```

## 文件迁移

| 当前路径 | 目标路径 | 说明 |
|---------|---------|------|
| src/types.rs | crates/openai/src/types.rs | OpenAI 类型 |
| src/chat.rs | crates/openai/src/chat.rs | OpenAI ChatBuilder |
| src/client.rs | crates/openai/src/client.rs | OpenAI Client |
| src/error.rs | crates/openai/src/error.rs | OpenAI 错误类型 |
| src/anthropic_types.rs | crates/anthropic/src/types.rs | Anthropic 类型 |
| src/anthropic.rs | crates/anthropic/src/client.rs | Anthropic Client |
| src/lib.rs | crates/minimax/src/main.rs | 测试入口 |

## 依赖关系

```
minimax (测试程序)
  └── openai (path dependency)
      └── reqwest, serde, thiserror
  └── anthropic (path dependency)
      └── reqwest, serde, thiserror
```

## crate 依赖

### openai/Cargo.toml

```toml
[package]
name = "openai"
version = "0.1.0"
edition = "2021"

[dependencies]
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
```

### anthropic/Cargo.toml

```toml
[package]
name = "anthropic"
version = "0.1.0"
edition = "2021"

[dependencies]
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
```

### minimax/Cargo.toml

```toml
[package]
name = "minimax"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "minimax"
path = "src/main.rs"

[dependencies]
openai = { path = "../openai" }
anthropic = { path = "../anthropic" }
```

## API 设计

### OpenAI 接口

```rust
// crates/openai/src/client.rs
pub struct Client {
    api_key: String,
    http_client: reqwest::blocking::Client,
}

impl Client {
    pub fn new(api_key: &str) -> Result<Self>;
    pub fn chat(&self) -> ChatBuilder;
}

// crates/openai/src/chat.rs
pub struct ChatBuilder { ... }
impl ChatBuilder {
    pub fn model(mut self, model: &str) -> Self;
    pub fn messages<T: Into<Vec<Message>>>(mut self, messages: T) -> Self;
    pub fn send(self) -> Result<ChatResponse>;
}
```

### Anthropic 接口

```rust
// crates/anthropic/src/client.rs
pub struct AnthropicClient {
    api_key: String,
    http_client: reqwest::blocking::Client,
}

impl AnthropicClient {
    pub fn new(api_key: &str) -> Result<Self>;
    pub fn anthropic(&self) -> ChatBuilder;
}

// crates/anthropic/src/chat.rs
pub struct ChatBuilder { ... }
impl ChatBuilder {
    pub fn model(mut self, model: &str) -> Self;
    pub fn messages<T: Into<Vec<Message>>>(mut self, messages: T) -> Self;
    pub fn max_tokens(mut self, tokens: i32) -> Self;
    pub fn send(self) -> Result<MessageResponse>;
}
```

## minimax 测试程序

```rust
// crates/minimax/src/main.rs

fn main() {
    let api_key = std::env::var("MINIMAX_API_KEY")
        .expect("MINIMAX_API_KEY must be set");

    println!("Testing MiniMax API with OpenAI format...");
    test_openai(&api_key);

    println!("\nTesting MiniMax API with Anthropic format...");
    test_anthropic(&api_key);
}

fn test_openai(api_key: &str) {
    let client = openai::Client::new(api_key).unwrap();
    let response = client.chat()
        .model("MiniMax-M2.7")
        .messages([openai::Message::user("Hello")])
        .send()
        .unwrap();
    println!("OpenAI Response: {:?}", response);
}

fn test_anthropic(api_key: &str) {
    let client = anthropic::AnthropicClient::new(api_key).unwrap();
    let response = client.anthropic()
        .model("MiniMax-M2.7")
        .messages([anthropic::Message::user("Hello")])
        .send()
        .unwrap();
    println!("Anthropic Response: {:?}", response);
}
```

## 实现顺序

1. 创建 workspace 结构
2. 创建 openai crate
3. 创建 anthropic crate
4. 创建 minimax 测试程序
5. 验证编译和测试
