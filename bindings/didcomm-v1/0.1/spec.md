---
slug: didcomm-v1
version: "0.1"
title: DIDComm v1 transport binding
summary: Carries Trust Task documents to Aries-lineage agents inside DIDComm v1 authcrypt envelopes, as an attachment on a basic-message; the connection's theirDid maps to the framework's transport-authenticated party identity.
status: draft
targetFrameworkVersion: "0.3"
bindingURI: https://trusttasks.org/binding/didcomm-v1/0.1
envelopeType: "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/basicmessage/1.0/message"
authors:
  - Glenn Gore (https://github.com/stormer78)
  - Alberto L (https://github.com/albertoleon7794)
---

## Abstract

This binding specifies how *Trust Task documents* are exchanged over **DIDComm v1**. The framework's other DIDComm binding targets v2.1; Credo — and therefore essentially every Aries-lineage wallet — speaks v1 and only v1, so without this those stacks cannot carry a Trust Task at all.

A producer places the document in an `~attach` decorator on an Aries `basic-message` and packs the message with v1 *authcrypt*. On unpack the connection's `theirDid` becomes the framework's *transport-authenticated sender* for the purposes of [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity) precedence.

## Status of This Document

`0.1` **draft**, and explicitly a starting point rather than a settled specification. It was written from a working reference implementation ([`trust-tasks-didcomm-v1`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/tree/main/trust-tasks-didcomm-v1)) so that the open questions are concrete, and it is offered to the DTG Core Credentials task force — who proposed the binding and have the Aries deployment experience — to take over and amend. The choices flagged **⚠ open** below are the ones most likely to change; nothing depends on them yet. Discussion on [issue #173](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues/173).

Targets **framework `0.3`**.

## 1. Binding URI

| Resource | URI |
|---|---|
| Binding identifier | `https://trusttasks.org/binding/didcomm-v1/0.1` |
| Message `@type` | `did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/basicmessage/1.0/message` |
| Attachment `@id` | `trust-task` |

The *binding identifier* is the stable URI for this binding specification ([SPEC §9.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#93-binding-namespace)). It is not a *Type URI* and **MUST NOT** appear in a *Trust Task document*'s `type` member.

Unlike the v2.1 binding, this one defines **no envelope type of its own**: it rides an existing Aries protocol. v1 message types have two interchangeable document URIs — the `did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/…` form above and `https://didcomm.org/basicmessage/1.0/message` — and Credo emits the latter by default. A *consumer* **MUST** treat them as equivalent and **MUST NOT** compare `@type` by string equality, which silently drops conforming peers.

## 2. Document carriage

⚠ **Open.** This is the decision most in need of review.

DIDComm v2.1 has an obvious home for the document: the message `body`. **v1 has none** — the payload is flattened at top level beside `@id`, `@type` and the decorators — and [RFC 0095](https://github.com/hyperledger/aries-rfcs/tree/main/features/0095-basic-message) types `basic-message`'s `content` as a *string* meant for human display, which Credo renders as chat text. A binding must therefore choose a slot and say so normatively, or two implementations will choose differently.

This binding carries the document in an **`~attach` decorator** ([RFC 0017](https://github.com/hyperledger/aries-rfcs/tree/main/concepts/0017-attachments)), inline as `data.json`, under the reserved attachment `@id` `trust-task`:

```json
{
  "@id": "8f1c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/basicmessage/1.0/message",
  "~thread": { "thid": "4a0e2b77-88c1-4d55-9f2a-6c3d1e5b7a92" },
  "content": "Trust Task: https://trusttasks.org/spec/acl/grant/0.1",
  "~attach": [{
    "@id": "trust-task",
    "mime-type": "application/json",
    "data": { "json": { "id": "…", "type": "…", "payload": {} } }
  }]
}
```

A *producer* **MUST**:

1. Set `@type` to one of the two `basic-message` URIs.
2. Include exactly one `~attach` entry whose `@id` is `trust-task`, whose `mime-type` is `application/json`, and whose `data.json` is the *Trust Task document* as a JSON object.
3. Pack the message with **authcrypt** (see [§3](#3-identity-mapping)).

A *producer* **SHOULD** set `content` to a short human-readable summary naming the document's Type URI. RFC 0095 says `content` is for display and a wallet will show it; a summary means a human reading a chat log sees something meaningful, and it duplicates nothing a *consumer* parses. A *producer* **MUST NOT** place the document in `content`.

A message **MAY** carry other `~attach` entries. A *consumer* **MUST** select by `@id` rather than position, and **MUST** reject a message with no `trust-task` attachment.

Implementations differ on which decorators they add to a `basic-message` — return-route, timing, localization, acknowledgement requests — and none of them carry binding semantics. A *consumer* **MUST** ignore decorators it does not recognize.

`sent_time` is **not** a decorator: RFC 0095 defines it as a top-level member of the message, alongside `content`. Stacks differ on whether they populate it — Credo always does, the reference implementation does not. A *producer* **SHOULD** set it; a *consumer* **MUST NOT** require it.

### 2.1 Why an attachment

Recorded because the alternatives are reasonable and a reviewer should see them weighed.

**JSON in `content`** reaches every v1 agent with no special handling, which is the strongest argument for it. The cost is that a human-facing wallet renders each Trust Task as a wall of double-encoded JSON — the primary experience for an Aries user rather than an edge case — and the payload becomes a string inside a string, which nothing can schema-check at the transport layer.

**A sibling top-level member** keeps `content` readable and nothing rejects it, since v1 parsers ignore unknown members. But it invents a slot no Aries reader looks in, so it gets none of the ecosystem's existing tooling while still requiring both ends to agree.

**`~attach`** is the idiomatic Aries home for a structured payload. It keeps the document as JSON, leaves `content` free for the summary, and uses tooling that already exists.

⚠ **Open, and worth revisiting:** whether `basic-message` is the right carrier at all. Its assumed advantage is reach — unmodified wallets already surface it — but a Trust Task *consumer* needs a handler under any of the three options above, and v1 mediator forwarding is type-agnostic, so a dedicated `@type` would route just as well and would mirror `binding/didcomm/0.1` more closely. The trade is ecosystem familiarity against not pretending a Trust Task is a chat message.

### 2.2 Implementation notes (Aries frameworks)

Recorded from the first Credo implementation (Credo 0.6.3 — the framework current Aries-lineage wallets ship), exercised agent-to-agent and through a production Aries mediator; the carriage above worked unchanged in both topologies.

- **Producing takes the message layer, not the chat API.** The high-level basic-message APIs (Credo: `basicMessages.sendMessage(connectionId, content)`) accept a display string only — there is no attachment parameter. A *producer* constructs the message at the framework's message layer (in Credo: a `DidCommBasicMessage` with `appendedAttachments`, dispatched through the message sender). A Trust Task client is therefore a small module of its own, not a wrapper over the chat API.
- **The attachment may not survive transport storage.** Credo's persisted basic-message record keeps `content` only; the `~attach` decorator is reachable solely on the in-flight message event. A *consumer* **MUST** obtain the document from the received message (or persist the document itself) rather than relying on the transport's message store.

## 3. Identity mapping

The mapping into [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity) precedence:

| Framework concept | DIDComm v1 value |
|---|---|
| *Transport-authenticated sender* | The connection's `theirDid` — the DID bound to the verkey that authenticated the envelope — in bare-DID form. |
| *Transport-authenticated recipient* | The DID the unwrapping agent unpacked for. |
| In-band `issuer` (when set) | Compared against the transport-authenticated sender per §4.8.1; mismatch is `identityMismatch`. |
| In-band `recipient` (when set) | Compared against the transport-authenticated recipient; mismatch is `identityMismatch`. |

**Attribution takes a step more than it does in v2.1, and the difference is normative.** A v2.1 envelope authenticates a DID directly: the verified `sender_kid` reduces to one. **A v1 envelope contains no DID at all.** It authenticates a bare base58 Ed25519 *verkey*, and the verkey-to-DID binding is connection state the agent holds — not something the wire carries.

Three outcomes follow, and a *consumer* **MUST** distinguish them:

1. **Authcrypt, verkey bound to a known DID.** The only case that yields a transport-authenticated sender. Proceed to the §7.2 pipeline.
2. **Authcrypt, verkey bound to no known DID.** The envelope is cryptographically sound — someone holds the secret half of that key — but this agent cannot name them, and §4.8.1 needs a party identity rather than a key. The document **MUST NOT** enter the pipeline with a fabricated sender.
3. **Anoncrypt or plaintext.** No authenticated sender at all.

Cases 2 and 3 are both unattributable and are treated the same by [§4](#4-error-mapping), but a *consumer* **SHOULD** distinguish them in logs: the first is a missing connection record on this side, the second is a message that was never authenticated. Collapsing them hides an operational fault behind what looks like a hostile message.

Where the in-band member is absent, a *consumer* **MAY** treat the transport-derived value as if carried in-band — authcrypt provides authenticated identity end-to-end, so omitting `issuer`/`recipient` on a v1-only exchange is conformant per §4.8.1.

When applying the [§8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#81-the-trust-task-error-specification) routing rule under `identityMismatch`, a *consumer* **MUST** address the error response to the connection it actually authenticated and **MUST NOT** address it to the contested in-band `issuer`.

### 3.1 Thread correlation

Per [SPEC §9.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#91-what-a-transport-binding-specifies), a binding mapping its transport's correlation identifiers onto the framework's must state the mapping. v1 carries them in the `~thread` decorator ([RFC 0008](https://github.com/hyperledger/aries-rfcs/tree/main/concepts/0008-message-id-and-threading)):

| Framework member | `~thread` field |
|---|---|
| `threadId` ([§4.9](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#49-the-threadid-member)) | `thid` |
| `parentThreadId` ([§4.9.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#492-the-parentthreadid-member)) | `pthid` |

**Producers.** A *producer* **SHOULD** set `thid` from the document's `threadId`, and `pthid` from its `parentThreadId`. Where the document carries no `threadId`, the *producer* **SHOULD** set `thid` to the document's `id` — §4.9's own fallback, so the v1 thread and the Trust Task exchange are named by one value. Where there is no `parentThreadId`, `pthid` is omitted.

Populating the decorator *from* the members, rather than the reverse, is what makes the layers agree. A producer that lets v1 default its own threading produces a message whose `thid` is the DIDComm `@id`, which is a different identifier space entirely — the v1 `@id` is the transport's, unrelated to the document's `id`.

**Representability.** RFC 0008 shapes thread ids as `[-_./a-zA-Z0-9]{8,64}`, and major Aries stacks enforce it: Credo validates every `~thread` field against exactly that pattern (or a bare DID) and refuses to pack a message that fails. A framework id in URI form — `urn:uuid:…`, with its colons — therefore cannot ride the decorator at all: the send is rejected client-side before the envelope is built, on the very stacks this binding exists to reach. The colon is the common case rather than the only one: the shape also excludes any value shorter than eight characters, and the `+` and `=` of base64, so a producer that tests only the URN form will meet this again.

Accordingly:

- A *producer* **MUST NOT** emit a `~thread` field that does not satisfy RFC 0008's shape.
- Where a document's `threadId` or `parentThreadId` is not representable, the *producer* **MUST omit** that field — never truncate or rewrite it. A rewritten value would disagree with the in-band member, and this section's own comparison rule makes that `malformedRequest`.
- A *producer* **initiating** an exchange intended for this binding **SHOULD** mint a `threadId` that is itself RFC 0008-conformant. §4.9 lets an initiator omit `threadId`, mint a fresh value, or reuse the document's own `id`, so the choice is free — and initiation is the only point in an exchange where it is. A *responder* inherits `threadId` from the originating document under §4.9's convention and cannot repair a value another party minted, possibly on another transport.
- The same applies to the document's `id`, which §4.9 makes the fallback `thid`, and to `parentThreadId`, which a producer inherits from the containing exchange on the same terms. A bare UUID satisfies both the framework's §4.3 uniqueness obligation and this transport's shape; §4.3 places no constraint on the form of an `id` beyond uniqueness, so the `urn:uuid:` URIs the framework's examples use are a convention rather than a requirement.

**What an omission costs.** Omitting the field does not leave the message unthreaded. RFC 0008 **defaults** an absent `thid` to the message `@id`, so each message of the exchange falls into a v1 thread of its own, named by a value the transport minted. Nothing the *framework* relies on is lost — the in-band members are authoritative, and `threadId` carries no normative validation semantics (§4.9) — but transport-level correlation is unavailable for that exchange, and tooling that groups by `~thread` will not group it. A *consumer* **MUST NOT** infer thread continuation from a defaulted `thid`, for the reason the comparison rule below gives.

**Consumers.** Where **both** a `~thread` field and its framework member are explicitly present, they **MUST** be equal, and a *consumer* **MUST** reject a mismatch with `malformedRequest` — **not** `identityMismatch`, which is reserved for a contested party identity and carries §8.1's suppression rules. A thread disagreement contests nobody's identity.

Where only one side is present, no comparison is made. This scoping is essential rather than lenient: RFC 0008 **defaults** an absent `thid` to the message `@id`, and §4.9 falls back to the document's `id`. A *consumer* **MUST** compare the `thid` *as it appeared on the wire* and **MUST NOT** compare the defaulted value — the default is a value the transport synthesised, not one the sender asserted, and comparing it manufactures disagreements out of conforming messages.

A *consumer* **MUST NOT** populate an absent `threadId` or `parentThreadId` from the decorator. Unlike `issuer` and `recipient`, where an authenticated transport genuinely carries the value the framework needs, a thread field is the transport's own correlation; asserting it as the document's fabricates a member the *producer* chose not to send.

## 4. Error mapping

| Transport-level condition | Framework disposition |
|---|---|
| Anoncrypt or plaintext (no authenticated sender) | Transport-level reject; **MUST NOT** enter the framework pipeline. Where the *consumer* has some other means of replying — a v1 message arrives over a connection even when the envelope authenticates nobody — `proofRequired` is the applicable code: an in-band `proof` is the only thing that could still attribute the document. A *consumer* with no such means **SHOULD NOT** reply at all. |
| Authcrypt, but the verkey is bound to no known DID | As above, and for the same reason: cryptographically sound but unattributable, so `proofRequired`. |
| Decryption or signature verification failure | DIDComm-level error; **MUST NOT** enter the pipeline. |
| `@type` is neither `basic-message` URI | Reject at the DIDComm layer; do not enter the pipeline. |
| No `~attach` entry with `@id` `trust-task` | `malformedRequest`. |
| The attachment does not deserialise as a *Trust Task document* | `malformedRequest`. |
| `thid`/`threadId` or `pthid`/`parentThreadId` both present and disagreeing | `malformedRequest` (see [§3.1](#31-thread-correlation)). Not `identityMismatch`. |

Error responses generated by the framework pipeline **SHOULD** be returned as a `trust-task-error` document carried the same way, over the same connection, with `~thread.thid` continuing the exchange where [§3.1](#31-thread-correlation)'s representability rule permits it. Where it does not, the field is omitted and the reply is correlated by its in-band `threadId` — and by `inResponseTo` ([SPEC §8.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#82-error-payload)), which names the reported-on document directly and does not depend on the transport carrying a thread at all.

## 5. Proof interaction

A document delivered over a v1 **authcrypt** envelope between two connected parties enjoys integrity and sender authentication from the transport, so per [§4.7.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#471-when-to-include-a-proof) the in-band `proof` **MAY** be omitted — subject to the attribution caveat in §3: that guarantee holds only where the authenticating verkey is bound to a known DID.

A *Trust Task specification* declaring `proof` **REQUIRED** ([§7.3 item 8](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#73-specification-requirements)) overrides this allowance, per variant where it declares per variant. Such specifications produce documents intended to be replayed past the original transport hop, and a v1 connection's authentication does not travel with the document.

## 6. Versioning

This binding follows the framework's `MAJOR.MINOR` versioning ([§5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#5-versioning)). A `MINOR` revision **MUST** remain backwards-compatible with consumers implementing this version: the message type, the attachment id and the carriage rules are preserved, and only additive conventions, identity-mapping refinements, or stricter error mappings may be introduced. Changing the carriage — a different attachment id, a move to `content` or to a dedicated `@type` — is breaking and requires a `MAJOR` bump and a new binding URI.

Given §2 is flagged open, a carriage change before adoption is expected to be handled as a correction to this draft rather than a new major version. `draft` status means the specification may change without notice ([§5.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#53-maturity-levels)).

## 7. References

- [Aries RFC 0095: Basic Message](https://github.com/hyperledger/aries-rfcs/tree/main/features/0095-basic-message)
- [Aries RFC 0017: Attachments](https://github.com/hyperledger/aries-rfcs/tree/main/concepts/0017-attachments)
- [Aries RFC 0008: Message ID and Threading](https://github.com/hyperledger/aries-rfcs/tree/main/concepts/0008-message-id-and-threading)
- [Aries RFC 0019: Encryption Envelope](https://github.com/hyperledger/aries-rfcs/tree/main/features/0019-encryption-envelope)
- [Trust Tasks framework specification](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md), §4.8.1, §4.9, §4.9.2, §7.2, §8, §9
- [`trust-tasks-didcomm-v1`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/tree/main/trust-tasks-didcomm-v1) — the reference implementation this was written from
