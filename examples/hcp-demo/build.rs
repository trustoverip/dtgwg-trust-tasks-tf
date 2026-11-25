// Generates $OUT_DIR/trust_tasks_generated.rs from trust-tasks.lock and the
// bundles vendored under trust-tasks/, after checking each bundle's digest.
// The generated code reaches the runtime through the facade.
fn main() -> Result<(), trust_tasks_build::Error> {
    trust_tasks_build::Config::new()
        .runtime_path("::trust_tasks")
        .compile()?;
    Ok(())
}
