# Tether trace format, schema v0

Status: draft, frozen at v0.3. Changes before then go through `tether-types`
and this file in the same PR.

## Files

```
<run>.jsonl                          one Event per line, append-only, UTF-8
<run>.blobs/blake3/<ab>/<full hex>   content-addressed payloads over 32 KB
```

`tether pack <run>` bundles both into one tar. All tools accept either form.

## Envelope

Every line is one JSON object with these fields, in this order:

| Field | Type | Meaning |
| --- | --- | --- |
| `v` | integer | Schema version. `0`. |
| `run` | string | Run id. |
| `seq` | integer | Dense, starts at 0, increments by exactly 1. |
| `ts` | RFC 3339 string | Wall clock at write time, UTC. |
| `step` | integer | Agent turn this event belongs to. |
| `agent` | string | Emitting agent. `"main"` for single-agent runs. |
| `parent` | integer, optional | `seq` of the request this event answers. |
| `hash` | string | `blake3:<hex>`. See Hash chain. |
| `type` | string | Event type. See below. |
| `data` | object | Type-specific payload. |

Example:

```json
{"v":0,"run":"run_01J...","seq":42,"ts":"2026-11-17T19:47:03.118Z","step":7,"agent":"main","parent":41,"hash":"blake3:...","type":"tool.call","data":{}}
```

## Hash chain

`hash` is BLAKE3 over the canonical JSON encoding of the event with its own
`hash` field replaced by the previous event's `hash` (empty string for
`seq = 0`). Canonical means: keys in the order above, no whitespace, UTF-8.
`tether verify` recomputes the chain and checks every referenced blob exists.

## Blobs

A payload larger than 32 KB (`INLINE_LIMIT`) is written to the blob store and
the event carries `{"blob":"blake3:<hex>","bytes":N}` in its place.

Model message history is content-addressed **per message**: `model.request.data.messages`
is a list of blob hashes, one per message. A step adds one new blob; the
history is never copied.

## Event types

| `type` | `data` |
| --- | --- |
| `run.created` | agent id, model config, policy snapshot (full TOML), sandbox backend, merged tool list, schema version |
| `run.state` | `{state, reason}`; state is one of `created`, `running`, `suspended`, `halted`, `finished` |
| `run.child` | `{child}` run id of a spawned sub-agent process |
| `run.finished` | outcome, totals, final budget |
| `model.request` | provider, model, `messages` (blob hashes), `tools`, params |
| `model.response` | `message` (blob hash), `tool_calls`, usage, `latency_ms`, `ttft_ms`, `stream`, `provider_request_id`, `raw` (blob hash) |
| `tool.call` | id, name, server, arguments, `external` |
| `tool.result` | id, content (inline or blob), `is_error`, `duration_ms` |
| `policy.decision` | `subject` seq, rule, `{outcome: allow \| deny \| ask, ...}`, budget snapshot |
| `policy.ask` | `subject`, prompt |
| `policy.answer` | `subject`, `answered_by`, `allow` |
| `sandbox.exec` | command, cwd, env hash |
| `sandbox.exit` | exit code, stdout and stderr refs |
| `budget.tick` | tokens, usd, wall seconds, tool calls: used and limit |
| `checkpoint` | `step`, optional sandbox `snapshot` id |
| `nondeterminism` | clock reads, random seeds, env lookups |
| `ext.*` | anything; ignored by replay and diff |

## Ordering guarantees

For every tool call the trace contains, in this order and with the same
`step`: `tool.call`, `policy.decision`, then `tool.result`. A `deny` still
produces a `tool.result` with `is_error: true` and the reason.

For every model call: `model.request`, `policy.decision`, `model.response`.

## Size limits

One serialized event is at most 1 MB. Anything larger must have spilled to a
blob; a writer that would exceed the cap fails the run with
`EventTooLarge` rather than writing a bad line.
