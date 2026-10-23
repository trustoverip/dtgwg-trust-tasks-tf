---
slug: vta/witness/list
version: "0.2"
title: "VTA Witness — List"
summary: "A client reads back from its VTA the Verifiable Witness Credentials stored for its principal, each with the outcome evidence a verifier needs to pair it with its session."
status: draft
targetFrameworkVersion: "0.6.0"
category: credentials
keywords:
  - witness
  - vwc
  - relationship
  - outcome-evidence
  - vta
  - credential-vault
parties:
  - role: Client acting for the principal
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: The principal's VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The response returns credential material — the VWCs and the two documents each was issued in — so the VTA
    must be able to attribute every read to a specific key of the caller, not to whatever holds the transport
    session, as for vault/credentials/get. A response that carries secret material back MUST be proven (SPEC
    §7.3 item 8, §4.7.1): the documents inside it are evidence on the witness's own proofs, but only the VTA's
    proof attributes the listing — what was returned, and that a page was the whole of it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A request that discloses credential material is worth replaying: captured once, it discloses the same
    credentials each time it is resent. `issuedAt` bounds how old a request the VTA will act on. Consequential
    under SPEC §3 (`exposure.discloses: secret`), so SPEC §7.3 item 17 sets REQUIRED as the floor.
sideEffects:
  level: none
  rationale: >-
    A read of what vta/witness/accept stored. Nothing is created, altered or consumed, and no session record is
    involved.
exposure:
  discloses: secret
  ingests: none
  actsAsSubject: false
  rationale: >-
    Out: each VWC — a statement about the principal's relationship DID and the digest of its relationship
    credential — with the witness/session document and the witness's submit response it was issued in, which
    carry the relationship's identifiers, its threads and both parties' relationship DIDs. That is the material a
    verifier sees only when the principal presents it. In: optional filters naming DIDs the principal already
    holds. The VTA signs nothing.
retention:
  class: transient
  rationale: >-
    The request is consumed and discarded. The VTA keeps its audit record of the read; what it lists, it already
    kept under vta/witness/accept's durable retention.
errorCodes:
  - code: vta/witness/list:cursorInvalid
    meaning: >-
      The supplied `cursor` cannot be honoured — expired, malformed, or issued against vault state no longer
      recognised. Consumers SHOULD retry from the first page without a cursor.
    retryable: true
related:
  - vta/witness/accept
  - vta/witness/open
  - vtc/members/personhood/assert
  - vault/credentials/get
---

## Abstract

[`vta/witness/accept`](../../accept/0.2/spec.md) stores a Verifiable Witness Credential (VWC) in the principal's credential vault **with its evidence**: the `witness/session` document that opened the session, and the witness's `witness/session/submit` `#response` that closed it. A VWC shown without both counts for nothing, because a verifier cannot pair it with a session that completed ([`witness/session/submit/0.2`](../../../../witness/session/submit/0.2/spec.md), *This response is the outcome evidence*).

Until now the only copy of that evidence a client received was `accept`'s own answer. A client that lost it — it stopped between `accept` returning and recording the result, or it is a second device of the same principal — could see the VWC in the vault and could not present it: [`vault/credentials/query/0.1`](../../../../vault/credentials/query/0.1/spec.md) returns body-free descriptors and [`vault/credentials/get/0.1`](../../../../vault/credentials/get/0.1/spec.md) the credential alone.

**VTA Witness — List** reads back every stored VWC the caller may read, each with its evidence exactly as stored, so the principal's client can rebuild its record of a witnessing at any time.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Changes from 0.1

Paging moves to the registry-wide pagination convention — request `cursor` + `limit`, response `nextCursor` absent on the last page — so every list task pages the same way:

* Request **`pageSize`** → **`limit`**. Same type and bounds (integer, 1–100).
* Response **`cursor`** → **`nextCursor`**. Same opaque string (1–4096 characters), sent back as the next request's `cursor`.
* Response **`truncated`** is **removed**, and `witnessings` is now the only required response member. In 0.1 it meant the VTA stopped before the last match, which is exactly what the presence of `nextCursor` now says; its absence marks the last page.

The request's `cursor` member keeps its name and meaning. Nothing else in the payload moves.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.1` remains published.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements), and refuse a caller without standing with `permissionDenied`.
2. List only VWCs that `vta/witness/accept` stored — under any version of it — in a context the caller may act in ([Authorization](#authorization)), and only those whose vault entry is active. An archived or deleted entry is not listed.
3. Apply every filter present: `relationshipDid` against the VWC's `credentialSubject.id`, and `witness` against its `issuer`. A filter naming a DID the VTA does not hold, or holds in a context the caller cannot act in, matches nothing; the answer is then an empty `witnessings`, exactly as for a DID with no VWCs. It **MUST NOT** answer either case with an error, so the answer reveals nothing about DIDs the caller cannot see.
4. Return each entry's `evidence.session` and `evidence.outcome` exactly as `vta/witness/accept` stored them, byte for byte, and its `credentialId`, `witness` and `vwcDigestMultibase` as `accept` answered them. It **MUST NOT** rebuild, re-sign or re-serialise either document.
5. Order entries by `acceptedAt`, oldest first, and keep that order across the pages of one cursor.
6. Honor `limit`, and return a `nextCursor` if, and only if, it stopped before the last match. It **MUST NOT** accept a `cursor` it did not issue; a cursor it cannot honour is `vta/witness/list:cursorInvalid`.

A conforming **producer** (the client) **MUST NOT** construct or parse a `cursor`, and **MUST NOT** read a page carrying a `nextCursor` as a complete account of what is stored.

## Authorization

The authority is **standing over the context in which each VWC is stored** — the context of the relationship DID's key, where `vta/witness/accept` stored it. Where that entitlement is expressed as a capability, the registered value is **`witnessRequest`**, as for the rest of the family ([conventions §4](../../../_shared/0.6/CONVENTIONS.md#4-authorization)). A VTA **MUST NOT** satisfy this task under a grant of `sign` alone: it reads the credential vault, which `sign` does not confer.

A VTA **MUST NOT** list a VWC stored in a context the caller cannot act in, whatever the filters name. Each page contains only what the caller could have received from `accept` itself.

Verifying the document's `issuer`, transport identity or `proof` establishes who asked ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)), never that they may.

## Definitions

- **`relationshipDid`** — filter: the relationship DID a VWC names as `credentialSubject.id`.
- **`witness`** — filter: the witness that issued a VWC.
- **`cursor`**, **`limit`** — paging, as in [`vta/credentials/list`](../../../credentials/list/0.2/spec.md).
- **`witnessings`** (response) — one entry per stored VWC that matched.
  - **`credentialId`** — the vault handle `accept` returned.
  - **`witness`** — the VWC's `issuer`.
  - **`relationshipDid`** — the VWC's `credentialSubject.id`.
  - **`vwcDigestMultibase`** — the VWC's digest, as the witness returned it and the VTA checked it.
  - **`acceptedAt`** — when `accept` stored it.
  - **`evidence`** — `session` and `outcome`, the witness family's shared [`OutcomeEvidence`](../../../../witness/_shared/0.1/outcome-evidence.schema.json), of whichever version the session was conducted under.
- **`nextCursor`** (response) — the VTA stopped early; continues the listing, and is absent on the last page.

### Why the VWC is not repeated in an entry

As in `vta/witness/accept`: the VWC travels once, inside `evidence.outcome`, under the witness's proof. A second copy beside it could only disagree with it, and a stored credential is never re-serialised.

### Why there is no lookup by session

A VWC's `taskContext` is its session id, and `evidence.session.id` is the same value, so a client looking for one session's VWC filters by relationship DID and reads it from the entries. A session-id filter would add a second key to the same record with no new capability.

## Request

The client sends the request to its VTA. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### The VWCs of one relationship

```json
{
  "id": "urn:uuid:5b0f3c1e-0000-4000-8000-000000000301",
  "type": "https://trusttasks.org/spec/vta/witness/list/0.2#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-10-16T08:00:00Z",
  "payload": {
    "relationshipDid": "did:peer:2.Ez6LSAliceForBobRelationship.Vz6MkAliceForBobRelationship",
    "limit": 20
  }
}
```

## Response

The VTA answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### One stored VWC, with its evidence

`evidence.session` and `evidence.outcome` are the documents from the [`vta/witness/accept`](../../accept/0.2/spec.md#response) example, abbreviated here to their first members.

```json
{
  "id": "urn:uuid:5b0f3c1e-0000-4000-8000-000000000302",
  "type": "https://trusttasks.org/spec/vta/witness/list/0.2#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-10-16T08:00:01Z",
  "threadId": "urn:uuid:5b0f3c1e-0000-4000-8000-000000000301",
  "payload": {
    "witnessings": [
      {
        "credentialId": "vwc-alice-to-bob",
        "witness": "did:webvh:QmWitness:witness.example.org:a",
        "relationshipDid": "did:peer:2.Ez6LSAliceForBobRelationship.Vz6MkAliceForBobRelationship",
        "vwcDigestMultibase": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
        "acceptedAt": "2026-10-15T09:00:07Z",
        "evidence": {
          "session": {
            "id": "urn:uuid:3a2b1c0d-0000-4000-8000-000000000201",
            "type": "https://trusttasks.org/spec/witness/session/0.2#request",
            "payload": {
              "parties": [
                "did:peer:2.Ez6LSAliceForBobRelationship.Vz6MkAliceForBobRelationship",
                "did:peer:2.Ez6LSBobForAliceRelationship.Vz6MkBobForAliceRelationship"
              ]
            }
          },
          "outcome": {
            "id": "urn:uuid:3a2b1c0d-0000-4000-8000-000000000204",
            "type": "https://trusttasks.org/spec/witness/session/submit/0.2#response",
            "payload": {
              "vwc": { "type": ["VerifiableCredential", "DTGCredential", "StatementCredential"] },
              "vwcDigestMultibase": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR"
            }
          }
        }
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

Out: each matching VWC with both of its evidence documents, which together name the relationship DID, the counterparty's relationship DID, the witness, the relationship thread and the session. In: filters naming DIDs the principal holds. Under the member link the member DID appears in no VWC and in neither evidence document, so this task never returns it.

### Correlation

Every entry is the material a verifier would see when the principal presents that VWC, and the same exact join keys ([`vta/witness/accept`](../../accept/0.2/spec.md#correlation)). Returned to the principal's own client it discloses nothing the principal did not already hold; that is why the authorization rule confines a page to contexts the caller may act in, and why an unreachable DID matches nothing rather than answering an error. A listing of every relationship's witnessings is a map of the principal's witnessed relationships, so a client **SHOULD** filter by `relationshipDid` when it needs one relationship's.

### Retention

The VTA keeps nothing from the request beyond its audit record. A client that rebuilds its records from a listing holds the evidence on the same terms as one that kept `accept`'s answer, and **SHOULD** keep both documents for as long as it keeps the VWC: either one alone cannot be presented.

### Consent/purpose

The listing exists so the principal's client can present a stored VWC, on the principal's selection, to a verifier such as [`vtc/members/personhood/assert/0.2`](../../../../vtc/members/personhood/assert/0.2/spec.md). Listing a VWC is not presenting it. This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
