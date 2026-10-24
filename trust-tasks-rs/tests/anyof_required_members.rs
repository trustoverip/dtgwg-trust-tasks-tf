//! Payloads whose schema says "at least one of these members" with an
//! `anyOf` of bare `required` clauses.
//!
//! typify used to model that `anyOf` as an untagged enum with one variant per
//! branch, each holding only its own branch's member under
//! `deny_unknown_fields`, so a payload carrying two of the members — which the
//! schema accepts — deserialized into no variant. The codegen now drops the
//! `anyOf` from the copy of the schema typify sees
//! (`drop_anyof_of_required_members` in `trust-tasks-codegen`): the members
//! are ordinary optional fields of one struct, and the wire schema behind
//! `ValidatedPayload::validate_value` still enforces "at least one".
//!
//! Each case pins both halves: the combined payload round-trips through the
//! generated type, and a payload with none of the members is still refused by
//! validation.

use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};

/// Deserialize `value` into `T`, re-serialize it, and require the same JSON
/// back — the property the untagged enum broke.
#[allow(dead_code)]
fn round_trips<T: DeserializeOwned + Serialize>(value: Value) -> T {
    let parsed: T = serde_json::from_value(value.clone())
        .unwrap_or_else(|e| panic!("deserialize {value}: {e}"));
    let rendered = serde_json::to_value(&parsed).expect("re-serialize");
    assert_eq!(rendered, value, "round-trip changed the payload");
    parsed
}

#[cfg(feature = "rooms")]
mod rooms {
    use super::*;
    use trust_tasks_rs::specs::rooms::keys::chain::v0_2 as chain;
    use trust_tasks_rs::specs::rooms::keys::profile::write::v0_1 as profile_write;
    use trust_tasks_rs::specs::rooms::records::curate::v0_1 as curate;

    fn chain_with(members: Value) -> Value {
        let mut v = json!({ "roomId": "did:webvh:example.com:rooms:northwind" });
        v.as_object_mut()
            .unwrap()
            .extend(members.as_object().unwrap().clone());
        v
    }

    fn rungs() -> Value {
        json!([
            { "epoch": 4, "wrapped": "Lp3wXc9TgYw2mQnR4vBk8Q", "nonce": "c1Au9Rn3Zr2tWwS1" },
            { "epoch": 3, "wrapped": "Tf6yNb2VhZx5pRoS7wCl9Q", "nonce": "d2Bv0So4As3uXxT2" }
        ])
    }

    /// The normal case for `rooms/keys/chain/0.2`: a member relays the rungs
    /// together with the floor the host served them with.
    #[test]
    fn chain_with_links_and_epoch_floor_round_trips() {
        let payload = chain_with(json!({ "links": rungs(), "epochFloor": 2 }));
        let parsed: chain::Payload = round_trips(payload);
        assert_eq!(parsed.links.len(), 2);
        assert_eq!(parsed.epoch_floor.map(|f| f.get()), Some(2));
    }

    #[test]
    fn chain_with_one_member_still_round_trips() {
        let _: chain::Payload = round_trips(chain_with(json!({ "links": rungs() })));
        let _: chain::Payload = round_trips(chain_with(json!({ "epochFloor": 3 })));
    }

    #[cfg(feature = "validate")]
    #[test]
    fn chain_validation_still_requires_links_or_epoch_floor() {
        use trust_tasks_rs::validate::ValidatedPayload;
        let both = chain_with(json!({ "links": rungs(), "epochFloor": 2 }));
        chain::Payload::validate_value(&both).expect("both members are valid");
        let neither = chain_with(json!({}));
        chain::Payload::validate_value(&neither)
            .expect_err("a payload with neither links nor epochFloor must be refused");
    }

    fn curate_with(members: Value) -> Value {
        let mut v = json!({
            "roomId": "did:webvh:example.com:rooms:northwind",
            "key": "notes/standup",
            "presentation": {
                "membership": "eyJhbGciOiJFZERTQSJ9.e30.sig",
                "authority": ["eyJhbGciOiJFZERTQSJ9.e30.sig"]
            }
        });
        v.as_object_mut()
            .unwrap()
            .extend(members.as_object().unwrap().clone());
        v
    }

    /// `status` and `pinned` are orthogonal: demoting and pinning a record in
    /// one curation is a single valid request.
    #[test]
    fn curate_with_status_and_pinned_round_trips() {
        let parsed: curate::Payload = round_trips(curate_with(
            json!({ "status": "deprecated", "pinned": true }),
        ));
        assert!(parsed.status.is_some());
        assert_eq!(parsed.pinned, Some(true));
    }

    #[cfg(feature = "validate")]
    #[test]
    fn curate_validation_still_requires_status_or_pinned() {
        use trust_tasks_rs::validate::ValidatedPayload;
        let both = curate_with(json!({ "status": "deprecated", "pinned": true }));
        curate::Payload::validate_value(&both).expect("both members are valid");
        curate::Payload::validate_value(&curate_with(json!({})))
            .expect_err("a curation with neither status nor pinned must be refused");
    }

    fn profile_write_with(members: Value) -> Value {
        let mut v = json!({
            "roomId": "did:webvh:example.com:rooms:northwind",
            "host": "did:webvh:example.com:hosts:a"
        });
        v.as_object_mut()
            .unwrap()
            .extend(members.as_object().unwrap().clone());
        v
    }

    #[test]
    fn profile_write_clear_alone_round_trips() {
        let _: profile_write::Payload = round_trips(profile_write_with(json!({ "clear": true })));
    }

    #[cfg(feature = "validate")]
    #[test]
    fn profile_write_validation_still_requires_profile_or_clear() {
        use trust_tasks_rs::validate::ValidatedPayload;
        profile_write::Payload::validate_value(&profile_write_with(json!({})))
            .expect_err("a write with neither profile nor clear must be refused");
    }

    /// `profile/write` is exactly-one, not at-least-one: its schema adds
    /// `dependentSchemas: {clear: {not: {required: [profile]}}}`. The old
    /// enum happened to encode that; the struct does not, so it is the wire
    /// schema that refuses both — which is where the guarantee always lived
    /// for every other binding.
    #[cfg(feature = "validate")]
    #[test]
    fn profile_write_validation_still_refuses_profile_with_clear() {
        use trust_tasks_rs::validate::ValidatedPayload;
        let profile = json!({ "name": "Northwind" });
        profile_write::Payload::validate_value(&profile_write_with(json!({ "profile": profile })))
            .expect("profile alone is valid");
        let both = profile_write_with(json!({ "clear": true, "profile": profile }));
        profile_write::Payload::validate_value(&both)
            .expect_err("profile and clear together must be refused");
    }
}

#[cfg(feature = "vtc")]
mod vtc {
    use super::*;
    use trust_tasks_rs::specs::vtc::directory::notes::append::v0_1 as append;

    /// A note that redacts an earlier one and says why in the same entry.
    #[test]
    fn notes_append_with_text_and_redacts_round_trips() {
        let payload = json!({
            "entryId": "dir_0123456789abcdefgh",
            "text": "Superseded: wrong contact.",
            "redacts": "note_0123456789abcdefgh"
        });
        let parsed: append::Payload = round_trips(payload);
        assert!(parsed.text.is_some());
        assert!(parsed.redacts.is_some());
    }

    #[cfg(feature = "validate")]
    #[test]
    fn notes_append_validation_still_requires_text_or_redacts() {
        use trust_tasks_rs::validate::ValidatedPayload;
        append::Payload::validate_value(&json!({ "entryId": "dir_0123456789abcdefgh" }))
            .expect_err("a note with neither text nor redacts must be refused");
    }
}
