use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub content: String,
    #[serde(default)]
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

    pub fn assistant(content: &str) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.to_string(),
            name: None,
        }
    }

    pub fn user_system(content: &str) -> Self {
        Self {
            role: "user_system".to_string(),
            content: content.to_string(),
            name: None,
        }
    }

    pub fn group(name: &str, content: &str) -> Self {
        Self {
            role: "group".to_string(),
            content: content.to_string(),
            name: Some(name.to_string()),
        }
    }

    pub fn sample_message_user(content: &str) -> Self {
        Self {
            role: "sample_message_user".to_string(),
            content: content.to_string(),
            name: None,
        }
    }

    pub fn sample_message_ai(content: &str) -> Self {
        Self {
            role: "sample_message_ai".to_string(),
            content: content.to_string(),
            name: None,
        }
    }

    pub fn tool(tool_use_id: &str, content: &str) -> Self {
        Self {
            role: "user".to_string(),
            content: serde_json::json!([{
                "type": "tool_result",
                "tool_use_id": tool_use_id,
                "content": content
            }]).to_string(),
            name: None,
        }
    }
}

/// Tool definition for Anthropic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "input_schema")]
    pub input_schema: serde_json::Value,
}

/// ToolUse block in response (when model calls a tool)
#[derive(Debug, Clone, Deserialize)]
pub struct ToolUseBlock {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

/// Content block item for response (includes tool_use)
pub enum ContentBlock {
    Text(TextBlock),
    Thinking(ThinkingBlock),
    ToolUse(ToolUseBlock),
}

impl std::fmt::Debug for ContentBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentBlock::Text(t) => f.debug_tuple("Text").field(&t.text).finish(),
            ContentBlock::Thinking(t) => f.debug_tuple("Thinking").field(&t.thinking).finish(),
            ContentBlock::ToolUse(t) => f.debug_tuple("ToolUse").field(&t.name).finish(),
        }
    }
}

impl Clone for ContentBlock {
    fn clone(&self) -> Self {
        match self {
            ContentBlock::Text(t) => ContentBlock::Text(TextBlock { text: t.text.clone() }),
            ContentBlock::Thinking(t) => ContentBlock::Thinking(ThinkingBlock { thinking: t.thinking.clone(), signature: t.signature.clone() }),
            ContentBlock::ToolUse(t) => ContentBlock::ToolUse(ToolUseBlock { id: t.id.clone(), name: t.name.clone(), input: t.input.clone() }),
        }
    }
}

impl<'de> Deserialize<'de> for ContentBlock {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum RawContentBlock {
            Text { #[serde(rename = "type")] type_: String, text: String },
            Thinking { #[serde(rename = "type")] type_: String, thinking: String, signature: Option<String> },
            ToolUse { #[serde(rename = "type")] type_: String, id: String, name: String, input: serde_json::Value },
        }

        let raw = RawContentBlock::deserialize(deserializer)?;
        match raw {
            RawContentBlock::Text { type_: _, text } => Ok(ContentBlock::Text(TextBlock { text })),
            RawContentBlock::Thinking { type_: _, thinking, signature } => Ok(ContentBlock::Thinking(ThinkingBlock { thinking, signature })),
            RawContentBlock::ToolUse { type_: _, id, name, input } => Ok(ContentBlock::ToolUse(ToolUseBlock { id, name, input })),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
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
pub struct MessageStop {
    #[serde(default)]
    pub type_: Option<String>,
}

impl Default for MessageStop {
    fn default() -> Self {
        Self { type_: None }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessageStart {
    pub message: MessageStartContent,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessageStartContent {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub role: String,
    pub model: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContentBlockStart {
    pub index: i32,
    pub content_block: ContentBlock,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContentBlockStop {
    pub index: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessageDelta {
    pub delta: MessageDeltaContent,
    #[serde(default)]
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessageDeltaContent {
    #[serde(default)]
    pub stop_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum StreamEvent {
    #[serde(rename = "message_start")]
    MessageStart(MessageStart),
    #[serde(rename = "content_block_start")]
    ContentBlockStart(ContentBlockStart),
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta(ContentBlockDelta),
    #[serde(rename = "content_block_stop")]
    ContentBlockStop(ContentBlockStop),
    #[serde(rename = "message_delta")]
    MessageDelta(MessageDelta),
    #[serde(rename = "message_stop")]
    MessageStop(MessageStop),
    #[serde(rename = "ping")]
    Ping,
}
