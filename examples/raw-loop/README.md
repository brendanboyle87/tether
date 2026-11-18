# examples/raw-loop

A raw agent loop wrapped by Tether, in one file: `main.rs`.

The agent asks the model, runs the tools it asks for, feeds the results back,
and stops when the model asks for none. Its two call sites go through a
`tether_core::Run` (`run.complete` and `run.call_tool`) instead of a provider
SDK and a tool dispatcher. That is the whole integration.

The model is the scripted `fake` adapter, so no API key is needed. The tools
are real: `write_file` and `exec` run in the process sandbox, in
`target/raw-loop/workspace`.

```
cargo run -p tether-core --example raw-loop
cargo run -p tether-cli -- inspect target/raw-loop/<run>.jsonl
cargo run -p tether-cli -- verify target/raw-loop/<run>.jsonl
```

The example is in-process because the SDK boundary (`tetherd`, gRPC) lands in
v0.3. Until then an agent has to be Rust code linked against `tether-core`.
