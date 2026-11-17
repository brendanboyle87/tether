//! Model adapters. One trait, one internal message format; the adapter's only
//! job is translation. Tether never modifies prompts.
//!
//! Streamed responses are collapsed into one `model.response` event with
//! `stream: true` and `ttft_ms`. Pricing tables live in `pricing.toml`, not
//! in code, and the price used is recorded in `budget.tick`.

use async_trait::async_trait;
use tether_types::{ModelRequest, ModelResponse, Usage};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Money(pub f64);

#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("provider error: {0}")]
    Provider(String),
    #[error("replay: no recorded response for request hash {0}")]
    ReplayMiss(String),
}

#[async_trait]
pub trait ModelAdapter: Send + Sync {
    fn id(&self) -> &str;
    async fn complete(&self, req: ModelRequest) -> Result<ModelResponse, ModelError>;
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
pub mod fake {
    //! Scripted responses for tests. No API key.
}
