//! The run loop.
//!
//! `tether-core` owns a run from `Created` to `Finished`, feeds every event to
//! the single trace writer, consults policy before every side effect, and
//! reads nondeterministic inputs (clock, rng, env) through traits so replay can
//! satisfy them from the trace.
//!
//! It has no opinion about the agent. The agent is a black box that emits
//! model requests and tool requests; this crate answers them.

/// Clock reads go through this so replay can substitute recorded values.
pub trait Clock: Send + Sync {
    fn now_ms(&self) -> u64;
}

/// Random seeds go through this for the same reason.
pub trait Rng: Send + Sync {
    fn seed(&self) -> u64;
}

pub mod run {
    //! Run lifecycle and the state machine (`RunState`).
}

pub mod checkpoint {
    //! `checkpoint` events every N steps; resume from the last one.
}

pub mod replay {
    //! Re-run a trace with the replay adapter and a replay sandbox.
}
