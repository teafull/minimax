# Tool Executor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add high-level tool calling API to `crates/openai` that automatically handles the tool execution loop

**Architecture:** Add `ToolExecutor` trait and `ChatWithExecutor` struct to `crates/openai`. `ChatWithExecutor::send()` performs the full tool-loop: send request → parse tool_calls → execute via executor → append results → repeat until final response.

**Tech Stack:** Rust, reqwest (blocking), serde_json, thiserror

---

## Task 1: Add `ToolExecutor` trait to `crates/openai/src/client.rs`

**Files:**
- Modify: `crates/openai/src/client.rs`

- [ ] **Step 1: Read current client.rs to understand line numbers**

Run: `wc -l crates/openai/src/client.rs crates/openai/src/chat.rs`

- [ ] **Step 2: Add `ToolExecutor` trait after the `Models` impl block (around line 95)**

```rust
/// Trait for executing tool calls returned by the model.
/// Implement this trait and pass it to `ChatWithExecutor` to handle
/// tool calls automatically.
pub trait ToolExecutor: Send + Sync {
    /// Execute a tool call with the given name and arguments.
    /// Returns the tool result as a JSON string, or an error string
    /// which will be passed to the model for handling.
    fn execute(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Box<dyn std::error::Error + Send + Sync>;
}
```

- [ ] **Step 3: Commit**

```bash
git add crates/openai/src/client.rs
git commit -m "feat(openai): add ToolExecutor trait"
```

---

## Task 2: Add `ChatWithExecutor` struct and methods to `crates/openai/src/client.rs`

**Files:**
- Modify: `crates/openai/src/client.rs`

- [ ] **Step 1: Add `ChatWithExecutor` struct after `ToolExecutor` trait (around line 108)**

```rust
/// Builder for chat requests with automatic tool execution.
/// Use `Client::chat_with_executor()` to create an instance.
pub struct ChatWithExecutor<'a> {
    client: &'a Client,
    tools: Vec<types::Tool>,
    executor: std::sync::Arc<dyn ToolExecutor>,
    model: String,
    messages: Vec<types::Message>,
    temperature: Option<f64>,
    top_p: Option<f64>,
    max_completion_tokens: Option<i32>,
}

impl<'a> ChatWithExecutor<'a> {
    /// Create a new ChatWithExecutor
    pub fn new(
        client: &'a Client,
        tools: Vec<types::Tool>,
        executor: std::sync::Arc<dyn ToolExecutor>,
    ) -> Self {
        Self {
            client,
            tools,
            executor,
            model: String::new(),
            messages: Vec::new(),
            temperature: None,
            top_p: None,
            max_completion_tokens: None,
        }
    }

    /// Set the model name (required before send)
    pub fn model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }

    /// Set the messages (required before send)
    pub fn messages(mut self, messages: Vec<types::Message>) -> Self {
        self.messages = messages;
        self
    }

    /// Set temperature
    pub fn temperature(mut self, temperature: f64) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set top_p
    pub fn top_p(mut self, top_p: f64) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Set max_completion_tokens
    pub fn max_completion_tokens(mut self, tokens: i32) -> Self {
        self.max_completion_tokens = Some(tokens);
        self
    }

    /// Send the request and execute tools automatically.
    /// Loops: send request -> if tool_calls present, execute each and resend
    pub fn send(self) -> crate::error::Result<types::ChatResponse> {
        if self.model.is_empty() {
            return Err(crate::error::Error::InvalidResponse("model is required".to_string()));
        }
        if self.messages.is_empty() {
            return Err(crate::error::Error::InvalidResponse("messages cannot be empty".to_string()));
        }

        let mut current_messages = self.messages;
        let mut final_response: Option<types::ChatResponse> = None;

        loop {
            // Build request body
            let mut body = serde_json::json!({
                "model": self.model,
                "messages": current_messages,
                "stream": false,
                "max_completion_tokens": self.max_completion_tokens,
                "temperature": self.temperature,
                "top_p": self.top_p,
            });

            if !self.tools.is_empty() {
                body["tools"] = serde_json::to_value(&self.tools).unwrap();
            }

            let url = format!("{}/chat/completions", self.client.base_url());
            let request = self.client.http_client()
                .post(&url)
                .json(&body);

            let response: types::ChatResponse = self.client.request(request)?;

            // Check if model wants to call tools
            let tool_calls = response.choices
                .first()
                .and_then(|c| c.tool_calls.clone());

            if tool_calls.is_none() {
                // No tool calls - this is the final response
                final_response = Some(response);
                break;
            }

            let tool_calls = tool_calls.unwrap();

            // Add assistant message with tool calls to conversation
            let assistant_msg = types::Message {
                role: "assistant".to_string(),
                content: response.choices.first()
                    .and_then(|c| c.message.content.as_ref())
                    .map(|s| s.as_str())
                    .unwrap_or("")
                    .to_string(),
                name: None,
                tool_calls: Some(tool_calls.clone()),
                tool_call_id: None,
            };
            current_messages.push(assistant_msg);

            // Execute each tool call and add results to conversation
            for call in &tool_calls {
                let args: serde_json::Value = serde_json::from_str(&call.function.arguments)
                    .unwrap_or(serde_json::Value::Null);

                let result = self.executor.execute(&call.function.name, args);

                let tool_result = match result {
                    Ok(output) => output,
                    Err(e) => format!(r#"{{"error": "{}"}}"#, e),
                };

                let tool_msg = types::Message {
                    role: "tool".to_string(),
                    content: tool_result,
                    name: None,
                    tool_calls: None,
                    tool_call_id: Some(call.id.clone()),
                };
                current_messages.push(tool_msg);
            }

            // Continue loop to send updated conversation
        }

        final_response.ok_or_else(|| {
            crate::error::Error::InvalidResponse("no response received".to_string())
        })
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add crates/openai/src/client.rs
git commit -m "feat(openai): add ChatWithExecutor with automatic tool loop"
```

---

## Task 3: Add `chat_with_executor()` method to `Client`

**Files:**
- Modify: `crates/openai/src/client.rs`

- [ ] **Step 1: Add method to `Client` impl block (after line 71, before `pub struct Models`)**

```rust
    /// Create a chat builder with automatic tool execution.
    /// The executor will be called whenever the model requests a tool.
    pub fn chat_with_executor(
        &self,
        tools: Vec<types::Tool>,
        executor: std::sync::Arc<dyn ToolExecutor>,
    ) -> ChatWithExecutor<'_> {
        ChatWithExecutor::new(self, tools, executor)
    }
```

- [ ] **Step 2: Commit**

```bash
git add crates/openai/src/client.rs
git commit -m "feat(openai): add Client::chat_with_executor method"
```

---

## Task 4: Re-export `ToolExecutor` from `crates/openai/src/lib.rs`

**Files:**
- Modify: `crates/openai/src/lib.rs`

- [ ] **Step 1: Add ToolExecutor to the pub use list (line 6-9)**

Change:
```rust
pub use chat::ChatBuilder;
pub use client::{Client, Models};
pub use error::{Error, Result};
pub use types::{ChatChunk, ChatResponse, Choice, Delta, FunctionDefinition, Message, Model, ModelList, Tool, ToolCall, ToolCallFunction, Usage};
```

To:
```rust
pub use chat::ChatBuilder;
pub use client::{Client, Models, ToolExecutor};
pub use error::{Error, Result};
pub use types::{ChatChunk, ChatResponse, Choice, Delta, FunctionDefinition, Message, Model, ModelList, Tool, ToolCall, ToolCallFunction, Usage};
```

- [ ] **Step 2: Commit**

```bash
git add crates/openai/src/lib.rs
git commit -m "feat(openai): re-export ToolExecutor trait"
```

---

## Task 5: Verify compilation

**Files:**
- Modify: (none, just build verification)

- [ ] **Step 1: Build the workspace**

Run: `cargo build --workspace`
Expected: Successful compilation with no errors or warnings

- [ ] **Step 2: Run clippy**

Run: `cargo clippy --workspace`
Expected: No warnings

---

## Task 6: Simplify `test_openai_function_call` in `crates/minimax/src/main.rs`

**Files:**
- Modify: `crates/minimax/src/main.rs`

- [ ] **Step 1: Read current test function to understand what to replace**

Run: `grep -n "fn test_openai_function_call" crates/minimax/src/main.rs`

- [ ] **Step 2: Replace the entire `test_openai_function_call` function (~100 lines) with simplified version (~40 lines)**

Replace the function body with:

```rust
fn test_openai_function_call(api_key: &str) {
    use openai::{Client, Message, Tool, FunctionDefinition, ToolExecutor};
    use std::sync::Arc;
    use serde_json::Value;

    let client = Client::new(api_key).expect("Failed to create OpenAI client");

    // Define weather tool
    let get_weather_tool = Tool {
        type_: "function".to_string(),
        function: FunctionDefinition {
            name: "get_weather".to_string(),
            description: "获取城市天气信息".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "城市名称"
                    }
                },
                "required": ["location"]
            }),
        },
    };

    // Define executor
    struct WeatherExecutor;
    impl ToolExecutor for WeatherExecutor {
        fn execute(&self, tool_name: &str, args: Value) -> Box<dyn std::error::Error + Send + Sync> {
            if tool_name == "get_weather" {
                let location = args["location"].as_str().unwrap_or("北京");
                let result = call_weather_api(location);
                Ok(result)
            } else {
                Err(format!("Unknown tool: {}", tool_name).into())
            }
        }
    }

    println!("Step 1: User asks about weather");
    let response = client.chat_with_executor(
        vec![get_weather_tool],
        Arc::new(WeatherExecutor),
    )
    .model("MiniMax-M2.7")
    .messages(vec![Message::user("北京今天天气怎么样？")])
    .max_completion_tokens(4096)
    .send()
    .expect("Failed to send message with tools");

    println!("  Final Response:");
    if let Some(choice) = response.choices.first() {
        println!("  Content: {}", choice.message.content);
    }
}
```

Note: The `call_weather_api` helper function should remain unchanged at the top of main.rs (around line 168).

- [ ] **Step 3: Verify it compiles**

Run: `cargo build --workspace`
Expected: Successful compilation

- [ ] **Step 4: Commit**

```bash
git add crates/minimax/src/main.rs
git commit -m "refactor(minimax): simplify function calling test with ChatWithExecutor"
```

---

## Spec Coverage Check

- [ ] `ToolExecutor` trait defined in client.rs
- [ ] `ChatWithExecutor` struct with builder methods (model, messages, temperature, top_p, max_completion_tokens)
- [ ] `ChatWithExecutor::send()` implements the full tool-loop (send → parse tool_calls → execute → resend)
- [ ] `Client::chat_with_executor()` method added
- [ ] `ToolExecutor` re-exported from lib.rs
- [ ] `test_openai_function_call` simplified in minimax
- [ ] Multi-turn support: loop continues until no tool_calls
- [ ] Error handling: errors passed as JSON string to model
