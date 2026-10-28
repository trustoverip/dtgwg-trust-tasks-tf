//! The registry's shared proof vectors (`fixtures/proof` at the repository
//! root), which every library's verifier runs, so a document one
//! implementation signs is checked by all four. They include a document whose
//! timestamps a typed parser would re-spell (SPEC §4.7, "What a consumer
//! verifies"): `verify_raw` must accept it, and must refuse its re-spelled twin.

#![cfg(feature = "affinidi")]

use std::path::PathBuf;
use std::sync::Arc;

use affinidi_did_resolver_cache_sdk::{config::DIDCacheConfigBuilder, DIDCacheClient};
use serde_json::Value;
use trust_tasks_proof::affinidi::{CachedDidResolver, Verifier};

/// The suites this backend implements. An entry for one of the others is
/// skipped here by name, so a skip is visible, and one for a suite in neither
/// list fails. `ecdsa-jcs-2019` is a SHOULD (SPEC §4.7.1) this backend does not
/// implement yet.
const SUPPORTED: &[&str] = &["eddsa-jcs-2022"];
const NOT_IMPLEMENTED: &[&str] = &["ecdsa-jcs-2019"];

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/proof")
}

#[tokio::test]
async fn every_shared_vector_reaches_its_expected_result() {
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(dir().join("manifest.json")).unwrap()).unwrap();
    let client = DIDCacheClient::new(DIDCacheConfigBuilder::default().build())
        .await
        .expect("local DIDCacheClient");
    let verifier = Verifier::with_resolver(Arc::new(CachedDidResolver::new(Arc::new(client))));

    let mut checked = 0;
    for entry in manifest["entries"].as_array().unwrap() {
        let file = entry["file"].as_str().unwrap();
        let suite = entry["cryptosuite"].as_str().unwrap();
        if NOT_IMPLEMENTED.contains(&suite) {
            continue;
        }
        assert!(SUPPORTED.contains(&suite), "{file}: no support for {suite}");
        let doc: Value = serde_json::from_slice(&std::fs::read(dir().join(file)).unwrap()).unwrap();
        let result = verifier.verify_raw(&doc).await;
        match entry["expect"].as_str().unwrap() {
            "valid" => assert!(result.is_ok(), "{file}: expected valid, got {result:?}"),
            "invalid" => assert!(result.is_err(), "{file}: expected invalid, but it verified"),
            other => panic!("{file}: unknown expect {other}"),
        }
        checked += 1;
    }
    assert!(checked >= 3, "only {checked} shared vector(s) checked");
}
