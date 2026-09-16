---
slug: vta/backup/put-chunk
version: "1.0"
title: "VTA Backup — Put Chunk"
summary: "Write one chunk of a chunked import bundle by index, checked against the manifest committed when the slot was opened."
status: draft
targetFrameworkVersion: "0.5.0"
category: key-management
keywords:
  - backup
  - disaster-recovery
  - restore
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
  response: RECOMMENDED
  rationale: >-
    The request places bytes inside an agent, and the only authority for that is having opened the slot — a comparison against an issuer that has to be attributable on every write. The response is small and is the producer's evidence that a chunk was stored; a signature makes that acknowledgement something the producer can rely on after the transport session is gone, and costs little, since it covers no chunk data.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A replayed write of identical bytes changes nothing, and a replay of different bytes is refused against the manifest — but a replay still extends the slot's expiry, and bounding it is what stops a captured request from keeping a writable slot open.
sideEffects:
  level: mutating
  rationale: >-
    Stages one chunk of an import bundle. Nothing the agent holds is read, altered or destroyed; staged chunks are inert until finalize-import applies the assembled bundle, and a slot that is never finalized expires having changed nothing.
exposure:
  discloses: metadata
  ingests: secret
  actsAsSubject: false
  rationale: >-
    Inbound, each request carries part of an encrypted, complete copy of an agent — classed with the whole bundle, because the chunks together are the bundle and the bundle is protected only by its password. Outbound, the response is an acknowledgement and a count, and says nothing about the chunk's content.
retention:
  class: exchange
  rationale: >-
    A staged chunk lives only as long as its slot: applied by finalize-import, discarded by abort, or discarded unread at expiry.
errorCodes:
  - code: vta/backup/put-chunk:notFound
    meaning: >-
      The recipient holds no open chunked import slot under this identifier that this producer may act on. Deliberately conflates "no such bundle", "an export bundle", "a stream slot", and "not yours" — see Correlation.
    retryable: false
  - code: vta/backup/put-chunk:chunkOutOfRange
    meaning: >-
      `index` is not below the manifest's `chunkCount`.
    retryable: false
  - code: vta/backup/put-chunk:digestMismatch
    meaning: >-
      `digestMultibase` does not equal the manifest's digest for `index`, or `data` does not hash to it. Nothing is stored, and a chunk already held at `index` is left untouched. `details.expectedDigestMultibase` restates the manifest entry so the producer can tell a wrong index from corrupted bytes.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [expectedDigestMultibase]
      properties:
        expectedDigestMultibase:
          $ref: "../../../../_framework/0.3/framework.schema.json#/$defs/DigestMultibase"
  - code: vta/backup/put-chunk:chunkSizeMismatch
    meaning: >-
      `data` decodes to the wrong number of bytes for `index`: not exactly `chunkSize` for any chunk but the last, or not the remainder for the last. Nothing is stored.
    retryable: false
  - code: vta/backup/put-chunk:terminalState
    meaning: >-
      The slot was already finalized, aborted or has expired. Nothing more will be accepted under this identifier; a new initiate-import is needed.
    retryable: false
related:
  - vta/backup/initiate-import
  - vta/backup/finalize-import
  - vta/backup/abort
---

## Abstract

The **VTA Backup — Put Chunk** Trust Task writes one chunk of an encrypted bundle into an import slot opened by [`initiate-import`](../../initiate-import/1.1/spec.md) with the `chunkedTrustTask` algorithm.

It is the import-side counterpart of [`get-chunk`](../../get-chunk/1.0/spec.md), for the same reason: an agent reachable only over DIDComm or TSP cannot accept a whole bundle in one message, but can accept it a bounded chunk at a time. The algorithm is defined once, under [Chunked transfer](../../initiate-export/1.1/spec.md#chunked-transfer).

In this direction the operator has the bytes, so the operator sends them — but every write is individually answered, and a chunk is not held until its response says it is. A send the transport accepts is not a chunk the agent stored, and the answer is what turns one into the other.

Every chunk is checked against the manifest the operator committed when the slot was opened, before the agent held any of it. A write cannot introduce bytes the operator did not commit to, and cannot change a chunk already stored.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here, and the requirements of [Chunked transfer](../../initiate-export/1.1/spec.md#chunked-transfer).

## Authorization

The authority is **having created the slot**. A recipient **MUST** refuse a write from any party other than the one that issued the `initiate-import` that opened it, and **MUST** answer such a request as `notFound` rather than as a refusal — see [Correlation](#correlation).

The entitlement is narrow because the decision that matters was already taken: `initiate-import` demanded custody of the agent before it opened the slot, and `finalize-import` demands it again before anything is applied. A write in between stages inert bytes that must match a commitment the creator made.

It replaces the bearer token of the `stream` algorithm. A `stream` write is admitted by possession of a token; a chunk is admitted by the identity of the requester, which the transport authenticates, compared against the slot's creator. A recipient **MUST** perform that comparison using the transport-authenticated sender or the verified `proof`, and **MUST** refuse a request whose in-band `issuer` is inconsistent with the transport-authenticated identity ([SPEC §7.2](/SPEC.md#72-consumer-requirements)).

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient`, transport identity or `proof` establishes who sent this document, never that they own the slot it names.

## Definitions

**`bundleId`** — the handle from the `initiate-import` descriptor. Opaque: a producer quotes what it was given and **MUST NOT** derive, guess or enumerate one.

**`index`** — which chunk, zero-based, below the manifest's `chunkCount`.

**`digestMultibase`** — this chunk's digest, restating the manifest entry for `index`.

**`data`** — the chunk's bytes, base64url-encoded without padding. Exactly `chunkSize` bytes once decoded, except for the last chunk, which holds the remainder.

**`stored`** — whether this request is what stored the chunk. `false` means identical bytes were already held at `index`, which is a success.

**`remainingCount`** — how many indices the slot still lacks after this write. Zero means `finalize-import` can be sent.

**`expiresAt`** — the slot's expiry as of this response, after any extension the recipient applied.

## Accepting a chunk

A recipient accepting `put-chunk`:

1. **MUST** locate an open `chunkedTrustTask` import slot under `bundleId` created by this producer, and otherwise answer `notFound` — for a slot that does not exist, belongs to another producer, is an export, or uses a different algorithm alike.
2. **MUST** answer `terminalState` for a slot that was finalized, aborted or has expired, to its creator.
3. **MUST** answer `chunkOutOfRange` for an index not below `chunkCount`.
4. **MUST** answer `chunkSizeMismatch` if `data` does not decode to the length the manifest implies for `index`.
5. **MUST** compare `digestMultibase` with the manifest entry for `index`, and the digest of the decoded `data` with that entry, both as decoded multihash bytes; on any difference it **MUST** answer `digestMismatch` and store nothing.
6. If a chunk is already held at `index`, **MUST NOT** replace it. Because it passed the same check, it is necessarily identical, and the recipient answers success with `stored: false`. Otherwise it stores the chunk and answers `stored: true`.
7. **MAY** extend the slot's expiry, within the ceiling fixed at `initiate-import`, and **MUST** report the resulting value in `expiresAt`.

The recipient **MUST** have durably staged the chunk before it answers `stored: true`, so that the acknowledgement means the chunk survives what the producer will do next — which may be to discard its own copy.

### Idempotence

A write is idempotent per index, and a conflicting write is impossible by construction: the manifest fixes each index's digest before any chunk arrives, so the only bytes a slot can accept at an index are the bytes committed for it. A retry after a lost response, a reconnect, or a producer restart simply sends the same chunk again and is answered `stored: false` if the first attempt landed.

Resuming an interrupted upload is therefore re-sending the chunks whose writes were not acknowledged. A producer that has lost that record learns what is missing from `finalize-import`, whose `incompleteUpload` refusal lists the missing indices and is raised before the password is used.

### Rate and concurrency

The per-address request limiting that commonly fronts an HTTPS endpoint does not reach this task: it arrives through a mediator, and every request from every operator can share one source. A recipient **SHOULD** bound `put-chunk` **per requester** and per slot, in both rate and concurrency, and answer a request beyond the bound with the standard `unavailable` code and a `retryAfter` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)). Because each write carries up to a quarter of a megabyte that the recipient must decode and hash before it can refuse it, an unbounded writer is a cost the recipient pays whether or not the writes succeed.

## Request

The producer is the operator that opened the slot; the recipient is the agent holding it. The request payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Writing the last chunk of the bundle in initiate-import's example

The last chunk of a 524300-byte bundle in 256 KiB chunks holds the 12-byte remainder, which keeps the example readable; a full chunk carries 349526 characters of `data`.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000019",
  "type": "https://trusttasks.org/spec/vta/backup/put-chunk/1.0#request",
  "issuer": "did:example:operator",
  "recipient": "did:example:agent",
  "issuedAt": "2026-01-01T03:01:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000fa",
  "payload": {
    "bundleId": "7e6d5c4b-3a29-4817-a6f5-e4d3c2b1a098",
    "index": 2,
    "digestMultibase": "zQmTcWLvAPe4Txz32vVqZe5jgX4nBPiaBsTTCp4bLXDMzTT",
    "data": "YmFja3VwLXRhaWwh"
  }
}
```

## Response

The producer of the response is the recipient of the request. Its payload is the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error` with one of the codes declared in the front matter, not a `#response` document.

### The chunk was stored and the upload is complete

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-00000000001a",
  "type": "https://trusttasks.org/spec/vta/backup/put-chunk/1.0#response",
  "issuer": "did:example:agent",
  "recipient": "did:example:operator",
  "issuedAt": "2026-01-01T03:01:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000fa",
  "payload": {
    "bundleId": "7e6d5c4b-3a29-4817-a6f5-e4d3c2b1a098",
    "index": 2,
    "stored": true,
    "remainingCount": 0,
    "expiresAt": "2026-01-01T03:11:01Z"
  }
}
```

## Security & Privacy

### Data carried

The request carries **part of a complete, encrypted copy of an agent**. One chunk alone is ciphertext with no context, but the chunks together are the bundle, and the bundle is protected only by the operator's password. A recipient **MUST NOT** log `data`, and **MUST NOT** place staged chunks anywhere other than the slot they were written to. Staged chunks are never readable back — there is no task that returns them, and a recipient **MUST NOT** provide one.

The response carries an acknowledgement, the index, and a count, and nothing from the chunk. `digestMismatch`'s `details` restates the manifest's digest for the index, which the producer committed and therefore already holds.

### Correlation

To intermediaries, a chunked upload is a run of `chunkCount` large, similarly sized requests from one operator to one agent, each followed by a small response. A mediator that cannot read them still learns that a large transfer of that shape happened, and roughly how large. That is intrinsic to moving a bundle over a mediated transport.

`bundleId` joins every write to the `initiate-import` that opened the slot and to its `finalize-import` or `abort`; `index` orders them. Both are intrinsic.

Four conditions are answered as `notFound`: no slot with that id, an export bundle, a `stream` slot, and a slot owned by another operator. Conflating them keeps an opaque handle from becoming an oracle over the agent's backup activity. The other codes are returned only to the slot's creator.

### Retention

A staged chunk is `exchange`-scoped: it is applied, as part of the assembled bundle, by `finalize-import`, or discarded by `abort` or at expiry. Chunks staged against a slot that expires **MUST** be discarded unread, as the whole bundle would be.

The recipient keeps which indices it holds for the life of the slot, since that is what `remainingCount` and `finalize-import`'s completeness check read. It **SHOULD NOT** keep a per-chunk log beyond the slot; the durable fact — that an import was initiated and whether it was finalized — is recorded by `initiate-import` and `finalize-import`.

### Consent/purpose

The purpose is staging a bundle for the recovery `initiate-import` states. A recipient **MUST NOT** inspect, index or partially apply staged chunks, and **MUST NOT** treat a completed upload as authority to apply it: application is `finalize-import`'s, and requires its own authorization and the password.

Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements), this specification does not declare a consent, approval or step-up requirement.
