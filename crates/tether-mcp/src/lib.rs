//! Tool layer: MCP client, registry and the built-in `sandbox` server.
//!
//! The agent sees one flat tool list. At run start every configured server is
//! connected, `tools/list` is called, and the merged list is recorded in
//! `run.created`. Name collisions are prefixed by server id.
//!
//! Each release pins one MCP protocol version and refuses servers that
//! negotiate another, with a clear error.
//!
//! Decision: use `rmcp` behind [`McpClient`] under a cargo feature; the
//! `minimal` module is the fallback if `rmcp` blocks stdio or streamable HTTP
//! in week 1.

use async_trait::async_trait;
use tether_types::{ToolCall, ToolResult};

/// Pinned for this release.
pub const PROTOCOL_VERSION: &str = "2025-06-18";

#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("protocol version mismatch: server offered {0}, this build pins {PROTOCOL_VERSION}")]
    ProtocolMismatch(String),
    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Clone)]
pub struct ToolSpec {
    pub name: String,
    pub server: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub external: bool,
}

#[async_trait]
pub trait McpClient: Send + Sync {
    async fn list_tools(&self) -> Result<Vec<ToolSpec>, McpError>;
    async fn call(&self, call: &ToolCall) -> Result<ToolResult, McpError>;
}

pub mod registry {
    //! Merge tools from every server; route calls back to the owner.
}

pub mod sandbox_server;

#[cfg(feature = "minimal")]
pub mod minimal {
    //! Hand-rolled MCP client. Fallback only.
}
