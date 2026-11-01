---
slug: vtc/elections/ballot-boxes/list
version: "0.1"
title: "VTC Elections — Ballot Boxes — List"
summary: "An administrator reads the voting services the community trusts to hold its elections' boards, with what members are told about each, so a manifest names one the community will accept."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - vtc
  - voting
  - election
  - ballot-box
  - registrar
parties:
  - role: administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: community (the registrar)
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: RECOMMENDED
  rationale: >-
    The answer goes only to a holder of `vtc.elections.manage`, which the community establishes from a verified proof or
    from a transport-authenticated sender. A proof makes the read attributable on every transport.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read that changes nothing. A bounded `issuedAt` still keeps a captured request from being replayed later, by
    someone who no longer holds the capability, to learn which services the community trusts.
sideEffects:
  level: none
  rationale: "Reads the community's configuration. Persists nothing."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Discloses the community's trusted voting services: each one's DID, who operates it, and whether the community's own
    operator does. Members learn the same about the one an election names, in that election's disclosure.
retention:
  class: transient
  rationale: "A view of the community's configuration; the caller reads it again when writing the next manifest."
errorCodes: []
related:
  - vtc/elections/create
  - vtc/elections/show
---

## Abstract

**VTC Elections — Ballot Boxes — List** reads the voting services a community will register an election with: its **trusted ballot boxes** (VTI-VOT-082). A manifest must name one of them in `ballotBox`, or [`vtc/elections/create`](../../../create/0.4/spec.md) refuses it with `ballotBoxNotTrusted`; until now an administrator writing a manifest had to learn the list from whoever operates the community's host. Each entry carries what members are told about that ballot box when an election names it — who operates it, and whether the community's own operator does (VTI-VOT-090) — so the administrator choosing one sees what their members will. The family's shared rules are in [`CONVENTIONS.md`](../../../../../voting/_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a requester without `vtc.elections.manage` (VTI-VOT-080), read from the signer's live access-control entry at execution time, with `permissionDenied`.
2. **MUST** answer every ballot box it trusts at the moment it answers, and only those: exactly the list [`vtc/elections/create`](../../../create/0.4/spec.md) checks `ballotBox` against. A ballot box listed here **MUST** be one a create naming it would not refuse as `ballotBoxNotTrusted`, and one it would refuse **MUST NOT** be listed.
3. **MUST** answer each entry's `did` exactly as `ballotBox` must name it — `create` compares the two verbatim — and its `operator` and `sameOperator` exactly as [`vtc/elections/show`](../../../show/0.3/spec.md) would disclose them, as `ballotBoxOperator` and `sameOperator`, for an election naming it. `sameOperator` **MUST** be true whenever the party operating the community's VTC also operates that ballot box.
4. **MUST** answer an empty `ballotBoxes` when it trusts none.

A conforming **administrator** (`issuer`) **MUST NOT** treat the answer as lasting: the list is the community's configuration, which its operator may change at any time, and `create` checks `ballotBox` against the list as it then stands.

## Authorization

The authority is the **`vtc.elections.manage` capability at the community** (VTI-VOT-080), read from the signer's access-control entry at execution time. The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). Which ballot boxes are trusted is the community's host configuration, not something this family changes: no Trust Task adds or removes one.

## Definitions

- **`ballotBoxes`** (response, REQUIRED) — every trusted ballot box, in the community's own order; at most 256.
- **`did`** — the ballot box's DID, as a manifest's `ballotBox` names it.
- **`operator`** — who operates it, as the community names them.
- **`sameOperator`** — whether the party operating the community's VTC also operates it. A member is told this before taking a ticket, because the separation between registrar and ballot box is what keeps a ballot unlinkable to its voter.

## Request

The payload is empty (or carries only `ext`) — the top-level schema in [`payload.schema.json`](payload.schema.json).

### Read the trusted ballot boxes

```json
{
  "id": "urn:uuid:4b1c2d3e-5f60-4718-89a0-b1c2d3e4f506",
  "type": "https://trusttasks.org/spec/vtc/elections/ballot-boxes/list/0.1#request",
  "issuer": "did:web:carol.example",
  "recipient": "did:webvh:QmCommunity:community.example",
  "issuedAt": "2026-11-01T09:00:00Z",
  "threadId": "urn:uuid:4b1c2d3e-5f60-4718-89a0-b1c2d3e4f506",
  "payload": {},
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:carol.example#key-1",
    "created": "2026-11-01T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z7cDExampleProofValue"
  }
}
```

## Response

The community answers with its trusted ballot boxes — the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). A refusal is a `trust-task-error`.

### Two trusted ballot boxes

```json
{
  "id": "urn:uuid:5c2d3e4f-6071-4829-9ab1-c2d3e4f50617",
  "type": "https://trusttasks.org/spec/vtc/elections/ballot-boxes/list/0.1#response",
  "issuer": "did:webvh:QmCommunity:community.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2026-11-01T09:00:01Z",
  "threadId": "urn:uuid:4b1c2d3e-5f60-4718-89a0-b1c2d3e4f506",
  "payload": {
    "ballotBoxes": [
      {
        "did": "did:webvh:QmBallots:votes.example.net",
        "operator": "Ballots Example Ltd",
        "sameOperator": false
      },
      {
        "did": "did:webvh:QmOwnBox:votes.community.example",
        "operator": "Community Example Hosting",
        "sameOperator": true
      }
    ]
  }
}
```

### None trusted

```json
{
  "id": "urn:uuid:6d3e4f50-7182-493a-8bc2-d3e4f5061728",
  "type": "https://trusttasks.org/spec/vtc/elections/ballot-boxes/list/0.1#response",
  "issuer": "did:webvh:QmCommunity:community.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2026-11-01T09:00:01Z",
  "threadId": "urn:uuid:4b1c2d3e-5f60-4718-89a0-b1c2d3e4f506",
  "payload": {
    "ballotBoxes": []
  }
}
```

### Refused: not an elections manager

```json
{
  "id": "urn:uuid:7e4f5061-8293-4a4b-9cd3-e4f506172839",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:webvh:QmCommunity:community.example",
  "recipient": "did:web:dave.example",
  "issuedAt": "2026-11-01T09:00:01Z",
  "threadId": "urn:uuid:8f506172-93a4-4b5c-8de4-f50617283940",
  "payload": {
    "code": "permissionDenied",
    "message": "Reading the trusted ballot boxes takes vtc.elections.manage.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The request carries nothing. The response names voting services and their operators — organisations, not members — and a fact about the community's own hosting (`sameOperator`). No member, ballot or ticket appears.

### Correlation

The community is declared `identifierScope: public`, as throughout the family (CONVENTIONS §3): it is addressed by the DID its manifests name as `community`. The ballot boxes are public services, addressed by the DIDs every manifest naming them carries; listing them links nothing that an election's own disclosure does not.

### Retention

Transient. The answer is a view of the community's configuration; nothing about the read is kept beyond the community's ordinary request log.

### Consent/purpose

The list is disclosed so that an administrator can write a manifest the community will accept, and see what their members will be told about the ballot box they choose. It is answered only to holders of `vtc.elections.manage`: which services a community trusts, and whether it runs one itself, says something about how it is hosted, which its members learn only for the ballot box an election actually names.
