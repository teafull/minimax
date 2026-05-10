use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub name: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(rename = "tool_call_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    pub fn user(content: &str) -> Self {
        Self {
            role: "user".to_string(),
            content: content.to_string(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn system(content: &str) -> Self {
        Self {
            role: "system".to_string(),
            content: content.to_string(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn assistant(content: &str) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.to_string(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn tool(tool_call_id: &str, content: &str) -> Self {
        Self {
            role: "tool".to_string(),
            content: content.to_string(),
            name: None,
            tool_calls: None,
            tool_call_id: Some(tool_call_id.to_string()),
        }
    }

    pub fn user_system(content: &str) -> Self {
        Self {
            role: "user_system".to_string(),
            content: content.to_string(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn group(name: &str, content: &str) -> Self {
        Self {
            role: "group".to_string(),
            content: content.to_string(),
            name: Some(name.to_string()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn sample_message_user(content: &str) -> Self {
        Self {
            role: "sample_message_user".to_string(),
            content: content.to_string(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn sample_message_ai(content: &str) -> Self {
        Self {
            role: "sample_message_ai".to_string(),
            content: content.to_string(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }
}

/// Tool call made by the model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub function: ToolCallFunction,
}

/// Function details in a tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String,
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    #[serde(rename = "type")]
    pub type_: String,
    pub function: FunctionDefinition,
}

/// Function definition in a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
    #[serde(rename = "tool_calls")]
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCall>>,
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
pub struct ChatChunk {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChunkChoice>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChunkChoice {
    pub index: i32,
    pub delta: Delta,
    #[serde(default)]
    pub finish_reason: Option<String>,
    #[serde(rename = "tool_calls")]
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Delta {
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(rename = "tool_call_id")]
    #[serde(default)]
    pub tool_call_id: Option<String>,
    #[serde(default)]
    pub function: Option<ToolCallFunction>,
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
