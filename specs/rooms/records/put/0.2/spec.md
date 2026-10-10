---
slug: rooms/records/put
version: "0.2"
title: Rooms Records — Put
summary: "A member writes a record to a data room, optionally carrying a file whose ciphertext the host stores apart from it, authorized by an authority chain the room itself issued rather than by anything the host stores."
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - room
  - record
  - write
  - file
  - blob
  - attenuation
  - encryption
parties:
  - role: Member
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Host
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "A write mutates durable shared state that other members will read and act on, and a record naming a blob keeps that blob alive and charged to someone; transport-independent integrity is required."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A write is authorized by a presentation whose validity is time-bounded, and a replayed put would restore a superseded record — and with it a reference to a blob a later version had released."
sideEffects:
  level: mutating
  rationale: "Stores or replaces a record in a room, and moves the reference counts of the blobs the old and new versions name."
subjectPath: /roomId
exposure:
  discloses: none
  ingests: personal
  actsAsSubject: false
  rationale: "On an `open` room the host reads record content in the clear. On `attributed` and `private` rooms it receives ciphertext and learns only the key, version, epoch and — when the record carries a file — which stored blob it names. The presentation discloses the acting member on `open` and `attributed`; on `private` it discloses only that some member acted."
retention:
  class: durable
  rationale: "A record persists until a member overwrites it or retracts it, and keeps the blob it names stored for as long; outliving the session that wrote it is the point of the task."
errorCodes:
  - code: rooms/records/put:notAuthorized
    meaning: "The presentation does not confer `write` at this room's scope, or its chain does not reach the room."
    retryable: false
  - code: rooms/records/put:versionConflict
    meaning: "`expectedVersion` did not match. The response carries the current version and record."
    retryable: false
  - code: rooms/records/put:chainTooDeep
    meaning: "The authority chain exceeds the maximum of 8 links."
    retryable: false
  - code: rooms/records/put:subjectBindingMissing
    meaning: "A `private` room presentation omitted the required same-subject proof."
    retryable: false
  - code: rooms/records/put:epochMismatch
    meaning: "The record was sealed under an epoch that is not the room's current one."
    retryable: false
  - code: rooms/records/put:recordTooLarge
    meaning: "The record exceeds the host's per-record limit."
    retryable: false
  - code: rooms/records/put:blobNotFound
    meaning: "A `blobs` entry names no blob committed in this room — never uploaded, still uploading, uploaded to another room, or already collected. Answered identically in every case, so the refusal does not say whether the blob exists elsewhere."
    retryable: false
related:
  - rooms/records/get
  - rooms/records/list
  - rooms/records/curate
  - rooms/blobs/upload/begin
  - rooms/blobs/upload/commit
  - rooms/blobs/get
  - rooms/epoch/mint
---

## Abstract

The **Rooms Records — Put** Trust Task writes a record to a **data room**: a shared space
whose access is governed by credentials the *room itself* issues, and whose contents a host
may be unable to read.

The property that distinguishes this family from every other stored-data task in this
registry is that **a host never consults a member list of its own**. Authorization is a
presentation carrying a membership credential and an authority chain, verified against the
room's identifier. A host that speaks this family can therefore host *any* room without
knowing anything about who belongs to it, and a room can move between hosts without a
single credential being reissued.

Version 0.2 lets a record **carry a file**. A file is a record plus a blob: the record is an
ordinary record, versioned, curated and sealed as before, whose sealed body describes the
file ([`FileManifest`](../../../_shared/0.1/blobs.schema.json)); the blob is the file's
ciphertext, uploaded beforehand with [`rooms/blobs/upload/*`](../../../blobs/upload/begin/0.1/spec.md)
and named on the record by its `blobRef`. The record is what members see, curate and sync;
the blob is what the host stores and counts.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

**Why a new version and not an edit to 0.1.** Adding an optional member is ordinarily a
minor, compatible change, and 0.1 is a draft that could take one in place. This one is
not compatible in the direction that matters: a 0.1 host does not know `blobs`, and a
host that ignored it would store a record **pointing at a blob it never checked** — one
that was never uploaded, belongs to another room, or has already been collected — and
would neither keep that blob alive nor charge anyone for it. A producer sending `blobs`
must therefore reach a host that has agreed to enforce it, and the type URI is how it
knows. A host that implements 0.2 **SHOULD** continue to accept 0.1, which cannot name a
blob; a 0.1 rewrite of a record whose current version names a blob releases that blob, as
any rewrite that omits `blobs` does.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the member, or an agent acting under an attenuated credential)
**MUST**:

1. Emit a *Trust Task document* of type `https://trusttasks.org/spec/rooms/records/put/0.2`.
2. Present the **entire** authority chain, leaf first, ending in a credential issued by the
   room. A producer MUST NOT rely on the host resolving a link it was not given.
3. Seal the record and set `sealed` on an `attributed` or `private` room, or set
   `cleartext` on an `open` one. Exactly one of the two.
4. Bind the sealed record's AEAD associated data to `roomId`, `key`, `version` and `epoch`.
5. On a `private` room, include `presentation.subjectBinding`.
6. When the record carries a file: commit its blob in this room first, put the file's
   [`FileManifest`](../../../_shared/0.1/blobs.schema.json) in the record's body (inside
   the seal on `attributed` and `private` rooms), and name the same `blobRef` in `blobs`.
   A producer **MUST NOT** name a blob in `blobs` that the record's body does not describe:
   the host cannot tell, and a reader would find a blob kept alive by a record that never
   mentions it.

A conforming **consumer** (the host) **MUST**:

1. Verify every link in the chain, and **reject the chain if any link widens** what its
   parent conferred — in actions, in scope, or in validity period. A host that verifies only
   the presented credential has verified nothing: anyone can mint a well-formed authority
   credential naming any scope, and what makes it worthless is that its chain does not reach
   the room.
2. Reject a chain of more than **8** links. Verification is linear in chain length and runs
   on every operation.
3. **Never dereference** a chain link's `parent` over the network. Doing so would make
   verification depend on availability, turn the identifier into a request the host can be
   induced to make against an address the *producer* chooses, and signal credential use to
   whoever hosts that identifier.
4. On a `private` room, **reject a presentation with no `subjectBinding`**. Without it two
   parties pool credentials — one contributes membership, the other authority — and the
   combination verifies as a single party holding both.
5. Reject a record sealed under an epoch that is not current.
6. Return the current version and record with a `versionConflict`, rather than a bare
   rejection.
7. Refuse with `blobNotFound` a record whose `blobs` names any blob that is not committed
   **in this room**. A record can never point at nothing, at an upload still in progress, or
   at another room's file. The refusal is the same whichever of those holds.
8. Keep a **reference count** per blob, changed atomically with the record write:
   - a put whose new version names a blob the previous version did not **references** it;
   - a put whose new version omits a blob the previous version named **releases** it;
   - a retraction ([`rooms/records/curate`](../../curate/0.1/spec.md) to `retracted`)
     releases every blob the record named, since a tombstone keeps no body.

   A blob whose count reaches zero is **orphaned**. It **MUST** stop counting against the
   room's and its uploader's usage at once — a member who deletes a file has the space
   back immediately — and **MAY** be deleted from storage after a grace window the host
   chooses. Deprecating a record releases nothing: its body is retained, and so is its file.
9. Accept a record naming a blob another record of the same room already names. The count
   is per blob, not per record, so two records sharing one blob is well-defined, and a blob
   is orphaned only when the last of them releases it.

A conforming host **MUST NOT** require a session or account of its own as a condition of
serving a `private` room. Authorizing by session would record which member acted on every
operation, and a period of such records reconstructs the membership the tier exists to
withhold — without breaking any cryptography.

## Authorization

Authority is **conferred by the room**, never by the host: the presentation's authority
chain, verified link by link to a credential the room issued, must confer `write` at this
room's scope, and its leaf's subject must be the party the host authenticated for this
request. The host consults no access-control list of its own (the family's invariant that
room authorization never uses host ACLs). Naming a blob needs no further authority: the
blob was committed in this room by someone the room authorized to write, and the reference
only keeps it stored.

## Definitions

- **Record**, **authority chain**, **presentation**, **epoch** — as in
  [`rooms/_shared/0.1/room.schema.json`](../../../_shared/0.1/room.schema.json).
- **Blob**, **`blobRef`**, **file manifest** — as in
  [`rooms/_shared/0.1/blobs.schema.json`](../../../_shared/0.1/blobs.schema.json).
- **Committed in this room** — uploaded through `rooms/blobs/upload/*` with this `roomId`,
  verified against its manifest at commit, and not yet collected.
- **Orphaned** — committed, but named by no current record version.

## Request

A member (`issuer`) writes to the room's host (`recipient`). The payload is the top-level
schema in [`payload.schema.json`](payload.schema.json).

### A sealed record carrying one file

The file's description — its name, type, plaintext size and digest, and the same `blobRef`
— is inside `sealed.ciphertext`. Only the reference is in the clear.

```json
{
  "id": "urn:uuid:7d0b3a52-1f6e-4c1b-9a51-0d0f3f3c2a10",
  "type": "https://trusttasks.org/spec/rooms/records/put/0.2#request",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:web:rooms.northwind.example",
  "issuedAt": "2026-10-10T09:30:00Z",
  "threadId": "urn:uuid:7d0b3a52-1f6e-4c1b-9a51-0d0f3f3c2aff",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal",
    "key": "giXFLTGBdnnQJRoIsktuIg",
    "presentation": {
      "membership": "eyJhbGciOiJFZERTQSJ9.membership.sig",
      "authority": [
        "eyJhbGciOiJFZERTQSJ9.authority-leaf.sig",
        "eyJhbGciOiJFZERTQSJ9.authority-root.sig"
      ]
    },
    "expectedVersion": 0,
    "sealed": {
      "ciphertext": "c2VhbGVkLXJlY29yZC1ib2R5LXdpdGgtZmlsZS1tYW5pZmVzdA",
      "nonce": "AAAAAAAAAAAAAAAB",
      "epoch": 7
    },
    "blobs": [
      "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR"
    ]
  }
}
```

## Response

The host (`recipient` of the request, now responding) answers with the sub-schema reachable
via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json): the record's key,
the version assigned and the epoch it is stored under. Failures use `trust-task-error`.

### Stored

```json
{
  "id": "urn:uuid:7d0b3a52-1f6e-4c1b-9a51-0d0f3f3c2a11",
  "type": "https://trusttasks.org/spec/rooms/records/put/0.2#response",
  "issuer": "did:web:rooms.northwind.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-10T09:30:01Z",
  "threadId": "urn:uuid:7d0b3a52-1f6e-4c1b-9a51-0d0f3f3c2aff",
  "payload": {
    "key": "giXFLTGBdnnQJRoIsktuIg",
    "version": 42,
    "epoch": 7
  }
}
```

### Refused: the blob is not in this room

```json
{
  "id": "urn:uuid:7d0b3a52-1f6e-4c1b-9a51-0d0f3f3c2a12",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:web:rooms.northwind.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-10T09:30:01Z",
  "threadId": "urn:uuid:7d0b3a52-1f6e-4c1b-9a51-0d0f3f3c2aff",
  "payload": {
    "code": "rooms/records/put:blobNotFound",
    "message": "No blob with that reference is committed in this room.",
    "retryable": false
  }
}
```

## Security & Privacy

**A host verifies chains; it does not keep a roster.** Authorization is decided entirely by
credentials the room issued. A host that consults state of its own has made the room
unmovable and has made itself part of the membership.

**Chain verification is the security of this family.** Anyone can mint a well-formed
authority credential naming any scope and any actions; what makes it worthless is that its
chain does not reach the room. Every link is verified, and any link that widens actions,
scope, or validity beyond its parent invalidates the chain. Chain depth is capped at 8,
and parents are never dereferenced, for the reasons given under Conformance.

**A record can only name what this room holds.** Without the `blobNotFound` check a member
of one room could reference — and so keep alive, and be served — a blob uploaded to another
room on the same host, or claim a file that was never uploaded. The check is against
blobs committed *in this room*, and its answer is the same whether the blob exists
elsewhere or nowhere, so it is not an oracle for other rooms' contents.

**The host cannot substitute a file.** The `blobRef` in the clear is also inside the sealed
body the author signed, beside the plaintext digest. A host serving a different blob under
the record is caught by the reader: either the ciphertext does not match the manifest the
`blobRef` names, or the decrypted bytes do not match the signed digest.

**Credential pooling.** Where membership and authority are presented with the subject
withheld, a host that does not require proof that both describe the same subject lets two
parties combine one's membership with the other's authority and present as a single party
holding both.

### Data carried

On an `open` room, the record itself in the clear. On `attributed` and `private` rooms, only
ciphertext plus its key, version and epoch — and, when the record carries a file, the
`blobRef` in the clear. That discloses to the host **that the record has a file**, and,
through the blob it names, **the file's ciphertext size**; never its name, type, contents or
plaintext digest, which are sealed. A room that wants sizes blurred pads its files
(`FileManifest.padding`). The presentation names the acting member on `open` and
`attributed`; on `private` it does not.

### Correlation

On `attributed`, a host can build a per-member write history, now including which members
attach files and how large they are. On `private` it cannot attribute writes, but it still
sees when they happen, how large records and blobs are, and from where. A `blobRef` is a
digest of ciphertext under a per-file key, so two rooms holding the same document hold
different blobs, and rooms cannot be linked by the files they share.

### Retention

Records are durable by design and persist until overwritten or retracted. A retraction is a
tombstone: the body goes, the key and version stay, and every blob the record named is
released. A released blob stops counting against usage at once and is deleted from storage
after the host's grace window. On a store that publishes ciphertext, deletion frees the
storage but cannot recall copies already made; there the only erasure is cryptographic —
pruning the epoch chain so nobody can derive the file key.

### Consent/purpose

A member writes into a room they joined by accepting an invitation; the membership
credential pair is that consent. Nothing here authorizes a host to read, index, scan or
derive from record or file content beyond storing it and serving it back — which, on the
sealed tiers, it could not do anyway.
