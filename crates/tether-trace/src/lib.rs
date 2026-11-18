//! Trace writer, reader, hash chain and blob store.
//!
//! Layout on disk, per run:
//!
//! ```text
//! <run>.jsonl               one Event per line, append-only
//! <run>.blobs/blake3/ab/abcd...   content-addressed payloads over INLINE_LIMIT
//! ```
//!
//! The writer is the single owner of the file; `tether-core` feeds it from a
//! channel so events have a total order and the hash chain is computed in one
//! place. Readers never need the index; `rusqlite` is an optional cache for
//! `inspect` and `diff` over large traces.
//!
//! Spec: `docs/TRACE_FORMAT.md`.

/// Payloads at or below this many bytes are stored inline in the event.
pub const INLINE_LIMIT: usize = 32 * 1024;

/// Hard cap on one serialized event. Anything larger must have spilled to a blob.
pub const MAX_EVENT_BYTES: usize = 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum TraceError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("hash chain broken at seq {seq}: expected {expected}, found {found}")]
    ChainBroken {
        seq: u64,
        expected: String,
        found: String,
    },
    #[error("missing blob {0}")]
    MissingBlob(String),
    #[error("event {seq} is {bytes} bytes, over the {limit} byte cap")]
    EventTooLarge {
        seq: u64,
        bytes: usize,
        limit: usize,
    },
}

pub mod blob {
    //! Content-addressed blob store. Keyed by `blake3:<hex>`.
}

pub mod writer {
    //! Append-only JSONL writer owning the hash chain.
}

pub mod reader {
    //! Streaming reader with chain verification.
}

pub mod diff {
    //! Align two traces by step and report the first divergence.
}
