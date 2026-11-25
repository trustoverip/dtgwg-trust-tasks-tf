//! The HCP-over-Trust-Tasks demo: run the five steps and print every document.
//!
//! ```sh
//! cargo run -p trust-tasks-example-hcp-demo
//! ```
//!
//! A DEMONSTRATION, not a reference provider. See README.md and
//! `examples/hcp-demo-ts/SCENARIO.md`.

use trust_tasks_example_hcp_demo::{run_demo, Error, World};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let world = World::new();
    let result = run_demo(&world).await?;
    print!("{}", result.transcript);
    let code = &result.post_revoke_error["payload"]["code"];
    if code != "hcp:invalidToken" {
        return Err(format!("expected hcp:invalidToken after revoke, got {code}").into());
    }
    println!(
        "\nhcp-demo: discover, grant request (held, then granted), query, revoke, notice and post-revoke refusal all OK"
    );
    Ok(())
}
