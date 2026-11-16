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

/// The longest delivery id the binding allows, in characters (binding
/// `tsp/0.2` §2: a `deliveryId` is a string of 1 to 256 characters).
pub const MAX_DELIVERY_ID_LEN: usize = 256;

/// A delivery id the binding does not allow: not 1 to
/// [`MAX_DELIVERY_ID_LEN`] characters. Returned by
/// [`Envelope::with_delivery_id`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("a delivery id is 1 to {MAX_DELIVERY_ID_LEN} characters (got {length})")]
pub struct InvalidDeliveryId {
    length: usize,
}

impl InvalidDeliveryId {
    /// The refused id's length, in characters.
    pub fn length(&self) -> usize {
        self.length
    }
}

fn delivery_id_length_ok(id: &str) -> bool {
    (1..=MAX_DELIVERY_ID_LEN).contains(&id.chars().count())
}

/// The binding envelope object (binding `tsp/0.1` and `tsp/0.2` §2): the TSP
/// message payload, `{ "type": ENVELOPE_TYPE, "document": <the Trust Task
/// document> }`, plus — `tsp/0.2` §2.1 — an optional `deliveryId` requesting
/// an end-to-end delivery receipt.
///
/// Its `type` is always [`ENVELOPE_TYPE`] (the same in both binding
/// versions): an `Envelope` is either built around a document here or parsed
/// from a payload whose `type` was checked, so it holds only the `document`
/// and the delivery id. Without a delivery id it is a `tsp/0.1` envelope,
/// byte-for-byte.
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
    delivery_id: Option<String>,
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
        Ok(Self::from_document(document))
    }

    /// Wrap a document already held as JSON, unchanged — a received document
    /// being forwarded, say. Nothing about it is checked.
    pub fn from_document(document: Value) -> Self {
        Self {
            document,
            delivery_id: None,
        }
    }

    /// Request an end-to-end delivery receipt (binding `tsp/0.2` §2.1): set
    /// the envelope's `deliveryId`, replacing any it had.
    ///
    /// The id is the producer's to choose under `messaging/receipt` — unique
    /// among its deliveries to this recipient, never reused for another
    /// recipient, not derived from the content, and kept on a retry of the
    /// same delivery. An envelope carrying a receipt **must not** have one.
    /// Fails when the id is not 1 to [`MAX_DELIVERY_ID_LEN`] characters.
    pub fn with_delivery_id(
        self,
        delivery_id: impl Into<String>,
    ) -> Result<Self, InvalidDeliveryId> {
        let delivery_id = delivery_id.into();
        if !delivery_id_length_ok(&delivery_id) {
            return Err(InvalidDeliveryId {
                length: delivery_id.chars().count(),
            });
        }
        Ok(Self {
            delivery_id: Some(delivery_id),
            ..self
        })
    }

    /// The delivery id the producer requested a receipt with, if any
    /// (binding `tsp/0.2` §2.1).
    ///
    /// A consumer that sends the receipt echoes it exactly, to the
    /// TSP-authenticated sender of the message that carried it. It is not the
    /// document `id`, not a `threadId`, and **must not** affect the framework
    /// pipeline or its duplicate-execution record.
    pub fn delivery_id(&self) -> Option<&str> {
        self.delivery_id.as_deref()
    }

    /// Parse a TSP message payload (the plaintext the seal protected).
    ///
    /// Fails with [`TspError::InvalidBody`] when the payload is not a JSON
    /// object with a string `type` and a `document` (or names either member
    /// twice), and with [`TspError::WrongEnvelopeType`] when its `type` is not
    /// [`ENVELOPE_TYPE`] — a valid TSP payload that is simply not a Trust Task
    /// (binding §2: not dispatched through the framework pipeline). The
    /// `document` is not parsed: see [`Self::into_trust_task`].
    ///
    /// A `deliveryId` is kept ([`Self::delivery_id`]) when it is a string of 1
    /// to [`MAX_DELIVERY_ID_LEN`] characters, and otherwise dropped, never
    /// refused: binding `tsp/0.2` §2.1 keeps it out of the framework pipeline,
    /// so a bad one costs the producer its receipt, not the document. Other
    /// members are ignored (§2).
    pub fn from_bytes(payload: &[u8]) -> Result<Self, TspError> {
        #[derive(serde::Deserialize)]
        struct Wire {
            #[serde(rename = "type")]
            type_: String,
            document: Value,
            #[serde(rename = "deliveryId", default)]
            delivery_id: Option<Value>,
        }

        // Binding §2: the payload is a JSON *object*. The derived struct
        // deserialiser would also take the members as an array, so look at the
        // first byte; deserialising straight into the struct (not through a
        // `Value`) keeps its refusal of a duplicated member, which a `Value`
        // would resolve silently to the last one.
        let first = payload
            .iter()
            .find(|b| !matches!(b, b' ' | b'\t' | b'\n' | b'\r'));
        if first != Some(&b'{') {
            return Err(TspError::InvalidBody(not_an_object(payload)));
        }
        let wire: Wire = serde_json::from_slice(payload).map_err(TspError::InvalidBody)?;
        if wire.type_ != ENVELOPE_TYPE {
            return Err(TspError::WrongEnvelopeType(wire.type_));
        }
        let delivery_id = match wire.delivery_id {
            Some(Value::String(id)) if delivery_id_length_ok(&id) => Some(id),
            _ => None,
        };
        Ok(Self {
            document: wire.document,
            delivery_id,
        })
    }

    /// The envelope's JSON serialisation: the plaintext to hand to the TSP
    /// seal. Without a delivery id, byte-for-byte what
    /// [`pack_trust_task`](crate::pack_trust_task) seals for the same
    /// document; `deliveryId` is written only when one is set. (Member order
    /// is not significant on the wire; a consumer must not depend on it.)
    pub fn to_bytes(&self) -> Vec<u8> {
        // Built with `json!` — a `serde_json::Map` — on purpose, not with a
        // derived `Serialize` struct: that is how the binding has always built
        // the envelope, so the member order (and so the bytes) stays what it
        // was, whether or not serde_json's `preserve_order` feature is on in
        // the build. A struct would fix one order regardless and change the
        // bytes in one of the two builds.
        let mut envelope = serde_json::json!({ "type": ENVELOPE_TYPE, "document": &self.document });
        if let Some(delivery_id) = &self.delivery_id {
            envelope["deliveryId"] = Value::String(delivery_id.clone());
        }
        envelope.to_string().into_bytes()
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

/// The `InvalidBody` error for a payload that is not a JSON object: the
/// parse error if it is not JSON at all, otherwise the kind of value it is.
fn not_an_object(payload: &[u8]) -> serde_json::Error {
    use serde::de::{Error, Unexpected};

    let value: Value = match serde_json::from_slice(payload) {
        Ok(value) => value,
        Err(e) => return e,
    };
    let unexpected = match &value {
        Value::Null => Unexpected::Unit,
        Value::Bool(b) => Unexpected::Bool(*b),
        Value::Number(n) => match (n.as_u64(), n.as_i64(), n.as_f64()) {
            (Some(u), _, _) => Unexpected::Unsigned(u),
            (_, Some(i), _) => Unexpected::Signed(i),
            (_, _, Some(f)) => Unexpected::Float(f),
            _ => Unexpected::Other("number"),
        },
        Value::String(s) => Unexpected::Str(s),
        Value::Array(_) => Unexpected::Seq,
        Value::Object(_) => Unexpected::Map,
    };
    serde_json::Error::invalid_type(unexpected, &"the binding envelope object")
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
            json!("a string"),
            json!(null),
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
    fn a_non_object_payload_names_what_it_is() {
        let err = |payload: &[u8]| match Envelope::from_bytes(payload) {
            Err(TspError::InvalidBody(e)) => e.to_string(),
            other => panic!("expected InvalidBody, got {other:?}"),
        };
        assert!(err(b" [1, 2]").contains("invalid type: sequence"));
        assert!(err(b"\"x\"").contains("invalid type: string \"x\""));
        assert!(err(b"7").contains("invalid type: integer `7`"));
        assert!(err(b"null").contains("invalid type: null"));
        // Not JSON at all: the parser's own error.
        assert!(err(b"[1,").contains("EOF"));
    }

    #[test]
    fn a_duplicated_member_is_invalid_body() {
        for payload in [
            format!(r#"{{"type":"x","type":"{ENVELOPE_TYPE}","document":{{}}}}"#),
            format!(r#"{{"type":"{ENVELOPE_TYPE}","document":{{}},"document":{{}}}}"#),
        ] {
            assert!(
                matches!(
                    Envelope::from_bytes(payload.as_bytes()),
                    Err(TspError::InvalidBody(_))
                ),
                "{payload}"
            );
        }
    }

    #[test]
    fn a_delivery_id_must_be_1_to_256_characters() {
        let envelope = || Envelope::from_document(document());
        assert_eq!(envelope().with_delivery_id("").unwrap_err().length(), 0);
        assert!(envelope().with_delivery_id("é".repeat(256)).is_ok());
        assert_eq!(
            envelope()
                .with_delivery_id("x".repeat(257))
                .unwrap_err()
                .length(),
            257
        );
    }

    #[test]
    fn a_delivery_id_the_binding_does_not_allow_is_dropped_not_refused() {
        for delivery_id in [json!(""), json!("x".repeat(257)), json!(7), json!(null)] {
            let bytes = serde_json::to_vec(&json!({
                "type": ENVELOPE_TYPE,
                "document": document(),
                "deliveryId": delivery_id,
            }))
            .unwrap();
            let envelope = Envelope::from_bytes(&bytes).expect("envelope");
            assert_eq!(envelope.delivery_id(), None, "{delivery_id}");
        }
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
