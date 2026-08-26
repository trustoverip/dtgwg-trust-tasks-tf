---
slug: chat/message
version: "0.1"
title: Chat — Message
summary: A conversational message between an AI agent and a messaging-platform bridge, signed by its author and hash-linked to the previous message to form a verifiable per-conversation chain for audit and dispute resolution.
status: draft
targetFrameworkVersion: "0.2"
category: chat
keywords:
  - chat
  - message
  - bridge
  - agent
  - conversation
  - mention
  - non-repudiation
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Message author
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Counterparty
    requirement: REQUIRED
    member: recipient
    identifierScope: pairwise
proofRequirement:
  requirement: REQUIRED
  rationale: Chat messages form an evidentiary chain that must be verifiable after the transport has closed. For audit and dispute resolution a third party must verify each message's author and its position in the conversation independently of the (ephemeral, authcrypted) DIDComm session that carried it — transport authentication alone is not portable.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A chat message is fire-and-forget with no response to correlate it by, so the timestamp on the document is the only thing that stops a captured message being re-delivered later and read as newly said.
sideEffects:
  level: mutating
  rationale: "Appends a signed, hash-linked message to the conversation chain; persisted history."
exposure:
  discloses: none
  ingests: personal
  actsAsSubject: false
  rationale: >-
    The document is the message. `text` is an unconstrained plain-text body — the
    actual words of a human conversation — and it arrives alongside
    `mentions[].displayName`, a real-world name the source platform supplied, and
    `attachments[].filename`, which routinely describes what a file contains.
    `isGroup`, `isMention`, `sentAt` and `replyToId` add signed conversational
    context. Being fire-and-forget the task discloses nothing back, but the
    counterparty receives all of it and appends it to a chain built to be kept.
retention:
  class: durable
  rationale: >-
    The chain is the deliverable. Each message is hash-linked to its predecessor
    so an auditor or dispute arbiter can verify authorship and ordering long after
    the transport closed, and evidence discarded is evidence that never existed —
    a consumer that drops history loses the ability to answer the questions the
    `proof` and `prev` members are there to answer. The corollary is deliberate
    and stated in the body: the same construction is what makes deleting an
    individual message costly.
errorCodes:
  - code: chat/message:unknownConversation
    meaning: No conversation matches `conversationId` at this consumer.
    retryable: false
  - code: chat/message:brokenChain
    meaning: '`prev` does not reference the consumer''s last-seen message for this conversation — a gap, reorder, or fork.'
    retryable: false
  - code: chat/message:unsupportedContent
    meaning: The message carries content (e.g. an attachment kind) the consumer or its platform cannot represent.
    retryable: false
related: []
---

## Abstract

**Chat — Message** is a single conversational message exchanged between an **AI
agent** and a **messaging-platform bridge** (a component that terminates an
external messaging platform — Signal, WhatsApp, Telegram, email — and relays it
to/from the agent). It is the message form for a *governed* agent-messaging data
plane where the agent never holds platform credentials and never reaches a
platform directly.

Each message is a Trust Task document **signed by its author** (the document
`proof`) and **hash-linked to the previous message** in the conversation
(`prev`). Together these make a conversation a verifiable, ordered chain: a
third party — an auditor or a dispute arbiter — can confirm *who* authored each
message and *where* it sits in the conversation, **after** the transport that
carried it has closed. Transport authentication (e.g. DIDComm authcrypt) proves
the sender only to the unpacking party, in the moment; it is not portable
evidence. The document proof is.

Conversations and contacts are referenced by **opaque, bridge-issued handles**
(`conversationId`), never raw platform addresses — the agent and any
intermediary see handles, not phone numbers or chat ids.

This task is **fire-and-forget**: it defines no success-response document.
Acknowledgement of receipt is a separate concern (a future `chat/delivery-receipt`
task) so that an acknowledgement is itself a signed, independently-verifiable
link in the chain rather than a transport-level ack.

## Conformance

A conforming **producer** (the message author) **MUST**:

1. Set `direction` to `outbound` when it is the agent sending toward the
   platform, or `inbound` when it is the bridge attesting a message it received
   from the platform and normalized.
2. Reference the conversation by its opaque `conversationId`; **MUST NOT** place
   a raw platform address anywhere in the payload.
3. On every message after the first in a conversation, populate `prev` with the
   `id` and content `digest` of the immediately preceding message it observed,
   so the chain is unbroken.
4. Sign the document with the author's key; because `proofRequirement` is
   `REQUIRED`, a proofless document is rejected with `proofRequired`.

A conforming **consumer** (the counterparty) **MUST**:

1. Resolve `conversationId`; unknown → `chat/message:unknownConversation`.
2. Verify the document `proof` and that its `verificationMethod` DID is the
   expected author for the `direction`.
3. Where it maintains chain state, verify `prev` references its last-seen
   message for the conversation; a mismatch → `chat/message:brokenChain`.
4. Reject content it cannot represent with `chat/message:unsupportedContent`.

## Authorization

*Stated in anticipation of [SPEC.md §7.3](/SPEC.md#73-specification-requirements)
item 15, which binds specifications targeting framework 0.4; this specification
targets 0.2, where the declaration is not yet required.*

The authorization evidence is **being the expected author for the
`conversationId` and `direction`**. The consumer resolves the conversation
(unknown → `chat/message:unknownConversation`) and checks the document proof's
`verificationMethod` DID against the party it expects on that side: the agent
for `outbound`, the bridge for `inbound`. A party that is not the expected
author for that direction on that conversation is not authorized to append to
it, however well its document verifies.

`proof` is REQUIRED here, and verifying it is what makes that comparison
possible — but per
[SPEC.md §7.2](/SPEC.md#72-consumer-requirements) item 10 it is not
itself the authorization. A validly signed message from a DID that is not this
conversation's expected author for its direction has established its author and
earned no entitlement.

The `prev` chain check is not an authorization input either. It establishes
that the conversation's history is unbroken, which is an integrity property; a
correctly chained message from an unexpected author is still unauthorized, and
a chain mismatch from the right author is reported separately as
`chat/message:brokenChain`.

## Payload

`conversationId` (REQUIRED) — opaque conversation handle. `direction` (REQUIRED)
— `inbound` | `outbound`. `sentAt` (REQUIRED) — RFC 3339 author timestamp.
`platform`, `text`, `mentions`, `attachments` (by reference), `replyToId`,
`isGroup`, `isMention`, and `prev` (the chain link: previous message `id` +
`digest`) are OPTIONAL — though `prev` is present on every message after the
first. See `payload.schema.json`.

`isGroup` and `isMention` are signed routing context. `isGroup` records whether
the conversation is a group/channel or a 1:1 DM; `isMention` records whether an
inbound message **addresses the agent** (an @-mention of the agent, or any DM),
so a group-aware consumer can decide a group message is for it without
agent-name heuristics. `isMention` is about the *agent as addressee* and is
distinct from `mentions`, which enumerates the participants referenced in the
body. Both are part of the signed record so the audit chain captures where a
message was sent, not just its text; both default to false when absent.

`mentions` carries the @-mentions in the body in a **platform-neutral** form.
Each entry references the mentioned party by an **opaque participant handle**
(`participant`, never a raw address — the same handle model as `conversationId`)
plus an optional `displayName` rendering hint. The body carries one `U+FFFC`
("object replacement character") sentinel per mention, and `mentions` is ordered
to match: the *Nth* `U+FFFC` in `text` binds to the *Nth* entry. `start`/`length`
are the source platform's native offsets and are advisory — the authoritative
binding is positional, because offset units differ across platforms. A producer
translating from a platform whose mentions are inline text (e.g. WhatsApp,
Slack, Matrix) replaces each mention span with a single `U+FFFC` sentinel and
records the corresponding entry.

## Examples

An **outbound** message from the agent to the bridge — the first in its
conversation, so it carries no `prev`:

```json
{
  "id": "urn:uuid:6f1c8b2a-0001-4a10-8a00-000000000001",
  "type": "https://trusttasks.org/spec/chat/message/0.1",
  "issuer": "did:key:z6MkAgentExampleAaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "recipient": "did:key:z6MkBridgeExampleBbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  "threadId": "urn:uuid:6f1c8b2a-conv-0000-0000-000000000000",
  "issuedAt": "2026-06-16T12:00:00Z",
  "payload": {
    "conversationId": "conv-9c2f",
    "direction": "outbound",
    "platform": "signal",
    "text": "Hi — confirming our meeting at 3pm.",
    "sentAt": "2026-06-16T12:00:00Z"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-06-16T12:00:00Z",
    "verificationMethod": "did:key:z6MkAgentExampleAaaaaaaaaaaaaaaaaaaaaaaaaaaa#z6MkAgentExampleAaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "proofPurpose": "assertionMethod",
    "proofValue": "z…(eddsa-jcs-2022 signature over the JCS-canonical document)"
  }
}
```

The **inbound** reply the bridge attests from the platform — chained to the
message above via `prev`:

```json
{
  "id": "urn:uuid:6f1c8b2a-0002-4a10-8a00-000000000002",
  "type": "https://trusttasks.org/spec/chat/message/0.1",
  "issuer": "did:key:z6MkBridgeExampleBbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  "recipient": "did:key:z6MkAgentExampleAaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "threadId": "urn:uuid:6f1c8b2a-conv-0000-0000-000000000000",
  "issuedAt": "2026-06-16T12:00:09Z",
  "payload": {
    "conversationId": "conv-9c2f",
    "direction": "inbound",
    "platform": "signal",
    "text": "Great, see you then!",
    "replyToId": "urn:uuid:6f1c8b2a-0001-4a10-8a00-000000000001",
    "prev": {
      "id": "urn:uuid:6f1c8b2a-0001-4a10-8a00-000000000001",
      "digest": "zQmaiQiEmnfNmSYMicJwuJcgiwRkRkALUPvVHyLWvtAbGyA"
    },
    "sentAt": "2026-06-16T12:00:09Z"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-06-16T12:00:09Z",
    "verificationMethod": "did:key:z6MkBridgeExampleBbbbbbbbbbbbbbbbbbbbbbbbbbbb#z6MkBridgeExampleBbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    "proofPurpose": "assertionMethod",
    "proofValue": "z…(eddsa-jcs-2022 signature over the JCS-canonical document)"
  }
}
```

An **inbound** group message that **@-mentions** a participant. The body carries
one `U+FFFC` sentinel where the mention sits, and the single `mentions` entry —
referenced by an opaque `participant` handle, with a `displayName` hint —
binds to it positionally:

```json
{
  "id": "urn:uuid:6f1c8b2a-0003-4a10-8a00-000000000003",
  "type": "https://trusttasks.org/spec/chat/message/0.1",
  "issuer": "did:key:z6MkBridgeExampleBbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  "recipient": "did:key:z6MkAgentExampleAaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "threadId": "urn:uuid:6f1c8b2a-conv-0000-0000-000000000000",
  "issuedAt": "2026-06-16T12:01:00Z",
  "payload": {
    "conversationId": "conv-9c2f",
    "direction": "inbound",
    "platform": "signal",
    "text": "￼ can you confirm the 3pm slot?",
    "mentions": [
      {
        "participant": "part-7b1e",
        "displayName": "Alice",
        "start": 0,
        "length": 1
      }
    ],
    "isGroup": true,
    "isMention": true,
    "prev": {
      "id": "urn:uuid:6f1c8b2a-0002-4a10-8a00-000000000002",
      "digest": "zQmaiQiEmnfNmSYMicJwuJcgiwRkRkALUPvVHyLWvtAbGyA"
    },
    "sentAt": "2026-06-16T12:01:00Z"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-06-16T12:01:00Z",
    "verificationMethod": "did:key:z6MkBridgeExampleBbbbbbbbbbbbbbbbbbbbbbbbbbbb#z6MkBridgeExampleBbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    "proofPurpose": "assertionMethod",
    "proofValue": "z…(eddsa-jcs-2022 signature over the JCS-canonical document)"
  }
}
```

## Security & Privacy

### Data carried

This task carries human conversation, and it is worth being blunt about that
rather than describing it as a message envelope. `text` is declared as a bare
string with no length bound, no pattern, and no structure: whatever a person
typed arrives verbatim. `mentions[].displayName` is a real-world human name the
source platform supplied. `attachments` are carried by reference rather than
inline, which keeps the bytes off this wire, but the reference itself is
descriptive — `filename` routinely says what a document is, `mediaType` and
`sizeBytes` narrow it further, and `digest` is a stable content fingerprint that
lets any party holding a candidate file confirm it is the one that was sent.
Around all of that sit `isGroup`, `isMention`, `replyToId` and `sentAt`, which
are deliberately part of the signed record so the chain captures where and when
a message was sent and not merely its words.

The one genuine minimisation the design does achieve is at the identifier layer.
`conversationId` and `mentions[].participant` are opaque, bridge-issued handles;
the schema forbids raw platform addresses in either, so no upstream party — the
agent, any intermediary, any later auditor — learns a phone number, a chat id, or
a platform member id. `displayName` is explicitly non-authoritative and
consumers **MUST NOT** treat it as an identity; resolving a handle to a real
person is deliberately out of band.

Beyond that, minimisation is the producer's to do and the specification cannot do
it for them. Only `conversationId`, `direction` and `sentAt` are REQUIRED. Every
member that carries content — `text`, `mentions`, `attachments`, `platform` — is
OPTIONAL, so a producer bridging a platform **SHOULD** carry the smallest form
that still makes the message intelligible to its counterparty, and **SHOULD NOT**
populate `attachments[].filename` where the media type and digest already suffice.

Confidentiality is not this task's job. The document proof authenticates and
orders; it does not encrypt. A deployment carrying sensitive conversation
**SHOULD** run it over a confidential binding such as DIDComm authcrypt, and
should understand that doing so protects the message in flight and not the chain
at rest.

Two smaller members are free text on the same terms and are bounded accordingly.
`mentions[].displayName` (256) is the name the *source platform* supplied for a
mentioned participant — not the bridge's own view of them and never an identity
claim, so a surface MUST render it as a hint and MUST NOT resolve a participant
by it. `attachments[].filename` (256) is whatever the sender's device called the
file, which is routinely more disclosive than the sender expects; it is carried
for display only. Both are read by whoever reads the message and, because the
chain is `durable`, both are retained for as long as the message is — a display
name that was accurate when the message was sent is retained unchanged
afterwards, and neither member is refreshed.

### Correlation

`conversationId` is stable for the life of a conversation by construction — it is
what makes a chain a chain — and `mentions[].participant` is stable for a
participant across every conversation the same bridge terminates. Neither reveals
a platform address, but both are perfectly good correlation keys within a
bridge's scope: a bridge, or anyone who obtains its records, can reconstruct who
spoke to whom, how often, in which groups, and at what hours, from
`participant`, `isGroup` and `sentAt` alone, without reading a single `text`.
Handles protect against upstream leakage of platform identity; they do not make
the traffic pattern private.

The chain adds a second kind of joinability that a message store does not have.
`prev` fixes each message's position relative to every other, so the sequence
itself is evidence: an arbiter can prove not just that a sentence was written but
that it was written *after* another one and *before* a third, with no gaps in
between. That is precisely the property the task is for, and it is also why the
history cannot be partially disavowed later.

Both parties declare `identifierScope: pairwise`. Nothing in the mechanism asks
the agent's or the bridge's DID to be recognisable outside their own
relationship — the counterparty check in *Authorization* is "is this the expected
author for this conversation and direction", which is a lookup local to the pair.
There is a real tension in that, and implementers should see it: the task's stated
purpose is third-party audit and dispute resolution, and a pairwise identifier
means an arbiter arriving later can verify that a specific key signed a specific
message without being able, from the chain alone, to say whose key it was. Binding
a pairwise DID to a responsible party is an out-of-band step that the deployment
must provide. A `public` scope would remove that step, at the cost of making every
conversation a bridge carries linkable to that party's activity everywhere else —
which is the wrong trade for a messaging data plane, and the reason it is not
declared here.

### Retention

Durable, and this is the hard part of the design rather than an afterthought.
Messages are meant to be kept: the value of the `proof` is that it survives the
transport, and the value of `prev` is that it survives reordering, so a chain a
consumer does not retain proves nothing at all.

The cost lands on deletion. `prev.digest` is taken over the JCS canonicalization
of the *whole preceding document*, payload included. Removing a message, or
editing a word of its `text`, therefore breaks the link for every message that
follows it — a consumer verifying the chain gets `chat/message:brokenChain`, which
is indistinguishable from tampering, because at the level of the mechanism it *is*
tampering. A conversation chain cannot honour a request to erase one message and
remain verifiable. The two properties are the same property viewed from opposite
sides, and no amount of implementation care reconciles them.

This specification does not resolve that, and implementers **MUST NOT** assume it
has. The choices a deployment actually has are to retain whole conversations for a
bounded, published period and then drop them entire — dropping a prefix of a chain
costs nothing, since verification runs backwards from the head — or to segment
conversations so a chain's span is small enough to discard as a unit, or to accept
that the chain is permanent and be honest with participants about it. What a
deployment **SHOULD NOT** do is offer per-message deletion and quietly leave the
chain broken, since the next verifier will read that as evidence of interference.

### Consent/purpose

The purpose is a governed messaging data plane in which an agent can converse
through a platform without ever holding that platform's credentials, and in which
what was said can later be established. The authority to append is narrow and
stated in *Authorization*: being the expected author for this `conversationId` and
`direction`. A valid signature from any other party is an authenticated message
and not an authorised one.

Two limits follow, and both are about reuse rather than access. First, the
evidentiary purpose is what justifies retaining the content at all; a consumer
that holds conversation history because dispute resolution may need it **SHOULD
NOT** mine the same history for unrelated ends — profiling participants,
enriching a contact graph, or training a model — since none of those is the reason
the participants' words were kept. Second, the human participants on the platform
side are not parties to this document and never see it: they are conversing on
Signal or WhatsApp, not consenting to a Trust Task chain. Whether they are told
that an agent is present, that the conversation is being attested, and that the
record is durable, is a deployment obligation that lives entirely outside this
payload — and per [SPEC.md §7.3](/SPEC.md#73-specification-requirements) item 13
this specification states the fact and declines to prescribe the gate.
