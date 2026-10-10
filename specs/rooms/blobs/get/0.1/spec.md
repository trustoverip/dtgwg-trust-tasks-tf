---
slug: rooms/blobs/get
version: "0.1"
title: "Rooms Blobs — Get"
summary: "A member opens a download of one file's ciphertext from a data room: the host returns the blob's manifest and a handle for fetching its chunks, to a room-issued chain conferring read."
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - room
  - file
  - blob
  - download
parties:
  - role: Member
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Host
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    The download handle is bound to the party the proof authenticates, and only that party may fetch its chunks; on a relayed transport the proof is what makes that binding hold. The response is RECOMMENDED: the manifest it carries is checked by the reader against the BlobRef it asked for, so a substituted manifest fails without the proof.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The presentation it carries is valid for a bounded time, and a replayed request must be placeable in the window in which that presentation was, or a captured request would keep opening downloads after the member's credentials lapsed.
sideEffects:
  level: none
  rationale: >-
    Opens a short-lived download handle and counts the download in the room's usage. Nothing in the room changes.
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Discloses the blob's ciphertext manifest — its size and chunk digests — to a party holding read authority in the room. Nothing about the file's contents, name or type, which are sealed in the record.
retention:
  class: exchange
  rationale: >-
    The download handle lives until it expires. The host counts the download in the room's usage figures, which keep counts, never which blob was read by whom.
errorCodes:
  - code: rooms/blobs/get:notAuthorized
    meaning: "The presentation does not confer `read` at this room's scope, its chain does not reach the room, or its leaf is not the party the host authenticated."
    retryable: false
  - code: rooms/blobs/get:notFound
    meaning: "No blob with this BlobRef is committed in this room. Says nothing about whether one exists in another room."
    retryable: false
related:
  - rooms/blobs/chunk
  - rooms/blobs/upload/begin
  - rooms/records/get
  - backup/get-chunk
---

## Abstract

A file in a data room is a record plus a blob: the record's sealed body describes the file and names its blob, and the blob is the file's ciphertext, held by the host. A member who has opened a record and wants the file asks the host for the blob it names.

The **Rooms Blobs — Get** Trust Task opens that download. It carries the room-issued authority to read and the **BlobRef** from the record. The host returns the blob's **ciphertext manifest** — size, chunk digests and whole digest, as committed at upload — and a short-lived `downloadId`. The member then fetches each chunk with [`rooms/blobs/chunk`](../../chunk/0.1/spec.md), checks it against the manifest on arrival, decrypts with the file's key, and checks the plaintext against the digest the record's author signed.

The reader trusts the host for nothing but availability. The manifest is named by its own digest, which is the BlobRef the reader already holds from a record its author signed, so a host cannot substitute one blob for another, truncate one, or reorder its chunks without the reader noticing.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the reader):

1. **MUST** present the entire authority chain, leaf first, ending in a credential issued by the room.
2. **MUST** check that the digest of the returned manifest's RFC 8785 canonicalization equals the requested BlobRef, and **MUST** refuse the download otherwise.
3. **MUST** check every chunk against the manifest on arrival and the reassembled ciphertext against its `digest`, and after decryption **MUST** check the plaintext against the file's signed digest, refusing the file on any mismatch.

A conforming **host** (`recipient`):

1. **MUST** authorize from the presentation alone, verifying every link of the chain against the room, and **MUST** refuse with `notAuthorized` unless it confers `read` at the room's scope and its leaf's subject is the party the host authenticated for this request.
2. **MUST** answer `notFound` unless the BlobRef names a blob committed **in this room** and not yet deleted from its store. It **MUST NOT** reveal whether a blob with that BlobRef exists in another room.
3. **MUST** return the manifest exactly as committed, and a fresh, unguessable `downloadId` bound to the room, the blob and the party it authenticated, with the handle's expiry.
4. **MUST** admit, on any binding, a `rooms/blobs/chunk` response up to that task's `maxDocumentBytes` toward the party that owns an open download.
5. **SHOULD** serve an orphaned blob that has not yet been collected, since a member may hold a record version that still names it; it **MAY** refuse one with `notFound`.

## Authorization

The authority this task presupposes is the **room's**: a membership credential and an authority chain the room issued, conferring `read` at the room's scope, presented by the party the host authenticated. A host never consults an access-control list of its own to decide it (rooms invariant I5). Verifying the proof establishes who asked; the chain establishes that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- **BlobRef**, **BlobManifest** — as [`rooms/_shared/0.1/blobs.schema.json`](../../../_shared/0.1/blobs.schema.json) defines them.
- **Download** — one handle for fetching a blob's chunks, identified by its `downloadId`, from get to expiry.

## Request

A member (`issuer`) asks the room's host (`recipient`) for a blob a record names. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Fetch the blob a record names

The presentation is shortened for the example.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000141",
  "type": "https://trusttasks.org/spec/rooms/blobs/get/0.1#request",
  "issuer": "did:example:member",
  "recipient": "did:example:host",
  "issuedAt": "2026-10-10T12:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000141ff",
  "payload": {
    "roomId": "did:example:room",
    "presentation": {
      "membership": "eyJhbGciOiJFZERTQSJ9.membership.signature",
      "authority": [
        "eyJhbGciOiJFZERTQSJ9.authority-leaf.signature",
        "eyJhbGciOiJFZERTQSJ9.authority-room.signature"
      ]
    },
    "blobRef": "zQmYtUc4iTCbbfVSDNKvtQqrfyezPPnFvE33wFmutw9PBBk"
  }
}
```

## Response

The host answers with the sub-schema reachable via `$anchor: "response"`: the manifest, a download handle and its expiry. A refusal is a `trust-task-error`.

### Download opened

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000142",
  "type": "https://trusttasks.org/spec/rooms/blobs/get/0.1#response",
  "issuer": "did:example:host",
  "recipient": "did:example:member",
  "issuedAt": "2026-10-10T12:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000141ff",
  "payload": {
    "manifest": {
      "size": 300000,
      "chunks": {
        "chunkSize": 262144,
        "chunkCount": 2,
        "chunkDigests": [
          "zQmRq2ZwqJ3vWJrK1v8R8oPqWXhD4nVb1JmT9pV6uYh3aBc",
          "zQmT5NvUtoM5nWFfrQdVrFtvGfKFmG7AHE8P34isapyhCxX"
        ]
      },
      "digest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR"
    },
    "downloadId": "7a1c2e4b-5d6f-4a8b-9c0d-1e2f3a4b5c6d",
    "expiresAt": "2026-10-10T12:10:01Z"
  }
}
```

## Security & Privacy

**The read chain gates the bytes only where the store does.** On a store that serves blobs only through the host, the room's read authority is what stands between the ciphertext and everyone else. A store that publishes ciphertext, so that anyone holding a blob's address can fetch it, removes that gate: the ciphertext is then protected by the file key alone. That is sound, because the key derives from a room storage key no host or outsider holds; but a removed member who kept a file's key can still open that file from such a store, where elsewhere they would also need the host to serve them the bytes. A host using such a store **SHOULD** say so to the room's members.

**The reader verifies; the host is trusted for availability only.** The BlobRef comes from a record its author signed, and the manifest must hash to it, so every substitution, truncation or reordering is caught by the reader.

### Data carried

A room identifier, a presentation and a BlobRef in; a ciphertext manifest, a handle and an expiry out.

### Correlation

On `open` and `attributed` rooms the presentation names the reader, so the host learns which member downloaded which blob and when, as it learns which member read which record. On `private` rooms it learns only that some member did. Download counts and egress feed the room's usage figures, which hold totals, not who read what.

### Retention

The download handle lives until its expiry, which a host **SHOULD** keep short (ten minutes in the reference design) and **MAY** extend as chunks are fetched.

### Consent/purpose

A member reads a file in a room they joined; their membership is the consent. The host's purpose is to serve the room's ciphertext to its members.
