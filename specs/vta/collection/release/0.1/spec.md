---
slug: vta/collection/release
version: "0.1"
title: "VTA Collection — Release"
summary: "A client releases the collection claim for a persona DID — its own, ending it at once once it has stopped collecting, or another client's, which then ends at its stated end without renewal — so that the VTA collects for the DID again."
status: draft
targetFrameworkVersion: "0.6.0"
category: messaging
keywords:
  - collection
  - mediator
  - persona
  - vta
  - lease
parties:
  - role: Client acting for the principal
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: >-
    Normally carried by an authenticated transport between a client and its own VTA. A release
    confers nothing and moves no key; the caller's standing in the context is what the VTA checks.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task changes a record, so the VTA must be able to place the request in a bounded window to
    absorb a repeat (SPEC §7.2 item 11).
sideEffects:
  level: mutating
  rationale: >-
    Ends a collection claim, or makes it unrenewable, so that the VTA collects for the identifier
    again when it ends. A repeat changes nothing.
subjectPath: /contextId
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: a context and an identifier. Out: the identifier and, where a claim was in force, when it
    ends.
retention:
  class: exchange
  rationale: >-
    Nothing is kept beyond the change to the claim, which the VTA keeps only until the claim ends.
errorCodes:
  - code: vta/collection/release:notCollectable
    meaning: >-
      `identifier` is not one the VTA collects for in `contextId` (VTI-NTF-080), so there is no claim
      to release.
    retryable: false
related:
  - vta/collection/claim
  - vta/collection/list
---

## Abstract

The **VTA Collection — Release** Trust Task ends a collection claim made with
[`vta/collection/claim`](../../claim/0.1/spec.md). A client that stops collecting for a persona DID
— signing out, or handing the persona back to the VTA — closes its connection first and then
releases its claim, and the VTA collects for the DID again at once. Any other party with standing
in the context may release a claim too — a principal cleaning up after a device it no longer has,
say — but the claimant may still be connected, and only its own clock tells it to stop, so such a
release makes the claim unrenewable and the VTA takes the DID back when the claim ends, never
before (VTI-NTF-084).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **producer** that releases its own claim **MUST** stop collecting for the identifier,
and close every connection that collects for it, before it sends this task (VTI-NTF-083).

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements), and refuse with
   `permissionDenied` a caller without standing under [Authorization](#authorization).
2. Refuse with `vta/collection/release:notCollectable` an `identifier` VTI-NTF-080 does not require
   it to collect for in `contextId`.
3. Where the caller is the claim's claimant, end the claim at once, resume collecting for the
   identifier, and answer with the instant it ended.
4. Where the caller is not the claimant, make the claim unrenewable, leave its end unchanged, and
   answer with that end; it **MUST NOT** resume collecting before that end plus its own clock-skew
   tolerance (VTI-NTF-084).
5. Where no claim is in force, change nothing and answer without `endsAt`.
6. Record the release in its audit trail, naming the caller and the claimant.

**Retry class** (VTI-OPS-060): the second — a durable effect whose repeat is harmless.

**A repeated request** sets a state: releasing an identifier with no claim in force, or one already
released, succeeds and changes nothing.

## Authorization

**Standing in the trust context** named by `contextId`, as for
[`vta/collection/claim`](../../claim/0.1/spec.md#authorization). A party other than the claimant
with that standing may release the claim, because the identifier is the principal's and not the
claimant's; it cannot end the claim before its end, because the claimant would then be the second
connection.

## Definitions

- **`contextId`** — the trust context the identifier belongs to.
- **`identifier`** — the DID whose claim to release.
- **`endsAt`** (response) — when the released claim ends; absent when no claim was in force.

## Request

The client sends its VTA the top-level schema in [`payload.schema.json`](payload.schema.json).

### A client hands its persona back to the VTA

```json
{
  "id": "urn:uuid:7f8091a2-b3c4-4d5e-8f6a-7b8c9d0e1f2a",
  "type": "https://trusttasks.org/spec/vta/collection/release/0.1",
  "issuer": "did:key:z6MkDesktopClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2027-03-01T18:00:00Z",
  "payload": {
    "contextId": "personal",
    "identifier": "did:webvh:QmPersona:persona.example"
  }
}
```

## Response

The VTA answers with the sub-schema reachable via `$anchor: "response"`.

### Released by its claimant

```json
{
  "id": "urn:uuid:8091a2b3-c4d5-4e6f-9a7b-8c9d0e1f2a3b",
  "type": "https://trusttasks.org/spec/vta/collection/release/0.1#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkDesktopClient",
  "issuedAt": "2027-03-01T18:00:01Z",
  "threadId": "urn:uuid:7f8091a2-b3c4-4d5e-8f6a-7b8c9d0e1f2a",
  "payload": {
    "identifier": "did:webvh:QmPersona:persona.example",
    "endsAt": "2027-03-01T18:00:01Z"
  }
}
```

## Security & Privacy

### Data carried

A context and an identifier in; the identifier and an instant out.

### Correlation

Nothing about a release reaches the mediator or any sender beyond the change of the connection
that collects for the identifier, which the mediator sees whichever party holds it.

### Retention

Nothing beyond the change to the claim and the audit record.

### Consent/purpose

A release returns the identifier to the VTA's collection. It takes nothing from the claimant but the
claim: its standing, and any keys it holds, are unchanged, and a principal who wants a client to stop
holding a persona's keys removes its standing and rotates those keys.
