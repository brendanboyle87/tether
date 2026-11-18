//! Default backend: runs commands as plain child processes.
//!
//! **Not a security boundary.** What it does today: starts every command in
//! the workspace directory, clears the environment (only `PATH` is passed
//! through), keeps `read` and `write` paths inside the workspace, and kills a
//! command that outlives the time limit. A command can still reach anything
//! the user running Tether can. rlimits and the Linux seccomp filter are not
//! implemented yet; `network`, `cpu_millis` and `memory_bytes` are ignored.

use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::{Exec, ExecResult, Handle, Result, Sandbox, SandboxError, SandboxSpec, SnapshotId};

struct Session {
    workspace: PathBuf,
    time_limit: Option<Duration>,
}

#[derive(Default)]
pub struct ProcessSandbox {
    sessions: Mutex<HashMap<String, Session>>,
    next_id: AtomicU64,
}

impl ProcessSandbox {
    pub fn new() -> Self {
        Self::default()
    }

    fn sessions(&self) -> MutexGuard<'_, HashMap<String, Session>> {
        self.sessions.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn session(&self, h: &Handle) -> Result<(PathBuf, Option<Duration>)> {
        self.sessions()
            .get(&h.0)
            .map(|s| (s.workspace.clone(), s.time_limit))
            .ok_or_else(|| SandboxError::Other(format!("no such sandbox: {}", h.0)))
    }

    fn resolve(&self, h: &Handle, path: &Path) -> Result<PathBuf> {
        Ok(self.session(h)?.0.join(inside(path)?))
    }
}

/// Accept only a relative path with no `..`, so it cannot name anything
/// outside the workspace. Symlinks inside the workspace are not followed up.
fn inside(path: &Path) -> Result<&Path> {
    let stays_inside = path
        .components()
        .all(|c| matches!(c, Component::Normal(_) | Component::CurDir));
    if stays_inside {
        Ok(path)
    } else {
        Err(SandboxError::Other(format!(
            "path leaves the workspace: {}",
            path.display()
        )))
    }
}

#[async_trait]
impl Sandbox for ProcessSandbox {
    async fn start(&self, spec: &SandboxSpec) -> Result<Handle> {
        tokio::fs::create_dir_all(&spec.workspace).await?;
        let workspace = tokio::fs::canonicalize(&spec.workspace).await?;
        let id = format!("process-{}", self.next_id.fetch_add(1, Ordering::SeqCst));
        let session = Session {
            workspace,
            time_limit: spec.time_limit_secs.map(Duration::from_secs),
        };
        self.sessions().insert(id.clone(), session);
        Ok(Handle(id))
    }

    async fn exec(&self, h: &Handle, cmd: Exec) -> Result<ExecResult> {
        let (workspace, time_limit) = self.session(h)?;
        let (program, args) = cmd
            .argv
            .split_first()
            .ok_or_else(|| SandboxError::Other("empty argv".into()))?;
        let cwd = match &cmd.cwd {
            Some(dir) => workspace.join(inside(dir)?),
            None => workspace,
        };

        let mut command = Command::new(program);
        command
            .args(args)
            .current_dir(cwd)
            .env_clear()
            .envs(std::env::var_os("PATH").map(|path| ("PATH", path)))
            .envs(cmd.env.iter().map(|(k, v)| (k, v)))
            .stdin(if cmd.stdin.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let started = Instant::now();
        let mut child = command.spawn()?;
        if let (Some(input), Some(mut stdin)) = (cmd.stdin, child.stdin.take()) {
            // A command that exits without reading its input is not an error.
            let _ = stdin.write_all(&input).await;
        }
        let output = match time_limit {
            // Dropping the timed-out future drops the child, which kills it.
            Some(limit) => tokio::time::timeout(limit, child.wait_with_output())
                .await
                .map_err(|_| {
                    SandboxError::Other(format!("timed out after {} s", limit.as_secs()))
                })??,
            None => child.wait_with_output().await?,
        };
        Ok(ExecResult {
            // No code means the command was killed by a signal.
            exit_code: output.status.code().unwrap_or(-1),
            stdout: output.stdout,
            stderr: output.stderr,
            duration_ms: started.elapsed().as_millis() as u64,
        })
    }

    async fn read(&self, h: &Handle, path: &Path) -> Result<Vec<u8>> {
        Ok(tokio::fs::read(self.resolve(h, path)?).await?)
    }

    async fn write(&self, h: &Handle, path: &Path, data: Vec<u8>) -> Result<()> {
        let path = self.resolve(h, path)?;
        if let Some(dir) = path.parent() {
            tokio::fs::create_dir_all(dir).await?;
        }
        Ok(tokio::fs::write(path, data).await?)
    }

    async fn snapshot(&self, _h: &Handle) -> Result<SnapshotId> {
        Err(SandboxError::Unsupported("snapshot"))
    }

    /// Forgets the session. The workspace and its files are left in place.
    async fn stop(&self, h: Handle) -> Result<()> {
        self.sessions().remove(&h.0);
        Ok(())
    }
}
