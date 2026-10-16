---
slug: rooms/keys/profile/write
version: "0.1"
title: "Rooms Keys — Profile — Write"
summary: "A member holding curate or admin asks their own agent to set a room's name and description: the agent signs the profile as the member and seals it into the room where only members can find it, so members see what the room is and its host cannot."
status: draft
targetFrameworkVersion: "0.6.0"
category: ai-agents
keywords:
  - room
  - profile
  - name
  - description
  - write
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
    The recipient signs as the member a statement every member of the room will be shown, and presents the member's authority to store it. A request whose integrity depended on the transport would let a compromised channel choose what a room is called, in the member's name.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed write would put back a name the room's curators have since changed.
sideEffects:
  level: mutating
  rationale: >-
    Writes, or retracts, one record in the room at its host, as the member. Reversible by another write; the earlier profile stays readable at earlier epochs on a `chained` room.
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    `actsAsSubject` is the task: the recipient signs the profile as its principal and presents its principal's authority to the host. The request carries a name, a description, tags and an emoji, written to be shown to the room's members; the response carries where the record now is.
retention:
  class: transient
  rationale: >-
    The recipient builds, signs, seals and stores one record and keeps none of it. The record is durable at the host, which is `rooms/records/put`'s retention, declared there.
errorCodes:
  - code: rooms/keys/profile/write:notAMember
    meaning: "The recipient holds no membership of this room for its principal — no group state on a sealed tier, no membership credential on an `open` one."
    retryable: false
  - code: rooms/keys/profile/write:noAuthority
    meaning: "The authority its principal holds for this room does not suffice: writing a profile takes `write`, to store it, and `curate` or `admin`, for readers to accept it; clearing one takes `curate`. Refused here rather than stored, because a profile readers will refuse is worse than none."
    retryable: false
  - code: rooms/keys/profile/write:hostUnreachable
    meaning: "The named host could not be resolved, advertises no transport this recipient speaks, or did not answer. Nothing is known to have been stored."
    retryable: true
  - code: rooms/keys/profile/write:hostRefused
    meaning: "The host answered and declined — commonly that the room's epoch has lapsed and it is read-only until renewed, that it does not serve this room, or that it refused the presentation. Its own code and message are carried in `details`."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        code:
          type: string
          maxLength: 256
        message:
          type: string
          maxLength: 500
  - code: rooms/keys/profile/write:contended
    meaning: "Other writes to the room kept landing between sealing the profile and storing it, so the host assigned a version other than the one the record was sealed for, and the record it stored does not open. Readers ignore such a record and fall back to the profile before it. Retrying is safe."
    retryable: true
related:
  - rooms/keys/profile/read
  - rooms/keys/seal
  - rooms/records/put
  - rooms/records/curate
  - rooms/owner/group/add
  - rooms/owner/group/remove
  - rooms/owner/group/renew
  - vtc/rooms/community/create
  - vtc/rooms/community/update
---

## Abstract

A room's members need to see what the room is — its name, what it is for — and its host, on a sealed tier, must not. **Rooms Keys — Profile — Write** lets a member entitled to name the room set its **profile** ([`RoomProfile`](../../../../_shared/0.1/room.schema.json)): their key holder signs it as them, with their authority chain inside it, seals it under the room's current epoch, and stores it at the host like any record — at a key derived from that epoch's storage key, so that to the host it is one more opaque record ([`RoomProfileStatement`](../../../../_shared/0.1/room.schema.json)).

It also carries the rule that keeps the profile readable: **a key holder that advances a room's epoch re-writes the profile under the new one**, so a member who joins later, or reads only from where they joined, still finds it.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Why a task, and not a reserved record key

The obvious design is a record at a well-known key — `profile` — written with [`rooms/records/put`](../../../../records/put/0.2/spec.md) and read with [`rooms/keys/read`](../../../read/0.1/spec.md), no new task at all. It is rejected on three counts.

- **It tells the host which record names the room.** The family requires a sealed room's keys to be opaque ([`RecordMetadata.key`](../../../../_shared/0.1/room.schema.json)) because a descriptive key leaks what the ciphertext beside it protects. A fixed key leaks less than a slug, but exactly what matters here: which record is the room's name, when it changes, who changed it on an `attributed` room, and roughly how long it is — and it makes withholding or rolling back the room's name a one-line policy for a host.
- **A member's surface cannot compute anything better.** A key hidden from the host must be derived from something the host does not hold, and on a sealed tier that is the epoch's storage key — which never leaves the member's key holder. So the key holder has to be in the loop for a sealed room either way; a task makes that one round trip instead of a key-disclosure call followed by a read.
- **Checking who wrote it is the hard part, and belongs where the keys are.** A host stores a profile under `write`, since it cannot tell the profile from other records, so readers must check the profile's own authority chain for `curate` or `admin`. That is chain verification — the host's work in the rest of the family — and a wallet or a portal page should not have to carry it.

The alternatives considered and set aside: a profile in the room's credentials, or in its host record, or in `rooms/owner/register` — all host-visible, and the last explicitly forbidden to carry a name; a random key members find by opening every record — a cost linear in the room, paid by every new member; a profile signed by the room itself rather than by a member — it would need the room's own key for every change, and on an `attributed` room the host records the presenting member as author, which a room-signed body would contradict.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** **MUST** send exactly one of `profile` and `clear`, and **MUST NOT** put personal data or a credential in any member of `profile`.

A conforming **recipient** (the key holder):

1. **MUST** refuse with `notAMember` a room it holds no membership of for its principal.
2. **MUST** refuse with `noAuthority`, before contacting the host, when the authority its principal holds for the room confers no `write`, or neither `curate` nor `admin` — or, for `clear`, no `curate`.
3. To write, on an `attributed` or `private` room, **MUST** use the epoch `E` it is at, store the profile at `profile_key(E)`, and build a [`SealedRecordBody`](../../../../_shared/0.1/room.schema.json) whose `author` is its principal and whose `roomProfile` is a `RoomProfileStatement` carrying `profile`, the current time as `issuedAt`, and its principal's authority chain conferring `curate` or `admin`, leaf first, as the credentials themselves. It **SHOULD** carry the shortest such chain it holds and **MAY** attenuate it to `curate` alone. It **MUST** sign the body as its principal as [`rooms/keys/seal`](../../../seal/0.2/spec.md) signs under `signAsPrincipal`, **SHOULD** pad it as `RoomProfileStatement` describes, and **MUST** seal it under `E` with associated data binding `roomId`, the key, the version and `E`.
4. To write on an `open` room, **MUST** build the same signed body for the key `room-profile` and store its UTF-8 JSON as `cleartext.body`, with `cleartext.title` the profile's `name`.
5. **MUST** store the record with [`rooms/records/put`](../../../../records/put/0.2/spec.md) as its principal, presenting a `write` grant and nothing wider, with the principal as the document's `issuer` and signer, and its own identifier reaching the host nowhere — as [`rooms/keys/read`](../../../read/0.1/spec.md) requires of a read.
6. **MUST** compare the version the host assigned with the version it sealed for. On a mismatch the record does not open: it **MUST** build, sign, seal and store the profile again for the next version it predicts, and after attempts of its own bounding **MUST** answer `contended` rather than success. It **MUST NOT** answer `written` for a record that does not open.
7. To clear, **MUST** retract the record at the current epoch's profile location with [`rooms/records/curate`](../../../../records/curate/0.1/spec.md), presenting a `curate` grant and nothing wider. Where that location holds no record, it **MUST** first store one — a profile is required to retract — and **MAY** do so by writing the current profile there.
8. **MUST NOT** retain the profile, the signed body, the ciphertext or the presentation beyond the request.

### Keeping the profile readable across epochs

A profile written under epoch `E` is at `profile_key(E)`, sealed under `E`. A member who joins at a later epoch finds it only by walking back, and on a `fromJoin` room nobody can walk back below the epoch they joined at. So the profile is re-written at every advance, by the party that makes the advance — on a sealed room, the owner's key holder, which is the room's sole committer.

A key holder that implements this task, and advances a sealed room's epoch for its principal by any task at any version — [`rooms/owner/group/add`](../../../../owner/group/add/0.2/spec.md), [`group/remove`](../../../../owner/group/remove/0.2/spec.md), [`group/renew`](../../../../owner/group/renew/0.1/spec.md) or another:

1. **SHOULD**, before the advance, resolve the room's profile as [`rooms/keys/profile/read`](../../read/0.1/spec.md) does. Before, because after it a key holder of a `fromJoin` room can no longer open what was sealed under the epoch it left.
2. When that found a profile and its principal holds the authority item 2 above requires, **SHOULD**, once the host has accepted the advance, write the same `RoomProfile` under the new epoch as its principal, as this task writes one. The re-written statement is its principal's, signed now; the earlier author's statement stays where it was, under the earlier epoch.
3. **MUST NOT** let the re-write fail, delay or roll back the advance. A membership change is the act; the profile follows it. A re-write that fails **SHOULD** be retried.
4. When the resolve answered `cleared` or `absent`, **MUST NOT** write a profile on the room's behalf.
5. **SHOULD NOT** retract the profile under the earlier epoch. On a `chained` room it is the room's history, and a retraction landing with the re-write would tell the host which two records are the same thing.

A reader of a room whose owner does not do this reads `absent` once the room advances past its profile, or `stale` where it can walk back. A surface **SHOULD** fall back to a name it was given another way — the label of the offer that brought the member in, or one the member chose — and **SHOULD** say the room has not given itself a name this member can read, rather than inventing one.

## Authorization

**The entitlement is being — or acting as — the recipient's own principal**, as with every custody task in this family. The recipient signs as its principal and presents its principal's authority; it **MUST** honour this only from a party it accepts as acting **as** its principal, and **MUST** refuse a caller to which the principal has delegated narrower standing with the standard `permissionDenied` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)). An agent acting under an attenuated chain may hold `curate` of its own; it writes a profile, if at all, as its own author with its own key holder, never as the principal.

**What the room requires is the principal's.** Storing the record takes `write`, which the host checks. Being *accepted* as the room's profile takes `curate` or `admin`, which every reader checks from the chain the statement carries ([`RoomProfileStatement`](../../../../_shared/0.1/room.schema.json), item 3). Item 2 refuses a write the readers would refuse, so a member is told at once rather than discovering that nobody sees what they wrote.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient` or `proof` establishes *who sent this* and *that it is unaltered* — never that they were entitled to the outcome.

## Definitions

**`roomId`**, **`host`** — as in [`rooms/keys/read`](../../../read/0.1/spec.md).

**`profile`** — a [`RoomProfile`](../../../../_shared/0.1/room.schema.json). It replaces the whole of the room's profile.

**`clear`** — `true` retracts the current epoch's profile record.

**Profile location**, **`profile_key(E)`** — see [`RoomProfileStatement`](../../../../_shared/0.1/room.schema.json).

## Request

A **Member** sends this to their **KeyHolder**. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Name a room

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/rooms/keys/profile/write/0.1#request",
  "issuer": "did:example:member",
  "recipient": "did:example:keyholder",
  "issuedAt": "2026-10-16T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "host": "did:webvh:example.com:community",
    "profile": {
      "name": "Northwind renewal",
      "description": "Everything for the 2027 Northwind contract renewal: pricing, legal mark-ups and the call notes.",
      "tags": ["deal", "renewal"],
      "emoji": "🤝"
    }
  }
}
```

### Clear a room's profile

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/rooms/keys/profile/write/0.1#request",
  "issuer": "did:example:member",
  "recipient": "did:example:keyholder",
  "issuedAt": "2026-10-16T10:10:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000003ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "host": "did:webvh:example.com:community",
    "clear": true
  }
}
```

### What the recipient signs and seals

Not a wire document — the plaintext of the record, as every reader opens it. On an `open` room this is `cleartext.body`.

```json
{
  "roomId": "did:webvh:example.com:rooms:northwind",
  "key": "Qm4c8XkR2pVt7sLw0aJbYg",
  "version": 214,
  "author": "did:example:member",
  "roomProfile": {
    "profile": {
      "name": "Northwind renewal",
      "description": "Everything for the 2027 Northwind contract renewal: pricing, legal mark-ups and the call notes.",
      "tags": ["deal", "renewal"],
      "emoji": "🤝"
    },
    "issuedAt": "2026-10-16T10:00:01Z",
    "authority": [
      "eyJhbGciOiJFZERTQSJ9.authority-curate.sig"
    ],
    "padding": "                                        "
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:member#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3FXQjecWufY46yg5abdVZsXqLhxhueuSoZgNSARiKBk9czhSePTFehP8c3PGfb6a22gkfUKods5D2UxazYzwBJMr"
  }
}
```

## Response

The **KeyHolder** answers with the sub-schema reachable via `$anchor: "response"`. Failures use a `trust-task-error` document rather than a `#response`, per [SPEC §8.1](/SPEC.md#81-the-trust-task-error-specification).

### Written

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/keys/profile/write/0.1#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:member",
  "issuedAt": "2026-10-16T10:00:02Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "outcome": "written",
    "key": "Qm4c8XkR2pVt7sLw0aJbYg",
    "version": 214,
    "epoch": 6
  }
}
```

### Refused: the member may write, but not name the room

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000004",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:member",
  "issuedAt": "2026-10-16T10:00:02Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "code": "rooms/keys/profile/write:noAuthority",
    "message": "Naming this room takes curate or admin; you hold write.",
    "retryable": false
  }
}
```

## Security & Privacy

**The name never reaches the host on a sealed tier.** The profile is sealed like any record, at a key the host cannot compute or tell from a random one, padded so its length says little. It is never put in a credential, in the room's host record, in `rooms/create` or [`rooms/owner/register`](../../../../owner/register/0.1/spec.md), or in any cleartext member of anything sent to the host. What the host can still see is pattern: a record written by the owner just after each epoch advance. It can guess; it cannot read, and it can do nothing to that record it could not do to any other.

**A member holding only `write` can overwrite the profile's record, and cannot rename the room.** The host cannot stop the overwrite, because it cannot tell the record is the profile — any writer can already overwrite any record key in a room. What it cannot do is make readers accept it: readers ignore a statement whose chain lacks `curate` or `admin`, fall back to the profile before it, and say who tried. Somebody holding `curate` repairs it by writing again.

**Every writer is attributed.** The statement is signed by its author and carries their chain, so every member who reads the profile can see who named the room and on what authority — on a `private` room too, where the host learns nothing of who wrote anything.

### Data carried

The request carries a profile — a name, a description, tags and an emoji, all written to be shown to the room's members — or a request to clear it. The recipient learns it, as it learns everything its principal writes. The host receives ciphertext on a sealed tier and the profile in the clear on an `open` one. A producer **MUST NOT** put personal data or a credential in the profile, and **MUST NOT** put any of it in `ext`.

### Correlation

On an `attributed` room the host records which member stored the record, as for any write, and learns that this member wrote *a* record. Under the epoch-advance rule the owner's key holder writes one at every advance, which is the pattern described above. A surface **SHOULD NOT** write the profile more often than it changes.

### Retention

The recipient keeps nothing. The record is durable at the host until overwritten or retracted; under each earlier epoch, the profile written then stays as the room's history on a `chained` room, readable by members who can derive that epoch's key.

### Consent/purpose

A member entitled to govern or curate a room describing it to the room's own members. It confers nothing on anyone: what a member can do in a room is what their credentials say, whatever the room is called.

This specification describes purpose only. Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that consent, approval or a step-up is required.
