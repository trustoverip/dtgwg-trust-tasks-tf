//! [`Envelope`] — the binding's envelope object, without the TSP seal.
//!
//! [`pack_trust_task`](crate::pack_trust_task) and
//! [`unpack_trust_task`](crate::unpack_trust_task) build and open the envelope
//! *and* seal and unseal it with raw VID keys. An application whose own TSP
//! stack seals and opens the message — a messaging SDK that holds the keys and
//! the relationship state — needs only the envelope: the plaintext TSP seals,
//! and the plaintext it hands back. [`Envelope`] is that, and is the same code
//! the pack and unpack functions use, so the bytes cannot drift from theirs.

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use trust_tasks_rs::TrustTask;

use crate::error::TspError;

/// Envelope `type` URI for Trust Tasks carried over TSP.
///
/// Conforming consumers reject payloads with any other `type` via
/// [`TspError::WrongEnvelopeType`].
pub const ENVELOPE_TYPE: &str = "https://trusttasks.org/binding/tsp/0.1/envelope";

/// The binding envelope object (binding `tsp/0.1` §2): the TSP message
/// payload, `{ "type": ENVELOPE_TYPE, "document": <the Trust Task document> }`.
///
/// Its `type` is always [`ENVELOPE_TYPE`]: an `Envelope` is either built
/// around a document here or parsed from a payload whose `type` was checked,
/// so it holds only the `document`.
///
/// ```
/// use serde_json::{json, Value};
/// use trust_tasks_rs::TrustTask;
/// use trust_tasks_tsp::Envelope;
///
/// let doc: TrustTask<Value> = serde_json::from_value(json!({
///     "id": "urn:uuid:3f1c0b8e-5d0a-4c1e-9a7b-2e6d4f8a1c3b",
///     "type": "https://example.com/spec/grant/0.1",
///     "payload": { "subject": "did:example:carol" }
/// }))?;
///
/// // Producer: the plaintext to hand to the TSP seal.
/// let payload: Vec<u8> = Envelope::new(&doc)?.to_bytes();
///
/// // Consumer: the plaintext the TSP open returned.
/// let envelope = Envelope::from_bytes(&payload)?;
/// let received: TrustTask<Value> = envelope.into_trust_task()?;
/// assert_eq!(received.id, doc.id);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envelope {
    document: Value,
}

impl Envelope {
    /// Wrap a Trust Task document — typed, `TrustTask<serde_json::Value>`, or
    /// an [`ErrorResponse`](trust_tasks_rs::ErrorResponse).
    ///
    /// Fails with [`TspError::SerialiseBody`] only if the document does not
    /// serialise to JSON.
    pub fn new<P>(doc: &TrustTask<P>) -> Result<Self, TspError>
    where
        P: Serialize,
    {
        let document = serde_json::to_value(doc).map_err(TspError::SerialiseBody)?;
        Ok(Self { document })
    }

    /// Wrap a document already held as JSON, unchanged — a received document
    /// being forwarded, say. Nothing about it is checked.
    pub fn from_document(document: Value) -> Self {
        Self { document }
    }

    /// Parse a TSP message payload (the plaintext the seal protected).
    ///
    /// Fails with [`TspError::InvalidBody`] when the payload is not a JSON
    /// object with a string `type` and a `document`, and with
    /// [`TspError::WrongEnvelopeType`] when its `type` is not
    /// [`ENVELOPE_TYPE`] — a valid TSP payload that is simply not a Trust Task
    /// (binding §2: not dispatched through the framework pipeline). The
    /// `document` is not parsed: see [`Self::into_trust_task`].
    pub fn from_bytes(payload: &[u8]) -> Result<Self, TspError> {
        #[derive(serde::Deserialize)]
        struct Wire {
            #[serde(rename = "type")]
            type_: String,
            document: Value,
        }

        // Binding §2: the payload is a JSON *object*. A derived struct
        // deserialiser would also take the members as an array.
        let value: Value = serde_json::from_slice(payload).map_err(TspError::InvalidBody)?;
        if !value.is_object() {
            return Err(TspError::InvalidBody(serde::de::Error::invalid_type(
                serde::de::Unexpected::Other("a JSON value other than an object"),
                &"the binding envelope object",
            )));
        }
        let wire: Wire = serde_json::from_value(value).map_err(TspError::InvalidBody)?;
        if wire.type_ != ENVELOPE_TYPE {
            return Err(TspError::WrongEnvelopeType(wire.type_));
        }
        Ok(Self {
            document: wire.document,
        })
    }

    /// The envelope's JSON serialisation: the plaintext to hand to the TSP
    /// seal. Byte-for-byte what [`pack_trust_task`](crate::pack_trust_task)
    /// seals for the same document. (Member order is not significant on the
    /// wire; a consumer must not depend on it.)
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::json!({ "type": ENVELOPE_TYPE, "document": &self.document })
            .to_string()
            .into_bytes()
    }

    /// The carried document, as received.
    ///
    /// A `proof` is verified over this object (with only its top-level
    /// `proof` removed), never over a re-serialisation of the typed document —
    /// binding §2, SPEC §4.7.
    pub fn document(&self) -> &Value {
        &self.document
    }

    /// The carried document, as received — see [`Self::document`].
    pub fn into_document(self) -> Value {
        self.document
    }

    /// Deserialise the carried document as a `TrustTask<P>` —
    /// `serde_json::Value` for a consumer that routes on `type` itself.
    ///
    /// Fails with [`TspError::InvalidBody`] when it does not deserialise.
    /// This runs none of the SPEC §7.2 consumer checks; a consumer that is not
    /// using [`TspConsumer`](crate::TspConsumer) runs them itself.
    pub fn into_trust_task<P>(self) -> Result<TrustTask<P>, TspError>
    where
        P: DeserializeOwned,
    {
        serde_json::from_value(self.document).map_err(TspError::InvalidBody)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn document() -> Value {
        json!({
            "id": "urn:uuid:3f1c0b8e-5d0a-4c1e-9a7b-2e6d4f8a1c3b",
            "type": "https://example.com/spec/grant/0.1",
            "payload": { "subject": "did:example:carol" }
        })
    }

    /// What `pack_inner_direct` sealed before the envelope was a type: the
    /// envelope must keep producing exactly these bytes.
    #[test]
    fn to_bytes_is_the_json_the_binding_sealed_before() {
        let doc = document();
        let before =
            serde_json::to_vec(&json!({ "type": ENVELOPE_TYPE, "document": doc.clone() })).unwrap();
        assert_eq!(Envelope::from_document(doc).to_bytes(), before);
    }

    #[test]
    fn a_payload_missing_a_member_is_invalid_body() {
        for payload in [
            json!({ "type": ENVELOPE_TYPE }),
            json!({ "document": document() }),
            json!({ "type": 7, "document": document() }),
            json!([ENVELOPE_TYPE, document()]),
        ] {
            let bytes = serde_json::to_vec(&payload).unwrap();
            assert!(
                matches!(Envelope::from_bytes(&bytes), Err(TspError::InvalidBody(_))),
                "{payload}"
            );
        }
        assert!(matches!(
            Envelope::from_bytes(b"not json"),
            Err(TspError::InvalidBody(_))
        ));
    }

    #[test]
    fn another_type_is_wrong_envelope_type() {
        let bytes = serde_json::to_vec(
            &json!({ "type": "https://example.org/other", "document": document() }),
        )
        .unwrap();
        match Envelope::from_bytes(&bytes) {
            Err(TspError::WrongEnvelopeType(t)) => assert_eq!(t, "https://example.org/other"),
            other => panic!("expected WrongEnvelopeType, got {other:?}"),
        }
    }

    #[test]
    fn an_undeserialisable_document_is_invalid_body() {
        let envelope = Envelope::from_document(json!({ "id": 1 }));
        assert!(matches!(
            envelope.into_trust_task::<Value>(),
            Err(TspError::InvalidBody(_))
        ));
    }
}
