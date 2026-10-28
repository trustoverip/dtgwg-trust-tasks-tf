use super::*;

/// A throwaway directory per call.
fn scratch(tag: &str) -> PathBuf {
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "tt-publish-{tag}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn put(root: &Path, rel: &str, text: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

// ─── lifecycles (moved from trust-tasks-codegen) ───────────────────────────

fn lifecycles(entries: &[(&str, &str, &str)]) -> BTreeMap<(String, String), Lifecycle> {
    let root = scratch("lifecycle");
    for (slug, version, fm) in entries {
        put(
            &root,
            &format!("{slug}/{version}/spec.md"),
            &format!("---\n{fm}\n---\n# x\n"),
        );
    }
    let out = compute_lifecycles(&root).unwrap();
    fs::remove_dir_all(&root).unwrap();
    out
}

fn get<'a>(
    map: &'a BTreeMap<(String, String), Lifecycle>,
    slug: &str,
    version: &str,
) -> &'a Lifecycle {
    &map[&(slug.to_string(), version.to_string())]
}

fn deprecated(note: &str) -> Lifecycle {
    Lifecycle::Deprecated(note.to_string())
}

#[test]
fn a_later_version_at_equal_maturity_supersedes_and_numeric_order_wins() {
    let map = lifecycles(&[
        ("a", "0.9", "status: draft"),
        ("a", "0.10", "status: \"draft\""),
    ]);
    assert_eq!(get(&map, "a", "0.9"), &deprecated("superseded by a/0.10"));
    assert_eq!(get(&map, "a", "0.10"), &Lifecycle::Live);
}

#[test]
fn a_later_draft_does_not_supersede_a_standard() {
    let map = lifecycles(&[
        ("a", "1.0", "status: standard"),
        ("a", "1.1", "status: draft"),
    ]);
    assert_eq!(get(&map, "a", "1.0"), &Lifecycle::Live);
    assert_eq!(get(&map, "a", "1.1"), &Lifecycle::Live);
}

#[test]
fn a_retired_or_withdrawn_later_version_supersedes_nothing() {
    let map = lifecycles(&[
        ("a", "0.1", "status: draft"),
        ("a", "0.2", "status: retired"),
        ("a", "0.3", "status: withdrawn"),
    ]);
    assert_eq!(get(&map, "a", "0.1"), &Lifecycle::Live);
    assert_eq!(get(&map, "a", "0.2"), &deprecated("retired; use a/0.1"));
    assert_eq!(get(&map, "a", "0.3"), &Lifecycle::Withdrawn);
}

#[test]
fn a_retired_version_names_its_successor() {
    let map = lifecycles(&[
        ("old", "0.1", "status: retired\nsupersededBy: new/0.1"),
        ("old", "0.2", "status: retired\nsupersededBy: new"),
        ("old", "0.3", "status: retired\nsupersededBy: gone"),
        ("old", "0.4", "status: retired"),
        ("new", "0.1", "status: draft"),
        ("new", "0.2", "status: draft"),
    ]);
    assert_eq!(get(&map, "old", "0.1"), &deprecated("retired; use new/0.1"));
    assert_eq!(get(&map, "old", "0.2"), &deprecated("retired; use new/0.2"));
    assert_eq!(get(&map, "old", "0.3"), &deprecated("retired"));
    assert_eq!(get(&map, "old", "0.4"), &deprecated("retired"));
}

#[test]
fn canonical_json_sorts_and_refuses_ambiguous_numbers() {
    let v = json!({"b": 1, "a": [true, null, "x\u{1}"], "\u{e9}": 0.5});
    assert_eq!(
        canonical_json(&v).unwrap(),
        r#"{"a":[true,null,"x\u0001"],"b":1,"é":0.5}"#
    );
    assert!(canonical_json(&json!(1.0)).is_err());
    assert!(canonical_json(&json!(1e21)).is_err());
}

// ─── third-party registries ───────────────────────────────────────────────

const PREFIX: &str = "https://tasks.example.com";

/// A third party's `specs/`: `payments/settle` 1.0 and 1.1 (1.1 extends 1.0),
/// whose payload `$ref`s a local shared file and the registry's `AclEntry`
/// by absolute URL.
fn third_party_tree(prefix: &str) -> PathBuf {
    let root = scratch("third-party");
    let specs = root.join("specs");
    put(
        &specs,
        "payments/_shared/1.0/money.schema.json",
        r#"{"$schema":"https://json-schema.org/draft/2020-12/schema","$defs":{"Money":{"type":"object","additionalProperties":false,"required":["currency","amount"],"properties":{"currency":{"type":"string"},"amount":{"type":"string"}}}}}"#,
    );
    for (version, extra_fm, extra_prop) in [
        ("1.0", "", ""),
        ("1.1", "extends: \"1.0\"\n", r#","memo":{"type":"string"}"#),
    ] {
        let uri = type_uri_under(prefix, "payments/settle", version);
        put(
            &specs,
            &format!("payments/settle/{version}/spec.md"),
            &format!(
                "---\nslug: payments/settle\nversion: \"{version}\"\nstatus: draft\n{extra_fm}\
                 proofRequirement:\n  requirement: REQUIRED\nparties:\n  - member: recipient\n    \
                 requirement: REQUIRED\nerrorCodes:\n  - code: payments/settle:insufficientFunds\n    \
                 meaning: Not enough.\n    retryable: false\n---\n# Settle\n"
            ),
        );
        put(
            &specs,
            &format!("payments/settle/{version}/payload.schema.json"),
            &format!(
                r#"{{"$schema":"https://json-schema.org/draft/2020-12/schema","$id":"{uri}","type":"object","additionalProperties":false,"required":["amount","payer"],"properties":{{"payer":{{"type":"string"}},"amount":{{"$ref":"../../_shared/1.0/money.schema.json#/$defs/Money"}},"grant":{{"$ref":"https://trusttasks.org/specs/acl/_shared/0.2/acl-entry.schema.json#/$defs/AclEntry"}}{extra_prop}}},"$defs":{{"Response":{{"type":"object","additionalProperties":false,"required":["settled"],"properties":{{"settled":{{"type":"boolean"}}}}}}}}}}"#
            ),
        );
    }
    root
}

fn official_checkout() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../specs")
}

#[test]
fn a_third_party_tree_builds_into_verifiable_bundles() {
    let root = third_party_tree(PREFIX);
    let refs = Resolver::local().with_official_specs(official_checkout());
    let built = build_third_party(&root.join("specs"), PREFIX, &refs).unwrap();
    assert_eq!(built.bundles.len(), 2);

    // Each bundle reads back through the consumer-side parser, names its own
    // authority, and carries the inlined local and registry definitions.
    for b in &built.bundles {
        let parsed = crate::Bundle::from_slice(&b.bytes).unwrap();
        assert_eq!(
            parsed.type_uri,
            format!("{PREFIX}/spec/payments/settle/{}", b.version)
        );
        assert!(parsed.policy.proof_required.request);
        assert!(parsed.policy.recipient_required.request);
        assert_eq!(parsed.error_codes.len(), 1);
        let defs = parsed.schema["$defs"].as_object().unwrap();
        assert!(defs.contains_key("Money") && defs.contains_key("AclEntry"));
        assert!(defs.contains_key("Ext"), "AclEntry's own refs are followed");
        assert_eq!(parsed.property_order["/schema"][0], "payer");
        // `$defs` in the order the TS, Go and Dart inliners leave them: the
        // schema's own, then a last-in-first-out frontier — so the registry's
        // `AclEntry` (the payload's last `$ref`) before the local `Money`.
        let raw: Value = serde_json::from_slice(&b.bytes).unwrap();
        let order: Vec<&str> = raw["defsOrder"]["/schema"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n.as_str().unwrap())
            .collect();
        assert_eq!(order[..2], ["Response", "AclEntry"]);
        assert_eq!(order.last(), Some(&"Money"));
        assert_eq!(
            raw["defsOrder"]["/responseSchema"],
            raw["defsOrder"]["/schema"]
        );
        // Origins: the registry component by its `$id`; the local file, which
        // has none, by its path below the Type URI prefix.
        assert_eq!(
            raw["defOrigins"]["AclEntry"],
            "https://trusttasks.org/spec/acl/_shared/0.2/acl-entry#/$defs/AclEntry"
        );
        assert_eq!(
            raw["defOrigins"]["Money"],
            format!("{PREFIX}/spec/payments/_shared/1.0/money#/$defs/Money")
        );
        assert!(
            raw["defOrigins"].get("Response").is_none(),
            "own definitions have no origin"
        );
        // The bundle generates.
        crate::generate_module(&parsed, &crate::Target::application()).unwrap();
        assert_eq!(b.digest, sha256_digest(&b.bytes));
    }
    assert_eq!(
        built.bundles[0].deprecation.as_deref(),
        Some("superseded by payments/settle/1.1")
    );
    assert_eq!(built.bundles[1].extends.as_deref(), Some("1.0"));

    // Written as the served layout, with the by-digest store, repeatably.
    let out = root.join("site/specs");
    let w = write_registry(&out, &built.bundles, &built.index, true).unwrap();
    assert_eq!(w.by_digest_added, 2);
    let again = write_registry(&out, &built.bundles, &built.index, true).unwrap();
    assert_eq!(again.by_digest_added, 0);
    let index: Value = serde_json::from_slice(&fs::read(out.join(INDEX_FILE)).unwrap()).unwrap();
    for entry in index["specs"].as_array().unwrap() {
        let bytes = fs::read(out.join(entry["path"].as_str().unwrap())).unwrap();
        assert_eq!(entry["digest"], json!(sha256_digest(&bytes)));
        let hex = entry["digest"]
            .as_str()
            .unwrap()
            .trim_start_matches("sha256:");
        assert_eq!(
            fs::read(out.join(format!("by-digest/sha256-{hex}.json"))).unwrap(),
            bytes
        );
    }
    // A corrupted store entry is never overwritten.
    let hex = built.bundles[0].digest.trim_start_matches("sha256:");
    fs::write(out.join(format!("by-digest/sha256-{hex}.json")), b"{}").unwrap();
    assert!(write_registry(&out, &built.bundles, &built.index, true).is_err());
    fs::remove_dir_all(&root).ok();
}

#[test]
fn registry_refs_are_fetched_when_there_is_no_checkout() {
    let root = third_party_tree(PREFIX);
    let checkout = official_checkout();
    let fetched = std::cell::RefCell::new(Vec::new());
    let fetch = |url: &str| -> Result<String> {
        fetched.borrow_mut().push(url.to_string());
        let rel = url
            .strip_prefix("https://trusttasks.org/specs/")
            .ok_or_else(|| anyhow!("unexpected {url}"))?;
        Ok(fs::read_to_string(checkout.join(rel))?)
    };
    let refs = Resolver::local().with_fetch(&fetch);
    let by_fetch = build_third_party(&root.join("specs"), PREFIX, &refs).unwrap();
    let by_checkout = build_third_party(
        &root.join("specs"),
        PREFIX,
        &Resolver::local().with_official_specs(checkout.clone()),
    )
    .unwrap();
    assert!(!fetched.borrow().is_empty());
    for (a, b) in by_fetch.bundles.iter().zip(&by_checkout.bundles) {
        assert_eq!(a.bytes, b.bytes, "fetching and reading a checkout agree");
    }
    // Without either, a registry ref is an error rather than a silent gap.
    let err = build_third_party(&root.join("specs"), PREFIX, &Resolver::local()).unwrap_err();
    assert!(format!("{err:#}").contains("--official-specs"), "{err:#}");
    fs::remove_dir_all(&root).ok();
}

#[test]
fn a_component_id_resolves_through_the_checkout() {
    // `_framework/0.4/framework.schema.json`'s `$id` does not follow the
    // path convention; with a checkout it is found by `$id` anyway.
    let root = scratch("by-id");
    let uri = format!("{PREFIX}/spec/x/1.0");
    put(&root, "specs/x/1.0/spec.md", "---\nstatus: draft\n---\n");
    put(
        &root,
        "specs/x/1.0/payload.schema.json",
        &format!(
            r#"{{"$schema":"https://json-schema.org/draft/2020-12/schema","$id":"{uri}","type":"object","additionalProperties":false,"properties":{{"ext":{{"$ref":"https://trusttasks.org/spec/trust-task/0.4/framework#/$defs/Ext"}}}}}}"#
        ),
    );
    let refs = Resolver::local().with_official_specs(official_checkout());
    let built = build_third_party(&root.join("specs"), PREFIX, &refs).unwrap();
    let parsed = crate::Bundle::from_slice(&built.bundles[0].bytes).unwrap();
    assert!(parsed.schema["$defs"]["Ext"].is_object());
    fs::remove_dir_all(&root).ok();
}

#[test]
fn urn_and_did_prefixes_build() {
    for prefix in ["urn:example:tasks", "did:web:tasks.example.com"] {
        let root = third_party_tree(prefix);
        let refs = Resolver::local().with_official_specs(official_checkout());
        let built = build_third_party(&root.join("specs"), prefix, &refs).unwrap();
        let parsed = crate::Bundle::from_slice(&built.bundles[0].bytes).unwrap();
        assert_eq!(
            parsed.type_uri,
            type_uri_under(prefix, "payments/settle", "1.0")
        );
        fs::remove_dir_all(&root).ok();
    }
    assert_eq!(
        type_uri_under("urn:example:tasks", "payments/settle", "1.0"),
        "urn:example:tasks:spec:payments:settle:1.0"
    );
}

#[test]
fn third_party_refusals() {
    let refs = Resolver::local().with_official_specs(official_checkout());
    let err = |root: &Path, prefix: &str| {
        format!(
            "{:#}",
            build_third_party(&root.join("specs"), prefix, &refs).unwrap_err()
        )
    };

    // The registry's authority, and http.
    let root = third_party_tree(PREFIX);
    assert!(err(&root, "https://trusttasks.org").contains("SPEC §6.5 item 1"));
    assert!(err(&root, "https://TrustTasks.org/x").contains("SPEC §6.5 item 1"));
    assert!(err(&root, "http://tasks.example.com").contains("must not be http"));
    // A prefix the schemas' `$id`s do not name.
    assert!(err(&root, "https://elsewhere.example").contains("$id must be the Type URI"));
    fs::remove_dir_all(&root).ok();

    // A framework slug.
    let root = third_party_tree(PREFIX);
    fs::rename(
        root.join("specs/payments"),
        root.join("specs/trust-task-pay"),
    )
    .unwrap();
    assert!(err(&root, PREFIX).contains("reserves trust-task"));
    fs::remove_dir_all(&root).ok();

    // A slug outside the SPEC grammar.
    let root = third_party_tree(PREFIX);
    fs::rename(root.join("specs/payments"), root.join("specs/Payments")).unwrap();
    assert!(err(&root, PREFIX).contains("is not a SPEC §6.1 slug"));
    fs::remove_dir_all(&root).ok();

    // Front matter naming another version.
    let root = third_party_tree(PREFIX);
    let md = root.join("specs/payments/settle/1.0/spec.md");
    let text = fs::read_to_string(&md)
        .unwrap()
        .replace("version: \"1.0\"", "version: \"1.3\"");
    fs::write(&md, text).unwrap();
    assert!(err(&root, PREFIX).contains("does not match the directory"));
    fs::remove_dir_all(&root).ok();

    // An `extends` naming a version that does not exist.
    let root = third_party_tree(PREFIX);
    fs::remove_dir_all(root.join("specs/payments/settle/1.0")).unwrap();
    assert!(err(&root, PREFIX).contains("extends names 1.0"));
    fs::remove_dir_all(&root).ok();
}

#[test]
fn a_shared_file_without_an_id_is_named_below_a_urn_prefix() {
    let prefix = "urn:example:tasks";
    let root = third_party_tree(prefix);
    let refs = Resolver::local().with_official_specs(official_checkout());
    let built = build_third_party(&root.join("specs"), prefix, &refs).unwrap();
    let raw: Value = serde_json::from_slice(&built.bundles[0].bytes).unwrap();
    assert_eq!(
        raw["defOrigins"]["Money"],
        "urn:example:tasks:spec:payments:_shared:1.0:money#/$defs/Money"
    );
    fs::remove_dir_all(&root).ok();
}
