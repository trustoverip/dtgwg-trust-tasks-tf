---
slug: rooms/keys/file-key
version: "0.1"
title: Rooms Keys — File Key
summary: "A member's own client asks the holder of the member's room keys for one file's key — bound to the room, the epoch and the file, opening that file and nothing else — so file bytes never pass through the key holder."
status: draft
targetFrameworkVersion: "0.6.0"
category: key-management
keywords:
  - room
  - file
  - key-release
  - custody
  - encryption
  - wallet
parties:
  - role: Member's client
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Oracle
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "The oracle releases key material derived from its principal's room keys. A request whose integrity depended on the transport would let a compromised channel choose which file's key is released, and to whom."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Consequential, because the response discloses a secret, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed request would release a key at a time the principal did not choose — after the caller's authorization was withdrawn, for instance."
sideEffects:
  level: none
  rationale: "A pure derivation from keys the oracle already holds. Nothing is stored or changed; the same request always yields the same key."
subjectPath: /roomId
exposure:
  discloses: secret
  ingests: none
  actsAsSubject: false
  rationale: "Returns a file key: secret material that decrypts one file in the room, indefinitely. That is the whole of what crosses — never the room's storage key, from which every file's key derives, and never any file's bytes, which the oracle does not see."
retention:
  class: transient
  rationale: "The oracle derives the key on demand and keeps nothing but an audit line. The caller holds it for the duration of one encryption or decryption and discards it."
errorCodes:
  - code: rooms/keys/file-key:notAuthorized
    meaning: "The caller is not authorized to seal or open files for this room."
    retryable: false
  - code: rooms/keys/file-key:unknownEpoch
    meaning: "The oracle cannot derive a key for the requested epoch. `details.reason` says why: `notDelivered` — the room has moved to an epoch whose commit has not reached the oracle yet, so retrying after it arrives may succeed; `beyondChain` — the oracle's epoch chain reaches back only to `details.earliestEpoch`, and the file was sealed before the principal could read it, or before the room pruned its chain."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [reason]
      properties:
        reason:
          type: string
          enum: [notDelivered, beyondChain]
        earliestEpoch:
          type: integer
          minimum: 1
related:
  - rooms/keys/open
  - rooms/keys/seal
  - rooms/blobs/get
  - rooms/blobs/upload/begin
  - rooms/records/put
---

## Abstract

A data room can hold files: a record whose sealed body describes the file, plus a blob — the
file's ciphertext — stored by the host apart from the record. The file is encrypted under a
**file key** of its own, derived from the room's storage key for one epoch and bound to the
room, the epoch and the file's random identifier
([`FileManifest`](../../../_shared/0.1/blobs.schema.json)).

The **Rooms Keys — File Key** Trust Task asks the party holding a member's room keys — the
oracle, typically the member's own agent — for one file's key, either to seal a file about to
be uploaded or to open one already in the room. The member's client then encrypts or
decrypts the file's bytes itself.

## Why this task releases a key, when `rooms/keys/open` deliberately does not

[`rooms/keys/open`](../../open/0.1/spec.md) is a decryption oracle: the caller sends a
sealed record and gets its plaintext back, and **the key never crosses**. That is the right
shape for records, and the wrong one for files, for three reasons:

1. **File bytes must not pass through the oracle.** A file can be a gigabyte. Routing it
   through the member's agent — over a mediator, chunk by chunk, in both directions — would
   put the bulk of a room's traffic through the one component whose job is to hold keys, and
   bound file sizes by what that component and its transports can carry.
2. **The released key opens exactly one file.** It is derived per file and bound to the
   room, the epoch and the file's identifier. Holding it reveals nothing about any other
   file, any record, or the room's storage key, and cannot be used to derive a sibling. The
   cost of a released key is that one file, not the room.
3. **The intended caller is the member's own client**, not a web page. Where a member uses a
   room from a browser, the caller is their wallet extension, which requests the key, runs
   the encryption in its own context and streams plaintext to and from the page. The page
   receives the bytes the member is viewing or uploading — which it must have anyway — and
   never a key.

The third point is a deployment requirement, not a formality. **A file key does not
expire.** A key leaked from a web page — by script injection on the page, a compromised
dependency, or another extension reading the page's memory — opens that file's ciphertext
**forever**, and on a store that publishes ciphertext, such as a public blob network, that
ciphertext is available to anyone forever. A key held by the member's own client, used and
discarded, limits the same compromise to what the member viewed while it lasted. An oracle
**SHOULD** therefore answer this task only to callers its principal has authorized for it
specifically, and a client **MUST NOT** hand the key to page script.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **oracle** (`recipient`) **MUST**:

1. Refuse a caller its principal has not authorized to open files for this room with
   `notAuthorized`.
2. Refuse with `malformedRequest` a `seal` request carrying `epoch`, and an `open` request
   without one.
3. Derive the key exactly as `FileManifest` defines it:
   `file_key = HKDF-SHA256(ikm = storage_key(E), salt = fileId, info = "openvtc/room/file/v1" || roomId || u64be(E))`,
   where `fileId` is the decoded 32 bytes, `roomId` the identifier's UTF-8 bytes, and
   `storage_key(E)` the room's storage key for epoch `E` — the one records sealed under `E`
   use.
   - For `seal`, `E` is the room's **current** epoch as the oracle knows it, and the
     response names it.
   - For `open`, `E` is the requested epoch. Where `E` is earlier than the oracle's current
     epoch, the oracle reaches `storage_key(E)` through the room's epoch chain, exactly as
     `rooms/keys/open` does for a record sealed under `E`.
4. Refuse with `unknownEpoch` an epoch it cannot reach, saying which of the two cases holds:
   `notDelivered` when `E` is later than any epoch it holds (a commit has not been
   delivered), `beyondChain` with `earliestEpoch` when the chain does not reach back to `E`.
   It **MUST NOT** derive under a different epoch and return that.
5. Return only the derived key and its epoch. It **MUST NOT** return `storage_key(E)`, an
   epoch-chain link, or any key that is not the one file key requested.

A conforming oracle **SHOULD** record each release in its audit trail with the room, the file
identifier, the purpose and the epoch, and **MUST NOT** record the key. That record is how a
member sees which files their agents and clients opened.

A conforming **caller** **MUST** mint a fresh random `fileId` for every file it seals and
never seal two files under one, since the STREAM construction's nonces are counters and two
files under one key would reuse them; **MUST** hold the key only for as long as one
encryption or decryption takes; and **MUST NOT** store, log, or pass it to web page script or
any other party.

Retrying is safe: the derivation is a pure function, and a repeated request returns the same
key.

## Authorization

The authority this task presupposes is the **principal's authorization of the caller** to
open files in this room — the same standing that lets a caller use
[`rooms/keys/open`](../../open/0.1/spec.md). The decision is the oracle's, made against what
its principal granted; verifying the proof establishes who asked, never that they may
([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

Where that grant is expressed as a device capability, the registered value is **`roomOpen`**
— see `Capability` in [`device/_shared`](../../../../device/_shared/0.2/device-binding.schema.json)
— for both purposes. Sealing a file needs it as much as opening one does: a sealing key is a
file key like any other, and opens the file it seals. It is distinct from `roomPresent`,
which produces presentations and decrypts nothing, and from `sign`, which would grant more
than this task needs.

## Definitions

- **Storage key** — the room's per-epoch secret that records are sealed under. Held by the
  oracle; never released by any task in this family.
- **File key** — the per-file key derived above. Opens one file's STREAM segments and
  nothing else.
- **File manifest**, **STREAM** — as in [`rooms/_shared/0.1/blobs.schema.json`](../../../_shared/0.1/blobs.schema.json).

## Request

The member's client (`issuer`) asks the oracle (`recipient`). The payload is the top-level
schema in [`payload.schema.json`](payload.schema.json).

### Sealing a new file

```json
{
  "id": "urn:uuid:3c4d5e6f-7081-4293-a4b5-c6d7e8f90a01",
  "type": "https://trusttasks.org/spec/rooms/keys/file-key/0.1#request",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:webvh:QmAgent:agent.alice.example",
  "issuedAt": "2026-10-10T09:28:00Z",
  "threadId": "urn:uuid:3c4d5e6f-7081-4293-a4b5-c6d7e8f90aff",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal",
    "fileId": "q83vEjRWeJq83vEjRWeJq83vEjRWeJq83vEjRWeJq80",
    "purpose": "seal"
  }
}
```

### Opening a file sealed under epoch 5

```json
{
  "id": "urn:uuid:3c4d5e6f-7081-4293-a4b5-c6d7e8f90a03",
  "type": "https://trusttasks.org/spec/rooms/keys/file-key/0.1#request",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:webvh:QmAgent:agent.alice.example",
  "issuedAt": "2026-10-10T09:40:00Z",
  "threadId": "urn:uuid:3c4d5e6f-7081-4293-a4b5-c6d7e8f90afe",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal",
    "fileId": "bm9ydGh3aW5kLXRlcm0tc2hlZXQtdjMtZmlsZS1pZDA",
    "purpose": "open",
    "epoch": 5
  }
}
```

## Response

The oracle answers with the sub-schema reachable via `$anchor: "response"` in
[`payload.schema.json`](payload.schema.json): the file key and the epoch it was derived under.
Failures use `trust-task-error`.

### The sealing key, under the current epoch

```json
{
  "id": "urn:uuid:3c4d5e6f-7081-4293-a4b5-c6d7e8f90a02",
  "type": "https://trusttasks.org/spec/rooms/keys/file-key/0.1#response",
  "issuer": "did:webvh:QmAgent:agent.alice.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-10T09:28:00Z",
  "threadId": "urn:uuid:3c4d5e6f-7081-4293-a4b5-c6d7e8f90aff",
  "payload": {
    "key": "Xk1jSgV0b3BzZWNyZXQtZmlsZS1rZXktZXhhbXBsZTA",
    "epoch": 7
  }
}
```

### Refused: the chain does not reach back that far

```json
{
  "id": "urn:uuid:3c4d5e6f-7081-4293-a4b5-c6d7e8f90a04",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:webvh:QmAgent:agent.alice.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-10T09:40:00Z",
  "threadId": "urn:uuid:3c4d5e6f-7081-4293-a4b5-c6d7e8f90afe",
  "payload": {
    "code": "rooms/keys/file-key:unknownEpoch",
    "message": "This file was sealed under epoch 5; the chain reaches back only to epoch 6.",
    "retryable": false,
    "details": { "reason": "beyondChain", "earliestEpoch": 6 }
  }
}
```

## Security & Privacy

**One file per release, and never the storage key.** The derivation is the containment: a
released file key cannot be turned into another file's key, a record's key, or the storage
key, because HKDF is one-way and the storage key never leaves the oracle. An oracle that
offered a room-wide key "for efficiency" would make every client compromise a room
compromise.

**Keep the key out of web pages.** Discussed above, and the most important deployment rule
here: a file key never expires, so where it ends up determines how long a compromise lasts.
The member's own client — a wallet extension, a CLI, the agent itself — does the
cryptography; a page gets plaintext, never a key.

**Removal is forward-only, as for records.** A member removed from the room cannot obtain
keys for files sealed after their removal, because later epochs derive from storage keys
they cannot reach. A file key they obtained before removal still opens that file; on a store
where the ciphertext remains available, so does the file. The only erasure for such a file
is cryptographic — pruning the room's epoch chain so that nobody can derive its key again.

**Never derive under the wrong epoch.** A key derived under an epoch other than the file's
would simply fail to open it — but a sealing caller given a stale epoch would produce a file
that members who joined since cannot open, and that a member removed since still can. The
oracle uses the current epoch for sealing and the stated one for opening, and refuses rather
than substitutes.

### Data carried

In: a room identifier, a file identifier, a purpose and possibly an epoch — no file bytes, no
file name. Out: one 32-byte key and an epoch.

### Correlation

The oracle learns which files its principal's clients sealed and opened, and when: a complete
picture of the principal's own file activity, which it is the principal's own infrastructure
to hold. It learns nothing about other members, and nothing about file contents or names. The
`fileId` is random per file and does not link rooms.

### Retention

The oracle keeps nothing but an audit line naming the room, file, purpose and epoch — never
the key. The caller discards the key after one encryption or decryption.

### Consent/purpose

The caller acts for the principal whose room keys the oracle holds, within the authorization
the principal granted it — for this task, the `roomOpen` capability or its equivalent. The
purpose is limited to sealing or opening the one named file.
