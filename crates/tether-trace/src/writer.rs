//! Append-only JSONL writer owning the hash chain.
//!
//! One `TraceWriter` owns one trace file. It assigns `seq`, computes the hash
//! chain and spills oversized tool results to the blob store. It is
//! synchronous and reads no clock: `tether-core` runs it on the single writer
//! task and supplies each event's timestamp.

use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use tether_types::{Event, EventType, RunId, SCHEMA_VERSION};
use time::OffsetDateTime;

use crate::chain::{chain_hash, split_hash};
use crate::{BlobStore, Result, TraceError, MAX_EVENT_BYTES};

/// An event before the writer has given it a `seq` and a `hash`.
#[derive(Debug, Clone, PartialEq)]
pub struct NewEvent {
    pub ts: OffsetDateTime,
    pub step: u64,
    pub agent: String,
    pub parent: Option<u64>,
    pub body: EventType,
}

#[derive(Debug)]
pub struct TraceWriter {
    file: File,
    run: RunId,
    next_seq: u64,
    prev_hash: String,
    blobs: BlobStore,
    /// Set when a write failed partway; the file may end in a partial line.
    failed: bool,
}

impl TraceWriter {
    /// Start a new trace at `path`. Fails if the file already exists: a trace
    /// is never overwritten.
    pub fn create(path: impl AsRef<Path>, run: RunId) -> Result<Self> {
        let path = path.as_ref();
        let file = OpenOptions::new()
            .append(true)
            .create_new(true)
            .open(path)?;
        Ok(Self {
            file,
            run,
            next_seq: 0,
            prev_hash: String::new(),
            blobs: BlobStore::for_trace(path),
            failed: false,
        })
    }

    /// The blob store beside this trace. Callers put model messages and raw
    /// provider bodies here and record the returned hashes in their events.
    pub fn blobs(&self) -> &BlobStore {
        &self.blobs
    }

    /// `seq` the next appended event will get.
    pub fn next_seq(&self) -> u64 {
        self.next_seq
    }

    /// Append one event and return it as written, with its `seq` and `hash`.
    ///
    /// An event over [`MAX_EVENT_BYTES`] is rejected with `EventTooLarge` and
    /// nothing is written; the writer stays usable.
    pub fn append(&mut self, new: NewEvent) -> Result<Event> {
        if self.failed {
            return Err(io::Error::other("trace writer failed on an earlier write").into());
        }
        let body = match new.body {
            EventType::ToolResult(mut result) => {
                result.content = self.blobs.spill(result.content)?;
                EventType::ToolResult(result)
            }
            other => other,
        };
        let mut event = Event {
            v: SCHEMA_VERSION,
            run: self.run.clone(),
            seq: self.next_seq,
            ts: new.ts,
            step: new.step,
            agent: new.agent,
            parent: new.parent,
            hash: String::new(),
            body,
        };

        let unhashed = serde_json::to_string(&event)?;
        let (head, _, tail) = split_hash(&unhashed).ok_or_else(|| TraceError::Malformed {
            line: self.next_seq + 1,
            reason: "serialized event has no hash field".into(),
        })?;
        let hash = chain_hash(head, &self.prev_hash, tail);
        let mut line = String::with_capacity(unhashed.len() + hash.len() + 1);
        line.push_str(head);
        line.push_str(&hash);
        line.push_str(tail);
        if line.len() > MAX_EVENT_BYTES {
            return Err(TraceError::EventTooLarge {
                seq: self.next_seq,
                bytes: line.len(),
                limit: MAX_EVENT_BYTES,
            });
        }
        line.push('\n');

        // One write per event, so an event boundary is always a line boundary.
        if let Err(e) = self.file.write_all(line.as_bytes()) {
            self.failed = true;
            return Err(e.into());
        }

        event.hash.clone_from(&hash);
        self.prev_hash = hash;
        self.next_seq += 1;
        Ok(event)
    }

    /// Flush the trace to disk. Call at points that must survive a crash.
    pub fn sync(&mut self) -> Result<()> {
        self.file.sync_data()?;
        Ok(())
    }
}
