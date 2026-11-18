#![cfg(unix)]

use std::path::{Path, PathBuf};

use tether_sandbox::process::ProcessSandbox;
use tether_sandbox::{Exec, Handle, Network, Sandbox, SandboxError, SandboxSpec};

async fn start(test: &str, time_limit_secs: Option<u64>) -> (ProcessSandbox, Handle, PathBuf) {
    let workspace = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(test);
    let _ = std::fs::remove_dir_all(&workspace);
    let sandbox = ProcessSandbox::new();
    let spec = SandboxSpec {
        workspace: workspace.clone(),
        network: Network::None,
        cpu_millis: None,
        memory_bytes: None,
        time_limit_secs,
    };
    let handle = sandbox.start(&spec).await.expect("start");
    (sandbox, handle, workspace)
}

fn sh(script: &str) -> Exec {
    Exec {
        argv: vec!["sh".into(), "-c".into(), script.into()],
        cwd: None,
        env: vec![],
        stdin: None,
    }
}

#[tokio::test]
async fn exec_runs_in_the_workspace_with_a_scrubbed_env() {
    let (sandbox, h, workspace) = start("process_exec", None).await;
    std::env::set_var("TETHER_TEST_SECRET", "leak");

    let out = sandbox
        .exec(
            &h,
            sh("pwd; echo ${TETHER_TEST_SECRET:-unset}; echo oops >&2; exit 3"),
        )
        .await
        .expect("exec");
    let stdout = String::from_utf8(out.stdout).expect("utf-8");
    let lines: Vec<&str> = stdout.lines().collect();
    let workspace = workspace.canonicalize().expect("canonicalize");
    assert_eq!(lines, [workspace.to_str().expect("utf-8"), "unset"]);
    assert_eq!(out.stderr, b"oops\n");
    assert_eq!(out.exit_code, 3);
}

#[tokio::test]
async fn exec_passes_stdin_and_explicit_env() {
    let (sandbox, h, _) = start("process_stdin", None).await;
    let mut cmd = sh("cat; echo $GREETING");
    cmd.stdin = Some(b"from stdin\n".to_vec());
    cmd.env = vec![("GREETING".into(), "hello".into())];

    let out = sandbox.exec(&h, cmd).await.expect("exec");
    assert_eq!(out.stdout, b"from stdin\nhello\n");
}

#[tokio::test]
async fn write_then_read_roundtrips_inside_the_workspace() {
    let (sandbox, h, workspace) = start("process_files", None).await;
    sandbox
        .write(&h, Path::new("src/a.txt"), b"hi".to_vec())
        .await
        .expect("write");
    assert_eq!(
        sandbox
            .read(&h, Path::new("src/a.txt"))
            .await
            .expect("read"),
        b"hi"
    );
    assert!(workspace.join("src/a.txt").is_file());
}

#[tokio::test]
async fn paths_that_leave_the_workspace_are_refused() {
    let (sandbox, h, _) = start("process_escape", None).await;
    for bad in ["../outside.txt", "/etc/passwd", "a/../../b"] {
        let err = sandbox.read(&h, Path::new(bad)).await.expect_err(bad);
        assert!(matches!(err, SandboxError::Other(_)), "{bad}: {err}");
        let err = sandbox
            .write(&h, Path::new(bad), vec![])
            .await
            .expect_err(bad);
        assert!(matches!(err, SandboxError::Other(_)), "{bad}: {err}");
    }
    let mut cmd = sh("pwd");
    cmd.cwd = Some("..".into());
    assert!(sandbox.exec(&h, cmd).await.is_err());
}

#[tokio::test]
async fn a_command_past_the_time_limit_is_stopped() {
    let (sandbox, h, _) = start("process_timeout", Some(1)).await;
    let err = sandbox
        .exec(&h, sh("sleep 30"))
        .await
        .expect_err("must time out");
    assert!(err.to_string().contains("timed out"), "{err}");
}

#[tokio::test]
async fn snapshot_is_unsupported_and_stop_forgets_the_session() {
    let (sandbox, h, _) = start("process_stop", None).await;
    assert!(matches!(
        sandbox.snapshot(&h).await,
        Err(SandboxError::Unsupported(_))
    ));
    sandbox.stop(h.clone()).await.expect("stop");
    assert!(sandbox.exec(&h, sh("true")).await.is_err());
}
