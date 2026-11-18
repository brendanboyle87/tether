//! Tool calls and results. Every tool the agent sees is an MCP tool,
//! including the built-in sandbox server.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCall {
    /// Provider-assigned id, echoed back in the result.
    pub id: String,
    /// Fully qualified name, prefixed by server id on collision (`github.create_issue`).
    pub name: String,
    /// Which MCP server owns this tool. `"sandbox"` for the built-in server.
    pub server: String,
    pub arguments: serde_json::Value,
    /// Tools with side effects outside the sandbox are never re-invoked on replay.
    #[serde(default)]
    pub external: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolResult {
    pub id: String,
    /// Inline content, or a blob reference for payloads over the inline limit.
    pub content: Payload,
    pub is_error: bool,
    pub duration_ms: u64,
}

/// Either a reference into the content-addressed blob store or inline JSON.
///
/// `Blob` is listed first on purpose: the enum is untagged and `Inline` holds
/// any JSON value, so it would otherwise swallow every blob reference on read.
/// An inline value that itself has the shape of a blob reference would be
/// ambiguous, so the trace writer always spills such a value to a blob.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Payload {
    Blob { blob: String, bytes: u64 },
    Inline(serde_json::Value),
}
