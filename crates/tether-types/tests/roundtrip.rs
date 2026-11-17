use tether_types::*;
use time::OffsetDateTime;

#[test]
fn event_roundtrips_through_json() {
    let ev = Event {
        v: SCHEMA_VERSION,
        run: RunId("run_01TEST".into()),
        seq: 0,
        ts: OffsetDateTime::UNIX_EPOCH,
        step: 0,
        agent: "main".into(),
        parent: None,
        hash: "blake3:0000".into(),
        body: EventType::RunStateChanged {
            state: RunState::Running,
            reason: "start".into(),
        },
    };
    let line = serde_json::to_string(&ev).expect("serialize");
    assert!(line.contains("\"type\":\"run.state\""), "{line}");
    let back: Event = serde_json::from_str(&line).expect("deserialize");
    assert_eq!(back, ev);
}

#[test]
fn unknown_event_type_is_ext() {
    let line = r#"{"v":0,"run":"r","seq":3,"ts":"1970-01-01T00:00:00Z","step":1,"agent":"main","hash":"blake3:00","type":"ext.custom","data":{"k":1}}"#;
    let ev: Event = serde_json::from_str(line).expect("deserialize");
    match ev.body {
        EventType::Ext { r#type, .. } => assert_eq!(r#type, "ext.custom"),
        other => panic!("expected Ext, got {other:?}"),
    }
}
