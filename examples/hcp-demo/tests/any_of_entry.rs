//! A Rust-only check of the types `build.rs` generates (the application
//! target), not one of the demo's shared tests (those are `tests/demo.rs`,
//! the same in every port): `hcp/entries/query` 0.1's `entries[]` is an
//! `anyOf` of `Disclosure` and `Entry`, and a whole `Entry` reads as the
//! second branch. The demo's provider only ever answers with Disclosures,
//! and its client reads `entries` as JSON, so nothing else here decodes an
//! `Entry` through the generated type (`trust-tasks-build/src/any_of.rs`;
//! `trust-tasks-rs/tests/anyof_object_branches.rs` covers the prebuilt
//! modules).

use serde_json::json;
use trust_tasks_example_hcp_demo::query;

#[test]
fn a_whole_entry_reads_as_the_entry_branch() {
    // `hcp/entries/confirm` 0.1's response example (the `_shared` Entry).
    let entry = json!({
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
    });
    let item: query::ResponseEntriesItem =
        serde_json::from_value(entry.clone()).expect("a whole Entry reads");
    assert!(item.subtype_0.is_none() && item.subtype_1.is_some());
    assert_eq!(serde_json::to_value(&item).expect("serializes"), entry);
}
