//! `tetherd`: one daemon, many runs, Unix socket by default.
//! Service definition: `proto/tether.proto`. Wired up in v0.3 with the Python SDK.

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();
    anyhow::bail!("tetherd lands in v0.3; see docs/PLAN.md")
}
