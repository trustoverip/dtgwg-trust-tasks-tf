---
slug: vta/pcs/voter/read
version: "0.1"
title: "VTA PCS — Voter Read"
summary: "A voter's client asks its VTA for a vtc/elections/list or vtc/elections/show request signed as the voter's membership DID, so it can read the community's elections as the member without holding the member's key."
status: draft
targetFrameworkVersion: "0.6.0"
category: identity
keywords:
  - pcs
  - voting
  - election
  - vta
  - persona
parties:
  - role: Client driving the PCS engine
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: PCS engine (the VTA)
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    The request makes the VTA sign as the voter's membership DID. Where the transport does not
    already authenticate the caller, the proof is the only thing between that signature and
    anyone who can reach the VTA.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The answer is a signature as the member; placing the request in time bounds how long a
    captured request could be replayed to obtain one.
sideEffects:
  level: none
  rationale: >-
    The engine builds and signs one request document. Nothing it holds changes, and the read it
    signs changes nothing at the community either.
subjectPath: /contextId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    The VTA signs as the voter's membership DID. In: which of two reads, and its payload. Out:
    that read as a signed document — no key, and nothing the community would not show the
    member anyway.
retention:
  class: transient
  rationale: >-
    The engine keeps nothing; the signed document is the client's to send.
errorCodes:
  - code: vta/pcs/voter/read:memberNotHeld
    meaning: "`member` names a DID the engine holds no `authentication` key of, or one the caller may not act as."
    retryable: false
related:
  - vtc/elections/list
  - vtc/elections/show
  - vta/pcs/voter/register
---

## Abstract

The **VTA PCS — Voter Read** Trust Task signs one of the two reads a voter needs before taking
a ticket — [`vtc/elections/list`](../../../../../vtc/elections/list/0.1/spec.md) and
[`vtc/elections/show`](../../../../../vtc/elections/show/0.1/spec.md) — as the voter's
membership DID. A community answers those reads only to a party its roll admits, so the
request must be signed by the member; the member's key usually lives in their VTA, and this
task lets a client that holds none read as the member, exactly as
[`vta/pcs/voter/register`](../../register/0.1/spec.md)'s `ticketDocument` lets it ask for the
ticket. The family's shared rules are in
[`CONVENTIONS.md`](../../../../../voting/_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **producer** (the client) **MUST** send the community the returned `document`
unchanged, and **MUST NOT** ask for a read it does not mean to send.

A conforming **consumer** (the PCS engine) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse a caller without standing under [Authorization](#authorization) with `permissionDenied`.
3. Refuse with `malformedRequest` a `payload` that does not validate against the request schema of the task `read` names.
4. Refuse with `vta/pcs/voter/read:memberNotHeld` unless it holds an `authentication` key of `member` that the caller may act as, under the same rule by which it acts as a persona elsewhere.
5. Build the request document — a fresh random `id`, the type `https://trusttasks.org/spec/vtc/elections/list/0.1` or `…/show/0.1`, `issuer` `member`, `recipient` `community`, an `issuedAt` of now and an `expiresAt` no more than an hour later, and exactly `payload` — and sign it with that key for `authentication`.
6. Sign nothing else: no other type, no other payload. `read` is closed; a later read is a later version of this task.

## Authorization

The authority is **standing over the context** the voter belongs to ([conventions §4](../../../../../voting/_shared/0.1/CONVENTIONS.md#4-authorization)), expressed as the capability **`pcsVoter`**, and standing to act as `member`. It is not `sign`, and a VTA **MUST NOT** require `sign` for this task: the engine signs only these two reads, as `vta/pcs/voter/register` signs only the ticket request.

## Definitions

- **`contextId`** — the context the voter belongs to.
- **`community`** — the community's DID.
- **`member`** — the voter's membership DID at the community.
- **`read`** — `list` or `show`.
- **`payload`** — that read's request payload.
- **`document`** (response) — the signed read.

## Request

The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000000a02001",
  "type": "https://trusttasks.org/spec/vta/pcs/voter/read/0.1#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-11-02T09:00:00Z",
  "payload": {
    "contextId": "personal/communities",
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "member": "did:key:z6MkmMemberPersonaExampleKey22222222222222222222",
    "read": "show",
    "payload": { "electionId": "board-2026" }
  }
}
```

## Response

The VTA answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000000a02002",
  "type": "https://trusttasks.org/spec/vta/pcs/voter/read/0.1#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-11-02T09:00:01Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000000a02001",
  "payload": {
    "document": {
      "id": "urn:uuid:0e2f9b1c-5d4a-4e3b-9a8c-7d6e5f4a3b2c",
      "type": "https://trusttasks.org/spec/vtc/elections/show/0.1",
      "issuer": "did:key:z6MkmMemberPersonaExampleKey22222222222222222222",
      "recipient": "did:webvh:QmCommunity:vtc.example.org",
      "issuedAt": "2026-11-02T09:00:01Z",
      "expiresAt": "2026-11-02T10:00:01Z",
      "payload": { "electionId": "board-2026" },
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "verificationMethod": "did:key:z6MkmMemberPersonaExampleKey22222222222222222222#z6MkmMemberPersonaExampleKey22222222222222222222",
        "created": "2026-11-02T09:00:01Z",
        "proofPurpose": "authentication",
        "proofValue": "z4ExampleMemberProofValue"
      }
    }
  }
}
```

## Security & Privacy

### Data carried

The request names the member and which of the community's elections reads it signs; the
response is that read, signed. No key, ticket or ballot crosses.

### Correlation

The signed read is the member's own request to their own community, which the community
already attributes to them. Nothing in it links to a ticket or a cast: those are made under a
key per election that never signs as the member.

### Retention

Nothing is kept by the engine.

### Consent/purpose

The member's VTA signs as the member only to read the community's elections; a client holding
`pcsVoter` can make it sign nothing else.
