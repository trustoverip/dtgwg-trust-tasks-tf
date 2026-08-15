---
slug: didcomm-v1
version: "0.2"
title: DIDComm v1 transport binding
summary: Carries Trust Task documents to Aries-lineage agents inside DIDComm v1 authcrypt envelopes, as an attachment on a dedicated message type; the connection's theirDid maps to the framework's transport-authenticated party identity.
status: draft
targetFrameworkVersion: "0.4"
bindingURI: https://trusttasks.org/binding/didcomm-v1/0.2
envelopeType: https://trusttasks.org/binding/didcomm-v1/0.2/trust-task/1.0/task
authors:
  - Glenn Gore (https://github.com/stormer78)
  - Alberto L (https://github.com/albertoleon7794)
---

## Abstract

This binding specifies how *Trust Task documents* are exchanged over **DIDComm v1**. The framework's other DIDComm binding targets v2.1; Credo — and therefore essentially every Aries-lineage wallet — speaks v1 and only v1, so without this those stacks cannot carry a Trust Task at all.

A producer places the document in an `~attach` decorator on a message of this binding's own type and packs it with v1 *authcrypt*. On unpack the connection's `theirDid` becomes the framework's *transport-authenticated sender* for the purposes of [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity) precedence.

## Status of This Document

`0.2` **draft**. It settles the carriage question `0.1` left open: the document now rides a **dedicated message type** rather than an Aries `basic-message`. That change was not made on taste — it was made because the DTG Core Credentials task force built the identical exchange both ways against Credo 0.6.3 and measured it, and the measurement retired the argument `0.1` chose `basic-message` for. See [§2.1](#21-why-a-dedicated-message-type). Discussion on [issue #173](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues/173).

`0.1` remains published. A `0.2` *consumer* accepts both carriages; a `0.2` *producer* emits only this one. See [§7.1](#71-changes-from-01).

Targets **framework `0.4`**.

## 1. Binding URI

| Resource | URI |
|---|---|
| Binding identifier | `https://trusttasks.org/binding/didcomm-v1/0.2` |
| Message `@type` | `https://trusttasks.org/binding/didcomm-v1/0.2/trust-task/1.0/task` |
| Attachment `@id` | `trust-task` |

The *binding identifier* is the stable URI for this binding specification ([SPEC §9.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#93-binding-namespace)). Neither it nor the message `@type` is a *Type URI*, and neither **MUST** appear in a *Trust Task document*'s `type` member.

The message `@type` is a [RFC 0020](https://github.com/hyperledger/aries-rfcs/tree/main/concepts/0020-message-types) message type URI, and decomposes the way Aries tooling expects:

| RFC 0020 part | Value |
|---|---|
| Document URI | `https://trusttasks.org/binding/didcomm-v1/0.2` |
| Protocol name | `trust-task` |
| Protocol version | `1.0` |
| Message type name | `task` |

Two version numbers appear, and they answer to different audiences. The **document URI** carries the binding's own `MAJOR.MINOR` because [SPEC §9.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#93-binding-namespace) roots a binding's internal vocabulary under its versioned URI. The **protocol version** is the one Aries negotiates on, through [RFC 0031 discover-features](https://github.com/hyperledger/aries-rfcs/tree/main/features/0031-discover-features). The value above is minted at `0.2` and **pinned**: a later `MINOR` of this binding keeps it, exactly as `binding/didcomm/0.2` keeps `binding/didcomm/0.1/envelope`, so that two binding minors remain mutually intelligible on the wire.

Unlike `basic-message`, this type has **one** form. `0.1` had to treat `did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/basicmessage/1.0/message` and `https://didcomm.org/basicmessage/1.0/message` as equivalent, because both name the same Aries protocol and Credo emits the latter by default — a rule that silently dropped conforming peers wherever an implementation compared `@type` by string equality. Minting our own type removes that hazard rather than restating it: there is no second spelling to normalise.

The compatibility rule of [§2.3](#23-accepting-01-messages) is the one place a `0.2` consumer still meets both `basic-message` spellings.

## 2. Document carriage

DIDComm v2.1 has an obvious home for the document: the message `body`. **v1 has none** — the payload is flattened at top level beside `@id`, `@type` and the decorators — so a binding must choose a slot and say so normatively, or two implementations will choose differently.

This binding carries the document in an **`~attach` decorator** ([RFC 0017](https://github.com/hyperledger/aries-rfcs/tree/main/concepts/0017-attachments)), inline as `data.json`, under the reserved attachment `@id` `trust-task`, on a message of this binding's own type:

```json
{
  "@id": "8f1c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "@type": "https://trusttasks.org/binding/didcomm-v1/0.2/trust-task/1.0/task",
  "~thread": { "thid": "4a0e2b77-88c1-4d55-9f2a-6c3d1e5b7a92" },
  "comment": "Trust Task: https://trusttasks.org/spec/acl/grant/0.1",
  "~attach": [{
    "@id": "trust-task",
    "mime-type": "application/json",
    "data": { "json": { "id": "…", "type": "…", "payload": {} } }
  }]
}
```

A *producer* **MUST**:

1. Set `@type` to `https://trusttasks.org/binding/didcomm-v1/0.2/trust-task/1.0/task`.
2. Include exactly one `~attach` entry whose `@id` is `trust-task`, whose `mime-type` is `application/json`, and whose `data.json` is the *Trust Task document* as a JSON object.
3. Pack the message with **authcrypt** (see [§3](#3-identity-mapping)).

A *producer* **SHOULD** set `comment` to a short human-readable summary naming the document's Type URI, following the Aries convention for a human-readable line on a structured message. It is advisory: a *consumer* **MUST NOT** parse it, derive anything from it, or reject a message that omits it. A *producer* **MUST NOT** place the document in `comment`.

A message **MAY** carry other `~attach` entries. A *consumer* **MUST** select by `@id` rather than position, and **MUST** reject a message with no `trust-task` attachment.

Implementations differ on which decorators they add — return-route, timing, localization, acknowledgement requests — and none of them carry binding semantics. A *consumer* **MUST** ignore decorators it does not recognize, and **MUST NOT** require any top-level member beyond `@id`, `@type` and the `trust-task` attachment.

### 2.1 Why a dedicated message type

`0.1` carried the document on an Aries `basic-message`, and flagged the choice as open. It is now settled the other way, on measurement rather than argument.

The case for `basic-message` was **reach**: unmodified wallets already surface it, so a Trust Task would arrive somewhere visible even on a peer that knew nothing about this binding. The DTG Core Credentials task force built the identical exchange both ways against Credo 0.6.3 and found that argument does not survive contact:

- **A dedicated `@type` is *less* code**, not more — roughly 25 lines in Credo — once `basic-message`'s content-only high-level API and its attachment-dropping message store are priced in. `0.1`'s own §2.2 documented both of those as costs; they are the reason.
- **It crosses a mediator identically.** v1 mediator forwarding is type-agnostic — the type is inside the envelope — so nothing about routing favours an existing protocol.
- **It leaves the chat store alone.** `basic-message` writes every Trust Task into a wallet's conversation history, which is a real user-facing surface, not an implementation detail.
- **A consumer needs a handler either way.** No option lets an unmodified agent *act* on a Trust Task, so reach only ever bought visibility, not function.

What `basic-message` genuinely retained is **visible degradation in a mixed fleet**: an unaware wallet shows a harmless "Trust Task: …" line where a dedicated type is dropped silently. That is a real loss, and it is accepted here because [SPEC §11](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#11-discovery-and-capability-negotiation) capability negotiation — and RFC 0031 on the v1 side — is what gates a peer speaking Trust Tasks at all. A party that negotiated the capability and then dropped the message has a fault to surface; a party that never negotiated it should not be sent one.

Evidence: [`ref-06v1d-carrier`](https://github.com/berkmancenter/keyring-wallet/tree/doc/tsp-plan/tsp-reference/ref-06v1d-carrier) (both carriers built and compared), with the `~attach` carriage itself proven agent-to-agent in [`ref-06v1`](https://github.com/berkmancenter/keyring-wallet/tree/doc/tsp-plan/tsp-reference/ref-06v1-didcomm-v1-binding) and through a production Aries mediator in [`ref-06v1b`](https://github.com/berkmancenter/keyring-wallet/tree/doc/tsp-plan/tsp-reference/ref-06v1b-mediated).

The attachment itself is unchanged from `0.1`, and for the reasons `0.1` gave: `~attach` is the idiomatic Aries home for a structured payload, it keeps the document as JSON rather than a string inside a string, and a fixed attachment id means a consumer finds the document without scanning. Those arguments were about the *slot*; only the *carrier* moved.

### 2.2 Implementation notes (Aries frameworks)

Recorded from the Credo implementations (Credo 0.6.3 — the framework current Aries-lineage wallets ship), exercised agent-to-agent and through a production Aries mediator.

- **Register a message class and a handler.** A dedicated type is the ordinary Aries extension path: declare the message with its `@type`, register a handler, and dispatch through the message sender. This replaces `0.1`'s note that a producer had to reach past `basicMessages.sendMessage(connectionId, content)` because the chat API takes a display string and no attachment.
- **The transport's message store is not the document's store.** `0.1` recorded that Credo's persisted basic-message record keeps `content` only, so the attachment was reachable solely on the in-flight event. A dedicated type removes the specific trap, but the rule stands on its own: a *consumer* **MUST** obtain the document from the received message, or persist the document itself, rather than relying on the transport's storage of it.
- **Negotiate before sending.** Because an unaware peer drops an unknown type silently, a *producer* **SHOULD** establish that the peer speaks this binding — RFC 0031 discover-features, or the framework's own [§11](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#11-discovery-and-capability-negotiation) discovery — rather than inferring it from a connection existing.

### 2.3 Accepting `0.1` messages

`0.1` producers exist — the binding is published and implemented — so a `0.2` *consumer* **MUST** also accept the `0.1` carriage: a message whose `@type` is either `basic-message` document URI (`did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/basicmessage/1.0/message` or `https://didcomm.org/basicmessage/1.0/message`, which **MUST** be treated as equivalent) and which carries a `trust-task` attachment. Everything downstream of extraction — identity mapping, thread correlation, the §7.2 pipeline — is identical, because only the carrier changed.

A *consumer* **SHOULD** surface such a message as using a superseded carriage, so an operator can see which peers have not migrated. A *consumer* **MUST NOT** treat a `basic-message` with **no** `trust-task` attachment as a Trust Task: that is an ordinary chat message and belongs to whatever handles chat.

A *producer* **MUST NOT** emit the `0.1` carriage under this version. Receivers move first and senders follow, which is [SPEC §5.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#54-migrating-between-versions)'s expand-then-contract sequence; the contraction — dropping `0.1` acceptance — is a breaking change and belongs to a future `MAJOR`.

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
| `@type` is neither this binding's message type nor a `basic-message` URI accepted under [§2.3](#23-accepting-01-messages) | Not a message of this binding. Reject at the DIDComm layer; do not enter the pipeline, and do not reply — the sender was not addressing this binding. |
| `@type` is a `basic-message` URI carrying no `trust-task` attachment | Not a Trust Task at all ([§2.3](#23-accepting-01-messages)). Hand to whatever handles chat; **MUST NOT** be reported as a Trust Task error. |
| No `~attach` entry with `@id` `trust-task`, on this binding's own message type | `malformedRequest`. |
| The attachment does not deserialise as a *Trust Task document* | `malformedRequest`. |
| `thid`/`threadId` or `pthid`/`parentThreadId` both present and disagreeing | `malformedRequest` (see [§3.1](#31-thread-correlation)). Not `identityMismatch`. |

Error responses generated by the framework pipeline **SHOULD** be returned as a `trust-task-error` document carried the same way, over the same connection, with `~thread.thid` continuing the exchange where [§3.1](#31-thread-correlation)'s representability rule permits it. Where it does not, the field is omitted and the reply is correlated by its in-band `threadId` — and by `inResponseTo` ([SPEC §8.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#82-error-payload)), which names the reported-on document directly and does not depend on the transport carrying a thread at all.

## 5. Proof interaction

A document delivered over a v1 **authcrypt** envelope between two connected parties enjoys integrity and sender authentication from the transport, so per [§4.7.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#471-when-to-include-a-proof) the in-band `proof` **MAY** be omitted — subject to the attribution caveat in §3: that guarantee holds only where the authenticating verkey is bound to a known DID.

A *Trust Task specification* declaring `proof` **REQUIRED** ([§7.3 item 8](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#73-specification-requirements)) overrides this allowance, per variant where it declares per variant. Such specifications produce documents intended to be replayed past the original transport hop, and a v1 connection's authentication does not travel with the document.

## 6. Transport security profile

Required by [§9.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#91-what-a-transport-binding-specifies), and by
[§9.1.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#911-permitting-proof-to-be-omitted) because §5 permits `proof` to be
omitted. Each item §9.1.1 enumerates is addressed below; where v1 provides
nothing, that is stated rather than left out.

| Property | What DIDComm v1 provides |
|---|---|
| **Authenticated producer** | The authcrypt sender **verkey** — a bare base58 Ed25519 key, authenticated by the v1 unpack. Not a DID: see §3. |
| **Mapping to a VID** | Verkey → the connection's `theirDid`, resolved from **connection state this agent holds**, not from anything the wire carries. §3's three outcomes are normative, and only outcome 1 yields a transport-authenticated sender. |
| **Audience binding** | Cryptographic, not asserted: authcrypt seals the envelope to the recipient's key, so a party that cannot decrypt is not an audience. This is *transport* audience binding and does **not** satisfy [§4.8.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#482-audience-binding), which governs what a `proof` commits to and is unaffected by how the bytes travelled. |
| **Integrity across intermediaries** | The authcrypt AEAD covers the envelope end-to-end between the two connected agents. A mediator handles the outer `forward`; the inner envelope is opaque to it. |
| **Re-origination** | A mediator **cannot** modify the inner envelope undetected, nor originate one as the sender without the sender's secret key. It **can** drop, delay, reorder, and re-deliver. |
| **Freshness / replay** | **None.** v1 offers no anti-replay construct, and store-and-forward mediation admits arbitrary delay. See the note below — this is the item most likely to be assumed and is not provided. |
| **Key and credential status** | No status check exists at this layer. The verkey→DID binding is local connection state; a rotation the agent has not yet processed leaves a superseded verkey still authenticating. There is no revocation signal to consult. |
| **Where the guarantee stops** | At the connection. Outcomes 2 and 3 of §3 (verkey bound to no known DID; anoncrypt or plaintext) yield no authenticated sender at all, and the guarantee does not travel with the document once it leaves the connection. |

**The transport provides no replay protection, and that is load-bearing.**
Because v1 gives no freshness guarantee and a mediator may re-deliver, a
*consumer* over this binding carries the whole burden of
[§7.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#72-consumer-requirements) item 11 — duplicate-execution protection
keyed on the document `id` — for any *consequential Trust Task*. Nothing in this
transport will stop the same document arriving twice, and a *consumer* that
treats mediated delivery as at-most-once is wrong on both a hostile replay and
an ordinary mediator retry.

**What omitting `proof` costs.** The §5 allowance holds only inside the
connection and only for outcome 1. A document delivered without an in-band
`proof` carries no evidence of its producer once it leaves that connection: the
authentication was a property of the envelope, and the envelope is discarded on
unwrap. A *consumer* that retains such a document, forwards it, or offers it to
a third party is offering bytes nobody signed. Where a document is intended to
be retained or relied upon beyond the receiving agent,
[§4.7.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#471-when-to-include-a-proof) requires a `proof` regardless of this
binding's allowance, and a *Trust Task specification* declaring `proof`
**REQUIRED** settles it (§5).

## 7. Versioning

This binding follows the framework's `MAJOR.MINOR` versioning ([§5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#5-versioning)). A `MINOR` revision **MUST** remain backwards-compatible with *documents produced under* an earlier `MINOR` of this binding: the attachment id and the extraction rules are preserved, earlier carriages remain acceptable to a consumer, and only additive conventions, identity-mapping refinements, or stricter error mappings may be introduced. Dropping acceptance of an earlier carriage requires a `MAJOR` bump and a new binding URI.

### 7.1 Changes from `0.1`

**The carriage moved from `basic-message` to a dedicated message type** ([§2](#2-document-carriage)), on the measurement recorded in [§2.1](#21-why-a-dedicated-message-type). The attachment is unchanged: same `~attach` decorator, same reserved `@id` `trust-task`, same `data.json`. `content` is gone with `basic-message`; an optional advisory `comment` replaces it.

Also: the binding targets framework `0.4` rather than `0.3`, and `0.1`'s `@type`-equivalence rule survives only inside [§2.3](#23-accepting-01-messages), because this binding's own type has one spelling.

**On calling this a `MINOR`.** `0.1`'s §6 said a carriage change is breaking and requires a `MAJOR`, and in the general case it is right. Three things make `0.2` the honest number here rather than a convenience:

1. **A `0.2` consumer accepts everything a `0.1` producer emits** ([§2.3](#23-accepting-01-messages)). That is the compatibility direction [SPEC §5.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#52-compatibility-rules) makes normative — an implementation at `M.N` **MUST** accept documents conforming to any `M.K` where `K ≤ N` — and it is satisfied in full.
2. **`0.1` itself anticipated this.** Its §6 closed by saying that, with §2 flagged open, a carriage change before adoption should be handled as a correction to the draft rather than a new major. A new `MINOR` is the more conservative reading of that: it gives implementers a distinguishable URI instead of changing `0.1` underneath anyone who has already built against it — and `trust-tasks-didcomm-v1` 0.1.0 is published, so somebody could have.
3. **[SPEC §5.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#52-compatibility-rules) permits it explicitly** while an artifact is `draft`: a breaking change **MAY** be released as a `MINOR` increment.

What is **not** preserved, and should be stated plainly rather than implied: **forward compatibility**. A `0.1` consumer does not recognise a `0.2` producer's message and will drop it — silently, since an unknown `@type` is not an error condition it can report. Forward-minor compatibility is a `SHOULD` in §5.2, not a `MUST`, and it cannot be met across a carriage change by any versioning scheme. Deployments migrate receivers before senders ([§5.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#54-migrating-between-versions)), and negotiate capability before sending ([§2.2](#22-implementation-notes-aries-frameworks)).

## 8. References

- [Aries RFC 0020: Message Types](https://github.com/hyperledger/aries-rfcs/tree/main/concepts/0020-message-types) — the message type URI grammar this binding's `@type` follows
- [Aries RFC 0017: Attachments](https://github.com/hyperledger/aries-rfcs/tree/main/concepts/0017-attachments)
- [Aries RFC 0008: Message ID and Threading](https://github.com/hyperledger/aries-rfcs/tree/main/concepts/0008-message-id-and-threading)
- [Aries RFC 0019: Encryption Envelope](https://github.com/hyperledger/aries-rfcs/tree/main/features/0019-encryption-envelope)
- [Aries RFC 0031: Discover Features](https://github.com/hyperledger/aries-rfcs/tree/main/features/0031-discover-features) — capability negotiation on the v1 side
- [Aries RFC 0095: Basic Message](https://github.com/hyperledger/aries-rfcs/tree/main/features/0095-basic-message) — the `0.1` carriage, still accepted per [§2.3](#23-accepting-01-messages)
- [Trust Tasks framework specification](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md), §4.8.1, §4.9, §4.9.2, §5.2, §5.4, §7.2, §8, §9, §11
- [`bindings/didcomm-v1/0.1`](../0.1/spec.md) — the superseded carriage
- [`trust-tasks-didcomm-v1`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/tree/main/trust-tasks-didcomm-v1) — the reference implementation, which implements the `0.1` carriage and needs updating for this version
