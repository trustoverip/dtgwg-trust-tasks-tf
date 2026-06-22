---
slug: tsp
version: "0.1"
title: Trust Spanning Protocol (TSP) transport binding
summary: Carries Trust Task documents inside ToIP Trust Spanning Protocol messages; the authenticated sender VID maps to the framework's transport-authenticated party identity, with direct, nested, and routed carriage defined.
status: draft
targetFrameworkVersion: "0.2"
bindingURI: https://trusttasks.org/binding/tsp/0.1
envelopeType: https://trusttasks.org/binding/tsp/0.1/envelope
authors:
  - Glenn Gore (https://github.com/stormer78)
---

## Abstract

This binding specifies how *Trust Task documents* are exchanged inside [ToIP Trust Spanning Protocol (TSP)](https://trustoverip.github.io/tswg-tsp-specification/) messages. A producer places a *Trust Task document* inside the framework-reserved *envelope object* defined below and sends it as the authenticated, encrypted payload of a TSP message addressed from the producer's *Verifiable Identifier* (VID) to the recipient's VID. TSP seals the payload with HPKE authenticated encryption and signs the message, so the recipient learns the sender's VID under the same step that protects the bytes. On unwrap, the authenticated sender VID becomes the framework's *transport-authenticated sender* for the purposes of [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity) precedence.

TSP can convey a document **directly** between two endpoints, or **routed** through one or more intermediaries, with optional **nesting** for metadata privacy. This binding defines the framework consequences of each mode — in particular, which modes provide end-to-end producer→consumer guarantees (and therefore permit `proof` to be omitted under [§4.7.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#471-when-to-include-a-proof)) and which do not.

## Status of This Document

`0.1` draft. Targets **framework `0.2`** and uses the framework's lowerCamelCase error-code vocabulary ([SPEC §4.10](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#410-error-codes), [§8.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#83-standard-error-codes)). It parallels the [`didcomm/0.1`](../../didcomm/0.1/spec.md) and [`https/0.1`](../../https/0.1/spec.md) bindings in structure; note those two predate the 0.2 convention and still use the frozen 0.1 snake_case codes (a separate sweep should align them). TSP envelope and message structure follow the ToIP TSP specification (Implementers Draft, Rev 2). A reference TSP implementation is [OpenWallet Foundation Labs `tsp`](https://github.com/openwallet-foundation-labs/tsp); the Affinidi `affinidi-tsp` crate is a second implementation.

## 1. Binding URI

| Resource              | URI                                                       |
|-----------------------|-----------------------------------------------------------|
| Binding identifier    | `https://trusttasks.org/binding/tsp/0.1`                  |
| Envelope `type` value | `https://trusttasks.org/binding/tsp/0.1/envelope`         |

The *binding identifier* is the stable URI for this binding specification ([SPEC §9.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#93-binding-namespace)). The *envelope type* is the value a producer assigns to the binding *envelope object*'s `type` member (see [§2](#2-document-carriage)) when carrying a Trust Task over TSP. Per [SPEC §9.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#93-binding-namespace), neither URI is a *Type URI* and neither **MUST** appear in a Trust Task document's `type` member.

## 2. Document carriage

Because TSP is a general-purpose secure-messaging substrate, a TSP message payload may carry content other than a Trust Task. This binding therefore frames the document in a small, self-describing **envelope object** so a receiver can recognise — and dispatch — Trust-Task-carrying payloads without ambiguity. (This parallels the DIDComm binding's `type`-tagged envelope; TSP has no envelope-level content-type field of its own.)

The TSP **message payload** (the plaintext that TSP seals) **MUST** be the JSON serialisation of the envelope object:

```json
{
  "type": "https://trusttasks.org/binding/tsp/0.1/envelope",
  "document": { /* the Trust Task document, as a JSON object (not a string) */ }
}
```

| Envelope member | Value                                                                          |
|-----------------|--------------------------------------------------------------------------------|
| `type`          | `https://trusttasks.org/binding/tsp/0.1/envelope`                              |
| `document`      | The *Trust Task document* serialised as a JSON object (UTF-8, no BOM).         |

The producer then sends this payload as a TSP message:

| TSP element      | Value                                                                          |
|------------------|--------------------------------------------------------------------------------|
| `VID_sndr`       | The producer's VID (the identity TSP authenticates the message from).          |
| `VID_rcvr`       | The recipient's VID (the framework *consumer*, or, in routed mode, the first hop — see [§5](#5-routed-and-nested-carriage)). |
| message type     | `Direct`, or `Routed`/`Nested` per [§5](#5-routed-and-nested-carriage).        |
| payload          | The envelope object above, sealed by TSP's HPKE authenticated encryption.      |

A TSP message carrying a Trust Task **MUST** be sealed and signed by TSP's authenticated encryption (TSP does not define an anonymous/unauthenticated sender mode; see [§4](#4-error-mapping)).

A consumer **MUST**:

1. Receive and verify the TSP message — checking the message signature and decrypting the payload — recovering the authenticated `VID_sndr` and the addressed `VID_rcvr`.
2. Parse the decrypted payload as JSON and reject it if its `type` member is not exactly `https://trusttasks.org/binding/tsp/0.1/envelope` (see [§4](#4-error-mapping)). A payload that is valid TSP but is not a Trust Task envelope is not an error of this binding; it is simply not dispatched through the framework pipeline.
3. Deserialise `document` as a *Trust Task document* (`TrustTask<P>` for some payload type `P` selected by the document's `type` member).
4. Proceed with the framework's [§7.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#72-consumer-requirements) pipeline.

## 3. Identity mapping

A TSP VID is itself a *Verifiable Identifier* in the framework's sense ([SPEC §4.8](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#48-the-issuer-and-recipient-members) places no constraint on the VID scheme), so — unlike the DIDComm binding, which normalises a `sender_kid` to its bare DID — no transformation is applied: the TSP VID string **is** the framework VID. Comparison is exact string equality per [SPEC §4.8](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#48-the-issuer-and-recipient-members).

The mapping into the framework's [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity) precedence is:

| Framework concept                           | TSP value                                                                                       |
|---------------------------------------------|-------------------------------------------------------------------------------------------------|
| *Transport-authenticated sender*            | The authenticated `VID_sndr` of the TSP message that the *consumer* actually verified and decrypted. In nested/routed carriage this is the `VID_sndr` of the **innermost** message the consumer opens (see [§5](#5-routed-and-nested-carriage)). |
| *Transport-authenticated recipient*         | The consumer's own VID — the `VID_rcvr` it unwrapped the (innermost) message for.               |
| Producer's *in-band* `issuer` (when set)    | Compared by exact string equality against the transport-authenticated sender per [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity); mismatch is `identityMismatch`. |
| Producer's *in-band* `recipient` (when set) | Compared by exact string equality against the transport-authenticated recipient per [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity); mismatch is `identityMismatch`. |

Where the in-band member is absent, the *consumer* **MAY** treat the TSP-authenticated value as if it were carried in-band — TSP provides authenticated identity end-to-end for the message the consumer opens, so omitting the in-band `issuer`/`recipient` is conformant per [SPEC §4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity) — **subject to the routing caveat in [§5.3](#53-proof-and-identity-under-routing)**.

When applying the §8.1 error-response routing rule under `identityMismatch`, the *consumer* **MUST** route its `trust-task-error/0.2` response to the transport-authenticated sender (the VID it actually authenticated) and **MUST NOT** route to the contested in-band `issuer`.

## 4. Error mapping

| Transport-level condition                                                          | Framework disposition                                                                 |
|------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------|
| TSP signature or HPKE authentication failed (sender not authenticated / payload tampered) | Transport-level authentication failure; the message **MUST NOT** enter the framework pipeline. No framework error code applies — the sender is not authenticated, so no `trust-task-error` response can be routed. |
| Decrypted payload is not a JSON object, or its `type` is not the envelope type     | Reject at the binding layer; do not enter the framework pipeline.                     |
| `document` fails to deserialise as `TrustTask<P>`                                  | `malformedRequest` per [SPEC §8.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#83-standard-error-codes). |
| A routed message addressed to this VID as an **intermediary** (not the exit)       | Not a framework input — forward per [§5](#5-routed-and-nested-carriage); the document is never opened by an intermediary. |

Error responses generated by the framework pipeline (a `trust-task-error/0.2` document) **SHOULD** be returned by packing the response into a fresh TSP message — itself a Trust Task envelope per [§2](#2-document-carriage) — addressed to the authenticated `VID_sndr` of the originating message, over the established TSP relationship (see [§6](#6-responses-and-error-delivery)).

## 5. Routed and nested carriage

TSP can carry a message **directly** (`VID_sndr → VID_rcvr`) or **routed** through intermediaries, optionally with **nesting** for metadata privacy. The framework treats only the endpoints that open the *Trust Task document* as *parties*; intermediaries are transport actors.

### 5.1 Direct mode

The producer addresses the recipient VID directly. The TSP message provides end-to-end integrity, sender authentication, and confidentiality between producer and consumer. This is the simplest and strongest case for the framework.

### 5.2 Routed and nested mode

In **routed mode** ([TSP spec §5.3](https://trustoverip.github.io/tswg-tsp-specification/)), the message traverses one or more intermediaries; the hop list is carried in confidential routing fields. An intermediary opens only the layer addressed to **its** VID, reads the next hop, re-addresses the message, and forwards it — it **MUST NOT** be treated as the framework *recipient* and does not run the [§7.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#72-consumer-requirements) pipeline.

In **nested mode** ([TSP spec §5.5](https://trustoverip.github.io/tswg-tsp-specification/)), the producer wraps an inner TSP message — sealed and signed from the producer's VID to the final recipient's VID — inside one or more outer messages addressed to intermediaries. The Trust Task envelope ([§2](#2-document-carriage)) is the payload of the **innermost** message. Intermediaries see only outer-layer VIDs; the final recipient opens the inner message and recovers a producer→recipient end-to-end authenticated, confidential Trust Task.

A producer that requires metadata privacy or end-to-end confidentiality across intermediaries **SHOULD** use nested mode so that the Trust Task envelope is sealed to the final recipient and never exposed to intermediaries.

### 5.3 Proof and identity under routing

The framework guarantees a consumer can rely on for an opened Trust Task depend on the mode:

| Mode                                  | Producer→consumer end-to-end auth + confidentiality? | In-band `proof` |
|---------------------------------------|------------------------------------------------------|-----------------|
| Direct                                | Yes (TSP seals/signs producer→consumer)              | **MAY** be omitted ([§4.7.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#471-when-to-include-a-proof)) |
| Routed **with nesting** (inner sealed to final recipient) | Yes (innermost message is producer→consumer)         | **MAY** be omitted |
| Routed **without nesting** (single payload, intermediaries decrypt to route) | **No** — intermediaries can read the payload, and the final recipient authenticates only the last relaying hop, not the producer | **SHOULD** be included so the consumer can verify the producer; and the payload is exposed to intermediaries |

Accordingly, the §3 allowance to omit in-band `issuer`/`recipient` applies in direct and nested modes. In **non-nested routed mode**, a producer that omits both an in-band `proof` and in-band `issuer` gives the final recipient no verifiable producer identity; such a producer **SHOULD** either nest the message or carry an in-band `proof` bound to an in-band `recipient` ([§4.8.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#482-audience-binding)). A *Trust Task specification* whose `proof` requirement is **REQUIRED** ([SPEC §7.3 item 8](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#73-specification-requirements)) settles the matter regardless of mode.

### 5.4 Relationship prerequisite

TSP exchanges presuppose a bidirectional relationship between the two VIDs of each hop (established via TSP relationship-forming, [TSP spec §7.1](https://trustoverip.github.io/tswg-tsp-specification/)). Establishing or maintaining that relationship is a TSP-layer concern outside this binding; a Trust Task exchange simply requires that the necessary relationships exist for the chosen direct/routed path.

## 6. Responses and error delivery

A `#response`-variant *Trust Task document* and a `trust-task-error/0.2` document are both returned by packing them into a fresh Trust Task envelope ([§2](#2-document-carriage)) and sending them as a TSP message back toward the originating producer:

* **Direct / nested:** address the response from the consumer's VID to the authenticated producer VID over the existing relationship.
* **Routed:** use the TSP routed reply path ([TSP spec §7.1.3](https://trustoverip.github.io/tswg-tsp-specification/)) when one was established; otherwise the response follows whatever return path the relationship provides.
* **Fire-and-forget:** where no return relationship exists, response and error delivery are best-effort and **MAY** be undeliverable. Producers that require a response **SHOULD** ensure a return path exists before sending. The framework's `id`-keyed idempotency ([SPEC §10.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#101-cross-recipient-replay)) lets a producer safely retry where the binding cannot guarantee delivery.

## 7. Transport security profile

For [SPEC §4.7.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#471-when-to-include-a-proof) evaluation, a TSP message the consumer opens provides:

* **Sender authentication** — TSP HPKE authenticated encryption binds the sender's static key into the key agreement; a successful open proves the message came from the holder of `VID_sndr`'s keys.
* **Integrity** — a signature over the TSP envelope and payload.
* **Confidentiality** — authenticated encryption of the payload to the recipient's key. Confidentiality from **intermediaries** holds only in direct mode and in the nested layers sealed to the final recipient (see [§5.3](#53-proof-and-identity-under-routing)).
* **Freshness** — TSP relationship-forming carries nonces; data messages do not inherently prevent replay. Consumers whose tasks have persistent effect **SHOULD** apply the framework's `id`-keyed idempotency cache ([SPEC §10.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#101-cross-recipient-replay)).

These guarantees apply end-to-end between producer and consumer in **direct** and **nested** modes; in **non-nested routed** mode they apply only hop-by-hop (see [§5.3](#53-proof-and-identity-under-routing)).

## 8. Discovery wiring

A consumer **MAY** advertise the set of *Type URIs* it dispatches by handling `https://trusttasks.org/spec/trust-task-discovery/0.1` ([SPEC §11](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#11-discovery-and-capability-negotiation)). A discovery request is carried as an ordinary Trust Task envelope over TSP per [§2](#2-document-carriage); no separate mechanism is defined.

## 9. Versioning

This binding follows the framework's `MAJOR.MINOR` versioning ([SPEC §5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#5-versioning)). A `MINOR` revision **MUST** remain backwards-compatible with consumers implementing this version: the envelope type and object shape are preserved, the carriage and identity-mapping rules are preserved, and only additive conventions, additional carriage modes, or stricter error/identity mappings may be introduced. Breaking changes — a new envelope type, a different carriage, an incompatible identity-mapping or routing rule — require a `MAJOR` bump and a new binding URI.

## 10. References

- [Trust Spanning Protocol (TSP) Specification](https://trustoverip.github.io/tswg-tsp-specification/) — ToIP, Implementers Draft (Rev 2), §3 (message structure), §5.3 (routed mode), §5.5 (nesting), §7.1 (relationship forming).
- [DIDComm v2.1 transport binding](../../didcomm/0.1/spec.md) and [HTTPS transport binding](../../https/0.1/spec.md) — sibling bindings in this framework.
- [Trust Tasks framework specification](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md), §4.7.1, §4.8, §4.8.1, §7.2, §8, §9.
