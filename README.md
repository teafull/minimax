# MiniMax SDK

MiniMax API 的 Rust SDK，支持 OpenAI 和 Anthropic API 兼容格式。

## 项目结构

```
minimax-workspace/
├── Cargo.toml (workspace)
└── crates/
    ├── openai/        # OpenAI 兼容接口
    ├── anthropic/     # Anthropic 兼容接口
    └── minimax/       # 测试程序
```

## 安装

```toml
[dependencies]
openai = { path = "crates/openai" }
anthropic = { path = "crates/anthropic" }
```

## 快速开始

### OpenAI 格式

```rust
use openai::{Client, Message};

let client = Client::new("your-api-key")?;
let response = client.chat()
    .model("MiniMax-M2.7")
    .messages(vec![Message::user("你好")])
    .send()?;
```

### Anthropic 格式

```rust
use anthropic::{AnthropicClient, Message};

let client = AnthropicClient::new("your-api-key")?;
let response = client.anthropic()
    .model("MiniMax-M2.7")
    .messages(vec![Message::user("你好")])
    .max_tokens(1024)
    .send()?;
```

## 测试程序

```bash
MINIMAX_API_KEY=your-api-key cargo run -p minimax
```

## 构建

```bash
cargo build --workspace
```
