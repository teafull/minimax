# Tool Executor Design for OpenAI Crate

## Overview

Extend `crates/openai` with a high-level tool calling API that handles the full tool execution loop: parsing tool calls, executing them via a user-provided executor, and feeding results back to the model automatically. This simplifies `crates/minimax` function calling tests from ~100 lines to ~20 lines.

## Architecture

### New Trait: `ToolExecutor`

```rust
pub trait ToolExecutor: Send + Sync {
    fn execute(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Box<dyn std::error::Error + Send + Sync>;
}
```

- `Send + Sync`: Required because the executor may be invoked from a multi-threaded context
- Returns `String`: Tool result serialized as JSON string, fed back to the model
- Errors are returned as strings and passed to the model as tool result content

### New Struct: `ChatWithExecutor`

Wraps `ChatBuilder` with tools and an executor, providing the automatic tool-loop logic.

### Flow

```
1. client.chat().tools_with_executor(tools, executor).send()
        |
        v
2. Build request with tools, send to API
        |
        v
3. Receive response
        |
        v
4. Is tool_calls present?
    No  --> return response (final answer)
    Yes --> continue
        |
        v
5. For each tool_call:
       executor.execute(tool_name, arguments) -> result (or error string)
       Add Tool message to conversation
        |
        v
6. Go to step 2 with updated messages
        |
        v
7. Repeat until no tool_calls in response
```

## API

### Additions to `crates/openai/src/client.rs`

```rust
// New method on Client
pub fn chat_with_executor(
    &self,
    tools: Vec<types::Tool>,
    executor: Arc<dyn ToolExecutor>,
) -> ChatWithExecutor<'_>
```

### New struct `ChatWithExecutor`

```rust
pub struct ChatWithExecutor<'a> {
    builder: ChatBuilder<'a>,
    tools: Vec<types::Tool>,
    executor: Arc<dyn ToolExecutor>,
}
```

### Methods on `ChatWithExecutor`

```rust
impl<'a> ChatWithExecutor<'a> {
    pub fn model(self, model: &str) -> Self { ... }
    pub fn messages(self, messages: Vec<Message>) -> Self { ... }
    pub fn temperature(self, temperature: f64) -> Self { ... }
    pub fn top_p(self, top_p: f64) -> Self { ... }
    pub fn max_completion_tokens(self, tokens: i32) -> Self { ... }
    pub fn send(self) -> Result<ChatResponse>
}
```

## Error Handling

When `executor.execute()` returns an error:
- The error message is serialized as a string and passed to the model as the tool result
- Example: `r#"{"error": "Failed to call weather API: connection timeout"}"#`
- The model decides how to handle the error (retry, ask user, etc.)

## Multi-turn Support

The executor loop continues until the model returns a response without `tool_calls`. There is no hard limit on iterations; callers should set an appropriate `max_completion_tokens` to prevent infinite loops.

## Example Usage

```rust
use openai::{Client, Message, Tool, FunctionDefinition, ToolExecutor};
use std::sync::Arc;
use serde_json::Value;

struct WeatherExecutor;

impl ToolExecutor for WeatherExecutor {
    fn execute(
        &self,
        tool_name: &str,
        arguments: Value,
    ) -> Box<dyn std::error::Error + Send + Sync> {
        if tool_name == "get_weather" {
            let location = arguments["location"].as_str().unwrap();
            let result = call_weather_api(location);
            Ok(result)
        } else {
            Err(format!("Unknown tool: {}", tool_name).into())
        }
    }
}

let weather_tool = Tool {
    type_: "function".to_string(),
    function: FunctionDefinition {
        name: "get_weather".to_string(),
        description: "Get weather for a city".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "location": {"type": "string", "description": "City name"}
            },
            "required": ["location"]
        }),
    },
};

let response = client.chat_with_executor(
    vec![weather_tool],
    Arc::new(WeatherExecutor),
)
.model("MiniMax-M2.7")
.messages(vec![Message::user("北京今天天气怎么样？")])
.max_completion_tokens(4096)
.send()?;
```

## Changes to `crates/minimax`

The `test_openai_function_call` function will be refactored to:
1. Define a `WeatherExecutor` struct implementing `ToolExecutor`
2. Use `client.chat_with_executor().model().messages().send()` instead of manual tool-call parsing and re-request loop
3. Estimated reduction: from ~100 lines to ~20 lines

## Files to Modify

- `crates/openai/src/client.rs` - Add `ToolExecutor` trait, `ChatWithExecutor` struct and methods
- `crates/openai/src/lib.rs` - Re-export `ToolExecutor`
- `crates/minimax/src/main.rs` - Simplify `test_openai_function_call` and `test_anthropic_function_call`

## Open Questions

None remaining — design approved by user.
