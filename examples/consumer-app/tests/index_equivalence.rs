//! The `schema_index` module `trust-tasks-build` generates with
//! `Config::index(true)` answers as `trust_tasks_rs::schema_index` does, URI
//! for URI, for every version both know — so an application dropping the
//! prebuilt families for generated ones keeps the same lookups.
//!
//! This test enables the families it compares against (`[dev-dependencies]`
//! in Cargo.toml), which the example's binary does not: under resolver 2 they
//! are compiled for the test only.

// Superseded versions are locked on purpose; their types are deprecated.
#![allow(deprecated)]

use trust_tasks_rs::schema_index as prebuilt;
use trust_tasks_rs::type_index::{lookup, IndexEntry};

/// The example's own lock: acl/grant 0.2, keys/list 0.2, vault/get 0.3, and
/// acme's vault/get 0.3.
mod tasks {
    include!(concat!(env!("OUT_DIR"), "/trust_tasks_generated.rs"));
}

/// `index-equivalence/trust-tasks.lock`: acl/list 0.1 (superseded),
/// keys/sign-document 0.1 (request and response bounds), rooms/blobs/chunk 0.1
/// (a response bound only), sync/event 0.1 (superseded, no response) and
/// vault/get 0.3 — the last also in `tasks`, as another generated crate's
/// entry for the same version would be.
mod wide {
    include!(concat!(
        env!("OUT_DIR"),
        "/index-equivalence/trust_tasks_generated.rs"
    ));
}

const ACME: &str = "https://tasks.example.com/spec/vault/get/0.3";

/// The generated functions of one index, side by side with the prebuilt ones.
struct Generated {
    type_uris: &'static [&'static str],
    index: &'static [IndexEntry],
    schema_for: fn(&str) -> Option<&'static str>,
    spec_policy_for: fn(&str) -> Option<trust_tasks_rs::SpecPolicy>,
    error_codes_for: fn(&str) -> Option<&'static [trust_tasks_rs::DeclaredErrorCode]>,
    max_document_bytes_for: fn(&str) -> Option<usize>,
}

macro_rules! generated {
    ($m:path) => {{
        use $m as index;
        Generated {
            type_uris: index::TYPE_URIS,
            index: index::INDEX,
            schema_for: index::schema_for,
            spec_policy_for: index::spec_policy_for,
            error_codes_for: index::error_codes_for,
            max_document_bytes_for: index::max_document_bytes_for,
        }
    }};
}

/// Every function of `g` answers for `uri` as the prebuilt index does, and
/// `INDEX` through `lookup` agrees with both.
fn assert_same(g: &Generated, uri: &str) {
    assert_eq!(
        (g.schema_for)(uri),
        prebuilt::schema_for(uri),
        "schema_for({uri})"
    );
    assert_eq!(
        (g.spec_policy_for)(uri),
        prebuilt::spec_policy_for(uri),
        "spec_policy_for({uri})"
    );
    assert_eq!(
        (g.error_codes_for)(uri),
        prebuilt::error_codes_for(uri),
        "error_codes_for({uri})"
    );
    assert_eq!(
        (g.max_document_bytes_for)(uri),
        prebuilt::max_document_bytes_for(uri),
        "max_document_bytes_for({uri})"
    );
    let entry = lookup(&[g.index], uri);
    assert_eq!(
        entry.and_then(|e| e.schema),
        prebuilt::schema_for(uri),
        "INDEX schema {uri}"
    );
    assert_eq!(
        entry.and_then(|e| e.policy),
        prebuilt::spec_policy_for(uri),
        "INDEX policy {uri}"
    );
    assert_eq!(
        entry.and_then(|e| e.error_codes),
        prebuilt::error_codes_for(uri),
        "INDEX error codes {uri}"
    );
    assert_eq!(
        entry.and_then(|e| e.max_document_bytes),
        prebuilt::max_document_bytes_for(uri),
        "INDEX size {uri}"
    );
}

/// Each locked request Type URI, its `#response` and `#request` forms, and a
/// URI no index knows, compared function by function. `TYPE_URIS` is exactly
/// the URIs the prebuilt index has a schema for.
fn check(g: &Generated, official: &[&str]) {
    let requests: Vec<&str> = g
        .type_uris
        .iter()
        .copied()
        .filter(|u| !u.contains('#'))
        .collect();
    let mut known = Vec::new();
    for request in &requests {
        if !official.contains(request) {
            continue;
        }
        for uri in [
            request.to_string(),
            format!("{request}#response"),
            format!("{request}#request"),
        ] {
            assert_same(g, &uri);
            if prebuilt::schema_for(&uri).is_some() {
                known.push(uri);
            }
        }
    }
    let listed: Vec<String> = g
        .type_uris
        .iter()
        .filter(|u| official.iter().any(|o| u.starts_with(o)))
        .map(|u| u.to_string())
        .collect();
    assert_eq!(listed, known, "TYPE_URIS");
    let listed_index: Vec<&str> = g.index.iter().map(|e| e.type_uri).collect();
    assert_eq!(listed_index, g.type_uris, "INDEX and TYPE_URIS");
    // A URI neither index knows: both answer `None`.
    for unknown in [
        "https://trusttasks.org/spec/vault/get/9.9",
        "https://trusttasks.org/spec/vault/get/9.9#response",
        "",
    ] {
        assert_same(g, unknown);
        assert!((g.schema_for)(unknown).is_none(), "{unknown}");
    }
    // A version the prebuilt index knows (its family is compiled for this
    // test) and the lock does not pin: the generated index does not know it.
    let unlocked = "https://trusttasks.org/spec/vault/upsert/0.4";
    assert!(prebuilt::schema_for(unlocked).is_some());
    assert!((g.schema_for)(unlocked).is_none());
    assert!((g.spec_policy_for)(unlocked).is_none());
    assert!((g.error_codes_for)(unlocked).is_none());
    assert!(lookup(&[g.index], unlocked).is_none());
}

#[test]
fn the_wide_lock_matches_the_prebuilt_index() {
    let g = generated!(wide::schema_index);
    let official = [
        "https://trusttasks.org/spec/acl/list/0.1",
        "https://trusttasks.org/spec/keys/sign-document/0.1",
        "https://trusttasks.org/spec/rooms/blobs/chunk/0.1",
        "https://trusttasks.org/spec/sync/event/0.1",
        "https://trusttasks.org/spec/vault/get/0.3",
    ];
    check(&g, &official);
    // Every case the lock was chosen for is present, so the comparison above
    // is not vacuous.
    let sign = "https://trusttasks.org/spec/keys/sign-document/0.1";
    assert!(prebuilt::max_document_bytes_for(sign).is_some());
    assert!(prebuilt::max_document_bytes_for(&format!("{sign}#response")).is_some());
    let chunk = "https://trusttasks.org/spec/rooms/blobs/chunk/0.1";
    assert!(prebuilt::max_document_bytes_for(chunk).is_none());
    assert!(prebuilt::max_document_bytes_for(&format!("{chunk}#response")).is_some());
    let event = "https://trusttasks.org/spec/sync/event/0.1";
    assert!(prebuilt::schema_for(event).is_some());
    assert!(prebuilt::schema_for(&format!("{event}#response")).is_none());
    assert!(
        !prebuilt::error_codes_for("https://trusttasks.org/spec/vault/get/0.3")
            .unwrap()
            .is_empty()
    );
    assert_eq!(g.type_uris.len(), 9);
}

#[test]
fn the_example_lock_matches_the_prebuilt_index() {
    let g = generated!(tasks::schema_index);
    let official = [
        "https://trusttasks.org/spec/acl/grant/0.2",
        "https://trusttasks.org/spec/keys/list/0.2",
        "https://trusttasks.org/spec/vault/get/0.3",
    ];
    check(&g, &official);

    // acme's vault/get is a specification the prebuilt index cannot know; the
    // generated one answers for it, and not with the registry's answers.
    assert!(prebuilt::schema_for(ACME).is_none());
    assert!(tasks::schema_index::schema_for(ACME).is_some());
    assert!(tasks::schema_index::schema_for(&format!("{ACME}#response")).is_some());
    assert_ne!(
        tasks::schema_index::schema_for(ACME),
        tasks::schema_index::schema_for("https://trusttasks.org/spec/vault/get/0.3")
    );
    assert_eq!(tasks::schema_index::error_codes_for(ACME), Some(&[][..]));
    assert_eq!(
        tasks::schema_index::spec_policy_for(&format!("{ACME}#response")),
        None
    );
}

#[test]
fn indexes_from_two_generated_crates_combine_with_lookup() {
    // What a facade over two generated crates writes: no merge, just the list.
    static TABLES: &[&[IndexEntry]] = &[tasks::schema_index::INDEX, wide::schema_index::INDEX];
    let tables = TABLES;
    for uri in tasks::schema_index::TYPE_URIS
        .iter()
        .chain(wide::schema_index::TYPE_URIS)
    {
        let entry = lookup(tables, uri).expect("every listed URI is found");
        assert_eq!(entry.type_uri, *uri);
        if !uri.starts_with(ACME) {
            assert_eq!(entry.schema, prebuilt::schema_for(uri), "{uri}");
            assert_eq!(entry.policy, prebuilt::spec_policy_for(uri), "{uri}");
        }
    }
    assert!(lookup(tables, "https://trusttasks.org/spec/vault/upsert/0.4").is_none());
}
