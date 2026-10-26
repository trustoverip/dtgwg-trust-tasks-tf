//! `trust-tasks` — pin the Trust Task specifications an application uses.
//!
//! See README.md, and docs/design/consumer-side-generation.md in the
//! repository for the model: the application records its intent in
//! `trust-tasks.toml`, this command resolves it against the registry into
//! `trust-tasks.lock`, and vendors each pinned version's bundle beside it,
//! verified by digest. Builds read only the vendored files.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

mod lock;
mod manifest;
mod names;
mod ops;
mod project;
mod registry;

#[derive(Parser)]
#[command(
    name = "trust-tasks",
    version,
    about = "Pin the Trust Task specifications an application uses, vendor their bundles by digest, and verify them.",
    after_help = "Files: trust-tasks.toml (intent, yours), trust-tasks.lock (resolution, written by this command), \
trust-tasks/<slug>/<M.N>.bundle.json (vendored bundles). Commit all three."
)]
struct Cli {
    /// The registry: an https:// site root, a file:// URL or a directory with
    /// specs/index.json under it. Defaults to the manifest's `registry`, then
    /// https://trusttasks.org.
    #[arg(long, global = true, env = "TRUST_TASKS_REGISTRY", value_name = "URL")]
    registry: Option<String>,

    /// The project directory (the one holding trust-tasks.toml). Defaults to
    /// the nearest one at or above the current directory.
    #[arg(short = 'C', long = "dir", global = true, value_name = "DIR")]
    dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Pin tasks: `vault/upsert` (latest live version, pinned exactly),
    /// `vault/upsert/0.4` (that version), `vault/upsert@latest` (follow the
    /// latest live version on every update).
    Add {
        #[arg(required = true, value_name = "SLUG[/M.N|@latest]")]
        tasks: Vec<String>,
    },
    /// Unpin tasks (`vault/upsert`) or one version of one (`vault/upsert/0.3`),
    /// deleting their vendored bundles.
    Remove {
        #[arg(required = true, value_name = "SLUG[/M.N]")]
        tasks: Vec<String>,
    },
    /// Re-resolve against the registry: `latest` intents move to the latest
    /// live version, and an exact pin whose draft changed in place takes the
    /// new bytes. Refuses withdrawn versions. No slugs means every task.
    Update {
        #[arg(value_name = "SLUG")]
        slugs: Vec<String>,
        /// Also move exact pins to the latest live version (rewriting
        /// trust-tasks.toml).
        #[arg(long)]
        to_latest: bool,
    },
    /// Verify, offline, that the vendored bundles match trust-tasks.lock and
    /// that the lock matches trust-tasks.toml. Exits non-zero on any problem.
    Check,
    /// Restore missing or altered vendored bundles from the registry, by the
    /// digest trust-tasks.lock pins. Never moves a pin.
    Fetch,
    /// List the pinned tasks with their status and deprecation notes.
    List {
        /// Also ask the registry for newer versions.
        #[arg(long)]
        remote: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let ctx = ops::Ctx {
        dir: cli.dir.as_deref(),
        registry: cli.registry.as_deref(),
    };
    let result = match &cli.command {
        Command::Add { tasks } => ops::add(&ctx, tasks).map(|()| true),
        Command::Remove { tasks } => ops::remove(&ctx, tasks).map(|()| true),
        Command::Update { slugs, to_latest } => ops::update(&ctx, slugs, *to_latest),
        Command::Check => ops::check(&ctx),
        Command::Fetch => ops::fetch_locked(&ctx).map(|()| true),
        Command::List { remote } => ops::list(&ctx, *remote).map(|()| true),
    };
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}
