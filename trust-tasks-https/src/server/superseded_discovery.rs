// --- trust-task-discovery 0.1 / 0.2 / 0.3 (superseded) ----------------------
//! `trust-task-discovery` 0.1, 0.2 and 0.3 on the HTTPS server — superseded by
//! 0.4 and served only because a responder answers each version in the version
//! it was asked.
//!
//! Everything the server does for those versions is in this file; the
//! withdrawal batch deletes it together with the marked blocks in
//! `server.rs` that name it.

// The generated 0.1, 0.2 and 0.3 types are deprecated (superseded); naming them
// is the point of this file.
#![allow(deprecated)]

use std::sync::Arc;

use trust_tasks_rs::{
    discovery::DiscoveryRegistry, specs::trust_task_discovery::v0_1 as discovery_v0_1,
    specs::trust_task_discovery::v0_2 as discovery_v0_2,
    specs::trust_task_discovery::v0_3 as discovery_v0_3,
};

use super::{refuse_unauthenticated_discovery, HttpsServerBuilder};

/// Advertise discovery 0.1, 0.2 and 0.3 themselves in an `enable_discovery()`
/// registry.
pub(super) fn register(registry: &mut DiscoveryRegistry) {
    registry.register_payload::<discovery_v0_1::Payload>();
    registry.register_payload::<discovery_v0_2::Payload>();
    registry.register_payload::<discovery_v0_3::Payload>();
}

impl HttpsServerBuilder {
    /// Install the 0.1, 0.2 and 0.3 discovery handlers, answering from
    /// `registry` under the same discovery-privacy gate as 0.4. With
    /// `advertise_freshness`, a 0.3 response states the window of the
    /// freshness policy the built server applies, without the expiry horizon
    /// 0.3 cannot carry.
    pub(super) fn with_superseded_discovery(
        self,
        registry: DiscoveryRegistry,
        advertise_freshness: bool,
    ) -> Self {
        let public_v0_1 = Arc::clone(&self.public_discovery);
        let public_v0_2 = Arc::clone(&self.public_discovery);
        let public_v0_3 = Arc::clone(&self.public_discovery);
        let applied_freshness = Arc::clone(&self.applied_freshness);
        let registry_v0_2 = registry.clone();
        let registry_v0_3 = registry.clone();
        self.on_sync::<discovery_v0_1::Payload, _>(
            move |req, ctx| {
                refuse_unauthenticated_discovery(&public_v0_1, ctx.authenticated_sender.is_none())?;
                Ok(registry.respond_to(&req.payload))
            },
            false,
        )
        .on_sync::<discovery_v0_2::Payload, _>(
            move |req, ctx| {
                refuse_unauthenticated_discovery(&public_v0_2, ctx.authenticated_sender.is_none())?;
                Ok(registry_v0_2.respond_to_v0_2(&req.payload))
            },
            false,
        )
        .on_sync::<discovery_v0_3::Payload, _>(
            move |req, ctx| {
                refuse_unauthenticated_discovery(&public_v0_3, ctx.authenticated_sender.is_none())?;
                Ok(match applied_freshness.get() {
                    Some(policy) if advertise_freshness => registry_v0_3
                        .clone()
                        .acceptance_window_from(policy)
                        .respond_to_v0_3(&req.payload),
                    _ => registry_v0_3.respond_to_v0_3(&req.payload),
                })
            },
            false,
        )
    }
}
// --- end trust-task-discovery 0.1 / 0.2 / 0.3 (superseded) ------------------
