---
slug: vta/pcs/trustee/read
version: "0.2"
title: "VTA PCS — Trustee Read"
summary: "A trustee's client asks its VTA for a voting/trustees/status or voting/election/show request signed as the trustee, so it can follow the ceremony at the ballot box without holding the trustee's key."
status: draft
targetFrameworkVersion: "0.6.0"
category: identity
keywords:
  - pcs
  - voting
  - trustee
  - vta
  - persona
parties:
  - role: Client driving the trustee's VTA
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: The trustee's VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    The request makes the VTA sign as the trustee. Where the transport does not already
    authenticate the caller, the proof is the only thing between that signature and anyone who
    can reach the VTA.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The answer is a signature as the trustee; placing the request in time bounds how long a
    captured request could be replayed to obtain one.
sideEffects:
  level: none
  rationale: >-
    The engine builds and signs one request document. Nothing it holds changes, and the read it
    signs changes nothing at the ballot box either.
subjectPath: /contextId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    The VTA signs as the trustee. In: which of two reads, of which election. Out: that read as a
    signed document — no key, and nothing the ballot box would not show the trustee anyway.
retention:
  class: transient
  rationale: >-
    The engine keeps nothing; the signed document is the client's to send.
errorCodes:
  - code: vta/pcs/trustee/read:trusteeNotHeld
    meaning: "`trustee` names a DID the engine holds no `authentication` key of, or one the caller may not act as."
    retryable: false
related:
  - voting/trustees/status
  - voting/election/show
  - vta/pcs/trustee/enrol
---

## Abstract

The **VTA PCS — Trustee Read** Trust Task signs one of the two reads a trustee needs to take its
part in an election's ceremony — [`voting/trustees/status`](../../../../../voting/trustees/status/0.1/spec.md),
which says what the ceremony needs next, and [`voting/election/show`](../../../../../voting/election/show/0.2/spec.md),
which carries the manifest the trustee tasks take and the time voting closed — as the trustee.
On a private board the ballot box answers those reads only to a party it can identify as one of
the manifest's trustees (CONVENTIONS §9.6), so the request must be signed by the trustee; the
trustee's key lives in its VTA, and this task lets a client that holds none read as the trustee,
as [`vta/pcs/voter/read`](../../../voter/read/0.2/spec.md) does for a voter. The family's shared
rules are in [`CONVENTIONS.md`](../../../../../voting/_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Changes from 0.1

- The `election` read signs a [`voting/election/show/0.2`](../../../../../voting/election/show/0.2/spec.md) request, not `0.1`: the response's `document.type` enum names `https://trusttasks.org/spec/voting/election/show/0.2` in place of `…/0.1`, and the `read` and `ReadDocument` descriptions and Conformance item 4 say so. The `status` read is unchanged: `voting/trustees/status` has no version but `0.1`.

`voting/election/show/0.2` renames the response manifest's window members (`registrationOpens`, `votingOpens`, `votingCloses` → `…At`); its request is the same `{ community, electionId }`. A trustee client that follows this task reads the manifest it gets back, so the version it is signed for decides which spelling the client receives. `0.2` names the newer read only, rather than also accepting `0.1`: `read` is closed (Conformance item 5), the client does not choose the version, and a client built for this version of the signer pairs with the newer version of the read. A client that still parses the `0.1` manifest uses `vta/pcs/trustee/read/0.1`.

The `document.type` enum changes value, a change on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.1` remains published.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **producer** (the client) **MUST** send the ballot box the returned `document`
unchanged, and **MUST NOT** ask for a read it does not mean to send.

A conforming **consumer** (the PCS engine) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse a caller without standing under [Authorization](#authorization) with `permissionDenied`.
3. Refuse with `vta/pcs/trustee/read:trusteeNotHeld` unless it holds an `authentication` key of `trustee` that the caller may act as, under the same rule by which it acts as the trustee in the other `vta/pcs/trustee/*` tasks.
4. Build the request document — a fresh random `id`, the type `https://trusttasks.org/spec/voting/trustees/status/0.1` for `status` or `https://trusttasks.org/spec/voting/election/show/0.2` for `election`, `issuer` `trustee`, `recipient` `ballotBox`, an `issuedAt` of now and an `expiresAt` no more than an hour later, and the payload `{ community, electionId }` from the request — and sign it with that key for `authentication`.
5. Sign nothing else: no other type, no other payload. `read` is closed; a later read is a later version of this task.

## Authorization

The authority is **standing over the context** the trustee persona lives in, expressed as the capability **`pcsTrustee`**, and standing to act as `trustee` ([conventions §4](../../../../../voting/_shared/0.1/CONVENTIONS.md#4-authorization)). It is not `sign`, and a VTA **MUST NOT** require `sign` for this task: the engine signs only these two reads.

## Definitions

- **`contextId`** — the VTA context the trustee's keys live in.
- **`community`** — the community's DID.
- **`electionId`** — the election.
- **`trustee`** — the trustee's DID.
- **`ballotBox`** — the ballot box's DID, the manifest's `ballotBox`.
- **`read`** — `status` or `election`.
- **`document`** (response) — the signed read.

## Request

The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000000a03001",
  "type": "https://trusttasks.org/spec/vta/pcs/trustee/read/0.2#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-11-02T09:00:00Z",
  "payload": {
    "contextId": "personal/trustee",
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "electionId": "board-2026",
    "trustee": "did:webvh:QmTrustee1:trustees.example.org",
    "ballotBox": "did:webvh:QmVote:vote.example.net",
    "read": "status"
  }
}
```

## Response

The VTA answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000000a03002",
  "type": "https://trusttasks.org/spec/vta/pcs/trustee/read/0.2#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-11-02T09:00:01Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000000a03001",
  "payload": {
    "document": {
      "id": "urn:uuid:0e2f9b1c-5d4a-4e3b-9a8c-7d6e5f4a3b2d",
      "type": "https://trusttasks.org/spec/voting/trustees/status/0.1",
      "issuer": "did:webvh:QmTrustee1:trustees.example.org",
      "recipient": "did:webvh:QmVote:vote.example.net",
      "issuedAt": "2026-11-02T09:00:01Z",
      "expiresAt": "2026-11-02T10:00:01Z",
      "payload": {
        "community": "did:webvh:QmCommunity:vtc.example.org",
        "electionId": "board-2026"
      },
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "verificationMethod": "did:webvh:QmTrustee1:trustees.example.org#key-0",
        "created": "2026-11-02T09:00:01Z",
        "proofPurpose": "authentication",
        "proofValue": "z4ExampleTrusteeProofValue"
      }
    }
  }
}
```

## Security & Privacy

### Data carried

The request names the trustee, the election and which of two reads it signs; the response is that
read, signed. No key, share or ballot crosses.

### Correlation

The trustee is named in the manifest, and its every step at the ballot box is signed as it
already: a read signed as the trustee tells the ballot box nothing it could not attribute anyway.

### Retention

Nothing is kept by the engine.

### Consent/purpose

The trustee's VTA signs as the trustee only to read the election it is a trustee of; a client
holding `pcsTrustee` can make it sign nothing else through this task.
