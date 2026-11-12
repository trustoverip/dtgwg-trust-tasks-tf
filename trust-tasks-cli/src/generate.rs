//! `trust-tasks generate --lang <rs|ts|go|dart>`: verify the lock offline, then
//! run that language's generator over it.
//!
//! Each language's generator ships with its library, in that library's own
//! toolchain — `@openvtc/trust-tasks-gen` (npm), `cmd/trust-tasks-gen` in the
//! Go module, `trust_tasks:generate` in the Dart package — so this command only
//! runs the one the application already depends on, from the project
//! directory, with `--lock` and `--out` filled in. It never installs anything.
//!
//! Rust's generator, `trust-tasks-build`, is a library this command links, so
//! `--lang rs` runs it in-process: it writes the file a `build.rs` would write
//! into `OUT_DIR`, for reading or reviewing the generated code. An application
//! still generates in its `build.rs`.

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
    /// Rust: `trust-tasks-build`, run in-process — the file `build.rs`
    /// would write.
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
    let Some((program, args, hint)) = generator(lang) else {
        return generate_rust(&root, &absolute(out), extra).map(|()| true);
    };
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

/// `--lang rs`: run `trust-tasks-build` over the lock and write
/// `<out>/trust_tasks_generated.rs` — the file `Config::compile` writes into
/// `OUT_DIR` from a `build.rs`. The arguments after `--` are this generator's
/// options: `--index` (the `schema_index` module, `Config::index`) and
/// `--runtime-path <path>` (`Config::runtime_path`).
fn generate_rust(root: &Path, out: &Path, extra: &[String]) -> Result<()> {
    let mut config = trust_tasks_build::Config::new()
        // Absolute, so the build crate's `CARGO_MANIFEST_DIR` base never applies.
        .lockfile(root.join(crate::project::LOCKFILE))
        .out_dir(out);
    let mut args = extra.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--index" => config = config.index(true),
            "--runtime-path" => {
                let path = args
                    .next()
                    .ok_or_else(|| anyhow!("--runtime-path needs a path, e.g. `::trust_tasks`"))?;
                if !is_rust_path(path) {
                    bail!("--runtime-path `{path}` is not a Rust path, e.g. `::trust_tasks`");
                }
                config = config.runtime_path(path);
            }
            other => bail!(
                "--lang rs takes `--index` and `--runtime-path <path>` after `--`, not `{other}`"
            ),
        }
    }
    let generated = config.generate().map_err(|e| anyhow!(e))?;
    std::fs::create_dir_all(out).with_context(|| format!("creating {}", out.display()))?;
    let path = out.join(trust_tasks_build::GENERATED_FILE);
    std::fs::write(&path, &generated.source)
        .with_context(|| format!("writing {}", path.display()))?;
    println!("{:>9} {}", "Generated", path.display());
    println!(
        "note: this is for reading the generated code — an application generates it in build.rs \
         with `trust_tasks_build::Config::new().compile()?` (see the trust-tasks-build README)"
    );
    Ok(())
}

/// `::a::b`, `a::b` or `crate::x` — segments of ASCII letters, digits and `_`,
/// none starting with a digit.
fn is_rust_path(path: &str) -> bool {
    let body = path.strip_prefix("::").unwrap_or(path);
    !body.is_empty()
        && body.split("::").all(|seg| {
            seg.chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
                && seg.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                && seg != "_"
        })
}

#[cfg(test)]
mod tests {
    use super::is_rust_path;

    #[test]
    fn runtime_paths() {
        for ok in ["::trust_tasks_rs", "trust_tasks", "crate::rt", "::a::b2"] {
            assert!(is_rust_path(ok), "{ok}");
        }
        for bad in ["", "::", "a::", "1a", "a b", "a::-b", "_", "a;b"] {
            assert!(!is_rust_path(bad), "{bad}");
        }
    }
}
