---
slug: vta/pcs/voter/accept
version: "0.1"
title: "VTA PCS — Voter Accept"
summary: "A voter's client hands its VTA the community's answer to a ticket request; the VTA unblinds it into the voter's ticket for the election and keeps it."
status: draft
targetFrameworkVersion: "0.6.0"
category: identity
keywords:
  - pcs
  - voting
  - election
  - zero-knowledge
  - vta
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
    The request makes the VTA act with the election's voter key and store the ticket that
    admits the voter's ballot. Where the transport does not already authenticate the caller,
    the proof is the only thing between that key and anyone who can reach the VTA.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential under SPEC §3, so SPEC §7.3 item 17 sets REQUIRED as the floor: a duplicate
    can only be absorbed inside a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Stores the voter's ticket for the election, and discards the blinding state it was
    unblinded with.
subjectPath: /contextId
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    The VTA exercises the election's voter key to unblind the community's answer into a ticket
    the community has never seen. Out: an acknowledgement only, never the ticket.
retention:
  class: durable
  rationale: >-
    The ticket is what admits the voter's ballot; it is kept until the election is tallied or
    cancelled (CONVENTIONS §5.3).
errorCodes:
  - code: vta/pcs/voter/accept:notFound
    meaning: "The engine records no ticket request for this election in this context and community: voter/register never ran, or the election was forgotten after it was tallied or cancelled."
    retryable: false
  - code: vta/pcs/voter/accept:noRequestInFlight
    meaning: "The engine holds no blinding state for this election: the ticket was already accepted, and is held."
    retryable: false
  - code: vta/pcs/voter/accept:credentialInvalid
    meaning: "The answer does not unblind into a ticket that verifies under the voting key for the class `voter/<electionId>`. The blinding state is kept, so a correct answer can still be accepted."
    retryable: false
  - code: vta/pcs/voter/accept:parametersInvalid
    meaning: "The voting parameters cannot be decoded, or name a voting key other than the one the request was registered under."
    retryable: false
related:
  - vtc/elections/tickets/issue
  - vta/pcs/voter/register
  - vta/pcs/vote
  - vta/pcs/root-accept
---

## Abstract

The **VTA PCS — Voter Accept** Trust Task is the second half of obtaining a voter's ticket. The client passes the community's [`vtc/elections/tickets/issue`](../../../../../vtc/elections/tickets/issue/0.1/spec.md) response; the VTA unblinds it with the state [`vta/pcs/voter/register`](../../../../../vta/pcs/voter/register/0.1/spec.md) kept, checks the ticket against the community's voting key, and keeps it wrapped like every other PCS secret of the engine ([VTA conventions §2](../../../../../vta/_shared/0.4/CONVENTIONS.md#2-custody)). The family's shared rules are in [`CONVENTIONS.md`](../../../../../voting/_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the [voting conventions](../../../../../voting/_shared/0.1/CONVENTIONS.md) and of the [PCS engine conventions](../../../../../vta/_shared/0.4/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the client) **MUST**:

1. Pass the community's response payload as `response`, without its `ext`.

A conforming **consumer** (the PCS engine) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse a caller without standing under [Authorization](#authorization) with `permissionDenied`.
3. Refuse with `vta/pcs/voter/accept:notFound` if it records no election `electionId` for (`contextId`, `community`).
4. Refuse with `malformedRequest` a `response` whose `electionId` is not `electionId` or whose `label` is not `voter/<electionId>`.
5. Refuse with `vta/pcs/voter/accept:parametersInvalid` a `votingKey` it cannot decode or that is not the one recorded when the request was registered.
6. Refuse with `vta/pcs/voter/accept:noRequestInFlight` if it holds no blinding state for the election.
7. Unblind `response.preCredential` with the kept blinding state and verify the result under `votingKey` for the class `voter/<electionId>` under the deployment `<community>#voting-pcs` ([CONVENTIONS §3.1–3.2](../../../../../voting/_shared/0.1/CONVENTIONS.md#3-cryptographic-binding)); refuse with `vta/pcs/voter/accept:credentialInvalid` if it does not verify, and keep the blinding state.
8. On success keep the ticket wrapped ([CONVENTIONS §5.3](../../../../../voting/_shared/0.1/CONVENTIONS.md#5-custody-vta)), discard the blinding state and the kept request, and answer `ticketHeld: true`. The ticket is never returned.

## Authorization

The authority is **standing over the context** the voter belongs to ([conventions §4](../../../../../voting/_shared/0.1/CONVENTIONS.md#4-authorization)). Where it is expressed as a capability, the registered value is **`pcsVoter`** — see `Capability` in [`device/_shared`](../../../../../device/_shared/0.2/device-binding.schema.json). It is not `sign`, and a VTA **MUST NOT** require `sign` for this task, nor satisfy it under `pcsVetter` or `pcsApplicant`. Verifying the `issuer`, the transport identity or the `proof` establishes who asked ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)), never that they may.

## Definitions

- **`contextId`**, **`community`**, **`electionId`** — the election's record in the engine.
- **`parameters`** — the community's published voting parameters.
- **`response`** — the community's answer to the ticket request.
- **`ticketHeld`** (response) — always `true` on success.

## Request

The client sends the request to its VTA. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000000a02001",
  "type": "https://trusttasks.org/spec/vta/pcs/voter/accept/0.1#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-11-02T09:00:05Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000000a02009",
  "payload": {
    "contextId": "personal/communities",
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "electionId": "board-2026",
    "parameters": {
      "suite": "ps-ddh-bls12381",
      "votingKey": "z6MkvVotingKey1abcXYZ"
    },
    "response": {
      "electionId": "board-2026",
      "label": "voter/board-2026",
      "preCredential": "zv4FAsGW8WPSZcPpmLuwdzejotKtvy9ep"
    }
  }
}
```

## Response

The VTA answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000000a02002",
  "type": "https://trusttasks.org/spec/vta/pcs/voter/accept/0.1#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-11-02T09:00:06Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000000a02009",
  "payload": {
    "electionId": "board-2026",
    "ticketHeld": true
  }
}
```

## Security & Privacy

### Data carried

A pre-credential in, useless to anyone without the blinding state. An acknowledgement out.

### Correlation

None beyond what voter/register already established. The ticket is one the community has never seen, so nothing the community holds pairs it with the voter. A client **SHOULD NOT** build the cast in the same exchange that accepted the ticket ([CONVENTIONS §6.3](../../../../../voting/_shared/0.1/CONVENTIONS.md#6-privacy), VTI-VOT-033).

### Retention

The ticket is kept until the election is tallied or cancelled, then discarded ([CONVENTIONS §5.3](../../../../../voting/_shared/0.1/CONVENTIONS.md#5-custody-vta)).

### Consent/purpose

To hold the voter's ticket for the election. This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
