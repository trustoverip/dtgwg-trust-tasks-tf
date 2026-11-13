//! Runs the shared forward-routing vectors in
//! `fixtures/consume/extends-routing.json` at the repository root, which the
//! TypeScript, Go and Dart runtimes also run: a consumer implementing `M.K`
//! reads a later version whose declared `extends` chain reaches `M.K` as
//! `M.K`, pruned, and answers it typed `<received type>#response`; anything
//! else of the same specification is `unsupportedVersion`, and anything else
//! at all `unsupportedType` (SPEC §5.2 "Declared extension", §7.2 item 3).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use trust_tasks_rs::extension::{declare_extends, read_as};
use trust_tasks_rs::{Payload, RejectReason, StandardCode, TrustTask};

/// The implemented version — the fixture's `implements` — with a payload type
/// that holds whatever it is handed, so the vector sees exactly what pruning
/// left.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
struct Implemented(Value);

impl Payload for Implemented {
    const TYPE_URI: &'static str = "https://example.com/spec/vector/grant/0.1";
    const PAYLOAD_SCHEMA: Option<&'static str> = Some(
        r#"{"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"subject":{"type":"string"},"role":{"type":"string"}},"required":["subject"],"additionalProperties":false}"#,
    );
}

fn vectors() -> Value {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/consume/extends-routing.json");
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

/// `RejectReason`'s wire code.
fn code(reason: &RejectReason) -> String {
    reason.code().to_string()
}

#[test]
fn shared_extends_routing_vectors() {
    let vectors = vectors();
    assert_eq!(
        vectors["implements"]["typeUri"].as_str().unwrap(),
        Implemented::TYPE_URI
    );
    assert_eq!(
        vectors["implements"]["payloadSchema"].as_str().unwrap(),
        Implemented::PAYLOAD_SCHEMA.unwrap()
    );

    // The declarations in force, handed to the application-side table.
    let table: Vec<(&'static str, &'static str)> = vectors["extends"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(uri, target)| -> (&'static str, &'static str) {
            (
                Box::leak(uri.clone().into_boxed_str()),
                Box::leak(target.as_str().unwrap().to_string().into_boxed_str()),
            )
        })
        .collect();
    declare_extends(Box::leak(table.into_boxed_slice()));

    let cases = vectors["cases"].as_array().unwrap();
    assert!(!cases.is_empty());
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let received: TrustTask<Value> =
            serde_json::from_str(case["document"].as_str().unwrap()).unwrap();
        let received_type = received.type_uri.clone();
        let verdict = case["verdict"].as_str().unwrap();
        match read_as::<Implemented>(received) {
            Ok(read) => {
                assert_eq!(verdict, "accept", "{name}: accepted");
                assert_eq!(read.payload.0, case["payload"], "{name}: payload");
                let extended = received_type.for_routing() != Implemented::type_uri().for_routing();
                assert_eq!(Value::Bool(extended), case["extended"], "{name}: extended");
                // The document keeps its received type, so the answer names it.
                assert_eq!(read.type_uri, received_type, "{name}: type kept");
                let answer = read.respond_with("urn:uuid:answer", ());
                assert_eq!(
                    answer.type_uri.to_string(),
                    case["answerType"].as_str().unwrap(),
                    "{name}: answer type"
                );
            }
            Err(reason) => {
                assert_eq!(code(&reason), verdict, "{name}: refused {reason:?}");
                assert!(
                    [
                        StandardCode::UnsupportedType.as_str(),
                        StandardCode::UnsupportedVersion.as_str()
                    ]
                    .contains(&verdict),
                    "{name}: an unexpected refusal"
                );
            }
        }
    }
}
