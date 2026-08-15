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
  - role: Counterparty
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Chat messages form an evidentiary chain that must be verifiable after the transport has closed. For audit and dispute resolution a third party must verify each message's author and its position in the conversation independently of the (ephemeral, authcrypted) DIDComm session that carried it — transport authentication alone is not portable.
sideEffects:
  level: mutating
  rationale: "Appends a signed, hash-linked message to the conversation chain; persisted history."
exposure:
  discloses: none
  actsAsSubject: false
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

*Stated in anticipation of [SPEC.md §7.3](../../../../SPEC.md#73-specification-requirements)
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
[SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements) item 10 it is not
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

**Portable, transport-independent evidence.** The value of this task is that the
`proof` survives the transport. authcrypt authenticates the sender to the
recipient at unpack time, but leaves nothing a third party can check later; the
`eddsa-jcs-2022` document proof lets an auditor or arbiter verify authorship and
ordering long after the session closed. This is why `proofRequirement` is
`REQUIRED`.

**Ordered, tamper-evident chain.** `prev` binds each message to the digest of
its predecessor, so a removed, reordered, or forged-in-the-middle message breaks
the chain and is detectable (`chat/message:brokenChain`). Producers SHOULD
compute the digest over the JCS-canonical previous document.

**Opaque handles.** `conversationId` and any contact reference — including a
mention's `participant` — are bridge-issued handles, never raw platform
addresses; upstream parties never learn the phone number, UUID, or chat id. A
mention's `displayName` is an optional, non-authoritative rendering hint and
MUST NOT be treated as an identity. Surfacing a real-world identity (e.g. for a
human approval step) is out of band and out of scope for this task.

**Confidentiality is the transport's job.** This task is about *authenticity and
ordering*, not secrecy. Carry it over a confidential binding (DIDComm authcrypt)
when message contents are sensitive; the document proof does not encrypt.
