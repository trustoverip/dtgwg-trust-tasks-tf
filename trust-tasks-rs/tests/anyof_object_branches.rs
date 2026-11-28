//! A member whose schema is an `anyOf` of object branches that share members.
//!
//! typify renders such an `anyOf` as a struct with one
//! `#[serde(flatten)] Option<Branch>` field per branch (`subtype_0`,
//! `subtype_1`, ...). serde deserializes flattened fields one after another
//! from one buffer, and a flattened struct takes the entries it recognises out
//! of that buffer even when it then fails. So when the first branch fails, the
//! members it shares with the next branch are already gone, and the next
//! branch fails too: `hcp/entries/create` 0.1's `entry`, an `anyOf` of
//! `Disclosure` and `Entry`, read a whole `Entry` as `ResponseEntry {}` and
//! wrote it back as `{}`.
//!
//! The generator now gives such a struct a `Deserialize` that reads each
//! branch from its own copy of the value (`apply_any_of_deserialize` in
//! `trust-tasks-build`), and refuses a value no branch reads. Each case pins a
//! round trip through the generated type.

// Exercises the prebuilt specification modules on purpose; they are deprecated
// in favour of generating the tasks an application uses (trust-tasks-build).
#![allow(deprecated)]

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

/// Deserialize `value` into `T`, re-serialize it, and require the same JSON
/// back.
#[allow(dead_code)]
fn round_trips<T: DeserializeOwned + Serialize>(value: Value) -> T {
    let parsed: T = serde_json::from_value(value.clone())
        .unwrap_or_else(|e| panic!("deserialize {value}: {e}"));
    let rendered = serde_json::to_value(&parsed).expect("re-serialize");
    assert_eq!(rendered, value, "round-trip changed the value");
    parsed
}

#[cfg(feature = "hcp")]
mod hcp {
    use super::*;
    use serde_json::json;
    use trust_tasks_rs::specs::hcp::entries::create::v0_1 as create;
    use trust_tasks_rs::specs::hcp::entries::get::v0_1 as get;
    use trust_tasks_rs::specs::hcp::entries::query::v0_1 as query;
    use trust_tasks_rs::specs::hcp::entries::revise::v0_1 as revise;

    /// `hcp/entries/create` 0.1's own response example: a `Disclosure`, the
    /// first branch.
    fn disclosure() -> Value {
        json!({
            "ref": "hcp-ref-c73e19",
            "version": 1,
            "content": {
                "profile": "https://hcp.example/spec/profiles/general-context/2.0",
                "value": "Prefers a window table.",
                "language": "en"
            },
            "assertion": {
                "declarationType": "model-inferred",
                "confidence": "medium",
                "basis": { "task": "Restaurant booking", "rationale": "Asked for a window seat at three bookings." }
            },
            "classification": { "categories": ["general_preferences"], "dimension": "seating" },
            "standing": { "assurance": "authenticated", "confirmed": false },
            "useUntil": "2026-11-26T10:00:01Z"
        })
    }

    /// A whole `Entry`, the second branch: `hcp/entries/confirm` 0.1's
    /// response example (the same `_shared` definition).
    fn entry() -> Value {
        json!({
            "schema": "https://hcp.example/spec/entry/2.0",
            "id": "urn:uuid:5affd201-734e-4217-a7b6-dc4dd177f561",
            "type": ["ContextEntry"],
            "subject": { "id": "https://context.provider.example/subjects/p-7f3a9c", "relationship": null },
            "content": {
                "profile": "https://hcp.example/spec/profiles/general-context/2.0",
                "value": "Prefers an aisle seat on flights longer than three hours.",
                "language": "en"
            },
            "assertion": {
                "declarationType": "model-inferred",
                "confidence": "medium",
                "basis": { "rationale": "Chose an aisle seat on each of the last four long-haul bookings." }
            },
            "classification": { "categories": ["general_preferences"], "dimension": "flight-seat" },
            "record": {
                "status": "active",
                "version": 2,
                "provenance": {
                    "assertedBy": { "id": "https://travel-assistant.example/clients/planner", "assurance": "authenticated" },
                    "assertedAt": "2026-11-20T14:02:00Z",
                    "source": "integration",
                    "submittedByClient": {
                        "id": "https://travel-assistant.example/clients/planner",
                        "name": "Trip Planner",
                        "operator": { "id": "https://travel-assistant.example", "name": "Travel Assistant Ltd" }
                    },
                    "authorizedUnder": "urn:uuid:eb54c220-b64f-40ad-a794-37a48095ce0b",
                    "recordedByProvider": "https://context.provider.example",
                    "recordedAt": "2026-11-20T14:02:01Z",
                    "confirmations": [
                        {
                            "disposition": "confirmed",
                            "confirmedBy": { "id": "https://context.provider.example/subjects/p-7f3a9c" },
                            "confirmedAt": "2026-11-25T09:00:01Z"
                        }
                    ]
                },
                "relationships": [
                    {
                        "type": "contradicts",
                        "target": "urn:uuid:b4f00822-4753-40a6-84cf-8fc51693b59e",
                        "assertedBy": { "role": "hcp-provider", "id": "https://context.provider.example" },
                        "recordedAt": "2026-11-22T08:15:00Z"
                    }
                ]
            }
        })
    }

    #[test]
    fn create_response_keeps_a_disclosure() {
        let r: create::Response = round_trips(json!({ "entry": disclosure() }));
        assert!(r.entry.subtype_0.is_some() && r.entry.subtype_1.is_none());
    }

    #[test]
    fn create_response_keeps_a_whole_entry() {
        let r: create::Response = round_trips(json!({ "entry": entry() }));
        assert!(r.entry.subtype_0.is_none() && r.entry.subtype_1.is_some());
    }

    #[test]
    fn revise_response_keeps_a_whole_entry() {
        let r: revise::Response = round_trips(json!({ "entry": entry() }));
        assert!(r.entry.subtype_1.is_some());
    }

    #[test]
    fn get_and_query_keep_either_branch() {
        for v in [disclosure(), entry()] {
            round_trips::<get::ResponseEntry>(v.clone());
            round_trips::<query::ResponseEntriesItem>(v);
        }
    }

    /// `anyOf` needs one branch to match. The flattened struct read a value
    /// no branch matches as an empty struct and wrote it back as `{}`.
    #[test]
    fn a_value_no_branch_reads_is_refused() {
        for v in [
            json!({}),
            json!({ "ref": "hcp-ref-c73e19" }),
            json!("entry"),
        ] {
            let err = serde_json::from_value::<create::ResponseEntry>(v.clone())
                .expect_err("a value matching no anyOf branch must not read");
            assert!(err.to_string().contains("ResponseEntry"), "{v}: {err}");
        }
    }

    /// `hcp/entries/create` 0.1's own request example.
    #[test]
    fn create_payload_example_round_trips() {
        round_trips::<create::Payload>(json!({
            "schema": "https://hcp.example/spec/entry/2.0",
            "subject": { "id": "urn:uuid:3f8a1c2e-5b7d-4e9f-a0c6-8d2b4e1f7a35" },
            "content": {
                "profile": "https://hcp.example/spec/profiles/general-context/2.0",
                "value": "Prefers a window table.",
                "language": "en"
            },
            "classification": { "categories": ["general_preferences"], "dimension": "seating" },
            "assertion": {
                "declarationType": "model-inferred",
                "confidence": "medium",
                "basis": { "task": "Restaurant booking", "rationale": "Asked for a window seat at three bookings." }
            },
            "record": {
                "provenance": {
                    "assertedBy": { "id": "https://booking-agent.example/model/v4" },
                    "source": "integration"
                }
            },
            "accessToken": "hcp_at_2b7f0c6e9d1a4e8b"
        }));
    }
}
