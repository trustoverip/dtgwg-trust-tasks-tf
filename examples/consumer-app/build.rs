// Generates $OUT_DIR/trust_tasks_generated.rs from trust-tasks.lock and the
// bundles vendored under trust-tasks/, after checking each bundle's digest.
fn main() -> Result<(), trust_tasks_build::Error> {
    trust_tasks_build::Config::new()
        .lockfile("trust-tasks.lock")
        // Also a `tasks::schema_index`: the application looks up a received
        // document's schema and policy by its Type URI (src/main.rs, step 7).
        .index(true)
        .compile()?;

    // A second, wider lock for tests/index_equivalence.rs only, which holds
    // the generated index to `trust_tasks_rs::schema_index`: request and
    // response bounds, a response-only bound, a task with no response, and
    // deprecated (superseded) versions.
    let out_dir = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR"));
    trust_tasks_build::Config::new()
        .lockfile("index-equivalence/trust-tasks.lock")
        .index(true)
        .out_dir(out_dir.join("index-equivalence"))
        .compile()?;
    Ok(())
}
