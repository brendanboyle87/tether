use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "tether",
    version,
    about = "Run agents you don't trust on infrastructure you do."
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Start a run under a policy.
    Run,
    /// Re-run a recorded trace with no network and no cost.
    Replay { run: String },
    /// Align two traces by step and report where they diverge.
    Diff { a: String, b: String },
    /// Run a suite of recorded tasks and score outcomes.
    Eval,
    /// Print a trace in readable form.
    Inspect { run: String },
    /// Continue a halted run from its last checkpoint.
    Resume { run: String },
    /// Verify the BLAKE3 hash chain and blob completeness.
    Verify { run: String },
    /// Bundle a trace and its blobs into one tar.
    Pack { run: String },
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Run => todo!("v0.1"),
        Cmd::Inspect { .. } => todo!("v0.1"),
        Cmd::Verify { .. } => todo!("v0.1"),
        Cmd::Replay { .. } | Cmd::Diff { .. } | Cmd::Resume { .. } | Cmd::Pack { .. } => {
            todo!("v0.3")
        }
        Cmd::Eval => todo!("v0.4"),
    }
}
