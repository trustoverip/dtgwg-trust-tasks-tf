//! The TSP version this binding speaks (binding `tsp/0.3` §2.2): TSP 1.0 and
//! nothing else. A message from a peer still on a TSP draft is refused from
//! its version marker as [`TspError::UnsupportedTspVersion`], before any key is
//! used or the sender resolved; an unknown 1.x MINOR is never a version
//! refusal.
//!
//! The fixtures are the ones the TypeScript, Go and Dart TSP bindings test
//! against (`trust-tasks-ts-tsp/test/fixtures.ts`,
//! `trust-tasks-go/tsp/interop_test.go`,
//! `trust-tasks-dart-tsp/test/interop_test.dart`). All three carry the
//! binding's `{type, document}` envelope from `did:example:alice` to
//! `did:example:bob`, sealed with the same parties' keys: alice signs with the
//! Ed25519 seed of 32 `0x01` bytes, and bob's X25519 private key is 32 `0x66`
//! bytes. HPKE draws a fresh ephemeral key on every seal, so none is
//! reproducible byte for byte.

use affinidi_tsp::PrivateVid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use trust_tasks_rs::{
    NoValidator, Payload, PayloadPolicy, ProofPolicy, ProofVerifier, TrustTask, VerificationError,
};
use trust_tasks_tsp::{
    advertised_sender, pack_trust_task, unpack_trust_task, TspConsumer, TspError, TSP_VERSION,
};

/// A TSP 1.0 (`YTSP-BAA`) Direct message packed by `affinidi-tsp` 0.5.0 — the
/// crate this binding seals with — through
/// `affinidi_tsp::message::direct::pack(body, MessageType::Direct, …)`.
const RUST_TSP_1_0: &[&str] = &[
    "f8408061348ff81000e41006006469643a6578616d706c653a616c696365e010056469643a6578616d706c65",
    "3a626f62e05070dcf39858172f2f081019d8b304436fcd20facc9141e75b3f6c86ebcb0c958f417d4b830b6e",
    "10d4ecbf10e157aba5e3c6617f8b29f4be9feb9afa07504ee530c8906b7219e42bc1b37295d3b39f52bc5c80",
    "6ad316e1e20a7e8237343c6b14cf72e92172a6f5cc30999beb2a3effcb88c18889dff54591796a86b8f9dc82",
    "142056d94a965154f737e6115be11a0edc7600ced661fbb77ac2de0c8a84f9c87079b2d3aa4284b05dc5005f",
    "dc62d8eee75ff181d57c0330a2abe170ff8bbb3dc81a63bb2db5fa23289d773b1da9411e5ac348b26453980b",
    "97af243cbe4e768637d2bb3c5092f28cdf84eb600e985cf74dc17f3a3ba2d89b97e4a313ee9d26e5b468f0df",
    "1bb0561ae1dc35499b840b88d2b756ca212c93702480ec3a8e5eabc69ffff53b316ad6c7495f596d7681eee3",
    "1ae7894348cae252c4971e4990155645751bbec2e53dfcad96902eb8e607c32aad7c64f82017f8a0160400f9",
    "07598aae27a4762637af72f2f233c9e4b34f2bcdea5c60b465fc32720fe0521ae78280fb57dd0268ea35ac08",
    "e1dae08152e5ac7e6116d1c8aad18b1ea0c703",
];

/// A TSP Rev 3 (`YTSP-AAC`) Direct message packed by `affinidi-tsp-go`
/// `v0.0.0-20260916130837-d32ca7cdde20`, the pseudo-version the Go binding
/// required before it moved to TSP 1.0.
const GO_REV_3: &[&str] = &[
    "f8407d61348ff80002e41006006469643a6578616d706c653a616c696365e010056469643a6578616d706c65",
    "3a626f62e0506d05e98823214d48869ab16c103da5127bb8a9ae817f0056d92531695cc2ee8a776c43cdaf47",
    "4e235166b8ab51f19703845f86e04d492bf2d1cbb16c2227de3c8fb536b2aa279d41c8703e00ce4e9d218f82",
    "b4357bd855304bbc85752371928a219380cc802c678bf64317c273d0fe1099a55c342289e348dce91a554b80",
    "d3bbf651f5e80c5b041bcf66fc9bc91ee017fb6121e5e499cf62fc715336768031399dd85677b26159fcc427",
    "e4ce41e8f8667d7b3954f1c3adc9e8559740f7fbf4081db8344b16fd5c8cae1c1b6c59387f47f020a8d9c8b5",
    "698ffe0fd88b266c0f5f6cb47c32aa544cfac271b970a6def8645b728d97ac4c394804e95c61738051b7bdcb",
    "7409ccdf264e2c71d6ade9651e0e740344e83e998d8cb34a3b7cdcd3a1f12556f66292b0f055f93c6ee0ebe7",
    "b479cab7e9cb8ade8fd38257ee0fa481a11f4c7de8713ae13bb3f82017f8a01604000d3c3aa590b545eb838f",
    "579da49ba05918bf6554dd3ef8d8a51aa08903582974d78dae43e304e1ac0ad03a3eb4f3d6b8a7111a56e69c",
    "f348d45ab64d64a6a205",
];

/// A TSP Rev 2 (`YTSP-AAB`) Direct message, HPKE-Auth sealed, packed by
/// `@openvtc/vti-tsp-js` 0.2.0, the version the TypeScript binding used before
/// it moved to TSP 1.0.
const VTI_TSP_JS_REV_2: &[&str] = &[
    "f8401061348ff80001e41006006469643a6578616d706c653a616c696365e010056469643a6578616d706c65",
    "3a626f625c0000e060674a8d67c84e02956663e9b8349532d9117d55f25a757553e916663b5067b1c085d813",
    "41b8239eaceaa32bd58cc2708cd5bfc53db86d409e8c1230e01081b0b5bb4e3d4724c66ddc1ef7c1849ce124",
    "148ec3f1a012a638e0c321826e0df592c2dfc64ac622d974bc1012608670a2b7ca51392a65332cb3c740b540",
    "8c8ab9d74a474d07c3a71ffc2f79070001e17d6b650b38dfb08f5283ecb8647114564bf62207e869afa01491",
    "ed0b49dc0dc2ffe970d12de025743b0595ff1594ab95816421d7e42cec61b55d57c83fa8be372d85f5c76f1a",
    "4b1b661d0695d366ceeb4d6760ec9e5c83a7eff71c959b18d68f7ff6ea6f64c419ac02493b4a14d6f99d076e",
    "1d3140ae44d00c8551d34a54abd5ff4f310f7e7b198f4a85086ce1dd5cca1bf1c002ed041d527b116f2d350a",
    "54989bfdc65e3597552b29f82016f8a016d010b694a72107551cc44102175cd59e4eaf4913ae3460707dac1f",
    "634e01bb289f4026f862d60930944ea09b62f2d621be936fb429118571de7038951feaa071ef05",
];

fn wire(chunks: &[&str]) -> Vec<u8> {
    let hex: String = chunks.concat();
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).expect("fixture hex"))
        .collect()
}

fn alice() -> PrivateVid {
    PrivateVid::from_keys("did:example:alice", [0x01; 32], [0x65; 32])
}

fn bob() -> PrivateVid {
    PrivateVid::from_keys("did:example:bob", [0x02; 32], [0x66; 32])
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Echo {
    text: String,
}

impl Payload for Echo {
    const TYPE_URI: &'static str = "https://trusttasks.org/spec/example/echo/0.1";
}

/// Pins `ProofPolicy`'s verifier type on the `AcceptUnverified` arm.
struct NoVerifier;

#[async_trait::async_trait]
impl ProofVerifier for NoVerifier {
    async fn verify<P>(&self, _doc: &TrustTask<P>) -> Result<(), VerificationError>
    where
        P: Serialize + Send + Sync,
    {
        Ok(())
    }
}

/// Just after the 1.0 fixture's `issuedAt`.
fn fixture_now() -> DateTime<Utc> {
    "2026-06-01T12:00:30Z".parse().unwrap()
}

/// Run `wire` through [`TspConsumer::receive`]. The handler panics unless
/// `expect_handled`.
async fn receive(wire: &[u8], sender: &PrivateVid, expect_handled: bool) -> Result<(), TspError> {
    let bob = bob();
    TspConsumer::new()
        .receive::<Echo, Echo, NoVerifier, NoValidator, _, _>(
            wire,
            &bob,
            &sender.to_resolved(),
            &bob.id,
            ProofPolicy::<NoVerifier>::AcceptUnverified,
            PayloadPolicy::<NoValidator>::AcceptUnvalidated,
            fixture_now(),
            || "urn:uuid:00000000-0000-4000-8000-0000000000e1".to_string(),
            |doc, _parties| async move {
                assert!(expect_handled, "the handler ran");
                Ok(doc.respond_with(
                    "urn:uuid:00000000-0000-4000-8000-0000000000b1",
                    Echo { text: "ok".into() },
                ))
            },
        )
        .await
        .map(|_| ())
}

#[test]
fn seals_tsp_1_0() {
    assert_eq!(TSP_VERSION, "1.0");
    let doc: TrustTask<Value> = serde_json::from_value(json!({
        "id": "urn:uuid:00000000-0000-4000-8000-0000000000a1",
        "type": "https://trusttasks.org/spec/example/echo/0.1",
        "payload": { "text": "hi" },
    }))
    .unwrap();
    let wire = pack_trust_task(&doc, &alice(), &bob().to_resolved()).unwrap();
    // `-E##` is three bytes and `YTSP` three more; the version code follows:
    // `f8 10 00` is `-BAA`, TSP 1.0.
    assert_eq!(wire[6..9], [0xf8, 0x10, 0x00], "sealed in TSP 1.0");
}

#[tokio::test]
async fn opens_a_tsp_1_0_message_packed_by_affinidi_tsp() {
    let wire = wire(RUST_TSP_1_0);
    assert_eq!(advertised_sender(&wire).unwrap(), "did:example:alice");

    let (doc, handler) =
        unpack_trust_task::<Echo>(&wire, &bob(), &alice().to_resolved()).expect("opens");
    assert_eq!(doc.payload.text, "from affinidi-tsp");
    assert_eq!(handler.peer(), Some("did:example:alice"));
    assert_eq!(handler.local(), Some("did:example:bob"));

    receive(&wire, &alice(), true).await.expect("handled");
}

/// A peer still on a TSP draft is refused from the version marker, before any
/// key is used: by `advertised_sender`, so the caller never resolves it; by
/// `unpack_trust_task` with the right sender and with an unrelated one; and by
/// the guarded consumer, without running the handler.
#[tokio::test]
async fn refuses_a_tsp_draft_before_the_sender_is_needed() {
    let stranger = PrivateVid::generate("did:example:stranger");
    for (name, chunks) in [("Rev 3", GO_REV_3), ("Rev 2", VTI_TSP_JS_REV_2)] {
        let wire = wire(chunks);
        let major = |r: Result<(), TspError>| match r {
            Err(TspError::UnsupportedTspVersion { major }) => major,
            Err(other) => panic!("{name}: expected UnsupportedTspVersion, got {other:?}"),
            Ok(()) => panic!("{name}: a draft message opened"),
        };

        assert_eq!(major(advertised_sender(&wire).map(|_| ())), 0, "{name}");
        assert_eq!(
            major(unpack_trust_task::<Echo>(&wire, &bob(), &alice().to_resolved()).map(|_| ())),
            0,
            "{name}"
        );
        assert_eq!(
            major(unpack_trust_task::<Echo>(&wire, &bob(), &stranger.to_resolved()).map(|_| ())),
            0,
            "{name}: refused whatever sender is passed"
        );
        assert_eq!(major(receive(&wire, &alice(), false).await), 0, "{name}");
    }
}

/// MINOR never gates a message (TSP 1.0 §3): a 1.x message with a MINOR this
/// binding does not know is read like any other, so one that then fails to
/// open is an ordinary failure, not a version refusal. The signature covers the
/// header, so the bumped message no longer verifies.
#[tokio::test]
async fn an_unknown_tsp_minor_is_not_a_version_refusal() {
    let mut wire = wire(RUST_TSP_1_0);
    assert_eq!(wire[6..9], [0xf8, 0x10, 0x00]);
    wire[8] = 0x05; // TSP 1.5

    assert_eq!(advertised_sender(&wire).unwrap(), "did:example:alice");
    let not_a_version_refusal = |r: Result<(), TspError>| match r {
        Err(TspError::UnsupportedTspVersion { .. }) => panic!("refused on MINOR"),
        Err(_) => {}
        Ok(()) => panic!("a tampered message opened"),
    };
    not_a_version_refusal(
        unpack_trust_task::<Echo>(&wire, &bob(), &alice().to_resolved()).map(|_| ()),
    );
    not_a_version_refusal(receive(&wire, &alice(), false).await);
}
