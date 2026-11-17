# tether

**Run agents you don't trust on infrastructure you do.**

Tether is a model-agnostic, MCP-native runtime that wraps any agent and gives
you the things frameworks leave out: a sandbox, hard budgets, a permission
layer, and a trace of every model call, tool call and policy decision that you
can replay, diff and eval against.

> Status: pre-alpha, v0.1 "it records" in progress. Nothing here runs yet.
> Read [docs/PLAN.md](docs/PLAN.md) for what ships when.

<!-- demo GIF: agent misbehaves, Tether blocks it, trace shows why -->

## What it is, and is not

Tether is a **harness**. It runs agents; it does not build them. Bring your
own loop, your own framework, your own prompts. Tether sits beside LangGraph,
the Claude Agent SDK, the OpenAI Agents SDK or a 50-line `while True`, and
owns everything that makes the run safe to leave unattended:

- **Sandboxed tool execution**: `process` (default), `docker`, `firecracker`
- **Policy**: allow / deny / ask rules in TOML, budgets for tokens, dollars, wall clock and tool calls, a kill switch
- **Trace**: one append-only JSONL per run, BLAKE3 hash chain, every decision recorded
- **Replay, diff, eval**: re-run a trace with no network and no cost, diff two runs, score a suite
- **Checkpoint and resume**: a 40-minute task survives a crash
- **OpenTelemetry** out of the box

Not in scope, ever: prompt templates, RAG, memory, agent-builder UIs.

## Integration is two call sites

```python
from tether import Run

with Run(agent="my-agent", policy="tether.toml") as run:
    msgs = [{"role": "user", "content": task}]
    while True:
        resp = run.complete(model="claude-sonnet-4-5", messages=msgs, tools=run.tools())
        if not resp.tool_calls:
            break
        for call in resp.tool_calls:
            msgs.append(run.call_tool(call))   # policy, sandbox, trace all happen here
```

## Layout

```
crates/   nine Rust crates; see CLAUDE.md
proto/    gRPC service for the SDKs
docs/     PLAN, ARCHITECTURE, TRACE_FORMAT
```

```sh
cargo check --workspace --all-targets
cargo test --workspace
```

## Honest note on sandboxing

The default `process` backend is rlimits, a seccomp filter and a cwd jail. It
is not a security boundary and we will never describe it as one. Use `docker`
(v0.2) for namespaces and `firecracker` (v1.0) for a separate kernel.

## License

Apache-2.0.
