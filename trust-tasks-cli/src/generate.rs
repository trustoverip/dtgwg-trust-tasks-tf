//! `trust-tasks generate --lang <rs|ts|go|dart>`: verify the lock offline, then
//! run that language's generator over it.
//!
//! Each language's generator ships with its library, in that library's own
//! toolchain — `trust-tasks-build` (a Cargo build-dependency),
//! `@openvtc/trust-tasks-gen` (npm), `cmd/trust-tasks-gen` in the Go module,
//! `trust_tasks:generate` in the Dart package — so this command only runs the
//! one the application already depends on, from the project directory, with
//! `--lock` and `--out` filled in. It never installs anything.

use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};
use clap::ValueEnum;

use crate::ops::{self, Ctx};
use crate::project::Project;

/// The Go generator, run with `go run` from the application's module, which
/// requires `trust-tasks-go` already.
const GO_GENERATOR: &str =
    "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/cmd/trust-tasks-gen";

/// A language to generate for.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Lang {
    /// Rust: generated at build time by `trust-tasks-build` in `build.rs`.
    Rs,
    /// TypeScript: `trust-tasks-gen-ts` from `@openvtc/trust-tasks-gen`.
    Ts,
    /// Go: `cmd/trust-tasks-gen` from the `trust-tasks-go` module.
    Go,
    /// Dart: `trust_tasks:generate` from the `trust_tasks` package.
    Dart,
}

/// The command for `lang`, and what to tell someone who does not have it.
fn generator(lang: Lang) -> Option<(&'static str, &'static [&'static str], &'static str)> {
    match lang {
        Lang::Rs => None,
        Lang::Ts => Some((
            "npx",
            &["--no-install", "trust-tasks-gen-ts"],
            "install it in the application: npm i -D @openvtc/trust-tasks-gen",
        )),
        Lang::Go => Some((
            "go",
            &["run", GO_GENERATOR],
            "run it from a Go module that requires github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go",
        )),
        Lang::Dart => Some((
            "dart",
            &["run", "trust_tasks:generate"],
            "run it from a Dart package that depends on trust_tasks",
        )),
    }
}

pub fn generate(ctx: &Ctx, lang: Lang, out: Option<&Path>, extra: &[String]) -> Result<bool> {
    let project = Project::locate(ctx.dir, false)?;
    if !ops::check(ctx)? {
        eprintln!("error: not generating from a lock that does not verify");
        return Ok(false);
    }
    let Some((program, args, hint)) = generator(lang) else {
        println!(
            "Rust generates at build time: add `trust-tasks-build` as a build-dependency and call \
             `trust_tasks_build::Config::new().compile()?` from build.rs (see the trust-tasks-build README)."
        );
        return Ok(true);
    };
    let out = out.ok_or_else(|| {
        let name = lang.to_possible_value().map(|v| v.get_name().to_string());
        anyhow!(
            "--out <dir> is required for --lang {}",
            name.unwrap_or_default()
        )
    })?;

    // The generator runs in the project directory (where its node_modules,
    // go.mod or pubspec.yaml is); paths given relative to here must survive that.
    let cwd = std::env::current_dir().context("reading the current directory")?;
    let absolute = |p: &Path| -> PathBuf { cwd.join(p) };
    let root = absolute(&project.root);
    let status = Command::new(program)
        .args(args)
        .arg("--lock")
        .arg(root.join(crate::project::LOCKFILE))
        .arg("--out")
        .arg(absolute(out))
        .args(extra)
        .current_dir(&root)
        .status();
    match status {
        Ok(status) if status.success() => Ok(true),
        Ok(status) => bail!("{program} {} exited with {status} — {hint}", args.join(" ")),
        Err(e) if e.kind() == ErrorKind::NotFound => {
            bail!("`{program}` is not on PATH — {hint}")
        }
        Err(e) => Err(e).with_context(|| format!("running {program}")),
    }
}
