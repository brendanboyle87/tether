//! The hash chain, shared by the writer and the reader so they cannot drift.
//!
//! An event's hash is BLAKE3 over its line exactly as written, with the value
//! of the envelope `hash` field replaced by the previous event's hash (empty
//! for `seq = 0`). Working on the line's bytes rather than on a re-serialized
//! struct keeps verification independent of this build's view of the schema.

const HASH_KEY: &str = ",\"hash\":\"";

/// Split a line around the value of its envelope `hash` field:
/// `(everything up to the value, the value, everything after it)`.
///
/// The envelope puts `hash` before `type` and `data`, and every field ahead
/// of it is a number or an escaped string, so the first match is the envelope
/// field and never something inside a payload.
pub(crate) fn split_hash(line: &str) -> Option<(&str, &str, &str)> {
    let start = line.find(HASH_KEY)? + HASH_KEY.len();
    let len = line[start..].find('"')?;
    Some((
        &line[..start],
        &line[start..start + len],
        &line[start + len..],
    ))
}

/// Hash of the line `head + prev + tail`, as `blake3:<hex>`.
pub(crate) fn chain_hash(head: &str, prev: &str, tail: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(head.as_bytes());
    hasher.update(prev.as_bytes());
    hasher.update(tail.as_bytes());
    format!("{}{}", crate::blob::HASH_PREFIX, hasher.finalize().to_hex())
}
