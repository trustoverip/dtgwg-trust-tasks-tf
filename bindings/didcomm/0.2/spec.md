---
slug: didcomm
version: "0.2"
title: DIDComm v2.1 transport binding
summary: Carries Trust Task documents inside DIDComm v2.1 authcrypt envelopes; the verified sender_kid maps to the framework's transport-authenticated party identity, and the DIDComm thread headers map to the framework's threadId and parentThreadId.
status: draft
targetFrameworkVersion: "0.3"
bindingURI: https://trusttasks.org/binding/didcomm/0.2
envelopeType: https://trusttasks.org/binding/didcomm/0.1/envelope
authors:
  - Glenn Gore (https://github.com/stormer78)
---

## Abstract

This binding specifies how *Trust Task documents* are exchanged inside [DIDComm v2.1](https://identity.foundation/didcomm-messaging/spec/) messages. A producer wraps a *Trust Task document* in a DIDComm message whose `type` is the framework-reserved *envelope type* defined below; the producer then signs and encrypts the envelope using DIDComm's authenticated encryption (authcrypt) so the recipient learns the sender's DID under the same step that protects the bytes. On unwrap, the verified `sender_kid` becomes the framework's *transport-authenticated sender* for the purposes of [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity) precedence.

## Status of This Document

`0.2` draft. Targets **framework `0.3`** and uses the framework's lowerCamelCase error-code vocabulary ([SPEC §4.10](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#410-naming-conventions), [§8.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#83-standard-error-codes)). The binding is implemented by [`trust-tasks-didcomm`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/tree/main/trust-tasks-didcomm) and round-trip-tested against `affinidi-messaging-test-mediator`.

## 1. Binding URI

| Resource              | URI                                                              |
|-----------------------|------------------------------------------------------------------|
| Binding identifier    | `https://trusttasks.org/binding/didcomm/0.2`                     |
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

Producers **MAY** include additional DIDComm headers — `created_time`, `expires_time`, attachments, etc. Except for the thread headers covered in [§3.1](#31-thread-correlation), the framework consumes none of them; the *Trust Task document*'s `issuedAt`, `expiresAt`, `threadId` and `parentThreadId` members ([SPEC §4.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#42-top-level-members)) remain authoritative for framework-level processing.

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
| Producer's *in-band* `issuer` (when set)   | Compared against the transport-authenticated sender per [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity); mismatch is `identityMismatch`. |
| Producer's *in-band* `recipient` (when set) | Compared against the transport-authenticated recipient per [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity); mismatch is `identityMismatch`. |

Where the in-band member is absent, the *consumer* **MAY** treat the DIDComm-derived value as if it were carried in-band — i.e. authcrypt provides authenticated identity end-to-end, so omitting the in-band `issuer`/`recipient` for a DIDComm-only exchange is conformant per [SPEC §4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity).

When applying the §8.1 error-response routing rule under `identityMismatch`, the *consumer* **MUST** route its `trust-task-error/0.2` response to the transport-authenticated sender (the DID it actually authenticated via authcrypt) and **MUST NOT** route to the contested in-band `issuer`.

### 3.1 Thread correlation

DIDComm carries its own correlation headers, `thid` and `pthid`, alongside the framework's `threadId` ([SPEC §4.9](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#49-the-threadid-member)) and `parentThreadId` ([SPEC §4.9.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#492-the-parentthreadid-member)). [SPEC §9.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#91-what-a-transport-binding-specifies) requires a binding that maps them to state the mapping. This binding maps them as follows.

| Framework member | DIDComm header |
|---|---|
| `threadId` | `thid` |
| `parentThreadId` | `pthid` |

**Producers.** A *producer* **SHOULD** set `thid` from the document's `threadId`, and `pthid` from its `parentThreadId`. Where the document carries no `threadId`, the *producer* **SHOULD** set `thid` to the document's `id` — the same fallback §4.9 defines for the framework member, so that the DIDComm thread and the Trust Task exchange are named by the same value. Where the document carries no `parentThreadId`, `pthid` is omitted.

Populating the headers *from* the members, rather than the reverse, is what makes the two layers agree. A producer that lets DIDComm default its own headers will produce a message whose `thid` is the DIDComm message `id` — a different identifier space entirely (see [§2](#2-document-carriage): the DIDComm `id` is independent of the Trust Task's `id`).

**Consumers.** Where **both** a DIDComm header and its framework member are explicitly present, they **MUST** be equal. A *consumer* **MUST** reject a mismatch with `malformedRequest` ([SPEC §8.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#83-standard-error-codes)) — **not** `identityMismatch`, which is reserved for a contested *party* identity and carries §8.1's response-suppression rules. A thread disagreement contests no one's identity; it is a structurally inconsistent document.

Where only one side is present, no comparison is made and no error arises. This is deliberate and is the whole reason the rule is scoped to both-present: DIDComm's `thid` defaults to the DIDComm message `id` when absent, and the framework's `threadId` falls back to the *Trust Task document*'s `id`. Those are different identifiers by construction, so a rule requiring equality unconditionally would reject exchanges that conform perfectly well on both layers.

The in-band members remain authoritative for all framework-level processing ([SPEC §4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity) applies the same precedence to identity). A *consumer* **MUST NOT** populate a missing `threadId` or `parentThreadId` from the transport headers: unlike `issuer` and `recipient`, where an authenticated transport genuinely carries the value the framework needs, a DIDComm thread header is the transport's own correlation and asserting it as the document's would fabricate a member the *producer* chose not to send.

## 4. Error mapping

| Transport-level condition                                                            | Framework disposition                                  |
|--------------------------------------------------------------------------------------|--------------------------------------------------------|
| Envelope arrived as anoncrypt or plaintext (no authenticated sender)                 | Transport-level reject; the message **MUST NOT** enter the framework pipeline. No framework error code applies — there is no authenticated sender to route a `trust-task-error` response to. |
| `type` field is not `https://trusttasks.org/binding/didcomm/0.1/envelope`            | Reject the envelope at the DIDComm layer; do not enter the framework pipeline. |
| `body` fails to deserialise as `TrustTask<P>`                                        | `malformedRequest` per [SPEC §8.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#83-standard-error-codes). |
| `thid` and `threadId`, or `pthid` and `parentThreadId`, are both present and disagree | `malformedRequest` (see [§3.1](#31-thread-correlation)). Not `identityMismatch`. |
| Decryption / signature verification failure                                          | DIDComm-level error; the message **MUST NOT** be passed to the framework pipeline. |

Error responses generated by the framework pipeline (a `trust-task-error/0.2` document) **SHOULD** be returned by packing the response back into a fresh authcrypt envelope addressed to the verified `sender_kid` of the originating message.

## 5. Proof interaction

A *Trust Task document* delivered over an authcrypt'd DIDComm envelope between two end-to-end parties enjoys integrity and sender authentication from the DIDComm layer. Per [SPEC §4.7.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#471-when-to-include-a-proof), the in-band `proof` member **MAY** be omitted in that case.

A *Trust Task specification* that declares `proof` as **REQUIRED** ([SPEC §7.3 item 8](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#73-specification-requirements)) overrides this binding-level allowance: the in-band `proof` is mandatory regardless of transport, because such specifications produce documents intended to be replayable past the original transport hop.

## 6. Transport security profile

*Stated in anticipation of [SPEC §9.1.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#911-permitting-proof-to-be-omitted),
which binds transport bindings targeting framework 0.4; this binding targets
0.3, where the profile is not yet required. It is stated because §5 permits
`proof` to be omitted, and §9.1.1's point is that such an allowance should never
rest on a transport's name.*

| Property | What DIDComm v2.1 authcrypt provides |
|---|---|
| **Authenticated producer** | The verified `sender_kid` of the authcrypt envelope, authenticated by the AEAD in the same step that protects the bytes. Anoncrypt and plaintext yield no authenticated sender and are rejected at the transport layer ([§4](#4-error-mapping)). |
| **Mapping to a VID** | `sender_kid` normalises to its bare DID **directly**. No external state is consulted — the substantive difference from the DIDComm v1 binding, where the key-to-DID binding is connection state the agent holds rather than something the wire carries. |
| **Audience binding** | Cryptographic, not asserted: authcrypt seals the envelope to the recipient's keys, so a party that cannot decrypt is not an audience. This is *transport* audience binding and does **not** satisfy [§4.8.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#482-audience-binding), which governs what a `proof` commits to and is unaffected by how the bytes travelled. |
| **Integrity across intermediaries** | The authcrypt AEAD covers the message end-to-end between the sender and recipient DIDs. A mediator handles the outer `forward`; the inner message is opaque to it. This is a tested path, not a theoretical one — see the Status section. |
| **Re-origination** | A mediator **cannot** modify the inner message undetected, nor originate one as the sender without the sender's secret key. It **can** drop, delay, reorder, and re-deliver. |
| **Freshness / replay** | **None guaranteed.** DIDComm's `expires_time` is optional and is a staleness hint, not an anti-replay construct, and a mediator may re-deliver. See the note below. |
| **Key and credential status** | Resolution of the sender DID to verification keys is the *consumer*'s own resolver and cache. A rotation the *consumer* has not yet observed leaves a superseded key verifying successfully. No revocation or status check is mandated at this layer. |
| **Where the guarantee stops** | At the message. The envelope is discarded on unwrap, and the guarantee does not travel with the document. |

**"Between two end-to-end parties" is doing real work in §5.** The allowance
holds for the message the *consumer* actually unwrapped, authenticated to the
DID that sealed it. It does not extend to a document that reached that sender by
some other path, nor to one the *consumer* forwards onward: in both cases the
party the authcrypt authenticated is not the party that composed the document.

**The transport provides no replay protection, and that is load-bearing.**
Because nothing here guarantees freshness and a mediator may re-deliver, a
*consumer* over this binding carries the whole burden of
[§7.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#72-consumer-requirements) item 11 — duplicate-execution protection
keyed on the document `id` — for any *consequential Trust Task*. A *consumer*
that treats mediated delivery as at-most-once is wrong on both a hostile replay
and an ordinary mediator retry.

**What omitting `proof` costs.** A document delivered without an in-band `proof`
carries no evidence of its producer once the envelope is gone: the
authentication was a property of the envelope, and the envelope is discarded on
unwrap. A *consumer* that retains such a document, forwards it, or offers it to
a third party is offering bytes nobody signed. Where a document is intended to
be retained or relied upon beyond the receiving agent,
[§4.7.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#471-when-to-include-a-proof) requires a `proof` regardless of this
binding's allowance, and a *Trust Task specification* declaring `proof`
**REQUIRED** settles it (§5).

## 7. Versioning

This binding follows the framework's `MAJOR.MINOR` versioning ([SPEC §5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#5-versioning)). A `MINOR` revision **MUST** remain backwards-compatible with consumers implementing this version: the envelope type is preserved, the carriage rules are preserved, and only additive header conventions, additional identity-mapping refinements, or stricter error mappings may be introduced. Breaking changes — a new envelope type, a different carriage, an incompatible identity-mapping rule — require a `MAJOR` bump and a new binding URI.

### 7.1 Changes from 0.1

Additive, and a `MINOR` increment accordingly: the envelope type is unchanged, so a `0.1` consumer recognises a `0.2` producer's messages and vice versa.

* Thread correlation ([§3.1](#31-thread-correlation)) is now mapped rather than ignored. `0.1` stated that the framework consumed no DIDComm headers, which left `thid` and the framework's `threadId` free to disagree with nothing detecting it.
* Targets framework `0.3`, which adds `parentThreadId` ([SPEC §4.9.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#492-the-parentthreadid-member)) — the member `pthid` maps to.
* One stricter error mapping, for a thread disagreement.

A `0.1` producer that never set `thid` remains conforming under `0.2`: the mapping is a `SHOULD` on the producer and the consumer comparison only engages when both values are present.

## 8. References

- [DIDComm Messaging v2.1](https://identity.foundation/didcomm-messaging/spec/) — Decentralized Identity Foundation.
- [Trust Tasks framework specification](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md), §4.8.1, §7.2, §8, §9.
