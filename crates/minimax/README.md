# MiniMax Test Program

This crate contains test programs demonstrating OpenAI and Anthropic API compatibility with MiniMax.

## 运行测试

```bash
cargo run -p minimax
```

## 测试内容

### 1. OpenAI 格式测试

- **Models API** - 列出模型列表和获取模型详情
- **Chat** - 文本对话
- **Streaming** - 流式响应
- **Function Calling** - 函数调用（天气查询示例）

### 2. Anthropic 格式测试

- **Models API** - 列出模型列表和获取模型详情
- **Chat** - 文本对话
- **Streaming** - 流式响应
- **Function Calling** - 函数调用（天气查询示例）

## Function Calling 示例

### OpenAI 格式

```rust
use openai::{Client, Message, Tool, FunctionDefinition, ToolCall, ToolCallFunction};

// 定义工具
let get_weather_tool = Tool {
    type_: "function".to_string(),
    function: FunctionDefinition {
        name: "get_weather".to_string(),
        description: "获取城市天气信息".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "location": { "type": "string", "description": "城市名称" }
            },
            "required": ["location"]
        }),
    },
};

// 第一次请求：获取工具调用
let response = client.chat()
    .model("MiniMax-M2.7")
    .messages(vec![Message::user("北京今天天气怎么样？")])
    .tools(vec![get_weather_tool.clone()])
    .send()?;

// 解析 tool_call_id 和参数，调用真实 API
// ...

// 第二次请求：带上工具结果
let response = client.chat()
    .model("MiniMax-M2.7")
    .messages(vec![
        Message::user("北京今天天气怎么样？"),
        Message { role: "assistant".to_string(), content: "".to_string(), name: None,
            tool_calls: Some(vec![ToolCall { id: tool_call_id, type_: "function".to_string(),
                function: ToolCallFunction { name: "get_weather".to_string(),
                    arguments: r#"{"location":"北京"}"#.to_string() } }]),
            tool_call_id: None },
        Message::tool(&tool_call_id, weather_result),
    ])
    .tools(vec![get_weather_tool])
    .send()?;
```

### Anthropic 格式

```rust
use anthropic::{AnthropicClient, Message, Tool};

// 定义工具
let get_weather_tool = Tool {
    name: "get_weather".to_string(),
    description: Some("获取城市天气信息".to_string()),
    input_schema: serde_json::json!({
        "type": "object",
        "properties": {
            "location": { "type": "string", "description": "城市名称" }
        },
        "required": ["location"]
    }),
};

// 第一次请求
let response = client.anthropic()
    .model("MiniMax-M2.7")
    .messages(vec![Message::user("北京今天天气怎么样？")])
    .tools(vec![get_weather_tool.clone()])
    .max_tokens(1024)
    .send()?;

// 从 ContentBlock::ToolUse 中提取 id 和参数，调用真实 API
// ...

// 第二次请求：带上工具结果
let response = client.anthropic()
    .model("MiniMax-M2.7")
    .messages(vec![
        Message::user("北京今天天气怎么样？"),
        Message::assistant(""),
        Message::tool(&tool_use_id, weather_result),
    ])
    .tools(vec![get_weather_tool])
    .max_tokens(1024)
    .send()?;
```

## API 端点

- **OpenAI**: `https://api.minimaxi.com/v1/chat/completions`
- **Anthropic**: `https://api.minimaxi.com/anthropic/v1/messages`

## 天气查询 API

测试中使用的天气接口: `https://uapis.cn/api/v1/misc/weather?city={城市名}`
