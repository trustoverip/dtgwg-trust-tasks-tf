---
slug: didcomm
version: "0.1"
title: DIDComm v2.1 transport binding
summary: Carries Trust Task documents inside DIDComm v2.1 authcrypt envelopes; the verified sender_kid maps to the framework's transport-authenticated party identity.
status: draft
targetFrameworkVersion: "0.1"
bindingURI: https://trusttasks.org/binding/didcomm/0.1
envelopeType: https://trusttasks.org/binding/didcomm/0.1/envelope
authors:
  - Glenn Gore (https://github.com/stormer78)
---

## Abstract

This binding specifies how *Trust Task documents* are exchanged inside [DIDComm v2.1](https://identity.foundation/didcomm-messaging/spec/) messages. A producer wraps a *Trust Task document* in a DIDComm message whose `type` is the framework-reserved *envelope type* defined below; the producer then signs and encrypts the envelope using DIDComm's authenticated encryption (authcrypt) so the recipient learns the sender's DID under the same step that protects the bytes. On unwrap, the verified `sender_kid` becomes the framework's *transport-authenticated sender* for the purposes of [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity) precedence.

## Status of This Document

`0.1` draft. Tracks `SPEC.md 0.1`. The binding is implemented by [`trust-tasks-didcomm`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/tree/main/trust-tasks-didcomm) and round-trip-tested against `affinidi-messaging-test-mediator`.

## 1. Binding URI

| Resource              | URI                                                              |
|-----------------------|------------------------------------------------------------------|
| Binding identifier    | `https://trusttasks.org/binding/didcomm/0.1`                     |
| Envelope `type` value | `https://trusttasks.org/binding/didcomm/0.1/envelope`            |

The *binding identifier* is the stable URI for this binding specification ([SPEC §9.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#93-binding-namespace)). The *envelope type* is the value a producer assigns to the DIDComm message `type` field when carrying a Trust Task. Per [SPEC §9.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#93-binding-namespace), neither URI is a *Type URI* and neither **MUST** appear in a Trust Task document's `type` member.

## 2. Document carriage

A DIDComm message carrying a *Trust Task document* **MUST** populate the following fields:

| Field      | Value                                                                              |
|------------|------------------------------------------------------------------------------------|
| `type`     | `https://trusttasks.org/binding/didcomm/0.1/envelope`                              |
| `id`       | A fresh message identifier (DIDComm's own — independent of the Trust Task's `id`). |
| `from`     | The DID URL of the sender's signing key.                                           |
| `to`       | A single-element array containing the recipient's DID.                             |
| `body`     | The *Trust Task document* serialised as a JSON object (not a string).              |

The DIDComm message **MUST** be encrypted using DIDComm's *authcrypt* algorithm (anoncrypt is not sufficient — see [§4. Error mapping](#4-error-mapping)).

Producers **MAY** include additional DIDComm headers — `created_time`, `expires_time`, `thid`, `pthid`, attachments, etc. The framework consumes none of them; the *Trust Task document*'s `issuedAt`, `expiresAt`, and `threadId` members ([SPEC §4.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#42-top-level-members)) remain authoritative for framework-level processing.

A consumer **MUST**:

1. Unpack the DIDComm envelope, verifying the authcrypt signature and decrypting the `body`.
2. Reject the envelope if its `type` field is not exactly `https://trusttasks.org/binding/didcomm/0.1/envelope` (see [§4](#4-error-mapping)).
3. Deserialise `body` as a *Trust Task document* (`TrustTask<P>` for some payload type `P`).
4. Proceed with the framework's [§7.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#72-consumer-requirements) pipeline.

## 3. Identity mapping

The mapping into the framework's [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity) precedence is:

| Framework concept                          | DIDComm value                                                                  |
|--------------------------------------------|--------------------------------------------------------------------------------|
| *Transport-authenticated sender*           | The verified `sender_kid` of the authcrypt envelope, normalised to its bare DID. |
| *Transport-authenticated recipient*        | The unwrapping party's own DID (i.e. the DID the receiving DIDComm agent unpacks for). |
| Producer's *in-band* `issuer` (when set)   | Compared against the transport-authenticated sender per [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity); mismatch is `identity_mismatch`. |
| Producer's *in-band* `recipient` (when set) | Compared against the transport-authenticated recipient per [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity); mismatch is `identity_mismatch`. |

Where the in-band member is absent, the *consumer* **MAY** treat the DIDComm-derived value as if it were carried in-band — i.e. authcrypt provides authenticated identity end-to-end, so omitting the in-band `issuer`/`recipient` for a DIDComm-only exchange is conformant per [SPEC §4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity).

When applying the §8.1 error-response routing rule under `identity_mismatch`, the *consumer* **MUST** route its `trust-task-error/0.1` response to the transport-authenticated sender (the DID it actually authenticated via authcrypt) and **MUST NOT** route to the contested in-band `issuer`.

## 4. Error mapping

| Transport-level condition                                                            | Framework disposition                                  |
|--------------------------------------------------------------------------------------|--------------------------------------------------------|
| Envelope arrived as anoncrypt or plaintext (no authenticated sender)                 | Reject; treat as `unauthenticated` per [SPEC §8.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#83-standard-error-codes). |
| `type` field is not `https://trusttasks.org/binding/didcomm/0.1/envelope`            | Reject the envelope at the DIDComm layer; do not enter the framework pipeline. |
| `body` fails to deserialise as `TrustTask<P>`                                        | `malformed_request` per [SPEC §8.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#83-standard-error-codes). |
| Decryption / signature verification failure                                          | DIDComm-level error; the message **MUST NOT** be passed to the framework pipeline. |

Error responses generated by the framework pipeline (a `trust-task-error/0.1` document) **SHOULD** be returned by packing the response back into a fresh authcrypt envelope addressed to the verified `sender_kid` of the originating message.

## 5. Proof interaction

A *Trust Task document* delivered over an authcrypt'd DIDComm envelope between two end-to-end parties enjoys integrity and sender authentication from the DIDComm layer. Per [SPEC §4.7.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#471-when-to-include-a-proof), the in-band `proof` member **MAY** be omitted in that case.

A *Trust Task specification* that declares `proof` as **REQUIRED** ([SPEC §7.3 item 8](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#73-specification-requirements)) overrides this binding-level allowance: the in-band `proof` is mandatory regardless of transport, because such specifications produce documents intended to be replayable past the original transport hop.

## 6. Versioning

This binding follows the framework's `MAJOR.MINOR` versioning ([SPEC §5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#5-versioning)). A `MINOR` revision **MUST** remain backwards-compatible with consumers implementing this version: the envelope type is preserved, the carriage rules are preserved, and only additive header conventions, additional identity-mapping refinements, or stricter error mappings may be introduced. Breaking changes — a new envelope type, a different carriage, an incompatible identity-mapping rule — require a `MAJOR` bump and a new binding URI.

## 7. References

- [DIDComm Messaging v2.1](https://identity.foundation/didcomm-messaging/spec/) — Decentralized Identity Foundation.
- [Trust Tasks framework specification](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md), §4.8.1, §7.2, §8, §9.
