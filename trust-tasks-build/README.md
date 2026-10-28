# trust-tasks-build

Build-time generator for [Trust Tasks](https://trusttasks.org/): your
`build.rs` turns the specification versions an application pins in
`trust-tasks.lock` into Rust types for [`trust-tasks-rs`](https://crates.io/crates/trust-tasks-rs).
An application compiles the three tasks it speaks, not the whole registry.

Pin tasks with the [`trust-tasks`](https://crates.io/crates/trust-tasks-cli) command
first. It writes `trust-tasks.toml` (what you asked for), `trust-tasks.lock` (the
exact versions and their `sha256:` digests) and the vendored bundles under
`trust-tasks/`. Commit all three. See
[consumer-side generation](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/docs/design/consumer-side-generation.md)
for the model.

## Use

```toml
# Cargo.toml
[dependencies]
trust-tasks-rs = { version = "0.35", default-features = false } # the runtime, no prebuilt modules

[build-dependencies]
trust-tasks-build = "0.1"
```

```rust
// build.rs
fn main() -> Result<(), trust_tasks_build::Error> {
    trust_tasks_build::Config::new() // ./trust-tasks.lock + ./trust-tasks/
        .compile()?;                // writes $OUT_DIR/trust_tasks_generated.rs
    Ok(())
}
```

```rust
// main.rs
mod tasks {
    include!(concat!(env!("OUT_DIR"), "/trust_tasks_generated.rs"));
}
use tasks::vault::get::v0_3 as vault_get;

tasks::declare_extends(); // once at start-up: routes newer versions that `extends` a locked one

let payload: vault_get::Payload = vault_get::Payload::builder().id("entry-42").try_into()?;
```

Each version sits at the module path `trust_tasks_rs::specs` gives it
(`<family>::<task>::v<M>_<N>`, hyphens as underscores), and holds what the
prebuilt module holds: the `Payload`/`Response` types and their builders, the
`trust_tasks_rs::Payload` impls with the version's policy constants, `EXTENDS`,
`DEPRECATION` and `ERROR_CODES`. The code is the same generator's output.

The application needs no serde, serde_json, regress or chrono dependency of its
own: generated code reaches them through `trust-tasks-rs`.

`Config` also takes `.lockfile(path)`, `.bundle_dir(path)` and `.out_dir(path)`
when the files are not in the default places.

## What it checks

Before generating anything, every vendored bundle is checked against the digest
the lockfile pins, and must name the slug, version and Type URI it is locked as.
A mismatch stops the build with `vendored bundle does not match trust-tasks.lock
— run trust-tasks check`. Builds never use the network.

## Third-party sources

A lockfile may also pin specifications from a third-party source: specifications
under an authority other than `https://trusttasks.org` (SPEC §6.5), declared
with `[sources.<name>]` in `trust-tasks.toml`. Each source's tasks get a
top-level module named for it, so source `acme`'s `payments/settle` 1.0 is
`tasks::acme::payments::settle::v1_0`. Identity is always the bundle's Type URI,
so a sourced `vault/get` and the registry's `vault/get` are different types and
never route into each other.

The build refuses:
- a source named like an official family in the same lock;
- a sourced bundle that claims the registry's authority;
- a sourced bundle with a framework slug (`trust-task*`, `trust-ceremony*`).

## Features

- **`publish`** (off by default) is the publisher's side. It builds registry
  bundles, `index.json` and the by-digest store from a `specs/` tree. It is what
  `trust-tasks registry build` and this repository's own codegen use. An
  application's build script never needs it.

## License

Apache-2.0.
