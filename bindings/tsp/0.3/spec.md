---
slug: tsp
version: "0.3"
title: Trust Spanning Protocol (TSP) transport binding
summary: Carries Trust Task documents inside ToIP Trust Spanning Protocol 1.0 messages; the authenticated sender VID maps to the framework's transport-authenticated party identity, with direct, nested, and routed carriage defined, and an optional delivery id requests an end-to-end delivery receipt. Requires TSP 1.0 and refuses a TSP draft.
status: draft
targetFrameworkVersion: "0.6.0"
bindingURI: https://trusttasks.org/binding/tsp/0.3
envelopeType: https://trusttasks.org/binding/tsp/0.1/envelope
authors:
  - Glenn Gore (https://github.com/stormer78)
---

## Abstract

This binding specifies how *Trust Task documents* are exchanged inside [ToIP Trust Spanning Protocol (TSP)](https://trustoverip.github.io/tswg-tsp-specification/) 1.0 messages. A producer places a *Trust Task document* inside the framework-reserved *envelope object* defined below and sends it as the authenticated, encrypted payload of a TSP message addressed from the producer's *Verifiable Identifier* (VID) to the recipient's VID. TSP signs every message and seals its payload to the recipient with the sender's VID inside the sealed payload, so the recipient learns the sender's VID under the same step that protects the bytes. On unwrap, the authenticated sender VID becomes the framework's *transport-authenticated sender* for the purposes of [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity) precedence.

TSP can convey a document **directly** between two endpoints, or **routed** through one or more intermediaries, with optional **nesting** for metadata privacy. This binding defines the framework consequences of each mode — in particular, which modes provide end-to-end producer→consumer guarantees (and therefore permit `proof` to be omitted under [§4.7.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#471-when-to-include-a-proof)) and which do not.

## Status of This Document

`0.3` draft. Targets **framework `0.6.0`** and uses the framework's lowerCamelCase error-code vocabulary ([SPEC §4.10](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#410-naming-conventions), [§8.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#83-standard-error-codes)). It parallels the [`didcomm/0.1`](../../didcomm/0.1/spec.md) and [`https/0.1`](../../https/0.1/spec.md) bindings in structure; note those two predate the 0.2 convention and still use the frozen 0.1 snake_case codes (a separate sweep should align them). TSP envelope and message structure follow the ToIP TSP specification **version 1.0** (version code `YTSP-BAA`), which this version requires ([§2.2](#22-tsp-version)). It supersedes [`tsp/0.2`](../0.2/spec.md), which cited the TSP Implementers Draft (Rev 2) and named no TSP version normatively; [§9.1](#91-changes-from-02) lists the changes. A reference TSP implementation is [OpenWallet Foundation Labs `tsp`](https://github.com/openwallet-foundation-labs/tsp); the Affinidi `affinidi-tsp` crate is a second implementation.

## 1. Binding URI

| Resource              | URI                                                       |
|-----------------------|-----------------------------------------------------------|
| Binding identifier    | `https://trusttasks.org/binding/tsp/0.3`                  |
| Envelope `type` value | `https://trusttasks.org/binding/tsp/0.1/envelope`         |

The *binding identifier* is the stable URI for this binding specification ([SPEC §9.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#93-binding-namespace)). The *envelope type* is the value a producer assigns to the binding *envelope object*'s `type` member (see [§2](#2-document-carriage)) when carrying a Trust Task over TSP. Per [SPEC §9.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#93-binding-namespace), neither URI is a *Type URI* and neither **MUST** appear in a Trust Task document's `type` member.

The envelope type is unchanged from `0.1`, and the envelope object is unchanged from `0.2`: a `0.2` envelope is a `0.1` envelope that may carry one more member ([§2.1](#21-delivery-receipts)). What `0.3` changes is the TSP message that carries the envelope, which must be TSP 1.0 ([§2.2](#22-tsp-version)).

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
| `deliveryId`    | OPTIONAL. A delivery id requesting an end-to-end delivery receipt ([§2.1](#21-delivery-receipts)). A string of 1 to 256 characters. |

The producer then sends this payload as a TSP message:

| TSP element      | Value                                                                          |
|------------------|--------------------------------------------------------------------------------|
| `VID_sndr`       | The producer's VID (the identity TSP authenticates the message from).          |
| `VID_rcvr`       | The recipient's VID (the framework *consumer*, or, in routed mode, the first hop — see [§5](#5-routed-and-nested-carriage)). |
| message type     | `Direct`, or `Routed`/`Nested` per [§5](#5-routed-and-nested-carriage).        |
| payload          | The envelope object above, as the confidential payload of a TSP 1.0 message: sealed to `VID_rcvr` with `VID_sndr` inside it (ESSR), and signed by `VID_sndr`. |

A TSP message carrying a Trust Task **MUST** be a TSP 1.0 message ([§2.2](#22-tsp-version)) whose payload is confidential: sealed to the recipient and signed by the sender (TSP does not define an anonymous/unauthenticated sender mode; see [§4](#4-error-mapping)).

A consumer **MUST**:

1. Receive and verify the TSP message — checking its TSP version ([§2.2](#22-tsp-version)), checking the message signature, decrypting the payload, and checking that the `VID_sndr` inside the payload matches the envelope's — recovering the authenticated `VID_sndr` and the addressed `VID_rcvr`.
2. Parse the decrypted payload as JSON and reject it if its `type` member is not exactly `https://trusttasks.org/binding/tsp/0.1/envelope` (see [§4](#4-error-mapping)). A payload that is valid TSP but is not a Trust Task envelope is not an error of this binding; it is simply not dispatched through the framework pipeline. An envelope member other than `type`, `document` and `deliveryId` is ignored.
3. Deserialise `document` as a *Trust Task document* (`TrustTask<P>` for some payload type `P` selected by the document's `type` member). Keep the received JSON object alongside it: the `proof` is verified over that object with only the top-level `proof` removed, never over a re-serialisation of the typed document, and the typed document is the one parsed from it ([SPEC §4.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#47-proof), "What a consumer verifies").
4. Proceed with the framework's [§7.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#72-consumer-requirements) pipeline.

### 2.1 Delivery receipts

A producer that wants end-to-end evidence that the consumer holds the message **MAY** add `deliveryId` to the envelope. Its presence is the request for a receipt; its absence asks for none. The receipt is the [`messaging/receipt`](../../../specs/messaging/receipt/0.1/spec.md) Trust Task, which states the rules for choosing the id, for sending and for settling, and is carried back as an ordinary envelope of this section.

Over TSP the receipt is the only end-to-end delivery evidence available. In routed and nested carriage ([§5](#5-routed-and-nested-carriage)) a hop that accepts or forwards a message is not the consumer, and a mediator that relays a message onward to another mediator can say nothing about whether the consumer collected it.

`deliveryId` sits on the envelope rather than in the *Trust Task document* because it describes this delivery, not the task. Re-sending the same document on a retry keeps the same `deliveryId`; sending it to a second recipient takes a new one.

A consumer that receives an envelope with `deliveryId`:

1. **SHOULD** send the receipt once it durably holds the message, as `messaging/receipt` requires, to the authenticated `VID_sndr` of the message that carried the id — the innermost one it opened, in nested carriage. It **MAY** decline, silently.
2. **MUST NOT** let `deliveryId` affect the framework pipeline. It is not the document `id`, not a `threadId`, and not an input to the duplicate-execution record of [§7.1](#71-duplicate-execution-record).
3. **MUST NOT** set `deliveryId` on the envelope carrying a receipt.

A `0.1` consumer ignores the member and sends no receipt, which the producer must already treat as no evidence rather than as failure.

### 2.2 TSP version

A *producer* **MUST** send the envelope in a TSP 1.0 message: TSP version MAJOR `1`, encoded as the version code `YTSP-BAA` for 1.0 ([TSP 1.0 §3, *TSP Envelope*; §9, *TSP Envelope Encoding*](https://trustoverip.github.io/tswg-tsp-specification/#serialization-and-encoding)).

A *consumer* **MUST** refuse a message whose TSP MAJOR is not `1` — among them the TSP drafts Rev 3 (`YTSP-AAC`) and Rev 2 (`YTSP-AAB`), whose MAJOR is `0` — before it uses any key or resolves the sender. It **MUST NOT** answer such a message: nothing in it has been authenticated, so there is no sender to route a reply to ([§4](#4-error-mapping)). It **SHOULD** report the refusal to its operator as a TSP version mismatch, distinct from a message it cannot open, so an operator can see which peers have not upgraded.

A *consumer* **MUST NOT** refuse a message for its TSP MINOR. TSP reserves a break in backward compatibility to the MAJOR ([TSP 1.0 §3, *TSP Envelope*](https://trustoverip.github.io/tswg-tsp-specification/#tsp-envelope), `TSP_Version`), so a 1.x message is a TSP 1.0 message as far as this binding is concerned. A 1.x message that then fails to verify or open is refused as any other message it cannot open is.

TSP has no version negotiation. A peer still on a TSP draft cannot exchange messages with a `0.3` implementation in either direction, and has to upgrade; nothing at this layer can bridge the two.

TSP 1.0 requires the sender's VID as a field of every confidential payload, following the ESSR construction, and a receiver checks it against the envelope's `VID_sndr` (TSP 1.0 §3, *TSP Payload* and *Receiver Procedure*, step 7). That checked `VID_sndr` is the one [§3](#3-identity-mapping) maps to the framework's *transport-authenticated sender*.

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

When applying the §8.1 error-response routing rule under `identityMismatch`, the *consumer* **MUST** route its `trust-task-error` response to the transport-authenticated sender (the VID it actually authenticated) and **MUST NOT** route to the contested in-band `issuer`.

## 4. Error mapping

| Transport-level condition                                                          | Framework disposition                                                                 |
|------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------|
| TSP MAJOR version is not `1` (a TSP draft, or a later MAJOR) | Refused at the binding layer before any key is used ([§2.2](#22-tsp-version)); the message **MUST NOT** enter the framework pipeline and **MUST NOT** be answered — the sender is not authenticated. **SHOULD** be reported to the operator as a TSP version mismatch. |
| TSP signature verification, decryption, or the ESSR sender check failed (sender not authenticated / payload tampered) | Transport-level authentication failure; the message **MUST NOT** enter the framework pipeline. No framework error code applies — the sender is not authenticated, so no `trust-task-error` response can be routed. |
| Decrypted payload is not a JSON object, or its `type` is not the envelope type     | Reject at the binding layer; do not enter the framework pipeline.                     |
| `document` fails to deserialise as `TrustTask<P>`                                  | `malformedRequest` per [SPEC §8.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#83-standard-error-codes). |
| A routed message addressed to this VID as an **intermediary** (not the exit)       | Not a framework input — forward per [§5](#5-routed-and-nested-carriage); the document is never opened by an intermediary. |

Error responses generated by the framework pipeline (a `trust-task-error` document) **SHOULD** be returned by packing the response into a fresh TSP message — itself a Trust Task envelope per [§2](#2-document-carriage) — addressed to the authenticated `VID_sndr` of the originating message, over the established TSP relationship (see [§6](#6-responses-and-error-delivery)).

### 4.1 Lifecycle mapping

Required by [SPEC §9.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#91-what-a-transport-binding-specifies), which makes the mapping a **MUST** for every binding: each state of [SPEC §4.12 *Document Lifecycle*](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#412-document-lifecycle) is named below against the TSP event that carries it, or declared to have no counterpart here.

TSP is a one-way message substrate. There is no request/response pair at the transport layer, so **every** reply named below is itself a *Trust Task document*, packed into a fresh Trust Task envelope ([§2](#2-document-carriage)) and sent back toward the producer per [§6](#6-responses-and-error-delivery). TSP's own events — a relationship formed, a hop accepted, a route taken — are not document states.

| Lifecycle state | TSP counterpart |
|---|---|
| `received` | **No counterpart** in the pipeline. Verifying the message, decrypting the payload and deserialising `document` happen inside the *consumer* and emit nothing. Where the producer requested one ([§2.1](#21-delivery-receipts)), a `messaging/receipt` reports that the *consumer*'s delivery layer durably holds the message — evidence that the document reached `received`, and of no later state. An **intermediary** accepting or forwarding a routed message is emphatically not this state — [§5.2](#52-routed-and-nested-mode) makes an intermediary a transport actor that never opens the document and never runs the pipeline. |
| `validated` | **No counterpart.** [SPEC §7.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#72-consumer-requirements) items 1–10 are internal to the *consumer* and are never signalled. |
| `accepted` | **No counterpart**, unless the *Trust Task specification* defines a reply that acknowledges acceptance. The duplicate-execution record of [§7.1](#71-duplicate-execution-record) is written internally. Where the *consumer* is understood but **blocked**, the framework's reserved `trust-task-next-step` document is carried as an ordinary envelope of [§2](#2-document-carriage) and the document remains `accepted` with the exchange open. |
| `executing` | **No counterpart.** Nothing is emitted when work begins. |
| `suspended` | The response to the `trust-task-control` document that suspended it ([SPEC §11](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#11-task-control)), packed per [§2](#2-document-carriage) and delivered per [§6](#6-responses-and-error-delivery). TSP expresses no transport-level counterpart: a relationship left idle, a route that stops carrying traffic, or an intermediary that stops forwarding is not suspension. |
| `responded` | A `<type>#response` document packed into a fresh Trust Task envelope and sent back toward the producer per [§6](#6-responses-and-error-delivery) — over the existing relationship in direct and nested mode, or over the routed reply path where one was established. Where the specification defines no success response, the framework's empty `{}` `#response` is the courtesy acknowledgement, sent the same way. |
| `errored` | A `trust-task-error` document, packed and delivered the same way ([§4](#4-error-mapping), [§6](#6-responses-and-error-delivery)). Three rows of [§4](#4-error-mapping)'s table produce no reply at all: a TSP version other than MAJOR `1` and a TSP signature, decryption or sender-check failure each leave no authenticated sender to route one to, and a payload whose `type` is not the envelope type is rejected at the binding layer without entering the pipeline. And see the return-path caveat below — over this transport `errored` can be reached and remain unobservable even when the *consumer* emits the error. |
| `cancelled` | The response to the `trust-task-control` document, for a *producer*-requested cancellation; a `trust-task-error` carrying `cancelled` where the *consumer* stopped of its own accord. Nothing at the TSP layer expresses this — see below. |
| `expired` | A `trust-task-error` carrying `expired`, where one can be returned. The document's own `expiresAt` is what expires; nothing in TSP's message handling evaluates it, and a route or relationship timing out is not it. |

Four substitutions this transport invites, stated so no implementer has to infer them:

* **A hop is not the *consumer*, so a hop's acceptance is not `received`.** In routed mode an intermediary opens only the layer addressed to its own VID, re-addresses the message and forwards it ([§5.2](#52-routed-and-nested-mode)). A successful hop, a routed message accepted for forwarding, or an intermediary's own transport-level acknowledgement tells a *producer* that its bytes moved — not that the *consumer* holds them, and certainly not that anything is executing. In **nested** mode the point is sharper still: the intermediaries cannot open the Trust Task envelope at all, so nothing they report can be about the document.
* **A reply that cannot be delivered leaves a terminal state unobservable.** [§6](#6-responses-and-error-delivery) already says it: where no return relationship exists, response and error delivery are best-effort and **MAY** be undeliverable. That is not a gap in this mapping but a fact about the transport — a document can reach `responded`, `errored`, `cancelled` or `expired`, the *consumer* can emit the correct reserved reply, and the *producer* can still hear nothing. A *producer* that requires a reply **SHOULD** ensure a return path exists before sending ([§5.4](#54-relationship-prerequisite)); one that does not **MUST NOT** read the resulting silence as any state, including success.
* **A relationship torn down is not `cancelled`.** Losing a TSP relationship, a route failing, or an intermediary dropping a message terminates a *delivery*. A document already accepted under [§7.1](#71-duplicate-execution-record) survives it and may still be executing. Semantic cancellation over this binding is a `trust-task-control` document and nothing else.
* **TSP's freshness is about the relationship, not the document.** [§7](#7-transport-security-profile) records that relationship-forming carries nonces while data messages do not inherently prevent replay. A nonce accepted at relationship-forming time establishes nothing about any document's state, and re-arrival of a document already `accepted` is not a state transition at all — [§7.1](#71-duplicate-execution-record) governs what, if anything, the *consumer* returns to it.

Because the transport is one-way, this binding expresses **no** counterpart for `validated`, `accepted` or `executing`, and none for `received` beyond the optional receipt of [§2.1](#21-delivery-receipts). Only the four states that have a reserved framework reply — `suspended`, `responded`, `errored`, `cancelled` — become observable, they become observable as *documents* rather than as TSP events, and under [§6](#6-responses-and-error-delivery)'s fire-and-forget case even they may not.

## 5. Routed and nested carriage

TSP can carry a message **directly** (`VID_sndr → VID_rcvr`) or **routed** through intermediaries, optionally with **nesting** for metadata privacy. The framework treats only the endpoints that open the *Trust Task document* as *parties*; intermediaries are transport actors.

### 5.1 Direct mode

The producer addresses the recipient VID directly. The TSP message provides end-to-end integrity, sender authentication, and confidentiality between producer and consumer. This is the simplest and strongest case for the framework.

### 5.2 Routed and nested mode

In **routed mode** ([TSP 1.0 §5, *Routed Messages Through Intermediaries*](https://trustoverip.github.io/tswg-tsp-specification/#routed-messages-through-intermediaries)), the message traverses one or more intermediaries; the hop list is carried in confidential routing fields. An intermediary opens only the layer addressed to **its** VID, reads the next hop, re-addresses the message, and forwards it — it **MUST NOT** be treated as the framework *recipient* and does not run the [§7.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#72-consumer-requirements) pipeline.

In **nested mode** ([TSP 1.0 §4, *Nested Messages*](https://trustoverip.github.io/tswg-tsp-specification/#nested-messages)), the producer wraps an inner TSP message — sealed and signed from the producer's VID to the final recipient's VID — inside one or more outer messages addressed to intermediaries. The Trust Task envelope ([§2](#2-document-carriage)) is the payload of the **innermost** message. Intermediaries see only outer-layer VIDs; the final recipient opens the inner message and recovers a producer→recipient end-to-end authenticated, confidential Trust Task.

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

TSP exchanges presuppose a bidirectional relationship between the two VIDs of each hop (established via TSP relationship forming, [TSP 1.0 §7, *Relationship Forming Protocol*](https://trustoverip.github.io/tswg-tsp-specification/#control-messages)). Establishing or maintaining that relationship is a TSP-layer concern outside this binding; a Trust Task exchange simply requires that the necessary relationships exist for the chosen direct/routed path.

## 6. Responses and error delivery

A `#response`-variant *Trust Task document* and a `trust-task-error` document are both returned by packing them into a fresh Trust Task envelope ([§2](#2-document-carriage)) and sending them as a TSP message back toward the originating producer:

* **Direct / nested:** address the response from the consumer's VID to the authenticated producer VID over the existing relationship.
* **Routed:** use the TSP routed reply path ([TSP 1.0 §7, *Relationship over a Routed Path*](https://trustoverip.github.io/tswg-tsp-specification/#control-messages)) when one was established; otherwise the response follows whatever return path the relationship provides.
* **Fire-and-forget:** where no return relationship exists, response and error delivery are best-effort and **MAY** be undeliverable. Producers that require a response **SHOULD** ensure a return path exists before sending. The framework's `id`-keyed idempotency ([SPEC §12.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#121-cross-recipient-replay)) lets a producer safely retry where the binding cannot guarantee delivery.

## 7. Transport security profile

For [SPEC §4.7.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#471-when-to-include-a-proof) evaluation, a TSP message the consumer opens provides:

* **Sender authentication** — the TSP signature by `VID_sndr`, together with the ESSR binding of `VID_sndr` into the sealed payload ([§2.2](#22-tsp-version)); a message that verifies and opens came from the holder of `VID_sndr`'s keys. TSP 1.0's HPKE-Base mode does not itself authenticate the sender, so the signature is what carries this property.
* **Integrity** — a signature over the TSP envelope and payload.
* **Confidentiality** — encryption of the payload to the recipient's key. Confidentiality from **intermediaries** holds only in direct mode and in the nested layers sealed to the final recipient (see [§5.3](#53-proof-and-identity-under-routing)).
* **Freshness** — TSP relationship-forming carries nonces; data messages do not inherently prevent replay. Consumers whose tasks have persistent effect **SHOULD** apply the framework's `id`-keyed idempotency cache ([SPEC §12.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#121-cross-recipient-replay)).

These guarantees apply end-to-end between producer and consumer in **direct** and **nested** modes; in **non-nested routed** mode they apply only hop-by-hop (see [§5.3](#53-proof-and-identity-under-routing)).

### 7.1 Duplicate-execution record

The freshness bullet above says a *consumer* whose task has persistent effect
**SHOULD** apply the framework's `id`-keyed idempotency; it names no mechanism,
and [SPEC §7.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#72-consumer-requirements) item 11 is in any case a **MUST** for a
*consequential Trust Task*. The mechanism is stated here.

A *consumer* implementing a *consequential Trust Task* over this binding
**MUST** keep the duplicate-execution record of item 11, and:

1. **The record is keyed on the *Trust Task document*'s `id`.** Nothing the TSP
   envelope carries — the envelope itself, its ephemeral material, or the
   relationship state that produced it — **MUST** substitute for it, per
   §7.2's keying paragraph. On this transport the point is stark: sealing the
   same document a second time produces an envelope sharing no bytes with the
   first, because TSP derives fresh ephemeral material per message. There is
   nothing about the envelope a record *could* key on and still absorb a
   re-send — which [§5.2](#52-routed-and-nested-mode) makes an ordinary event,
   since each intermediary may hold and re-forward the sealed inner message.
2. **Comparison is over the document, not the `id` alone.** A second document
   under a reused `id` whose content differs — including a re-signed or
   re-stamped `proof` — is **not** the retry of [§8.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#84-retry-semantics) and **MUST** be rejected
   with `idConflict`.
3. **Retention and the acceptance window are one bound.** The record **MUST**
   be retained at least as long as the *consumer* remains willing to execute
   the document — `expiresAt` where present, otherwise `issuedAt` plus the
   *consumer*'s acceptance window. A *consumer* that can establish neither
   **MUST NOT** execute a *consequential Trust Task* on the document. A routed
   path that may hold a message for longer than the *consumer*'s window widens
   both bounds together; there is no second retention setting to configure.
4. **A duplicate is not an error.** Where the *Trust Task specification*
   defines a success response the *consumer* **SHOULD** return the result the
   first execution produced; where the original execution is still running it
   **SHOULD** return or expose that execution's state rather than begin
   another; where the specification defines no response — including the
   fire-and-forget case [§6](#6-responses-and-error-delivery) describes, where
   no return relationship exists — silence is the correct disposition. In no
   case is a duplicate reported as `taskFailed`: the task did not fail, it
   already happened.
5. **A record that cannot be consulted fails closed.** Where the *consumer*
   cannot establish whether a document is a duplicate it **MUST NOT** execute,
   and **SHOULD** respond `unavailable` with `retryable` true: the producer's
   bit-for-bit resend is absorbed correctly once the record is reachable
   again.

A *consumer* replicated across processes **MUST** back the record with a store
every replica shares. Two replicas serving the same VID and each keeping their
own record accept the same document once apiece, which is the double execution
in a different shape.

## 8. Discovery wiring

A consumer **MAY** advertise the set of *Type URIs* it dispatches by handling `https://trusttasks.org/spec/trust-task-discovery/0.4` ([SPEC §10](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#10-discovery-and-capability-negotiation)). A discovery request is carried as an ordinary Trust Task envelope over TSP per [§2](#2-document-carriage); no separate mechanism is defined.

## 9. Versioning

This binding follows the framework's `MAJOR.MINOR` versioning ([SPEC §5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#5-versioning)). A `MINOR` revision **MUST** remain backwards-compatible with consumers implementing this version: the envelope type and object shape are preserved, the carriage and identity-mapping rules are preserved, and only additive conventions, additional carriage modes, or stricter error/identity mappings may be introduced. Breaking changes — a new envelope type, a different carriage, an incompatible identity-mapping or routing rule, a different TSP MAJOR — require a `MAJOR` bump and a new binding URI, except that while this binding is at `draft` status a breaking change **MAY** be released as a `MINOR` increment ([SPEC §5.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#52-compatibility-rules)), as `0.3` is.

### 9.1 Changes from 0.2

Not additive at the TSP layer, so a new version rather than an in-place note; a `MINOR` increment because the binding is at `draft` status ([SPEC §5.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#52-compatibility-rules)). The envelope type, the envelope object and every Trust Tasks rule are unchanged. What changes is which TSP messages conform: a `0.2` implementation on the TSP Rev 2 draft that `0.2` cited and a `0.3` implementation cannot exchange a single message, in either direction. A `0.2` implementation that already speaks TSP 1.0 conforms to `0.3` once it refuses TSP drafts as [§2.2](#22-tsp-version) requires.

* TSP 1.0 is required ([§2.2](#22-tsp-version)). `0.2` cited the TSP Implementers Draft (Rev 2) and named no TSP version normatively.
* A *consumer* refuses a TSP MAJOR other than `1`, before using any key and without answering, and never refuses on the TSP MINOR ([§2.2](#22-tsp-version), [§4](#4-error-mapping)).
* The *transport-authenticated sender* is the `VID_sndr` TSP 1.0 checks against the sealed payload ([§2.2](#22-tsp-version)); the security profile ([§7](#7-transport-security-profile)) attributes sender authentication to the TSP signature, since TSP 1.0's HPKE-Base mode does not authenticate the sender itself.
* TSP section references ([§5](#5-routed-and-nested-carriage), [§6](#6-responses-and-error-delivery), [§10](#10-references)) point at TSP 1.0.

### 9.2 Changes from 0.1 (in 0.2)

Additive, and a `MINOR` increment accordingly. The envelope type is unchanged, so a `0.1` consumer recognises a `0.2` producer's messages and vice versa.

* The envelope gains an optional `deliveryId` member, by which a producer requests an end-to-end delivery receipt ([§2.1](#21-delivery-receipts), [`messaging/receipt`](../../../specs/messaging/receipt/0.1/spec.md)). A consumer ignores envelope members it does not know.
* The `received` row of the lifecycle mapping ([§4.1](#41-lifecycle-mapping)) names the receipt as its evidence where one was requested.
* Targets framework `0.6.0`.

## 10. References

- [Trust Spanning Protocol (TSP) Specification](https://trustoverip.github.io/tswg-tsp-specification/) — ToIP, version 1.0 ([tswg-tsp-specification@ccda6b3](https://github.com/trustoverip/tswg-tsp-specification/tree/ccda6b33159d860f05c328eb436dabe83548b705/spec)): §3 (messages: envelope, payload, receiver procedure), §4 (nested messages), §5 (routed messages), §7 (relationship forming), §8 (cryptographic algorithms), §9 (serialization and the `YTSP-###` version code).
- [`tsp/0.2`](../0.2/spec.md) — the version this one supersedes ([§9.1](#91-changes-from-02)).
- [`messaging/receipt`](../../../specs/messaging/receipt/0.1/spec.md) — the delivery receipt a `deliveryId` requests.
- [DIDComm v2.1 transport binding](../../didcomm/0.1/spec.md) and [HTTPS transport binding](../../https/0.1/spec.md) — sibling bindings in this framework.
- [Trust Tasks framework specification](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md), §4.7.1, §4.8, §4.8.1, §7.2, §8, §9.
