use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use serde_json::json;
use tether_trace::{NewEvent, TraceWriter};
use tether_types::{EventType, Payload, RunId, RunState, ToolCall, ToolResult};
use time::OffsetDateTime;

fn tether(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tether"))
        .args(args)
        .output()
        .expect("run tether")
}

fn write_trace(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(test);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create test dir");
    let path = dir.join("run.jsonl");
    let mut w = TraceWriter::create(&path, RunId("run_test".into())).expect("create");
    let bodies = [
        EventType::RunStateChanged {
            state: RunState::Running,
            reason: "start".into(),
        },
        EventType::ToolCall(ToolCall {
            id: "call_1".into(),
            name: "sandbox.exec".into(),
            server: "sandbox".into(),
            arguments: json!({"argv": ["ls"]}),
            external: false,
        }),
        EventType::ToolResult(ToolResult {
            id: "call_1".into(),
            content: Payload::Inline(json!("README.md")),
            is_error: false,
            duration_ms: 4,
        }),
    ];
    for body in bodies {
        w.append(NewEvent {
            ts: OffsetDateTime::UNIX_EPOCH,
            step: 1,
            agent: "main".into(),
            parent: None,
            body,
        })
        .expect("append");
    }
    path
}

#[test]
fn verify_and_inspect_read_a_sound_trace() {
    let path = write_trace("cli_sound");
    let path = path.to_str().expect("utf-8 path");

    let out = tether(&["verify", path]);
    assert!(out.status.success(), "{out:?}");
    assert!(String::from_utf8_lossy(&out.stdout).starts_with("ok: 3 events, 0 blobs"));

    let out = tether(&["inspect", path]);
    assert!(out.status.success(), "{out:?}");
    let text = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 3, "{text}");
    assert!(
        lines[0].contains("run.state") && lines[0].contains("running: start"),
        "{text}"
    );
    assert!(
        lines[1].contains(r#"sandbox.exec {"argv":["ls"]}"#),
        "{text}"
    );
    assert!(lines[2].contains(r#"ok, 4 ms, "README.md""#), "{text}");
}

#[test]
fn verify_fails_on_a_tampered_trace() {
    let path = write_trace("cli_tampered");
    let text = fs::read_to_string(&path).expect("read");
    fs::write(&path, text.replace("README.md", "SECRETS.md")).expect("write");

    let out = tether(&["verify", path.to_str().expect("utf-8 path")]);
    assert!(!out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("hash chain broken at seq 2"),
        "{out:?}"
    );
}
