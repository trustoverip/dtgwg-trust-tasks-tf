---
slug: backup/initiate-export
version: "0.1"
title: "Backup — Initiate Export"
summary: "Mint an encrypted full-state export bundle and return the descriptor that fetches it, over HTTPS or chunk by chunk over the node's own Trust Task transport."
status: draft
targetFrameworkVersion: "0.5.0"
category: key-management
keywords:
  - backup
  - disaster-recovery
  - password
  - chunked-transfer
parties:
  - role: backup operator
    requirement: REQUIRED
    member: issuer
  - role: node
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: REQUIRED
  rationale: >-
    The request carries the password that protects the node's entire state, and the recipient stakes the whole export on the caller being entitled to it — an unattributable request is one there is no way to answer for afterwards. The response is REQUIRED for a different reason: it carries either a bearer capability or the per-chunk digests every later chunk is checked against, and an unsigned descriptor is one an intermediary can substitute — pointing the operator's download at bytes the node never minted, or blessing chunks the node never produced.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A replayed initiate-export mints a second bundle, and therefore a second live copy of the same state, from one authorization. Bounding replay is what keeps the number of outstanding copies equal to the number of times a human asked for one.
sideEffects:
  level: mutating
  rationale: >-
    Serializes the node's state into an encrypted bundle, stages the bytes, and mints a short-lived transport slot. Nothing existing is altered or destroyed — but a new copy of everything the node holds now exists and is retrievable, which is a change in the node's exposure even though it is not a change in its data.
exposure:
  discloses: secret
  ingests: secret
  actsAsSubject: false
  rationale: >-
    Both directions carry secret material and they are different secrets. Inbound, `password` is the key-derivation input protecting the whole export; the recipient uses it and MUST NOT retain it. Outbound, a `stream` descriptor's `transportToken` is a bearer credential for the byte endpoint, and the bundle it opens contains the node's key material — so that descriptor is, in effect, the export. A `chunkedTrustTask` descriptor carries no credential, but its manifest is what makes the chunks retrievable and verifiable, and is handled with the same care.
retention:
  class: exchange
  rationale: >-
    The staged bytes, and the token where there is one, live for the bundle's slot and no longer: fetched and acknowledged, aborted, or expired at `expiresAt`. The password is shorter still — it is consumed deriving the key and never belongs at rest.
errorCodes:
  - code: backup/initiate-export:transportUnavailable
    meaning: >-
      The recipient cannot move the bytes by the requested algorithm — for `stream`, it has no address at which it can publish them. Not a fault in the request — see Transport preconditions.
    retryable: false
  - code: backup/initiate-export:weakPassword
    meaning: >-
      The password is shorter than the recipient's floor. Refused before any state is serialized.
    retryable: false
  - code: backup/initiate-export:unsupportedAlgorithm
    meaning: >-
      The recipient does not implement the requested transport algorithm. The message names what it does implement.
    retryable: false
  - code: backup/initiate-export:tooManyOpenBundles
    meaning: >-
      This operator already holds the maximum number of live bundles. Abort one or wait for expiry.
    retryable: true
  - code: backup/initiate-export:bundleTooLarge
    meaning: >-
      The serialized bundle cannot be divided into at most 4096 chunks no larger than the chunk size the recipient may use for this producer. Only raised for `chunkedTrustTask`; the staged bytes are discarded before the error is returned.
    retryable: false
related:
  - backup/get-chunk
  - backup/complete-export
  - backup/abort
  - backup/put-chunk
---

## Abstract

The **Backup — Initiate Export** Trust Task asks a node to serialize everything it backs up — for an agent its keys, access-control entries and trust contexts, for a community its backed-up keyspaces and signing key bundle, and for either, optionally, its audit trail — into a single password-encrypted bundle, and to return the terms on which those bytes can be retrieved.

It is a Trust Task and not an API call because of what the bundle *is*. A node's value rests on the exclusivity of its key material; an export is the one operation that deliberately produces a second copy of it. The document that asks for one has to be attributable, placeable in time, and reviewable afterwards, and that is precisely the set of properties a Trust Task envelope provides and a bare HTTP request does not.

The bulk bytes deliberately do **not** travel in this envelope. This task is the control plane: it mints a bundle and hands back a [descriptor](#definitions). The operator then retrieves the bytes by one of two algorithms — an out-of-band HTTPS fetch (`stream`), or a sequence of [`get-chunk`](../../get-chunk/0.1/spec.md) tasks over the transport already carrying this one (`chunkedTrustTask`) — and acknowledges with [`complete-export`](../../complete-export/0.1/spec.md), or discards the bundle with [`abort`](../../abort/0.1/spec.md).

**One family for every node that backs itself up.** These tasks began as `vta/backup/*` 1.x, the agent's own descriptor-based backup and restore. Nothing in the transfer is specific to an agent — a slot, a pre-committed digest, chunks or a stream, a finalize step that previews before it replaces — and a community node needs exactly the same thing for the same reason: a backup of any real node is too large for one document, and a node that bounds a document's size before checking its proof cannot accept it in one. So the family is node-neutral here, and what a backup *contains* is the node's to define. `vta/backup/*` stays served while agents move to these tasks; `vtc/backup/{export,import}` remain for a node small enough to fit one document. The transfer shapes — descriptor, chunk manifest, chunk data — are the ones `vta/_shared/0.1/backup-transfer` already defines, and are referenced from there rather than copied, so the two families cannot drift and no generated library renames a published type.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).


## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

The authority is **custody of the node itself**. An export is not scoped to a resource inside the node that a capability could name — it reproduces the whole node, including the key material every other authority in the system is ultimately derived from. So the entitlement a producer needs is the one that admits of no narrower statement: the producer is an owner or operator of this node, at the level that could equally destroy it.

A recipient **MUST** refuse a producer holding anything less. In particular, a producer entitled to *read* a resource through this node is not thereby entitled to export it: the export bypasses every per-resource control by construction, so deriving export authority from any accumulation of narrower grants defeats them all.

The algorithm does not change the authority. A `chunkedTrustTask` export is retrieved through `get-chunk`, whose authority is *having created the bundle*; that narrower entitlement is sound only because this task, which created it, demanded custody.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient`, transport identity or `proof` establishes *who asked* and *that the document is unaltered*. None of it establishes entitlement, and a recipient that treats a valid signature from a known operator as sufficient has checked identity and called it authorization.

This specification does not, and **MUST NOT**, declare that a human approval or a step-up is required before an export ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)). That is consumer policy. It is worth being explicit that the policy has a limit, though, so that implementers do not over-read what one can achieve: an approval ceremony can establish that a human is present and intends the export, and cannot establish anything about the `password` member, which has already been chosen and transmitted by the time any approval is displayed. See [Data carried](#data-carried).

## Definitions

**`password`** — the secret from which the recipient derives the bundle's encryption key. Chosen by the producer, never by the recipient, and never recoverable from the recipient afterwards: a bundle whose password is lost is indistinguishable from random bytes. Recipients declare a minimum length and refuse anything shorter with `weakPassword`.

**`includeAudit`** — whether the node's audit trail is serialized into the bundle alongside its operational state. Separable because the trail answers a different question from the state and has a different sensitivity: it records who did what, which is a history of the operator's own actions and of every counterparty the node has dealt with. Absent means the trail is excluded.

**`algorithm`** — the transport mechanism the producer asks for, naming how the bytes will move rather than how they are encrypted. This version defines two values. `"stream"` — a single HTTPS transfer against the returned `transportUrl` — is what an absent member means. `"chunkedTrustTask"` — retrieval chunk by chunk through [`get-chunk`](../../get-chunk/0.1/spec.md) — is defined under [Chunked transfer](#chunked-transfer). The member is a string rather than an enumeration so that a recipient offering more (a pre-signed object-store URL, say) can be asked for it without a revision.

**`maxChunkSize`** — for `chunkedTrustTask` only, the largest chunk the producer can accept, when its transport is tighter than the one the normative ceiling assumes. Absent means the ceiling. A recipient ignores it for any other algorithm.

**`descriptor`** — the response's account of the transfer, in one of two shapes defined by the shared [`BundleDescriptor`](../../../vta/_shared/0.1/backup-transfer.schema.json). Both carry:

- **`bundleId`** — the handle for this bundle for its entire lifecycle. The producer quotes it to `get-chunk`, `complete-export` and `abort`.
- **`algorithm`** — the mechanism in use. A recipient **MUST** return the algorithm the request named (or `stream`, where it named none), or refuse; it **MUST NOT** substitute another, because a producer cannot be assumed to implement an algorithm it did not ask for.
- **`expectedSha256`** — the hash of the whole byte stream, as lowercase hex. A wire-integrity check independent of the encrypted envelope's own authentication tag, so that a truncated, substituted or misassembled transfer is detected before the password is ever applied to it.
- **`expectedSizeBytes`** — the total byte count, letting a producer detect a truncated transfer even without hashing.
- **`expiresAt`** — after which the bytes are collected and further retrieval refused. Short by design.

A **`stream`** descriptor additionally carries:

- **`transportUrl`** — where to fetch the bytes.
- **`transportToken`** — a bearer credential for that fetch, minted fresh per bundle, presented in the `X-Backup-Token` header. Recipients **SHOULD** store only a hash of it, so that a compromise of the recipient's own storage does not yield a usable token, and **SHOULD** accept it once.

A **`chunkedTrustTask`** descriptor instead carries **`chunks`**, the [chunk manifest](#chunked-transfer): `chunkSize`, `chunkCount`, and `chunkDigests`, one digest per chunk in index order.

**`completionHint`** — operator-facing text describing how to complete the download. Advisory, and safe to ignore; a producer **MUST NOT** parse it or derive behaviour from it.

## Channel requirement

This task **MUST** be carried over a channel confidential end-to-end between the producer and the recipient: one on which the producer encrypts to the recipient itself, such as the DIDComm binding with authenticated encryption, or the TSP binding. The request carries `password`, and the manifest the response returns is what retrieves the bundle it unlocks. A party that can read both holds the node's key material. A channel confidential only hop by hop does not qualify, the HTTPS binding included: TLS terminates wherever the recipient's operator terminates it (a load balancer, an ingress, a sidecar), and the plaintext document exists there.

A recipient **MUST** refuse this task with `permissionDenied` ([SPEC.md §8.3](/SPEC.md#83-standard-error-codes)) when it arrives over any other channel. It refuses after establishing entitlement and before serializing any state. The refusal **SHOULD** name the bindings the recipient accepts, since the producer's remedy is to send the same request again over one of them.

The requirement covers the tasks of the exchange, not the bytes. The bundle moved chunk by chunk is ciphertext.

## Transport preconditions

A recipient can only return a `transportUrl` if it knows an address at which it is reachable. That is a property of the recipient's deployment, not of the request, and it is commonly absent: a node that speaks only DIDComm or TSP has a perfectly good identity and no HTTPS address to publish — the arrangement much of this ecosystem is built to support.

Such a recipient, asked for `stream`, **MUST** refuse with `transportUnavailable` rather than returning a descriptor whose URL cannot be fetched. The distinction matters to the producer, because the two failures need opposite responses: a malformed request is fixed by the producer, and this one can only be fixed by whoever configures the node — or, where the recipient implements it, by asking again for `chunkedTrustTask`.

`chunkedTrustTask` needs no address of the recipient's own: it rides the transport the request arrived on. A recipient **SHOULD** implement it whenever it accepts this task over a transport other than HTTPS, since that is exactly the recipient `stream` cannot serve. A recipient that implements neither algorithm for the transport in use refuses with `transportUnavailable` if it recognises the algorithm and `unsupportedAlgorithm` if it does not.

## Chunked transfer

This section defines the `chunkedTrustTask` algorithm for the whole `vta/backup` family, in both directions. [`initiate-import`](../../initiate-import/0.1/spec.md), [`get-chunk`](../../get-chunk/0.1/spec.md), [`put-chunk`](../../put-chunk/0.1/spec.md) and [`finalize-import`](../../finalize-import/0.1/spec.md) refer to it rather than restating it.

### The manifest

A chunked bundle of `expectedSizeBytes` bytes is divided into `chunkCount` = ⌈`expectedSizeBytes` ÷ `chunkSize`⌉ chunks. Chunk *i* holds bytes *i* × `chunkSize` up to, but not including, (*i* + 1) × `chunkSize`, or the end of the bundle; every chunk but the last is exactly `chunkSize` bytes, and the last holds the remainder. `chunkDigests[i]` is the digest of chunk *i*'s raw bytes — not of their encoding — as a multibase-encoded multihash ([`DigestMultibase`](/SPEC.md#66-shared-schema-components)).

The manifest is a commitment made before any chunk moves. For an export the recipient computes it and returns it in this task's response, whose `proof` is REQUIRED, so every digest in it is authenticated by the node that minted the bytes. A party receiving a manifest **MUST** refuse it if `chunkCount` does not equal the formula above for the stated `expectedSizeBytes` and `chunkSize`, or if `chunkDigests` does not hold exactly `chunkCount` entries.

Every party **MUST** implement sha2-256, and a recipient **SHOULD** use it. Digests **MUST** be compared as decoded multihash bytes, never as encoded strings, since two conforming encodings of one digest differ. A party that does not implement the hash a digest names **MUST** treat the manifest as unverifiable and refuse it; it **MUST NOT** skip the check or substitute another hash.

### The chunk-size bound

`chunkSize` **MUST NOT** exceed **262144 bytes (256 KiB)**, and a recipient **MUST NOT** choose one larger than a request's `maxChunkSize`. The bound exists because the transports this algorithm is for impose a hard ceiling on a single message, and a chunk document that exceeds it is not delivered slowly — it is refused outright, often at an intermediary that tells neither party why.

The figure is derived, not chosen, from the commonest such ceiling: a mediator accepting at most **1 MiB (1048576 bytes)** per message, including every layer of encoding, a limit applied alike to DIDComm and to TSP traffic. The derivation, for a `get-chunk` response or a `put-chunk` request carrying a full chunk:

1. `data` carries the chunk base64url-encoded: 262144 bytes become 349526 characters.
2. The rest of the document — framework members, `proof`, `bundleId`, digest, `expiresAt` — is bounded by the schemas at well under 4 KiB. The plaintext message is therefore under 353622 bytes.
3. A DIDComm authenticated-encryption envelope carries its ciphertext base64url-encoded, multiplying by 4⁄3: under 471496 bytes.
4. A mediator forward wrapper encrypts and encodes that envelope again: under 628662 bytes.
5. A second, nested forward hop — a sender routing through its own mediator to the recipient's — does so once more: under 838216 bytes, which still leaves over 20% of a 1 MiB limit spare.

The next power of two does not survive the same path: a 524288-byte chunk is 699051 characters, over 932000 bytes once authenticated, and over 1242000 bytes after a single forward wrapper. TSP's CESR framing and HPKE sealing add overhead per hop rather than multiplying it, so a chunk within the DIDComm bound is within the TSP one with more margin. A deployment whose transport is tighter than this — a mediator configured below 1 MiB, a deeper routing path — is served by `maxChunkSize` on export and by choosing a smaller `chunkSize` on import; the floor of 16384 bytes exists so that a small chunk size still reaches a useful bundle size within the chunk-count bound.

`chunkCount` **MUST NOT** exceed **4096**. The manifest travels in one document too: at 4096 sha2-256 digests of roughly 50 characters each it is about 200 KiB before the same encoding path, which fits under the same ceiling. The two bounds together cap a chunked bundle at 1 GiB; a recipient whose serialized state does not fit refuses with `bundleTooLarge`.

### Retrieval, ordering and resume

The producer **pulls** each chunk with `get-chunk`, one request per index. Nothing is pushed. This is deliberate and it is the property the algorithm most depends on: mediated transports queue undelivered messages per recipient with a bounded queue and a bounded lifetime, and a recipient that pushed hundreds of chunks at a producer that was offline, slow or gone would fill that queue and lose chunks silently — a send that a transport reports as accepted is not a message that was delivered. When the producer asks for each chunk, a chunk that was lost in transit is one the producer notices it does not have, and asks for again.

Retrieval is **non-consuming**: a recipient **MUST** answer repeated `get-chunk` requests for the same index with the same bytes for as long as the bundle is live. A producer **MAY** request chunks in any order and **MAY** request several concurrently; a recipient **MAY** bound that concurrency per producer. Resuming an interrupted retrieval is therefore nothing more than requesting the indices not yet held — there is no resume token and no server-side cursor to lose.

A producer **MUST** verify each chunk against `chunkDigests` on arrival and **MUST** discard and re-request a chunk that fails. Once every index is held it **MUST** reassemble in index order and verify the result against `expectedSha256` and `expectedSizeBytes` before applying the password or writing the bundle anywhere it will be trusted.

### Expiry and completion

A chunked bundle is live from this task's response until the first of: an accepted [`complete-export`](../../complete-export/0.1/spec.md), an [`abort`](../../abort/0.1/spec.md), or `expiresAt`. After any of these the recipient **MUST** refuse `get-chunk` for the bundle and **SHOULD** discard its staged bytes.

A chunked retrieval takes longer than a single fetch, and its duration grows with the bundle. A recipient therefore **MAY** extend `expiresAt` each time it serves a chunk — a sliding window from the last activity — but **MUST NOT** extend it beyond a ceiling it fixes when the bundle is minted, measured from this task's response. The ceiling is recipient policy and is what bounds how long a copy of the node can remain retrievable however it is being used; a recipient **SHOULD** set it no longer than the largest bundle it will serve plausibly takes to move. Each `get-chunk` response reports the current `expiresAt`, so the producer never has to infer it.

For the purposes of `complete-export`'s `downloaded` member, a chunked bundle has been downloaded when every index has been served at least once.

## Request

The producer is the backup operator; the recipient is the node being exported. The request payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A full export including the audit trail

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/backup/initiate-export/0.1#request",
  "issuer": "did:example:operator",
  "recipient": "did:example:node",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "password": "correct horse battery staple",
    "includeAudit": true
  }
}
```

### A chunked export for a node with no HTTPS address

The same export asked of a node reachable only over DIDComm or TSP. The request travels over that transport, and so will every chunk.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000011",
  "type": "https://trusttasks.org/spec/backup/initiate-export/0.1#request",
  "issuer": "did:example:operator",
  "recipient": "did:example:node",
  "issuedAt": "2026-01-01T02:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000fd",
  "payload": {
    "password": "correct horse battery staple",
    "includeAudit": false,
    "algorithm": "chunkedTrustTask"
  }
}
```

## Response

The producer of the response is the recipient of the request. Its payload is the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error` with one of the codes declared in the front matter, not a `#response` document.

### The descriptor for the stream export above

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/backup/initiate-export/0.1#response",
  "issuer": "did:example:node",
  "recipient": "did:example:operator",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "descriptor": {
      "bundleId": "3f2504e0-4f89-41d3-9a0c-0305e82c3301",
      "algorithm": "stream",
      "transportUrl": "https://node.example/backup/blob/3f2504e0-4f89-41d3-9a0c-0305e82c3301",
      "transportToken": "dGhpcy1pcy1hLW9uZS1zaG90LWJlYXJlci10b2tlbg",
      "expectedSha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
      "expectedSizeBytes": 1048576,
      "expiresAt": "2026-01-01T00:05:01Z"
    },
    "completionHint": "GET the transportUrl with header X-Backup-Token, then send complete-export."
  }
}
```

### The manifest for the chunked export above

A 524300-byte bundle in 256 KiB chunks: two full chunks and a 12-byte remainder.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000012",
  "type": "https://trusttasks.org/spec/backup/initiate-export/0.1#response",
  "issuer": "did:example:node",
  "recipient": "did:example:operator",
  "issuedAt": "2026-01-01T02:00:02Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000fd",
  "payload": {
    "descriptor": {
      "bundleId": "5b1f4a2e-7c3d-4e8f-9a6b-2d0c1e3f4a5b",
      "algorithm": "chunkedTrustTask",
      "chunks": {
        "chunkSize": 262144,
        "chunkCount": 3,
        "chunkDigests": [
          "zQmehatQCtXyeV6kFkRVXjhDifqT3qARJ24248K2GJp7iWx",
          "zQmaFd25Uf6hJJ8xHX349DLzp4sryKmTGuyaSZWVtwsK5rM",
          "zQmTcWLvAPe4Txz32vVqZe5jgX4nBPiaBsTTCp4bLXDMzTT"
        ]
      },
      "expectedSha256": "20111c1d10631a5d6b2e9e06f558cd5c84e841d2bc5758bfda0d3b76bb4927f0",
      "expectedSizeBytes": 524300,
      "expiresAt": "2026-01-01T02:10:02Z"
    },
    "completionHint": "Send get-chunk for indices 0 to 2, verify each, then send complete-export."
  }
}
```

## Security & Privacy

### Data carried

The request carries **one secret going in**: `password`. This is unusual and worth stating plainly, because most of this task family's risk is read as being about what comes back. The password is the only thing standing between the exported bundle and whoever ends up holding it, and it is chosen by the producer at the moment of asking.

A recipient **MUST NOT** log it, echo it in a response, include it in an audit entry, or persist it in any form. It is consumed deriving the key and discarded. A producer, correspondingly, **MUST NOT** place it anywhere the envelope's confidentiality does not cover.

This has a direct consequence for producer *implementations*, and it is the reason the member is annotated `writeOnly`: the password's exposure is decided by where it is typed. A field in a shared or scriptable environment — a browser form reachable by autofill, a password manager, a co-resident extension or screen capture; a shell history; a CI variable — exposes it to everything with reach into that environment, and no property of this protocol recovers from that. Producers **SHOULD** collect it somewhere with a smaller reach, and **SHOULD NOT** offer to remember it.

A `stream` response carries **two secrets coming out**, and they compound: `transportToken` opens `transportUrl`, and the bytes behind it are the node's entire state. Anyone holding both holds the export, subject only to the password. A producer **MUST** treat that descriptor with the sensitivity of the bundle itself, and **MUST NOT** write it to a shared log or pass it through an intermediary that does not need it.

A `chunkedTrustTask` response carries **no bearer secret**, and that is a security property of the algorithm, not an omission. Each chunk is released only to a `get-chunk` request whose sender the transport authenticates and whose issuer is the bundle's creator, so there is no credential whose theft would let a third party retrieve the bundle. What the response does carry — the manifest — is integrity material: its digests say what the chunks must be. It is not confidential in itself, but a producer **SHOULD** keep it with the bundle, because it is what lets a later reader confirm the bundle is the one the node minted.

`includeAudit` widens what the bundle contains beyond the node's own state to a record of its dealings, including counterparties who were never party to this export and cannot object to it. A producer **SHOULD** set it only when the trail is part of what is being preserved.

`completionHint` is free text. A recipient **MUST NOT** place a token, a password, or any other secret in it — it is written to be shown to a human and is routinely copied into places the descriptor should never go.

### Correlation

The recipient learns when this operator exports and how often, which over time describes an operational rhythm — before migrations, before risky changes, after incidents. That is unavoidable: a node cannot export itself without knowing it did.

`bundleId` joins this document to the later `get-chunk`, `complete-export` or `abort` documents in the same lifecycle, and `threadId` joins the request to its response. Both are intrinsic to the exchange.

The producer's identifier must be **stable across the bundle's lifecycle**, because the recipient checks that whoever retrieves, acknowledges or aborts a bundle is the party that created it. A producer varying its identifier between documents will find the bundle reported as not found — deliberately, since reporting it as forbidden would confirm to a stranger that a bundle with that id exists. That check is what makes the identifier reused here, and it does not extend beyond this exchange: nothing requires the same identifier for the next export.

A `stream` `transportUrl` is fetched over a separate connection that carries no Trust Task envelope, so it is observable to the network as an ordinary HTTPS transfer of known size to a known node. A `chunkedTrustTask` retrieval is instead a burst of similarly sized encrypted messages between the two parties, visible to every intermediary on the route — a mediator learns that roughly `chunkCount` × `chunkSize` bytes moved from this node to this operator in a short interval, even though it can read none of them. Where the transport hides the final recipient from intermediaries, as TSP's routed mode does, that exposure is correspondingly smaller. In neither case does an observer learn anything of the bundle's content.

### Retention

The staged bytes, and a `stream` bundle's token, are `exchange`-scoped: they exist for one bundle's slot and are collected at `expiresAt` if nothing else ends them first. Recipients **SHOULD** keep that slot short — long enough for a retrieval, not for a forgotten bundle to linger as a retrievable copy of the node — and **SHOULD** cap the number a single operator may hold open, since each is another live copy. For a chunked bundle the sliding extension described under [Expiry and completion](#expiry-and-completion) is bounded by a ceiling for exactly this reason.

The recipient **MUST** record that an export was initiated, by whom, when and by which algorithm, durably and **before** it returns the descriptor, and **MUST** refuse the export when it cannot; it **SHOULD** retain that record beyond the bundle. A copy released unrecorded cannot be recalled, so the record comes first. "A copy of this node was made, on this date, at this operator's request" is the one fact about an export that stays relevant after the bundle is gone, and it is what a later investigation into a leaked copy has to start from.

The `password` is not retained at all, at any class.

### Consent/purpose

The purpose is continuity: preserving a node against loss, migration or corruption. A bundle produced for that purpose is a complete, offline, indefinitely-lived copy of the node, which makes reuse both easy and unobservable — nothing in the bundle reports having been opened.

A producer **SHOULD NOT** use an export to move data somewhere a narrower task would have supplied it, and **MUST NOT** use one to obtain material the node's own access controls would refuse it. That is the specific abuse this task's shape invites: exporting the whole node is often the path of least resistance to one thing inside it, and it silently discards every control that would have applied.

Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements), this specification does not declare a consent, approval or step-up requirement; the paragraph above states a purpose limitation, which is a different thing.
