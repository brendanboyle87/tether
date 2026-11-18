//! Policy engine: rules evaluated top to bottom, first match wins, default `ask`.
//!
//! Policies are TOML (`tether.toml`), loaded once per run and snapshotted into
//! the `run.created` event. This crate decides; it does not enforce. The
//! sandbox enforces.

use tether_types::{Decision, ToolCall};

/// Per-run context a rule may consult: budget so far, step, agent id.
#[derive(Debug, Clone, Default)]
pub struct RunCtx {
    pub step: u64,
    pub agent: String,
    pub budget: tether_types::BudgetSnapshot,
}

/// Every tool call and model call passes through exactly one of these.
pub trait Policy: Send + Sync {
    fn decide(&self, call: &ToolCall, ctx: &RunCtx) -> Decision;
}

/// Default when nobody answers a `policy.ask`, in seconds.
pub const DEFAULT_ASK_TIMEOUT_SECS: u64 = 600;

pub mod config {
    //! `tether.toml` parsing: `[budget]` and `[[rule]]` tables.
}

pub mod budget {
    //! Token, dollar, wall-clock and tool-call budgets.
}

pub mod kill {
    //! File-watch and Unix-signal kill switch; halts at the next event boundary.
}
