//! Scripted responses for tests. No API key.

use std::collections::VecDeque;
use std::sync::{Mutex, MutexGuard, PoisonError};

use async_trait::async_trait;
use tether_types::Usage;

use crate::{Completion, CompletionRequest, ModelAdapter, ModelError, Money};

/// Answers each request with the next completion in its script and remembers
/// every request it was sent. A request past the end of the script is a
/// provider error.
#[derive(Debug, Default)]
pub struct FakeAdapter {
    script: Mutex<VecDeque<Completion>>,
    requests: Mutex<Vec<CompletionRequest>>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

impl FakeAdapter {
    pub fn new(script: impl IntoIterator<Item = Completion>) -> Self {
        Self {
            script: Mutex::new(script.into_iter().collect()),
            requests: Mutex::default(),
        }
    }

    /// Every request received so far, oldest first.
    pub fn requests(&self) -> Vec<CompletionRequest> {
        lock(&self.requests).clone()
    }

    /// Scripted completions not yet used.
    pub fn remaining(&self) -> usize {
        lock(&self.script).len()
    }
}

#[async_trait]
impl ModelAdapter for FakeAdapter {
    fn id(&self) -> &str {
        "fake"
    }

    async fn complete(&self, req: CompletionRequest) -> Result<Completion, ModelError> {
        lock(&self.requests).push(req);
        lock(&self.script)
            .pop_front()
            .ok_or_else(|| ModelError::Provider("fake adapter: script exhausted".into()))
    }

    /// The fake is free.
    fn price(&self, _usage: &Usage) -> Option<Money> {
        Some(Money(0.0))
    }
}
