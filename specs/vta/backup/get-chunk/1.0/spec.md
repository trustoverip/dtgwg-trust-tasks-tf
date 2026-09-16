---
slug: vta/backup/get-chunk
version: "1.0"
title: "VTA Backup — Get Chunk"
summary: "Retrieve one chunk of a chunked export bundle by index, over the transport that carries the rest of the family."
status: draft
targetFrameworkVersion: "0.5.0"
category: key-management
keywords:
  - backup
  - disaster-recovery
  - chunked-transfer
parties:
  - role: backup operator
    requirement: REQUIRED
    member: issuer
  - role: verifiable trust agent
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: REQUIRED
  rationale: >-
    The request releases part of a copy of the agent, and the only authority for that is having created the bundle — a comparison against an issuer that has to be attributable, on every chunk, since each one is released separately. The response carries secret material and is REQUIRED on that ground (SPEC §7.3 item 8). Its integrity is also anchored independently, since every chunk is checked against a digest in the signed initiate-export manifest; the response proof is what attributes the release of each chunk to the agent that made it, which the manifest alone does not.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Retrieval is non-consuming, so a replay releases nothing the creator could not have asked for again — but it releases it to whoever can read the reply, and bounding replay is what limits a captured request to the window in which it was made.
sideEffects:
  level: mutating
  rationale: >-
    Returns bytes the recipient already holds and alters none of them, but it is not a pure read: the recipient records that the index was served — which decides complete-export's `downloaded` — and may extend the bundle's expiry. Both change what the agent will later do with the bundle.
exposure:
  discloses: secret
  ingests: none
  actsAsSubject: false
  rationale: >-
    Each response carries part of the encrypted export. One chunk alone is ciphertext without context, but the chunks together are the bundle, and the bundle is a complete copy of the agent protected only by its password — so every chunk is classed with the whole.
retention:
  class: exchange
  rationale: >-
    Nothing about the request needs keeping beyond the fact that the index was served, which lives only as long as the bundle does. The bytes themselves are the bundle's, and go when it goes.
errorCodes:
  - code: vta/backup/get-chunk:notFound
    meaning: >-
      The recipient holds no live chunked export bundle under this identifier that this producer may act on. Deliberately conflates "no such bundle", "an import bundle", "a stream bundle", and "not yours" — see Correlation.
    retryable: false
  - code: vta/backup/get-chunk:chunkOutOfRange
    meaning: >-
      `index` is not below the bundle's `chunkCount`. Returned only to the bundle's creator, who already holds the manifest, so it discloses nothing.
    retryable: false
  - code: vta/backup/get-chunk:terminalState
    meaning: >-
      The bundle was acknowledged with complete-export, aborted, or has expired. Nothing more will be served under this identifier; a new export is needed.
    retryable: false
related:
  - vta/backup/initiate-export
  - vta/backup/complete-export
  - vta/backup/abort
---

## Abstract

The **VTA Backup — Get Chunk** Trust Task asks an agent for one chunk of an export bundle minted by [`initiate-export`](../../initiate-export/1.1/spec.md) with the `chunkedTrustTask` algorithm.

It exists so that an agent reachable only over DIDComm or TSP can be backed up. Such an agent has no HTTPS address from which a whole bundle could be fetched, and its transport will not carry a whole bundle in one message; what it can carry is a Trust Task of bounded size. This task is that bound applied to the export: the bundle is divided into chunks small enough to fit one message each, and the operator asks for them one at a time.

It is a request the operator makes, not a stream the agent sends, and that is the design's load-bearing choice. Mediated transports hold undelivered messages in bounded, expiring queues, and an agent that pushed a bundle's worth of chunks at an operator who was offline or slow would lose some of them without either side being told. A chunk the operator must ask for is a chunk the operator knows it does not yet have. The algorithm — manifest, chunk-size bound, ordering, resume, expiry — is defined once, under [Chunked transfer](../../initiate-export/1.1/spec.md#chunked-transfer).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here, and the requirements of [Chunked transfer](../../initiate-export/1.1/spec.md#chunked-transfer).

## Authorization

The authority is **having created the bundle**. A recipient **MUST** refuse a chunk to any party other than the one that issued the `initiate-export` this bundle came from, and **MUST** answer such a request as `notFound` rather than as a refusal — see [Correlation](#correlation).

That entitlement is narrow, and it is enough only because of what preceded it: `initiate-export` demanded custody of the agent before it minted anything. Retrieval inherits that decision rather than repeating it, as `complete-export` and `abort` do.

It replaces the bearer token of the `stream` algorithm, and the replacement is a strengthening, not a relaxation. A `stream` fetch is admitted by possession of a token that anyone who obtained it could present. A chunk is admitted by the identity of the requester, which the transport authenticates — DIDComm's authenticated encryption and TSP both yield a sender the recipient did not have to take on trust — compared against the bundle's recorded creator. A recipient **MUST** perform that comparison using the transport-authenticated sender or the verified `proof`, and **MUST** refuse a request in which the in-band `issuer` is inconsistent with the transport-authenticated identity ([SPEC §7.2](/SPEC.md#72-consumer-requirements)); an `issuer` asserted but not authenticated is a claim, not a creator.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient`, transport identity or `proof` establishes who sent this document, never that they own the bundle it names. The ownership check is a separate comparison, and a recipient that omits it releases any bundle to any authenticated operator who can quote its identifier.

## Definitions

**`bundleId`** — the handle from the `initiate-export` descriptor. Opaque: a producer quotes what it was given and **MUST NOT** derive, guess or enumerate one.

**`index`** — which chunk, zero-based. Valid values run from 0 to the manifest's `chunkCount` − 1.

**`data`** — the chunk's bytes, base64url-encoded without padding. Every chunk except the last is exactly `chunkSize` bytes once decoded.

**`digestMultibase`** — this chunk's digest, restated from the manifest. A producer **MUST** verify `data` against the manifest entry for `index`, which arrived in a signed document; it **MUST NOT** treat this member as a substitute, since it travels with the bytes it describes.

**`expiresAt`** — the bundle's expiry as of this response, after any extension the recipient applied.

## Serving a chunk

A recipient serving `get-chunk`:

1. **MUST** locate a live `chunkedTrustTask` export bundle under `bundleId` created by this producer, and otherwise answer `notFound` — for a bundle that does not exist, belongs to another producer, is an import, or uses a different algorithm alike.
2. **MUST** answer `terminalState` for a bundle that was acknowledged, aborted or has expired, to its creator.
3. **MUST** answer `chunkOutOfRange` for an index not below `chunkCount`.
4. **MUST** return the chunk's bytes exactly as committed in the manifest. Repeated requests for one index **MUST** return identical `data` for as long as the bundle is live: retrieval is non-consuming, and that is what makes a retry — after a lost response, a timeout, a reconnect — safe without any coordination.
5. **MUST** record that the index has been served. When every index has been served at least once, a later `complete-export` reports `downloaded: true`.
6. **MAY** extend the bundle's expiry, within the ceiling fixed at `initiate-export`, and **MUST** report the resulting value in `expiresAt`.

After `complete-export` is accepted or `abort` is processed for the bundle, the recipient **MUST** refuse further `get-chunk` requests for it immediately, even where discarding the staged bytes completes later.

### Rate and concurrency

The per-address request limiting that commonly fronts an HTTPS endpoint does not reach this task: it arrives through a mediator, and every request from every operator can share one source. A recipient **SHOULD** therefore bound `get-chunk` **per requester** — per authenticated sender, and per bundle — in both rate and concurrency, and answer a request beyond the bound with the standard `unavailable` code and a `retryAfter` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)) rather than queueing it without limit. A bound expressed per bundle also caps what one compromised operator identity can extract in a given interval.

A producer **SHOULD** request chunks with bounded concurrency, and **SHOULD** treat a missing response as a reason to ask again after a delay, not as a failure of the transfer.

## Request

The producer is the operator that initiated the export; the recipient is the agent holding the bundle. The request payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Asking for the last chunk of the bundle in initiate-export's example

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000013",
  "type": "https://trusttasks.org/spec/vta/backup/get-chunk/1.0#request",
  "issuer": "did:example:operator",
  "recipient": "did:example:agent",
  "issuedAt": "2026-01-01T02:01:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000fb",
  "payload": {
    "bundleId": "5b1f4a2e-7c3d-4e8f-9a6b-2d0c1e3f4a5b",
    "index": 2
  }
}
```

## Response

The producer of the response is the recipient of the request. Its payload is the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error` with one of the codes declared in the front matter, not a `#response` document.

### The 12-byte final chunk

The last chunk of a 524300-byte bundle in 256 KiB chunks holds the 12-byte remainder, which keeps the example readable; a full chunk carries 349526 characters of `data`.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000014",
  "type": "https://trusttasks.org/spec/vta/backup/get-chunk/1.0#response",
  "issuer": "did:example:agent",
  "recipient": "did:example:operator",
  "issuedAt": "2026-01-01T02:01:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000fb",
  "payload": {
    "bundleId": "5b1f4a2e-7c3d-4e8f-9a6b-2d0c1e3f4a5b",
    "index": 2,
    "digestMultibase": "zQmTcWLvAPe4Txz32vVqZe5jgX4nBPiaBsTTCp4bLXDMzTT",
    "data": "YmFja3VwLXRhaWwh",
    "expiresAt": "2026-01-01T02:11:01Z"
  }
}
```

## Security & Privacy

### Data carried

The request carries a handle and a number. The response carries **part of a complete, encrypted copy of the agent**.

One chunk is ciphertext with no header, no key and no context, and reveals nothing of the agent's state on its own. That is not a reason to handle it loosely. The set of chunks is the bundle, the bundle is protected only by the operator's password, and a password is a secret whose strength is chosen by a human. A producer **MUST** treat each chunk with the sensitivity of the whole bundle, **MUST NOT** write chunks anywhere it would not write the bundle, and **SHOULD** discard partial retrievals it abandons.

The response carries no token and no address, and nothing that would let a third party retrieve another chunk: the next chunk is released only to the same authenticated requester.

A recipient **MUST NOT** place anything but the chunk and the members this schema defines in the response — in particular nothing from the bundle's plaintext, which it never needs in order to serve a chunk.

### Correlation

To intermediaries, a chunked retrieval is a run of `chunkCount` request/response pairs between one operator and one agent, the responses of nearly uniform size. A mediator that cannot read them still learns that a large transfer of that shape happened, and roughly how large. That is intrinsic to moving a bundle over a mediated transport. A producer **MAY** space its requests; it cannot hide the total. Where the transport conceals the final recipient from intermediaries, as TSP's routed mode does, an intermediary learns correspondingly less about who the transfer was between.

`bundleId` joins every chunk request to the export that created the bundle and to its eventual `complete-export` or `abort`; `index` orders them. Both are intrinsic.

Four conditions are answered as `notFound`: no bundle with that id, an import bundle, a `stream` export bundle, and a bundle owned by another operator. Conflating them keeps an opaque handle from becoming an oracle over the agent's backup activity. `chunkOutOfRange` and `terminalState` are distinguishable only because they are only ever returned to the bundle's creator, who already knows the bundle exists.

### Retention

The recipient keeps, for the life of the bundle, which indices have been served. It **MUST NOT** retain that record, or the bytes, beyond the bundle: the fact worth keeping durably — that an export was made and whether it was retrieved — is kept by `initiate-export` and `complete-export`, and a per-chunk log adds nothing to it except a detailed timeline of the operator's connection.

A producer holds chunks only until reassembly. Once the bundle is assembled and verified, the individual chunks have no further use.

### Consent/purpose

The purpose is retrieving an export the same operator initiated, for the continuity purpose `initiate-export` states. Nothing in this document licenses more: a recipient **MUST NOT** treat a `get-chunk` request as authority to mint a bundle, extend an expiry beyond its ceiling, or serve any bundle other than the one named.

Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements), this specification does not declare a consent, approval or step-up requirement.
