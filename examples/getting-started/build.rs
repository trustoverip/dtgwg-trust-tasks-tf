// Generates $OUT_DIR/trust_tasks_generated.rs from trust-tasks.lock and the
// bundle vendored under trust-tasks/, after checking its digest. The generated
// code reaches the runtime through the facade.
fn main() -> Result<(), trust_tasks_build::Error> {
    trust_tasks_build::Config::new()
        .runtime_path("::trust_tasks")
        .compile()?;
    Ok(())
}
