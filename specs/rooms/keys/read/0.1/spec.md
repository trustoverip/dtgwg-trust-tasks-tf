---
slug: rooms/keys/read
version: "0.1"
title: "Rooms Keys — Read"
summary: "A member asks their own agent to fetch one record from the room's host, check what the host asserted about it, and open it."
status: draft
targetFrameworkVersion: "0.5.0"
category: ai-agents
keywords:
  - room
  - record
  - read
  - trace
  - commitment
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Member
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: KeyHolder
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The request makes the recipient act outward in its principal's name — presenting their room credentials to a party this request names. One whose origin depended on the transport would let a compromised channel aim a member's credentials at a host of somebody else's choosing.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The response carries a verdict about a host's honesty, and a verdict that cannot be placed in time cannot be compared with a later one — which is the whole mechanism. An undated read is also one a duplicate-suppression window cannot absorb.
sideEffects:
  level: none
  rationale: >-
    Returns one record. The recipient makes an outbound request and MAY record what the host asserted, but the room is unchanged and nothing here is a write.
exposure:
  discloses: secret
  ingests: none
  actsAsSubject: true
  rationale: >-
    `actsAsSubject` is the point: the recipient presents its principal's membership and authority to the named host, as them. `discloses: secret` because the response carries the record's plaintext — on a sealed tier this is material the room withholds from its own host, and the recipient is the only party that can produce it.
retention:
  class: transient
  rationale: >-
    The record is returned and not kept; the presentation minted for the fetch is single-purpose. What the recipient MAY retain is what the host asserted about the room — see `verification.priorRoots` — which is a few dozen bytes and is not the record.
errorCodes:
  - code: rooms/keys/read:notAMember
    meaning: "The recipient holds no group state for this room, so it has nothing to present and nothing to open with."
    retryable: false
  - code: rooms/keys/read:hostUnreachable
    meaning: "The named host could not be resolved, advertises no transport this recipient speaks, or did not answer."
    retryable: true
  - code: rooms/keys/read:hostRefused
    meaning: "The host answered and declined. Its own code and reason are carried in `details` — commonly that it does not serve this room, or that no record has this key."
    retryable: false
  - code: rooms/keys/read:cannotOpen
    meaning: "The record was fetched and its epoch key is not held. `details.epoch` names the epoch; `rooms/keys/backfill` is the repair."
    retryable: false
related:
  - rooms/keys/browse
  - rooms/keys/open
  - rooms/keys/present
  - rooms/records/get
---

## Abstract

A member asks their **key holder** — the agent that holds their room keys — to read one record from the room's host: mint the presentation, make the request, check what came back, and open it.

Four acts, and today they belong to three parties. The presentation only the agent can mint ([`rooms/keys/present`](../../present/0.2/spec.md)); the request anyone with a channel to the host can make; the checking anyone can do *who has the room's state*; and the opening **only** the agent, because the epoch key never leaves it ([`rooms/keys/open`](../../open/0.1/spec.md)). The fourth decides where the other three go.

It exists for the same reason [`rooms/keys/backfill`](../../backfill/0.1/spec.md) does: the surfaces a person actually uses — a browser extension, a phone — hold a channel to their own agent and to nothing else. The alternative is a member who mints a presentation at their agent, carries it to the host, carries ciphertext back, and hands it to the same agent to open, holding a half-verified record in between.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

**The entitlement is being the recipient's own principal**, as with every custody task in this family. A key holder holds one member's room keys; this asks it to use them.

It is deliberately *not* gated on a room credential. The recipient already holds everything the room issued to its principal — that is what makes it their key holder — so requiring one here would ask the caller to present, to the party that mints the presentation, a credential it is about to present on their behalf.

A conforming consumer **MUST** accept this only from the party whose keys it holds, and **MUST** refuse it for a room it holds no group state for.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient` or `proof` establishes *who sent this* and *that it is unaltered* — never that they were entitled to the outcome.

### What the recipient presents, and to whom

The recipient **MUST** present a `read` grant scoped to this room and granted to the recipient itself. It **MUST NOT** present a grant conferring more than `read`.

**Presenter binding is what makes a caller-named host safe.** A presentation is granted to the recipient and usable by nobody else, so a host of the caller's choosing receives something it cannot act with. What such a host gains is **sight** of the presentation — the principal's membership and authority chain — which on a room that discloses its subjects names the principal. The caller is already a member and could have disclosed the same thing directly, so this widens correlation rather than access; a recipient that treats `host` as untrusted input is nonetheless right to, and **SHOULD** bound the outbound request by resolving `host` as a DID and speaking only the transports its document advertises.

## What the recipient checks

The reason this is not a proxy.

1. **The reply is signed by the host that was addressed.** [SPEC §7.3 item 7](/SPEC.md#73-specification-requirements) makes `rooms/records/get`'s response proof REQUIRED, and the recipient **MUST** verify it *and* bind the proven signer to the `host` it named. A proof that verifies against some other party is a reply from somebody else. An unsigned or misattributed reply is `hostRefused` with the reason stated — never a success with a caveat.
2. **The trace reaches the commitment served beside it.** The recipient **MUST** reassemble the leaf preimage from the response — that payload with its verification members removed, which is exactly `CommittedRecord` — hash it, replay the trace, and compare against the `dataCommitment` **in the same response**. Never one kept from an earlier read.
3. **The root against what it has seen before, at this `headVersion`.** See `verification.priorRoots`. An agent that keeps no history says `notChecked` rather than omitting the question.

### A verdict is never an error

**None of these findings fails the task.** A member's own agent refusing to hand over a record because the *host* misbehaved punishes the member for somebody else's act, and locks them out of the room holding the records that would show what happened.

The consequence belongs on the **write** path: continuing to hand material to a party you have caught is what compounds the damage, while reading is how a member gathers what they need. A consumer **MUST NOT** treat any value in `verification` as a failed read.

**What that costs is words, and this is the part that decides whether any of it works.** *Serve reads, refuse writes* is a rule nobody would guess. A member shown a bare warning icon dismisses it; a member later refused a write with no explanation concludes their own agent is broken. A detection the member attributes to the wrong party is worse than no detection, so a consumer **MUST** surface an adverse verdict in words that name what was observed — *two different record sets, both claimed as this room at version N* — rather than logging it or reducing it to a status colour.

### Opening, and when it cannot

The recipient opens the record with the epoch key for the epoch it was sealed under, which it derives by walking the chain it holds. Where that epoch is below its reach, the answer is `cannotOpen` with the epoch named, and [`rooms/keys/backfill`](../../backfill/0.1/spec.md) is the repair — a distinct outcome from a host refusing, and a member told only "could not read" cannot tell the two apart.

A `retracted` record has no body. That is an answer, not a failure: the tombstone, its version and its verdict are returned, and a consumer **MUST NOT** report it as an error.

## Definitions

**`roomId`** — the room to read from. The recipient **MUST** already hold group state for it.

**`host`** — the host to read from, as a DID. Named by the caller because **nothing maps a room to its host**: a room is portable, so a remembered host goes stale silently, and a room may be registered with more than one — a mirror serving reads while its primary takes writes. A caller who names the wrong host learns so as a refusal from a party that does not serve this room, which is loud and immediate.

**`plaintext`** / **`cleartext`** (response) — exactly one is present. `plaintext` is base64url, from a sealed tier, opened; `cleartext` is the body of an `open` room, where there was nothing to open. Carried differently on purpose: a member ought to be able to tell that this room's host can read what they just read.

**`verification`** (response) — REQUIRED. An agent that returns a record without saying what it checked has made the member's decision for them.

## Request

A **Member** sends this to their **KeyHolder**. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A member reads one record

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/rooms/keys/read/0.1#request",
  "issuer": "did:example:member",
  "recipient": "did:example:keyholder",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "host": "did:webvh:example.com:northwind-community",
    "key": "giXFLTGBdnnQJRoIsktuIg"
  }
}
```

## Response

The **KeyHolder** responds, using the sub-schema reachable via `$anchor: "response"`. Failures use a `trust-task-error` document rather than a `#response`, per [SPEC §6.4](/SPEC.md#64-error-documents).

### The record, and a host that checks out

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/keys/read/0.1#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:member",
  "issuedAt": "2026-01-01T00:00:02Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "key": "giXFLTGBdnnQJRoIsktuIg",
    "version": 412,
    "status": "active",
    "updatedAt": "2026-01-01T00:00:00Z",
    "plaintext": "eyJ0aXRsZSI6IlByaWNpbmcgaG9sZHMifQ",
    "verification": {
      "trace": "verified",
      "priorRoots": "agree",
      "head": {
        "dataCommitment": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
        "recordCount": 118,
        "headVersion": 412
      }
    }
  }
}
```

### The same record, from a host that has been caught

The record is still returned. The verdict is what changed.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/rooms/keys/read/0.1#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:member",
  "issuedAt": "2026-01-01T00:00:02Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "key": "giXFLTGBdnnQJRoIsktuIg",
    "version": 412,
    "status": "active",
    "updatedAt": "2026-01-01T00:00:00Z",
    "plaintext": "eyJ0aXRsZSI6IlByaWNpbmcgaG9sZHMifQ",
    "verification": {
      "trace": "verified",
      "priorRoots": "conflict",
      "head": {
        "dataCommitment": "zQmXo1sV5aJ7bT2kQdF9wRnPzYcH4uMgLtEjV6NrBqWsDpK",
        "recordCount": 117,
        "headVersion": 412
      }
    }
  }
}
```

`trace: "verified"` beside `priorRoots: "conflict"` is not a contradiction, and reading it as one is the mistake this shape exists to prevent. The trace proves the record sits under *the root the host just asserted*; the conflict says that root is not the one this host asserted for the same state before. **A host serving a private view of a room can build a consistent tree over it and trace every record in it perfectly.**

## Security & Privacy

### Data carried

The request carries three identifiers. What crosses the network *because of it* is the more interesting half: the recipient presents its principal's membership and authority to the named host, and receives back a record — ciphertext on the sealed tiers, which it then opens.

The response carries **plaintext**, which on a sealed tier is material the room withholds from its own host. It crosses one boundary, from the member's agent to the member, and a producer **MUST NOT** put record content in `ext`.

### Correlation

To an observer of the recipient's outbound traffic, this is a member of that room contacting that host. To the host, it is a member reading — which is what the presentation already said, and what reading a room requires.

The shape is informative in a way `backfill`'s is not: backfill happens about once, while this happens **whenever a member looks at something**. A recipient that turns this into a poll converts an access pattern into a timeline of what its principal was interested in, held by the host. A consumer **SHOULD NOT** poll, and **SHOULD** prefer one `rooms/keys/browse` and a read of what the member actually opened.

### Retention

The record is returned and not kept. The presentation minted for the fetch is single-purpose and **SHOULD NOT** be retained past the request it was made for.

What a recipient **MAY** retain, and what `verification.priorRoots` depends on, is what the host asserted about the room: `(headVersion, dataCommitment)`, a few dozen bytes per observation. That is not the record and does not disclose its content — but it is a record of *when this member read this room*, so a recipient **SHOULD** keep it bounded and **SHOULD NOT** keep it beyond the member's membership.

### Consent/purpose

The record is fetched so that a member's agent can read the room its principal is a member of. It confers nothing the member did not already hold: they hold a `read` grant the room issued, and this exercises it.

This specification describes purpose only. Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that consent, approval or a step-up is required.
