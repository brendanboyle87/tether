//! The single writer task.
//!
//! One thread owns the `TraceWriter`. Everything that wants an event in the
//! trace sends it here, so events have one total order and the hash chain is
//! computed in one place. The thread flushes and exits when the last handle
//! is dropped.

use tether_trace::{NewEvent, TraceError, TraceWriter};
use tether_types::Event;
use tokio::sync::{mpsc, oneshot};

type Reply<T> = oneshot::Sender<Result<T, TraceError>>;

enum Cmd {
    Append(Box<NewEvent>, Reply<Event>),
    PutBlob(Vec<u8>, Reply<String>),
    Sync(Reply<()>),
}

#[derive(Debug, Clone)]
pub(crate) struct TraceHandle {
    tx: mpsc::Sender<Cmd>,
}

fn gone() -> TraceError {
    TraceError::Io(std::io::Error::other("trace writer task is gone"))
}

impl TraceHandle {
    pub(crate) fn spawn(mut writer: TraceWriter) -> Self {
        let (tx, mut rx) = mpsc::channel(64);
        std::thread::spawn(move || {
            // A failed send means the caller stopped waiting; nothing to do.
            while let Some(cmd) = rx.blocking_recv() {
                match cmd {
                    Cmd::Append(event, reply) => {
                        let _ = reply.send(writer.append(*event));
                    }
                    Cmd::PutBlob(data, reply) => {
                        let _ = reply.send(writer.blobs().put(&data));
                    }
                    Cmd::Sync(reply) => {
                        let _ = reply.send(writer.sync());
                    }
                }
            }
            let _ = writer.sync();
        });
        Self { tx }
    }

    async fn ask<T>(&self, cmd: impl FnOnce(Reply<T>) -> Cmd) -> Result<T, TraceError> {
        let (reply, answer) = oneshot::channel();
        self.tx.send(cmd(reply)).await.map_err(|_| gone())?;
        answer.await.map_err(|_| gone())?
    }

    pub(crate) async fn append(&self, event: NewEvent) -> Result<Event, TraceError> {
        self.ask(|reply| Cmd::Append(Box::new(event), reply)).await
    }

    pub(crate) async fn put_blob(&self, data: Vec<u8>) -> Result<String, TraceError> {
        self.ask(|reply| Cmd::PutBlob(data, reply)).await
    }

    pub(crate) async fn sync(&self) -> Result<(), TraceError> {
        self.ask(Cmd::Sync).await
    }
}
