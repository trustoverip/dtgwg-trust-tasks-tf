---
slug: chat/direct/message
version: "0.1"
title: "Chat — Direct message"
summary: "One person writes a message to another they have a relationship with — and, as its author, edits it, withdraws it, or reacts to a message — using the same content vocabulary as a room record, carried without a proof over an end-to-end binding."
status: draft
targetFrameworkVersion: "0.6.0"
category: chat
keywords:
  - chat
  - direct message
  - dm
  - person-to-person
  - reply
  - edit
  - withdraw
  - unsend
  - reaction
authors:
  - Brendan A. Miller (https://github.com/bmiller59)
parties:
  - role: Sender
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Recipient
    requirement: REQUIRED
    member: recipient
    identifierScope: pairwise
proofRequirement:
  requirement: OPTIONAL
  rationale: >-
    A direct message is a conversation between two people, not a record meant to
    be relied on by anyone outside it, so the task asks for no transport-independent
    proof. Integrity and sender authentication come instead from the end-to-end
    binding the task is restricted to (SPEC §9.1.1): a producer sends it only over a
    binding mode that permits omitting `proof`. A producer that adds a proof anyway
    is not forbidden from doing so, and a consumer verifies it as it would any
    other (SPEC §7.2 item 7) — but the message then carries a signature any holder
    of the document can verify, which is more than the task needs.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task is mutating, so SPEC §7.3 item 17 makes `issuedAt` REQUIRED: a consumer
    can only absorb a redelivered message (SPEC §7.2 item 11) inside a bounded window,
    and direct messages are routinely composed offline and delivered late through a
    mediator, so the window has to be placeable. The acceptance window the
    specification expects is stated under Security & Privacy.
sideEffects:
  level: mutating
  rationale: >-
    Adds or rewrites the sender's own message in the recipient's copy of the
    conversation. A withdrawal makes the recipient discard that message's content.
    The content is the sender's own words, which the sender also holds, and nothing
    of the recipient's is destroyed.
exposure:
  discloses: none
  ingests: personal
  actsAsSubject: false
  rationale: >-
    The request is the message: `body` is whatever a person typed, verbatim, or a
    reaction they chose, and `writtenAt`, `editedAt` and `inReplyTo` add when it was
    written and edited and what it answers. Nothing is disclosed back — the task is
    fire-and-forget — but the recipient receives all of it and keeps it as part of
    the conversation.
retention:
  class: exchange
  rationale: >-
    The conversation is the exchange. A recipient keeps the current revision of each
    message for as long as it keeps the conversation, and drops it when the
    conversation is deleted. Nothing here is a record meant to outlive that: the
    documents carry no proof, so deleting them loses no evidence, and a withdrawal
    asks the recipient to discard the content sooner. The deduplication keys and
    withdrawal markers a consumer holds are kept for the acceptance window only.
maxDocumentBytes:
  request: 262144
  rationale: >-
    `body` is at most 16384 characters. Up to 4 bytes each in UTF-8, but an encoder
    that escapes every non-ASCII character writes an astral one as a surrogate
    pair of `\u` escapes, 12 bytes, so a body alone can reach about 196 KB. The
    bound covers that, the other members and envelope headroom. No proof is
    expected.
errorCodes:
  - code: chat/direct/message:notAuthor
    meaning: The document writes a `messageId` the consumer already holds for this conversation (the same `issuer`/`recipient` pair), and its authenticated issuer is not that message's author. Nothing was changed.
    retryable: false
  - code: chat/direct/message:revisionConflict
    meaning: The consumer holds this `revision` of this `messageId`, from the same author in the same conversation, as the message's current revision, with a different payload. The held revision stays. A revision lower than the current one is never refused; it is ignored silently.
    retryable: false
  - code: chat/direct/message:rewriteInvalid
    meaning: The rewrite changes the `kind` or `writtenAt` of the revision the consumer holds, or the `inReplyTo` of a reaction. The held revision stays.
    retryable: false
related:
  - chat/message
  - rooms/records/put
  - vrc/relationships/propose
  - messaging/message/status
---

## Abstract

**Chat — Direct message** is a message from one person to another, sent to the
pairwise identifier of someone they already have a relationship with, so that two
people who connected in different applications can write to each other. The same
task carries everything a person does to their own message after sending it: the
author **edits** it by sending it again under the same `messageId` with a greater
`revision`, **withdraws** it ("unsend") by sending a rewrite with an empty `body`,
and **reacts** to a message with a short message of `kind: reaction`.

The content vocabulary — `kind`, `body`, `inReplyTo`, `writtenAt`, `editedAt`, and
the rules for edits, withdrawal, replies and reactions — is the one a room record
uses (`rooms/_shared/0.1`, `SealedRecordBody`), and three of its definitions are
referenced from there directly. A client therefore renders a direct message and a
room message with one model. The container differs: a direct conversation is the
relationship between two pairwise identifiers, with no host, no membership and no
signed record body. Group conversations are `rooms/*`.

The documents carry no proof by design (`proofRequirement: OPTIONAL`), and a
producer sends them only over a binding mode that permits omitting `proof` under
[SPEC §9.1.1](/SPEC.md#911-permitting-proof-to-be-omitted).

This task is **fire-and-forget** ([SPEC §4.4.1](/SPEC.md#441-request-and-response-variants)):
it defines no success-response document. A consumer **MAY** send the courtesy
acknowledgement [SPEC §4.4.2](/SPEC.md#442-acknowledging-a-fire-and-forget-task)
describes, and failures use `trust-task-error`. Delivery and read receipts are a
separate task.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

### Carriage

`proofRequirement` is `OPTIONAL`, which is no weaker than
[SPEC §4.7.2](/SPEC.md#472-when-to-include-a-proof) only because this specification
restricts how the documents travel:

1. A producer **MUST** send a document of this task only over a transport binding
   mode whose binding specification permits `proof` to be omitted under
   [SPEC §9.1.1](/SPEC.md#911-permitting-proof-to-be-omitted). Which modes those
   are is stated by each binding, including where a binding's guarantee stops (a
   routing mode, a mediator, a proxy), and not by this specification.
2. A producer **SHOULD NOT** fall back to a mode with no such allowance when no
   permitted mode reaches the recipient. It **SHOULD** wait, or tell its user that
   the message cannot be sent, rather than send it over that mode or add a proof
   to make up for it.
3. A consumer that receives a proofless document of this task over a mode with no
   §9.1.1 allowance applies [SPEC §4.7.2](/SPEC.md#472-when-to-include-a-proof)
   and its own policy to it, as for any task; nothing here obliges it to accept
   one.

A present `proof` is verified as [SPEC §7.2](/SPEC.md#72-consumer-requirements)
item 7 says, and does not change any rule below.

### Producer

A conforming producer (the message's author):

1. **MUST** mint a fresh `messageId` (`MessageId`) for every new message and every
   new reaction, and **MUST NOT** derive it from the content, the counterparty, the
   time or any other identifier ([SPEC §13.1](/SPEC.md#131-identifier-correlation-and-linkability) item 2).
2. **MUST** set `revision: 1` on the first document it sends for a `messageId`, and
   a strictly greater integer on each rewrite of it. Gaps are allowed.
3. **MUST** send each revision as a new document with a fresh document `id`, and
   **MUST NOT** reuse a document `id` for a new revision.
4. **MUST** carry `body` whole on every revision: a rewrite replaces the message,
   it does not patch it.
5. **MUST** keep, on a rewrite, the `kind` and `writtenAt` of the revision it
   replaces, and on a `reaction` its `inReplyTo`. A rewrite of a `message` **MAY**
   change `inReplyTo`.
6. **MUST** set `editedAt` on every revision after the first, and **MUST NOT** set
   it on revision 1.
7. **MUST NOT** rewrite a `messageId` it did not author, and **MUST NOT** name a
   message of kind `reaction` in `inReplyTo`.
8. **SHOULD** omit `threadId` (see *Correlation*). A conversation-long `threadId`
   would join every message of the conversation for anyone who sees the documents.
9. **MUST NOT** put message content, contact details, location, device identifiers
   or handles in `ext`.

### Consumer

A conforming consumer applies the rules in *Rewriting a message* below, in
addition to [SPEC §7.2](/SPEC.md#72-consumer-requirements). In summary it
**MUST** refuse a rewrite by anyone but the author (`notAuthor`), a conflicting
copy of the current revision it holds (`revisionConflict`), and a rewrite that
changes what a new revision keeps (`rewriteInvalid`); it **MUST** ignore a stale
revision silently; and it **MUST NOT** answer anything about a `messageId` it does
not hold for the conversation the document belongs to.

## Authorization

The authority for a **new message** (revision 1, or a later revision of a
`messageId` the consumer does not hold) is **an accepted relationship between the
two pairwise identifiers**, `issuer` and `recipient`, established outside this task
— a TSP relationship, a DIDComm connection, a `vrc/relationships/*` exchange, or
whatever the deployment uses to decide who may write to whom. Whether a consumer
accepts messages from an identifier it has no accepted relationship with, holds
them as a message request, or drops them, is the consumer's policy; this
specification states the authority and does not prescribe the gate
([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13).

The authority for a **rewrite** — an edit, a withdrawal, a changed or withdrawn
reaction — is the same relationship **plus authorship of the `messageId`**. The
author of a `messageId` is the authenticated issuer of the first document the
consumer accepted for it in that conversation or, for a message the consumer's own
user sent, that user; only the author rewrites it (*Rewriting a message*, R1).

Neither is established by identity or proof alone. Per
[SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10, authenticating the sender
— the in-band `issuer` cross-checked against the transport-authenticated sender
([SPEC §4.8.1](/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity)),
or a verified `proof` — establishes *who* sent the document and that it is
unaltered. It is what lets the consumer compare the sender with the relationship
and with the message's author; it is not itself the entitlement.

## Definitions

- **`messageId`** (REQUIRED, `MessageId`) — names one message or one reaction
  across all of its revisions. Chosen by the author for revision 1; 32 random bytes
  in base64url, 43 characters. It is not the document `id`.
- **`revision`** (REQUIRED, `MessageRevision`) — the author's revision number for
  the `messageId`, starting at 1. The only key rewrites are ordered by.
- **`kind`** (REQUIRED, the rooms `RecordKind`) — `message`, a turn in the
  conversation, or `reaction`, a short response to another message. An open set,
  as in rooms: a reader presents a kind it does not know as rooms says (as a note,
  its `body` inert) and **MUST NOT** refuse a document for its kind.
- **`body`** (REQUIRED, `MessageBody`) — on a `message`, plain text, rendered as
  the rooms *What `body` is* rule says: never as markup, never following a URL by
  itself. On a `reaction`, the reaction's value (*Reactions*). Empty means
  withdrawn, and only a rewrite may be empty.
- **`inReplyTo`** (`MessageReference`) — OPTIONAL on a `message`, the message it
  answers; REQUIRED on a `reaction`, the message it responds to. It names a message
  of the same conversation only; the reader resolves it against messages it already
  holds for this conversation and **MUST NOT** fetch anything because of it. A reader
  ignores an `inReplyTo` that names the message's own `messageId`.
- **`writtenAt`** (REQUIRED, the rooms `RecordWrittenAt`) — when the author wrote
  the message, UTC. Unchanged across rewrites.
- **`editedAt`** (the rooms `RecordEditedAt`) — when the author made the edit that
  produced this revision, UTC. Absent on revision 1, REQUIRED on every later one.
  Display only.
- **`ext`** — the framework extension slot ([SPEC §4.5.1](/SPEC.md#451-the-ext-extension-member)).
  It **MUST NOT** carry content, and a rooms import drops it.

### The same vocabulary as rooms

Where a rule below says a reader applies a rooms rule, it is the rule of that name
in `rooms/_shared/0.1/room.schema.json` (`SealedRecordBody`), read with these
substitutions, because a direct conversation has no host:

| Rooms | Here |
|---|---|
| record `key` | `messageId` |
| record `version` (host-assigned) | `revision` (author-assigned) |
| `author`, inside the signed body | the authenticated issuer of the first accepted document for the `messageId`, or the consumer's own user for a message that user sent (R1) |
| the host's `createdAt` for a record | the instant the consumer first accepted the `messageId` |
| the host's `updatedAt` for a version | the instant the consumer received the revision |
| "the same room" | the same direct conversation (the same pair of identifiers) |
| a curator's retraction (`rooms/records/curate`) | none: only the author withdraws |

The rooms rules this specification applies are *What `body` is*, *When it was
written*, *Edits*, *What a new version keeps*, *Withdrawing a message*,
*Replies*, *Reactions*, *Counting reactions*, *A reaction's value* and *Nothing
answers a reaction*. A direct message is a strict subset of a rooms message: it
carries none of the library members (`title`, `description`, `tags`, the
provenance members, `roomProfile`), and `additionalProperties: false` keeps them
out.

## Rewriting a message

### R1. Only the original author rewrites

1. The **author** of a `messageId` is the authenticated issuer of the first
   document the consumer accepted for that `messageId` in the conversation or,
   for a message the consumer's own user sent, that user. Authenticated means the
   in-band `issuer` cross-checked against the transport-authenticated sender per
   [SPEC §4.8.1](/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity),
   or derived from it where the in-band member is absent.
2. A consumer **MUST NOT** apply a document for a `messageId` it holds for this
   conversation — the same `issuer`/`recipient` pair, in either direction — whose
   authenticated issuer is not that author. It answers
   `chat/direct/message:notAuthor` where a reply can be routed, and changes
   nothing. In a direct conversation this is the counterparty trying to rewrite
   the other's message. Every `messageId` lookup in this specification is scoped
   to the conversation the document belongs to: a `messageId` the consumer holds
   only for a different conversation is, for this one, a `messageId` it does not
   hold (R4).
3. A consumer **MUST NOT** accept a rewrite on the strength of an in-band `issuer`
   the transport did not authenticate. With no proof, the transport is the only
   authentication.

This is stricter than rooms, where a reader can only show a non-author's rewrite
as the new author's: here the consumer sees the authenticated sender of every
document and can refuse it.

### R2. Redelivery and new attempts

There are two layers, and a consumer **MUST NOT** merge them:

1. **The document layer is unchanged.** [SPEC §7.2](/SPEC.md#72-consumer-requirements)
   item 11 applies to every document by its `id`: the same `id` with identical
   content is a redelivery and is absorbed; the same `id` with different content is
   `idConflict`. The (`messageId`, `revision`) pair **MUST NOT** stand in for the
   document `id` in that record, just as a transport's own message identifier must
   not.
2. **The message layer is this task's.** Its key is (conversation, author,
   `messageId`, `revision`), where the conversation is the `issuer`/`recipient`
   pair, and it is compared only against the **current** revision the consumer
   holds for that `messageId`:
   - The same key with an identical payload — compared as the RFC 8785
     canonicalization of `payload` — under a **different** document `id` is a new
     attempt ([SPEC §8.4](/SPEC.md#84-retry-semantics)). It is absorbed: nothing
     changes, and no error is sent.
   - The same key with a different payload is refused with
     `chat/direct/message:revisionConflict`, and the held revision stays.
   - A revision lower than the current one is not compared at all, whatever its
     payload: it is stale, and R3 ignores it silently.

This specification does not declare repeated execution safe. Item 11 still applies
to every document.

### R3. Ordering rewrites, edits and withdrawal

1. **`revision` is the only ordering key.** A consumer applies a revision only if
   it is greater than the current one it holds for that `messageId`. A lower
   revision is stale whatever its payload: the consumer ignores it silently and
   sends no error, never `revisionConflict`, because mediated delivery reorders
   routinely. An equal revision is handled by R2 alone.
2. **`editedAt` is display only.** A reader shows a message carrying it as edited.
   It treats an `editedAt` earlier than `writtenAt`, or more than five minutes after
   the instant it received the revision, as implausible, and then shows that the
   message was edited but not when (the rooms *Edits* rule).
3. **What a rewrite keeps.** A rewrite **MUST** keep the `kind` and `writtenAt` of
   the revision it replaces, and on a `reaction` its `inReplyTo`. A consumer that
   holds an earlier revision refuses a rewrite that changes any of them with
   `chat/direct/message:rewriteInvalid`, and keeps what it holds. A rewrite of a
   `message` **MAY** change `inReplyTo`.
4. **Withdrawal.** A current revision with an empty `body` means the message is
   withdrawn — a tombstone. The consumer **MUST NOT** show any earlier revision's
   content and **SHOULD** discard it from storage. It keeps the `messageId`,
   `kind`, `revision`, `writtenAt` and `inReplyTo`, so that the message's place,
   replies to it, and R3 ordering all survive; replies are shown as answering a
   message that is not available. "Withdraw" is the author's act; this
   specification has no curator and no retraction.
5. **Withdrawal is not terminal**, as in rooms: a later revision with a non-empty
   `body` replaces it. An author can undo an unsend.

### R4. A revision of a message the consumer does not hold

"Does not hold" means does not hold for this conversation (R1): a `messageId` the
consumer holds only in a different conversation is unknown here, and is treated by
the rules below exactly as one it has never seen.

1. `revision: 1` for an unknown `messageId` is a new message. The consumer accepts
   it, subject to *Authorization*.
2. A revision greater than 1 for an unknown `messageId` means the earlier revisions
   were missed, reordered, or sent before this device had the conversation. Each
   revision is a whole message, so the consumer **MAY** accept it as the current
   state: it records the authenticated issuer as the author (R1), shows the
   message as edited, and applies R3 to anything later.
3. If that revision's `body` is empty — a withdrawal of a message never seen — the
   consumer records a tombstone for the `messageId` and shows nothing, so that a
   late revision 1 cannot bring the message back. It keeps tombstones at least for
   the acceptance window.
4. A consumer **MUST NOT** send an error about a `messageId` it does not hold for
   this conversation, including one it holds for another conversation. There is
   nothing to refuse, and silence avoids a state oracle
   ([SPEC §12.6](/SPEC.md#126-error-response-identity-leakage)): no answer
   confirms that a `messageId` exists anywhere outside the sender's own
   conversation.
5. An `inReplyTo` naming an unknown or withdrawn message: the consumer shows the
   reply, marked as answering a message that is not available, and fetches
   nothing.
6. A `reaction` whose target is unknown is not shown (the rooms *Reactions* rule).
   The consumer **MAY** hold it until the target arrives, within the acceptance
   window. A message or reaction whose `inReplyTo` names a `reaction` is ignored
   (the rooms *Nothing answers a reaction* rule).
7. A message the recipient deleted locally is the recipient's business. A consumer
   **SHOULD** keep a local deletion marker for it, so that a later revision does not
   bring back what its user deleted.

### R5. What a consumer keeps

Only the current revision of each `messageId`; for the acceptance window, the R2
key of that revision and a digest of its RFC 8785 canonical `payload`, without
which an identical new attempt cannot be told from a `revisionConflict` once the
content is withdrawn or discarded; and its tombstones and deletion markers. It **SHOULD NOT** keep earlier
revisions' content: edit history is not a requirement of this task, and keeping it
undermines withdrawal.

### R6. Ordering in the conversation

A reader orders the conversation by `writtenAt`, then by `messageId` where two are
equal, with the rooms *When it was written* rules, reading the host's `createdAt`
as the instant the consumer first accepted the `messageId`:

- A `writtenAt` more than five minutes after that instant is implausible: the
  reader orders the message by that instant and does not show the claimed time as
  when it was written.
- A `writtenAt` more than 24 hours before that instant claims to have been written
  well before it arrived: the reader orders the message by that instant and shows
  that it arrived later than it claims.

## Reactions

A reaction is a message of `kind: reaction` with its own `messageId`, its target
in `inReplyTo`, and its value in `body`. The rooms *Reactions*, *Counting
reactions* and *A reaction's value* rules apply as written, with `messageId` for
`key`:

- The value is `body` normalised to NFC with every U+FE0E and U+FE0F removed, at
  most 64 code points after normalisation, and one line. A reader applies the
  normalisation before both the length check and any comparison, so `❤` and `❤️`
  are one value. A reader **MUST NOT** show a reaction whose value breaks those
  limits.
- One reaction counts once per author, value and target, however many documents
  carry it. A rewrite with a different non-empty value changes that author's
  reaction; an empty one withdraws it.
- A producer reacting with a value it already holds a reaction for, on the same
  target, **SHOULD** rewrite that reaction's `messageId` rather than mint a new one,
  and one withdrawing a reaction **MUST** withdraw every reaction `messageId` its
  author holds with that value and target.
- A reader **MUST NOT** show a reaction as a message.

## Request

The sender sends the request to the recipient; its payload is the top-level schema
in [`payload.schema.json`](payload.schema.json). Every example below is unsigned, as
the task intends, and carries no `threadId`.

### A message

```json
{
  "id": "urn:uuid:7c1e0a52-3f4b-4d8e-9a61-0b2c3d4e5f60",
  "type": "https://trusttasks.org/spec/chat/direct/message/0.1",
  "issuer": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "recipient": "did:key:z6MkjchhfUsD6mmvni8mCdXHw216Xrm9bQe2mBH1P5RDjVJG",
  "issuedAt": "2026-11-08T09:14:02Z",
  "payload": {
    "messageId": "q3Xv9LbT2mWc7RkN4sYh1PdZ8uFa6JeG0oVi5xQnB2E",
    "revision": 1,
    "kind": "message",
    "body": "Hi Bo — are we still on for three o'clock?",
    "writtenAt": "2026-11-08T09:14:02Z"
  }
}
```

### A reply

The recipient answers, naming the first message in `inReplyTo`:

```json
{
  "id": "urn:uuid:8d2f1b63-4a5c-4e9f-8b72-1c3d4e5f6071",
  "type": "https://trusttasks.org/spec/chat/direct/message/0.1",
  "issuer": "did:key:z6MkjchhfUsD6mmvni8mCdXHw216Xrm9bQe2mBH1P5RDjVJG",
  "recipient": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "issuedAt": "2026-11-08T09:15:00Z",
  "payload": {
    "messageId": "Zr5tN1vB8kQw3HxM6cLp2YdS9gFa4JeU7oRi0nWqT1A",
    "revision": 1,
    "kind": "message",
    "body": "Yes, three works. See you there.",
    "inReplyTo": {
      "messageId": "q3Xv9LbT2mWc7RkN4sYh1PdZ8uFa6JeG0oVi5xQnB2E"
    },
    "writtenAt": "2026-11-08T09:15:00Z"
  }
}
```

### An edit

The author rewrites the first message: the same `messageId`, `revision: 2`, the
same `kind` and `writtenAt`, a new document `id`, and an `editedAt`:

```json
{
  "id": "urn:uuid:9e3a2c74-5b6d-4fa0-9c83-2d4e5f607182",
  "type": "https://trusttasks.org/spec/chat/direct/message/0.1",
  "issuer": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "recipient": "did:key:z6MkjchhfUsD6mmvni8mCdXHw216Xrm9bQe2mBH1P5RDjVJG",
  "issuedAt": "2026-11-08T09:16:30Z",
  "payload": {
    "messageId": "q3Xv9LbT2mWc7RkN4sYh1PdZ8uFa6JeG0oVi5xQnB2E",
    "revision": 2,
    "kind": "message",
    "body": "Hi Bo — are we still on for three o'clock at the library?",
    "writtenAt": "2026-11-08T09:14:02Z",
    "editedAt": "2026-11-08T09:16:30Z"
  }
}
```

### A withdrawal

The author withdraws the message with a further rewrite whose `body` is empty. The
recipient shows it as withdrawn, or not at all, and no earlier revision's content;
the reply above stays, answering a message that is not available:

```json
{
  "id": "urn:uuid:af4b3d85-6c7e-40b1-8d94-3e5f60718293",
  "type": "https://trusttasks.org/spec/chat/direct/message/0.1",
  "issuer": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "recipient": "did:key:z6MkjchhfUsD6mmvni8mCdXHw216Xrm9bQe2mBH1P5RDjVJG",
  "issuedAt": "2026-11-08T09:20:11Z",
  "payload": {
    "messageId": "q3Xv9LbT2mWc7RkN4sYh1PdZ8uFa6JeG0oVi5xQnB2E",
    "revision": 3,
    "kind": "message",
    "body": "",
    "writtenAt": "2026-11-08T09:14:02Z",
    "editedAt": "2026-11-08T09:20:11Z"
  }
}
```

### A reaction

The first sender reacts to the reply with its own `messageId`:

```json
{
  "id": "urn:uuid:b05c4e96-7d8f-41c2-9ea5-4f60718293a4",
  "type": "https://trusttasks.org/spec/chat/direct/message/0.1",
  "issuer": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "recipient": "did:key:z6MkjchhfUsD6mmvni8mCdXHw216Xrm9bQe2mBH1P5RDjVJG",
  "issuedAt": "2026-11-08T09:15:20Z",
  "payload": {
    "messageId": "Hk2pW8nQ4cZt6YvR1mLs9BxE3fJa7GuD5oTi0wKeN8M",
    "revision": 1,
    "kind": "reaction",
    "body": "👍",
    "inReplyTo": {
      "messageId": "Zr5tN1vB8kQw3HxM6cLp2YdS9gFa4JeU7oRi0nWqT1A"
    },
    "writtenAt": "2026-11-08T09:15:20Z"
  }
}
```

### Withdrawing the reaction

A rewrite of the reaction with an empty value, keeping its `kind`, `writtenAt` and
`inReplyTo`:

```json
{
  "id": "urn:uuid:c16d5fa7-8e90-42d3-8fb6-50718293a4b5",
  "type": "https://trusttasks.org/spec/chat/direct/message/0.1",
  "issuer": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "recipient": "did:key:z6MkjchhfUsD6mmvni8mCdXHw216Xrm9bQe2mBH1P5RDjVJG",
  "issuedAt": "2026-11-08T09:17:45Z",
  "payload": {
    "messageId": "Hk2pW8nQ4cZt6YvR1mLs9BxE3fJa7GuD5oTi0wKeN8M",
    "revision": 2,
    "kind": "reaction",
    "body": "",
    "inReplyTo": {
      "messageId": "Zr5tN1vB8kQw3HxM6cLp2YdS9gFa4JeU7oRi0nWqT1A"
    },
    "writtenAt": "2026-11-08T09:15:20Z",
    "editedAt": "2026-11-08T09:17:45Z"
  }
}
```

### Refused: a rewrite by someone other than the author

The recipient sends a document under the first sender's `messageId`. The first
sender's consumer holds that message from its own user, so it changes nothing and
answers `notAuthor`:

```json
{
  "id": "urn:uuid:d27e60b8-9fa1-43e4-90c7-618293a4b5c6",
  "type": "https://trusttasks.org/spec/trust-task-error/0.6",
  "issuer": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "recipient": "did:key:z6MkjchhfUsD6mmvni8mCdXHw216Xrm9bQe2mBH1P5RDjVJG",
  "issuedAt": "2026-11-08T09:18:03Z",
  "threadId": "urn:uuid:e38f71c9-a0b2-44f5-a1d8-7293a4b5c6d7",
  "payload": {
    "code": "chat/direct/message:notAuthor",
    "retryable": false,
    "inResponseTo": {
      "typeUri": "https://trusttasks.org/spec/chat/direct/message/0.1",
      "id": "urn:uuid:e38f71c9-a0b2-44f5-a1d8-7293a4b5c6d7"
    }
  }
}
```

### Refused: a conflicting copy of a revision

The consumer holds revision 2 of a message and receives, under a new document
`id`, a revision 2 from the same author with a different `body`. It keeps what it
holds and answers `revisionConflict`:

```json
{
  "id": "urn:uuid:f490a2da-b1c3-45a6-b2e9-8394a5b6c7d8",
  "type": "https://trusttasks.org/spec/trust-task-error/0.6",
  "issuer": "did:key:z6MkjchhfUsD6mmvni8mCdXHw216Xrm9bQe2mBH1P5RDjVJG",
  "recipient": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "issuedAt": "2026-11-08T09:19:12Z",
  "threadId": "urn:uuid:05a193eb-c2d4-46b7-83fa-94a5b6c7d8e9",
  "payload": {
    "code": "chat/direct/message:revisionConflict",
    "retryable": false,
    "inResponseTo": {
      "typeUri": "https://trusttasks.org/spec/chat/direct/message/0.1",
      "id": "urn:uuid:05a193eb-c2d4-46b7-83fa-94a5b6c7d8e9"
    }
  }
}
```

## Security & Privacy

### Data carried

The task carries human conversation: `body` is whatever a person typed, verbatim,
up to 16384 characters, or the reaction they chose. Around it sit `writtenAt` and
`editedAt` (when it was written and edited), `inReplyTo` (what it answers), and
`revision`, a per-message edit counter that discloses how often the author edited
that message — to the one party who sees the edits anyway. Nothing else is
REQUIRED, and there is nothing else to send: no display names, no contact details,
no conversation identifier. `ext` **MUST NOT** carry content, contact details,
location, device identifiers or handles; nothing in the toolchain catches that, so
it is stated here.

The documents carry no proof, so a document a consumer holds is not something its
author signed. Confidentiality and sender authentication come from the end-to-end
binding the task is restricted to (*Carriage*), and stop where that binding's
guarantee stops. What a mediator or other intermediary sees — routing, size,
timing — is the binding's to describe.

### Correlation

Both parties declare `identifierScope: pairwise`: the sender and the recipient use
identifiers minted for their relationship, so the documents of one conversation
cannot be joined to either person's other conversations by identifier.
`messageId` and every document `id` are fresh and random
([SPEC §13.1](/SPEC.md#131-identifier-correlation-and-linkability) item 2), and
there is deliberately no conversation identifier and no hash chain.

Within the conversation, `messageId` links the revisions of one message, which is
its purpose, and `inReplyTo` links a reply to what it answers. A producer
**SHOULD** omit `threadId`: a conversation-long `threadId` would join every message
of the conversation for anyone who can see the documents, and
[SPEC §13.1](/SPEC.md#131-identifier-correlation-and-linkability) item 2 scopes a
correlator to one exchange. The libraries stamp `issuedAt` in whole seconds, and a
producer **MAY** coarsen it further (item 3), at the cost of the consumer's clock
skew allowance. `writtenAt` and `editedAt` are the author's own times and a
producer **MAY** coarsen them likewise.

### Retention

The recipient keeps the current revision of each message for as long as it keeps
the conversation (`retention.class: exchange`). It **SHOULD NOT** keep earlier
revisions (R5). A withdrawal asks the recipient to remove the content from
display **and** from storage, and a conforming consumer does so (R3), but the
recipient holds the bytes and the sender cannot verify that they were discarded;
a user interface **SHOULD NOT** promise an author more than that. No proof means
nothing evidentiary is lost by deleting.

**Acceptance window.** Direct messages are composed offline and delivered through
mediators that hold them until the recipient comes online, so a short window
refuses messages people actually sent. A consumer **SHOULD** accept an `issuedAt`
up to **30 days** old ([SPEC §7.3](/SPEC.md#73-specification-requirements)
item 17), refusing older ones and ones beyond its clock skew in the future as
[SPEC §7.2](/SPEC.md#72-consumer-requirements) item 13 says, and **SHOULD** keep
its item 11 record, the R2 key and payload digest of each message's current
revision (R5), and its tombstones for the same window. A revision older than the
current one is ignored silently (R3), never refused, so no key for it is needed. This is
guidance; the window is the consumer's policy.

### Consent/purpose

The purpose is delivering one person's words to the person they addressed, in a
relationship both accepted, and nothing more. A consumer **SHOULD NOT** use a
conversation for anything its user did not ask for — profiling the sender,
building a contact graph, training a model. Whether a consumer accepts messages
from someone its user has no accepted relationship with is consumer policy; per
[SPEC §7.3](/SPEC.md#73-specification-requirements) item 13 this specification
describes the purpose and does not prescribe a consent gate. Errors are limited to
the three codes declared above, each about a message the consumer already holds
from the sender's own conversation, and an error's `message` text **MUST NOT**
echo internal state ([SPEC §12.6](/SPEC.md#126-error-response-identity-leakage)).
