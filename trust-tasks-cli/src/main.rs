//! `trust-tasks` — pin the Trust Task specifications an application uses.
//!
//! See README.md, and docs/design/consumer-side-generation.md in the
//! repository for the model: the application records its intent in
//! `trust-tasks.toml`, this command resolves it against the registry into
//! `trust-tasks.lock`, and vendors each pinned version's bundle beside it,
//! verified by digest. Builds read only the vendored files.
//!
//! Tasks may also come from third-party sources (`[sources.<name>]`, SPEC
//! §6.5), and `trust-tasks registry build` is the publisher's side of one.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

mod generate;
mod lock;
mod manifest;
mod names;
mod ops;
mod project;
mod publish;
mod registry;

#[derive(Parser)]
#[command(
    name = "trust-tasks",
    version,
    about = "Pin the Trust Task specifications an application uses, vendor their bundles by digest, and verify them.",
    after_help = "Files: trust-tasks.toml (intent, yours), trust-tasks.lock (resolution, written by this command), \
trust-tasks/<slug>/<M.N>.bundle.json (vendored bundles; trust-tasks/@<source>/… for a third-party source). Commit all three.\n\n\
A task from a third-party source is named <source>:<slug> (e.g. acme:payments/settle/1.0), with the source declared in \
trust-tasks.toml as [sources.<source>] url = \"https://…\" (or path = \"<dir>\"). TRUST_TASKS_CA_FILE names a PEM bundle \
to trust instead of the Mozilla roots."
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
    /// latest live version on every update). `acme:payments/settle/1.0` pins
    /// from the source `acme` declared in trust-tasks.toml.
    Add {
        #[arg(required = true, value_name = "[SOURCE:]SLUG[/M.N|@latest]")]
        tasks: Vec<String>,
    },
    /// Unpin tasks (`vault/upsert`) or one version of one (`vault/upsert/0.3`),
    /// deleting their vendored bundles.
    Remove {
        #[arg(required = true, value_name = "[SOURCE:]SLUG[/M.N]")]
        tasks: Vec<String>,
    },
    /// Re-resolve against the registry: `latest` intents move to the latest
    /// live version, and an exact pin whose draft changed in place takes the
    /// new bytes. Refuses withdrawn versions. No slugs means every task.
    Update {
        #[arg(value_name = "[SOURCE:]SLUG")]
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
        /// Also ask the registry (and each source) for newer versions.
        #[arg(long)]
        remote: bool,
    },
    /// Verify the lock offline, then run a language's generator over it:
    /// `npx trust-tasks-gen-ts`, `go run …/cmd/trust-tasks-gen` or
    /// `dart run trust_tasks:generate`, from the project directory, with
    /// `--lock` and `--out` filled in. Arguments after `--` go to the
    /// generator. Rust generates in build.rs instead.
    Generate {
        /// The language.
        #[arg(long, value_enum)]
        lang: generate::Lang,
        /// The directory to generate into (owned by the generator).
        #[arg(long, value_name = "DIR")]
        out: Option<PathBuf>,
        /// Passed to the generator unchanged (e.g. `-- --package tasks`).
        #[arg(last = true)]
        args: Vec<String>,
    },
    /// Publish a registry of your own: the served layout a `[sources.<name>]`
    /// reads.
    Registry {
        #[command(subcommand)]
        command: RegistryCommand,
    },
}

#[derive(Subcommand)]
enum RegistryCommand {
    /// Build bundles, specs/index.json and the by-digest store from a
    /// `specs/<slug>/<M.N>/{spec.md,payload.schema.json}` tree whose Type URIs
    /// sit under your own authority (SPEC §6.5).
    Build {
        /// The specifications: the directory holding `<slug>/<M.N>/`.
        #[arg(long, value_name = "DIR")]
        specs: PathBuf,
        /// Everything before `/spec/<slug>/<M.N>` in your Type URIs —
        /// `https://tasks.example.com`, `did:web:example.com` — or before
        /// `:spec:` in the URN form, `urn:example:tasks`. Never under
        /// https://trusttasks.org.
        #[arg(long, value_name = "PREFIX")]
        type_uri_prefix: String,
        /// The site root to write: `<out>/specs/…`.
        #[arg(long, value_name = "DIR")]
        out: PathBuf,
        /// A checkout of the official registry's `specs/`, for `$ref`s to its
        /// shared components (`https://trusttasks.org/specs/…`). Without it,
        /// they are fetched from trusttasks.org.
        #[arg(long, value_name = "DIR")]
        official_specs: Option<PathBuf>,
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
        Command::Generate { lang, out, args } => {
            generate::generate(&ctx, *lang, out.as_deref(), args)
        }
        Command::Registry {
            command:
                RegistryCommand::Build {
                    specs,
                    type_uri_prefix,
                    out,
                    official_specs,
                },
        } => publish::registry_build(specs, type_uri_prefix, out, official_specs.as_deref())
            .map(|()| true),
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
