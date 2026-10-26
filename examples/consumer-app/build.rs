// Generates $OUT_DIR/trust_tasks_generated.rs from trust-tasks.lock and the
// bundles vendored under trust-tasks/, after checking each bundle's digest.
fn main() -> Result<(), trust_tasks_build::Error> {
    trust_tasks_build::Config::new()
        .lockfile("trust-tasks.lock")
        .compile()?;
    Ok(())
}
