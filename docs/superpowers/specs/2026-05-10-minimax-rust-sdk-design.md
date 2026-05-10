# MiniMax Rust SDK 设计文档

## 概述

MiniMax Rust SDK 提供对 MiniMax LLM API 的访问，支持 OpenAI 和 Anthropic API 兼容格式。采用同步阻塞设计，组合器风格 API。

## 设计决策

| 决策项 | 选择 | 理由 |
|--------|------|------|
| 运行时 | 同步阻塞 | 适合脚本、CLI 工具 |
| API 风格 | 组合器链式调用 | 渐进式构建请求 |
| 错误处理 | thiserror 自定义枚举 | 类型安全 |
| 流式支持 | 是 | SSE 流式响应 |
| 响应类型 | 双模式 (完整 + 简化) | 灵活使用 |
| 模型接口 | 统一客户端接口 | 一致体验 |
| 认证方式 | 构造函数传入 | 简单明确 |
| 包结构 | 单 crate + feature flags | 平衡灵活性与复杂度 |

## 项目结构

```
src/
├── lib.rs           # 入口，导出模块
├── error.rs         # 自定义错误类型
├── types.rs         # 共享类型
├── client.rs        # Client 主结构
├── chat.rs          # 聊天接口
└── models.rs        # 模型接口
```

## API 设计

### 客户端创建

```rust
use minimax::Client;

let client = Client::new("your-api-key")?;
```

### 聊天接口

```rust
// 完整响应
let response = client.chat()
    .model("MiniMax-M2.7")
    .messages([Message::user("你好")])
    .send()?;

println!("{}", response.choices[0].message.content);
println!("Tokens: {}", response.usage.total_tokens);

// 简化响应
let content = client.chat()
    .model("MiniMax-M2.7")
    .messages([Message::user("你好")])
    .send_content()?;

// 流式响应
let mut stream = client.chat()
    .model("MiniMax-M2.7")
    .messages([Message::user("讲个故事")])
    .stream(true)?
    .send_stream();

for chunk in stream {
    println!("{}", chunk?.choices[0].delta.content);
}
```

### 模型接口

```rust
// 获取模型列表
let models = client.models().list()?;

// 获取单个模型
let model = client.models().get("MiniMax-M2.7")?;
```

## 核心类型

### Message

```rust
pub struct Message {
    pub role: String,
    pub content: String,
    pub name: Option<String>,
}

impl Message {
    pub fn user(content: &str) -> Self;
    pub fn system(content: &str) -> Self;
    pub fn assistant(content: &str) -> Self;
}
```

### ChatResponse

```rust
pub struct ChatResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

pub struct Choice {
    pub index: i32,
    pub message: Message,
    pub finish_reason: Option<String>,
}

pub struct Usage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}
```

### 错误类型

```rust
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
```

## Feature Flags

```toml
[features]
default = ["sync"]
sync = ["reqwest/blocking"]
stream = ["futures"]
```

- `sync` (默认): 同步阻塞 HTTP 客户端
- `stream`: 流式响应支持，启用 futures 依赖

## 依赖

```toml
[dependencies]
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
futures = { version = "0.3", optional = true }
tokio = { version = "1", optional = true }
```

## 实现顺序

1. `types.rs` - 定义所有类型
2. `error.rs` - 错误类型
3. `client.rs` - 客户端核心
4. `chat.rs` - 聊天接口
5. `models.rs` - 模型接口
6. `lib.rs` - 模块导出
7. 测试和文档
