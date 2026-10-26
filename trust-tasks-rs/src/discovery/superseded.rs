// --- trust-task-discovery 0.1 / 0.2 (superseded) ----------------------------
//! `trust-task-discovery` 0.1 and 0.2 — superseded by 0.3, and answered only
//! because a responder answers each version in the version it was asked.
//!
//! **Everything for those two versions is in this file**, so withdrawing them
//! is deleting it and the `mod superseded;` line in `discovery.rs` — the
//! methods [`DiscoveryRegistry::respond_to`] (0.1) and
//! [`DiscoveryRegistry::respond_to_v0_2`] (0.2), and their tests, go with it.
//! Nothing else in the crate calls them.

// The generated 0.1 and 0.2 types are deprecated (superseded); naming them is
// the point of this file.
#![allow(deprecated)]

use super::{framework_version_parts, DiscoveryRegistry};
use crate::specs::trust_task_discovery::v0_1 as wire;
use crate::specs::trust_task_discovery::v0_2 as wire_v0_2;

impl DiscoveryRegistry {
    /// Build a response to a `trust-task-discovery/0.1` `query`: the same
    /// matching as [`Self::respond_to_v0_3`], with `frameworkVersion` written
    /// `MAJOR.MINOR` — 0.1 admits nothing finer, so the configured release is
    /// written without its PATCH — and no acceptance window, which 0.1
    /// cannot carry.
    ///
    /// URIs with no `requiredExt` policy are emitted in shorthand form
    /// ([`wire::ResponseSupportedTypesItem::Uri`]); URIs with a policy
    /// declared via [`Self::with_required_ext`] / [`Self::require_ext`]
    /// are emitted in expanded form
    /// ([`wire::ResponseSupportedTypesItem::Object`]) carrying the
    /// `requiredExt` array.
    ///
    /// 0.1 is superseded by 0.3; this is removed when 0.1 is withdrawn.
    pub fn respond_to(&self, query: &wire::Payload) -> wire::Response {
        // Generated `Payload` wraps each pattern in a `PayloadPatternsItem`
        // newtype; deref to &str for matching.
        let patterns: Vec<&str> = query.patterns.iter().map(|p| p.as_str()).collect();
        let supported_types: Vec<wire::ResponseSupportedTypesItem> = self
            .matching(&patterns)
            .map(|uri| match self.required_ext_for(uri) {
                // Namespaces that don't match the reverse-DNS pattern are
                // skipped rather than panicking on caller-supplied input.
                Some(required_ext) => wire::ResponseSupportedTypesItem::Object {
                    type_: uri.to_string(),
                    required_ext: Some(required_ext.filter_map(|ns| ns.parse().ok()).collect()),
                },
                None => wire::ResponseSupportedTypesItem::Uri(uri.to_string()),
            })
            .collect();

        // A `framework_version` override that is not a version is omitted
        // rather than panicking on this caller-supplied value.
        let framework_version = self
            .framework_version
            .as_deref()
            .and_then(framework_version_parts)
            .and_then(|(major, minor, _)| format!("{major}.{minor}").parse().ok());

        wire::Response {
            supported_types,
            framework_version,
        }
    }

    /// Build a response to a `trust-task-discovery/0.2` `query`: the same
    /// matching as [`Self::respond_to_v0_3`], with `frameworkVersion` written
    /// as the full three-part release (SPEC §5.1.1) — a two-part configured
    /// value is written as its `.0` — and no acceptance window, which 0.2
    /// cannot carry.
    ///
    /// 0.2 is superseded by 0.3; this is removed when 0.2 is withdrawn.
    pub fn respond_to_v0_2(&self, query: &wire_v0_2::Payload) -> wire_v0_2::Response {
        let patterns: Vec<&str> = query.patterns.iter().map(|p| p.as_str()).collect();
        let supported_types: Vec<wire_v0_2::ResponseSupportedTypesItem> = self
            .matching(&patterns)
            .map(|uri| match self.required_ext_for(uri) {
                Some(required_ext) => wire_v0_2::ResponseSupportedTypesItem::Object {
                    type_: uri.to_string(),
                    required_ext: Some(required_ext.filter_map(|ns| ns.parse().ok()).collect()),
                },
                None => wire_v0_2::ResponseSupportedTypesItem::Uri(uri.to_string()),
            })
            .collect();

        wire_v0_2::Response {
            supported_types,
            framework_version: self
                .three_part_framework_version()
                .and_then(|v| v.parse().ok()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::type_uri::TypeUri;
    use std::num::NonZeroU64;

    fn uri_of(entry: &wire::ResponseSupportedTypesItem) -> &str {
        match entry {
            wire::ResponseSupportedTypesItem::Uri(s) => s.as_str(),
            wire::ResponseSupportedTypesItem::Object { type_, .. } => type_.as_str(),
        }
    }

    fn registry() -> DiscoveryRegistry {
        DiscoveryRegistry::new()
            .with_type_uri(TypeUri::canonical("acl/grant", 0, 2).unwrap())
            .with_type_uri(TypeUri::canonical("acl/revoke", 0, 2).unwrap())
            .with_type_uri(TypeUri::canonical("trust-task-discovery", 0, 3).unwrap())
    }

    #[test]
    fn v0_1_filters_by_pattern_and_writes_major_minor() {
        let response = registry().respond_to(&wire::Payload {
            patterns: vec!["acl/*".parse().unwrap()],
        });
        assert_eq!(
            response
                .supported_types
                .iter()
                .map(uri_of)
                .collect::<Vec<_>>(),
            vec![
                "https://trusttasks.org/spec/acl/grant/0.2",
                "https://trusttasks.org/spec/acl/revoke/0.2",
            ]
        );
        assert_eq!(
            response.framework_version.as_ref().map(|v| v.to_string()),
            Some("0.6".to_string())
        );
        let everything = registry().respond_to(&wire::Payload { patterns: vec![] });
        assert_eq!(everything.supported_types.len(), 3);
    }

    #[test]
    fn no_framework_version_suppresses_field() {
        let registry = DiscoveryRegistry::new().no_framework_version();
        let response = registry.respond_to(&wire::Payload { patterns: vec![] });
        assert!(response.framework_version.is_none());
        let response = registry.respond_to_v0_2(&wire_v0_2::Payload { patterns: vec![] });
        assert!(response.framework_version.is_none());
    }

    #[test]
    fn override_framework_version_is_emitted_verbatim_in_0_1() {
        let registry = DiscoveryRegistry::new().framework_version("0.2");
        let response = registry.respond_to(&wire::Payload { patterns: vec![] });
        assert_eq!(
            response.framework_version.as_ref().map(|v| v.to_string()),
            Some("0.2".to_string())
        );
    }

    #[test]
    fn v0_2_writes_the_full_three_part_release() {
        let response = registry().respond_to_v0_2(&wire_v0_2::Payload { patterns: vec![] });
        assert_eq!(
            response.framework_version.as_ref().map(|v| v.to_string()),
            Some("0.6.0".to_string())
        );
        assert_eq!(response.supported_types.len(), 3);
    }

    #[test]
    fn a_patch_release_survives_in_0_2_and_is_dropped_only_in_0_1() {
        // 0.1 cannot express a PATCH; 0.2 exists so that it can.
        let registry = DiscoveryRegistry::new().framework_version("0.6.1");
        let v1 = registry.respond_to(&wire::Payload { patterns: vec![] });
        let v2 = registry.respond_to_v0_2(&wire_v0_2::Payload { patterns: vec![] });
        assert_eq!(
            v1.framework_version.map(|v| v.to_string()),
            Some("0.6".into())
        );
        assert_eq!(
            v2.framework_version.map(|v| v.to_string()),
            Some("0.6.1".into())
        );
    }

    #[test]
    fn a_two_part_override_is_written_as_its_dot_zero_in_0_2() {
        // SPEC §5.1.1: a two-part M.N is the release M.N.0.
        let registry = DiscoveryRegistry::new().framework_version("0.5");
        let v2 = registry.respond_to_v0_2(&wire_v0_2::Payload { patterns: vec![] });
        assert_eq!(
            v2.framework_version.map(|v| v.to_string()),
            Some("0.5.0".into())
        );
    }

    #[test]
    fn the_0_2_schema_refuses_a_two_part_framework_version() {
        assert!("0.6"
            .parse::<wire_v0_2::ResponseFrameworkVersion>()
            .is_err());
        assert!("0.6.0"
            .parse::<wire_v0_2::ResponseFrameworkVersion>()
            .is_ok());
    }

    #[test]
    fn malformed_framework_version_override_is_omitted_not_panicked() {
        let registry = DiscoveryRegistry::new().framework_version("not-a-version");
        let response = registry.respond_to(&wire::Payload { patterns: vec![] });
        assert!(response.framework_version.is_none());
        let response = registry.respond_to_v0_2(&wire_v0_2::Payload { patterns: vec![] });
        assert!(response.framework_version.is_none());
    }

    #[test]
    fn required_ext_is_carried_in_expanded_form_and_a_window_never_is() {
        let grant_uri = TypeUri::canonical("acl/grant", 0, 2).unwrap();
        let registry = registry()
            .with_required_ext(grant_uri.clone(), ["vnd.affinidi.webvh"])
            .acceptance_window(NonZeroU64::new(600).unwrap(), 60)
            .with_acceptance_window_for(
                TypeUri::canonical("acl/revoke", 0, 2).unwrap(),
                NonZeroU64::new(120).unwrap(),
                30,
            );

        let v1 = registry.respond_to(&wire::Payload { patterns: vec![] });
        match &v1.supported_types[0] {
            wire::ResponseSupportedTypesItem::Object { required_ext, .. } => {
                let namespaces: Vec<String> = required_ext
                    .as_ref()
                    .expect("requiredExt populated")
                    .iter()
                    .map(|n| n.to_string())
                    .collect();
                assert_eq!(namespaces, vec!["vnd.affinidi.webvh".to_string()]);
            }
            other => panic!("expected expanded Object form, got {other:?}"),
        }
        // acl/revoke has only a window, which 0.1 cannot carry → shorthand.
        assert!(matches!(
            &v1.supported_types[1],
            wire::ResponseSupportedTypesItem::Uri(_)
        ));

        let v2 = registry.respond_to_v0_2(&wire_v0_2::Payload { patterns: vec![] });
        assert!(matches!(
            &v2.supported_types[0],
            wire_v0_2::ResponseSupportedTypesItem::Object { required_ext: Some(ns), .. } if ns.len() == 1
        ));
        assert!(matches!(
            &v2.supported_types[1],
            wire_v0_2::ResponseSupportedTypesItem::Uri(_)
        ));
        // Neither response type has anywhere to put a window: the closed 0.1
        // and 0.2 schemas would refuse one.
        let json = serde_json::to_value(&v2).unwrap();
        assert!(json.get("acceptanceWindow").is_none());
    }
}
// --- end trust-task-discovery 0.1 / 0.2 (superseded) ------------------------
