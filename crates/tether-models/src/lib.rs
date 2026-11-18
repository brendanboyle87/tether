//! Model adapters. One trait, one internal message format; the adapter's only
//! job is translation. Tether never modifies prompts.
//!
//! Streamed responses are collapsed into one `model.response` event with
//! `stream: true` and `ttft_ms`. Pricing tables live in `pricing.toml`, not
//! in code, and the price used is recorded in `budget.tick`.
//!
//! Adapters work on real message content ([`CompletionRequest`],
//! [`Completion`]) and know nothing about the trace. `tether-core` turns those
//! into the trace's `model.request` and `model.response` events by storing
//! each message as a blob.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tether_types::Usage;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Money(pub f64);

#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("provider error: {0}")]
    Provider(String),
    #[error("replay: no recorded response for request hash {0}")]
    ReplayMiss(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

/// One piece of a message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Block {
    Text {
        text: String,
    },
    /// The model asking for a tool. `name` is the name the agent offered.
    ToolUse {
        id: String,
        name: String,
        arguments: serde_json::Value,
    },
    ToolResult {
        id: String,
        content: serde_json::Value,
        #[serde(default)]
        is_error: bool,
    },
}

/// A message in Tether's own form. This is what a message blob holds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Vec<Block>,
}

impl Message {
    pub fn text(role: Role, text: impl Into<String>) -> Self {
        Self {
            role,
            content: vec![Block::Text { text: text.into() }],
        }
    }
}

/// A tool as offered to the model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// What the agent asks of a model, with message content in full.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub tools: Vec<ToolDef>,
    pub params: serde_json::Value,
}

/// What a model answered. Latency is measured by the caller, not the adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Completion {
    pub message: Message,
    pub usage: Usage,
    /// Set when the response was streamed and collapsed.
    pub ttft_ms: Option<u64>,
    pub stream: bool,
    pub provider_request_id: Option<String>,
    /// Raw provider body, kept for diagnosing format changes.
    pub raw: Option<Vec<u8>>,
}

impl Completion {
    /// An unstreamed assistant message with zero usage.
    pub fn new(content: Vec<Block>) -> Self {
        Self {
            message: Message {
                role: Role::Assistant,
                content,
            },
            usage: Usage::default(),
            ttft_ms: None,
            stream: false,
            provider_request_id: None,
            raw: None,
        }
    }

    pub fn text(text: impl Into<String>) -> Self {
        Self::new(vec![Block::Text { text: text.into() }])
    }

    pub fn tool_use(
        id: impl Into<String>,
        name: impl Into<String>,
        arguments: serde_json::Value,
    ) -> Self {
        Self::new(vec![Block::ToolUse {
            id: id.into(),
            name: name.into(),
            arguments,
        }])
    }

    pub fn with_usage(mut self, usage: Usage) -> Self {
        self.usage = usage;
        self
    }
}

#[async_trait]
pub trait ModelAdapter: Send + Sync {
    fn id(&self) -> &str;
    async fn complete(&self, req: CompletionRequest) -> Result<Completion, ModelError>;
    /// `None` when the adapter cannot price (local models).
    fn price(&self, usage: &Usage) -> Option<Money>;
}

#[cfg(feature = "anthropic")]
pub mod anthropic {}
#[cfg(feature = "openai_compat")]
pub mod openai_compat {}
#[cfg(feature = "ollama")]
pub mod ollama {}
#[cfg(feature = "replay")]
pub mod replay {
    //! Answers from recorded `model.response` events, matched by request hash.
}
#[cfg(feature = "fake")]
pub mod fake;
