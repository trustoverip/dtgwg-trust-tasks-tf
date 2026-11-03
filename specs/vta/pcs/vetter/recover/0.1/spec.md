---
slug: vta/pcs/vetter/recover
version: "0.1"
title: "VTA PCS — Vetter — Recover"
summary: "A vetter's client asks its VTA to restore the vetter key a community has bound the member to. The VTA searches the keys it can derive for the context and, if one matches, makes it the engine's key — so the vetter can enrol again without an administrator's reset."
status: draft
targetFrameworkVersion: "0.6.0"
category: identity
keywords:
  - pcs
  - hidden-vetting
  - vetting
  - recovery
  - vta
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Client driving the PCS engine
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: PCS engine (the VTA)
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: >-
    The caller already has standing over the context, and the task can only ever select a key the
    VTA derives for that context. A proof adds attribution in the audit trail.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A recovery replaces the key an engine uses. Placing the request in a window is what lets the
    VTA refuse a replay of an old recovery after the vetter has moved on.
sideEffects:
  level: mutating
  rationale: >-
    May replace the vetter engine's key reference with another key derived for the same context,
    and discard the engine's state under the replaced key. Refused when that state holds anything a
    community has issued.
subjectPath: /contextId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: a context, a community, a member DID and a public identifier. Out: the engine's summary.
retention:
  class: durable
  rationale: >-
    The engine record now names the recovered key, for the rest of the vetter's life in the
    community.
errorCodes:
  - code: vta/pcs/vetter/recover:notFound
    meaning: "No key this VTA can derive for the context has the PCS identifier `boundId`. The member asks the community's administrators for a reset (vtc/vetting/vetters/pcs-binding/reset/0.1)."
    retryable: false
  - code: vta/pcs/vetter/recover:heldInAnotherContext
    meaning: "A vetter engine for this community with the identifier `boundId` exists in another context the caller has standing over. Vet from that context; a key cannot move between contexts."
    retryable: false
  - code: vta/pcs/vetter/recover:memberMismatch
    meaning: "A vetter engine exists for this context and community under another member DID."
    retryable: false
  - code: vta/pcs/vetter/recover:engineEnrolled
    meaning: "The engine's current key holds a class credential, tokens or attestations from the community. A key a community has issued to is never replaced."
    retryable: false
related:
  - vtc/vetting/vetters/pcs-binding/show
  - vtc/vetting/vetters/pcs-binding/reset
  - vta/pcs/root-request
  - vta/pcs/vetter/show
---

## Abstract

A community binds each vetter to one PCS identifier at their first enrolment
([`vtc/vetting/vetters/pcs-root`](../../../../../vtc/vetting/vetters/pcs-root/0.1/spec.md)). The VTA
never stores the key behind it: it derives the key from the context's base, at a path the engine
record keeps ([conventions §2](../../../../_shared/0.4/CONVENTIONS.md#2-custody)). If that record is
replaced — a re-created engine, a restored VTA that lost the record but kept its seed — the next
[`vta/pcs/root-request`](../../../root-request/0.1/spec.md) creates a fresh key, and the community
refuses it with `identifierRebound`.

The key is usually not gone: the seed still derives it. **VTA PCS — Vetter — Recover** takes the
identifier the community expects ([`vtc/vetting/vetters/pcs-binding/show`](../../../../../vtc/vetting/vetters/pcs-binding/show/0.1/spec.md)),
searches the keys the VTA can derive for the context, and if one matches, makes it the engine's
key. The vetter then enrols as before, with no administrator involved and no change to who the
community counts. Only when this fails does the member need a reset
([`vtc/vetting/vetters/pcs-binding/reset`](../../../../../vtc/vetting/vetters/pcs-binding/reset/0.1/spec.md)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../../_shared/0.4/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **consumer** (the PCS engine) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements), and refuse a caller
   without standing under [Authorization](#authorization) with `permissionDenied`.
2. If an engine exists for (`contextId`, `community`) under a `memberDid` other than the one
   named, refuse with `memberMismatch`.
3. If the engine's key already has the identifier `boundId`, change nothing and answer
   `recovered: false`.
4. If the engine's current key holds a class credential, unspent or spent tokens, or attestations
   it made, refuse with `engineEnrolled`. Its key is one a community has issued to; replacing it
   would strand that state, and the community it was issued by bound that key, not `boundId`.
5. Search for a key with the identifier `boundId` among the keys it can derive for `contextId`
   under the suite the engine runs, and **MUST NOT** search any other context's keys. The search
   **MUST** be bounded (an implementation states its bound), and **MUST** consider every seed
   generation the VTA still holds.
6. If it finds one: replace the engine's key reference with the found key's, discard the engine
   state held under the replaced key (enrolments and draws in flight — rule 4 has established
   there is nothing else), create the engine if none existed, audit the recovery without the
   identifier, and answer `recovered: true` with the engine's summary.
7. If it finds none, and an engine for `community` whose identifier is `boundId` exists under
   another context the caller has standing over, refuse with `heldInAnotherContext`, naming that
   context in the message. Otherwise refuse with `notFound`.
8. Never return, log or audit a key, a seed or a derivation path.

[Conventions §3](../../../../_shared/0.4/CONVENTIONS.md#3-one-vetter-key-per-community) — one key per
(context, community) for the engine's whole life — holds across a recovery. The key the engine
ends with is the key the community bound; the key it discards is one no community accepted.

## Authorization

The authority is **standing over the context** the engine belongs to ([conventions §4](../../../../_shared/0.4/CONVENTIONS.md#4-authorization)). Where it is expressed as a capability, the registered value is **`pcsVetter`** — see `Capability` in [`device/_shared`](../../../../../device/_shared/0.2/device-binding.schema.json). It is not `sign`, and a VTA **MUST NOT** require `sign` for this task. Verifying the `issuer`, the transport identity or the `proof` establishes who asked ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)), never that they may.

## Definitions

- **`contextId`**, **`community`**, **`memberDid`** — the vetter engine, as in
  [`vta/pcs/root-request`](../../../root-request/0.1/spec.md).
- **`boundId`** — the identifier the community bound the member to.
- **`vetter`** (response) — the engine's summary (`VetterSummary`) after the task.

## Request

The client sends the request to its VTA. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:9d4e1a77-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/pcs/vetter/recover/0.1#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-11-03T09:00:00Z",
  "threadId": "urn:uuid:9d4e1a77-0000-4000-8000-0000000000ff",
  "payload": {
    "contextId": "personal/communities",
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "memberDid": "did:webvh:QmMember:vta.example.com:alice",
    "boundId": "z4FvpxtP31f2HAEo4V9ffW3VRH1yZJAXY"
  }
}
```

## Response

The VTA answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:9d4e1a77-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/pcs/vetter/recover/0.1#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-11-03T09:00:02Z",
  "threadId": "urn:uuid:9d4e1a77-0000-4000-8000-0000000000ff",
  "payload": {
    "recovered": true,
    "vetter": {
      "contextId": "personal/communities",
      "community": "did:webvh:QmCommunity:vtc.example.org",
      "memberDid": "did:webvh:QmMember:vta.example.com:alice",
      "suite": "ps-ddh-bls12381",
      "id": "z4FvpxtP31f2HAEo4V9ffW3VRH1yZJAXY",
      "enrolledLabels": [],
      "pendingLabels": [],
      "tokensFree": 0,
      "tokensHeld": 0,
      "drawsInFlight": [],
      "attested": 0
    }
  }
}
```

## Security & Privacy

### Data carried

A public identifier in, the engine's summary out. Nothing that would let the client act as the
vetter.

### Correlation

None. The identifier is the member's own, given to their own VTA.

### Retention

The engine record keeps the recovered key's path, as it would have had the record never been
lost.
