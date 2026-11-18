//! Sandbox: one trait, three backends.
//!
//! `process` (default) is rlimits, a seccomp filter on Linux, a cwd jail and
//! an env scrub. It is **not** a security boundary and is never described as
//! one. `docker` adds filesystem and network namespaces. `firecracker` adds a
//! separate kernel. The README states what each isolates and what it does not.

use std::path::{Path, PathBuf};

use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct SandboxSpec {
    pub workspace: PathBuf,
    pub network: Network,
    pub cpu_millis: Option<u32>,
    pub memory_bytes: Option<u64>,
    pub time_limit_secs: Option<u64>,
}

#[derive(Debug, Clone)]
pub enum Network {
    None,
    Allowlist(Vec<String>),
    All,
}

#[derive(Debug, Clone)]
pub struct Handle(pub String);

#[derive(Debug, Clone)]
pub struct SnapshotId(pub String);

#[derive(Debug, Clone)]
pub struct Exec {
    pub argv: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub env: Vec<(String, String)>,
    pub stdin: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct ExecResult {
    pub exit_code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub duration_ms: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("unsupported by this backend: {0}")]
    Unsupported(&'static str),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, SandboxError>;

#[async_trait]
pub trait Sandbox: Send + Sync {
    async fn start(&self, spec: &SandboxSpec) -> Result<Handle>;
    async fn exec(&self, h: &Handle, cmd: Exec) -> Result<ExecResult>;
    async fn read(&self, h: &Handle, path: &Path) -> Result<Vec<u8>>;
    async fn write(&self, h: &Handle, path: &Path, data: Vec<u8>) -> Result<()>;
    /// Backends that cannot snapshot return `Unsupported`; resume then
    /// replays writes from the trace into a fresh sandbox.
    async fn snapshot(&self, h: &Handle) -> Result<SnapshotId>;
    async fn stop(&self, h: Handle) -> Result<()>;
}

#[cfg(feature = "process")]
pub mod process;

#[cfg(feature = "docker")]
pub mod docker {
    //! Docker backend; snapshot via `commit`.
}

#[cfg(feature = "firecracker")]
pub mod firecracker {
    //! Firecracker microVM backend; snapshot via VM snapshot. v1.0.
}
