// --- trust-task-discovery 0.1 / 0.2 (superseded) ----------------------------
//! `trust-task-discovery` 0.1 and 0.2 on the HTTPS server — superseded by 0.3
//! and served only because a responder answers each version in the version it
//! was asked.
//!
//! Everything the server does for those two versions is in this file; the
//! withdrawal batch deletes it together with the marked blocks in
//! `server.rs` that name it.

// The generated 0.1 and 0.2 types are deprecated (superseded); naming them is
// the point of this file.
#![allow(deprecated)]

use std::sync::Arc;

use trust_tasks_rs::{
    discovery::DiscoveryRegistry, specs::trust_task_discovery::v0_1 as discovery_v0_1,
    specs::trust_task_discovery::v0_2 as discovery_v0_2,
};

use super::{refuse_unauthenticated_discovery, HttpsServerBuilder};

/// Advertise discovery 0.1 and 0.2 themselves in an `enable_discovery()`
/// registry.
pub(super) fn register(registry: &mut DiscoveryRegistry) {
    registry.register_payload::<discovery_v0_1::Payload>();
    registry.register_payload::<discovery_v0_2::Payload>();
}

impl HttpsServerBuilder {
    /// Install the 0.1 and 0.2 discovery handlers, answering from `registry`
    /// under the same discovery-privacy gate as 0.3.
    pub(super) fn with_superseded_discovery(self, registry: DiscoveryRegistry) -> Self {
        let public_v0_1 = Arc::clone(&self.public_discovery);
        let public_v0_2 = Arc::clone(&self.public_discovery);
        let registry_v0_2 = registry.clone();
        self.on::<discovery_v0_1::Payload, _>(move |req, ctx| {
            refuse_unauthenticated_discovery(&public_v0_1, ctx.authenticated_sender.is_none())?;
            Ok(registry.respond_to(&req.payload))
        })
        .on::<discovery_v0_2::Payload, _>(move |req, ctx| {
            refuse_unauthenticated_discovery(&public_v0_2, ctx.authenticated_sender.is_none())?;
            Ok(registry_v0_2.respond_to_v0_2(&req.payload))
        })
    }
}
// --- end trust-task-discovery 0.1 / 0.2 (superseded) ------------------------
