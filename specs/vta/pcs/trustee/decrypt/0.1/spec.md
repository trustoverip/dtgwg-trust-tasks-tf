---
slug: vta/pcs/trustee/decrypt
version: "0.1"
title: "VTA PCS — Trustee Decrypt"
summary: "The trustee's VTA makes its partial decryption of each target, with proofs, and returns the voting/trustees/decrypt request signed as the trustee."
status: retired
supersededBy: vta/pcs/trustee/decrypt/0.2
targetFrameworkVersion: "0.6.0"
category: identity
keywords:
  - pcs
  - voting
  - trustee
  - vta
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
    The request makes the VTA act as the trustee with keys that decide the election's secrecy. Where the transport does not already authenticate the caller, the proof is the only thing between those keys and anyone who can reach the VTA.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential under SPEC §3, so SPEC §7.3 item 17 sets REQUIRED as the floor.
sideEffects:
  level: none
  rationale: >-
    Uses the key share; keeps nothing new.
subjectPath: /contextId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    Out: partial decryptions with proofs.
retention:
  class: transient
  rationale: >-
    Nothing new is kept.
errorCodes:
  - code: vta/pcs/trustee/decrypt:trusteeNotHeld
    meaning: "The VTA holds no `authentication` key of `trustee` that the caller may act as, or the manifest does not name `trustee` as a trustee."
    retryable: false
  - code: vta/pcs/trustee/decrypt:keyNotHeld
    meaning: "This VTA holds no key share of the trustee for this election."
    retryable: false
  - code: vta/pcs/trustee/decrypt:notClosed
    meaning: "Voting has not closed: no decryption is made before it does (VTI-VOT-063)."
    retryable: false
related:
  - voting/trustees/decrypt
  - voting/trustees/status
---

## Abstract

The **VTA PCS — Trustee Decrypt** Trust Task makes the trustee's share of every decryption the result needs (CONVENTIONS §9.7), never before voting closes (VTI-VOT-063). The family's shared rules are in [`CONVENTIONS.md`](../../../../../voting/_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is **retired** per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); it is superseded by [`vta/pcs/trustee/decrypt/0.2`](../0.2/spec.md), which acts only on the signed evidence the ballot box cannot forge (CONVENTIONS §9.6 – 9.7) where this version took values the ballot box reported. The schema is frozen; the document is retained so already-issued documents remain verifiable.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../../../voting/_shared/0.1/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the client) **MUST**:

1. Pass the targets and the closing time voting/trustees/status and voting/election/show report, and send the returned document unchanged.

A conforming **consumer** (the VTA) **MUST**, in this order:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `vta/pcs/trustee/decrypt:trusteeNotHeld` unless it holds an `authentication` key of `trustee` the caller may act as.
3. Refuse with `vta/pcs/trustee/decrypt:keyNotHeld` unless it holds the trustee's key share for the election.
4. Refuse with `vta/pcs/trustee/decrypt:notClosed` while `closedAt` is in the future.
5. Return the voting/trustees/decrypt request with one partial decryption per target, each with its Chaum–Pedersen proof, signed as `trustee`.

## Authorization

The authority is **standing over the context** the trustee persona lives in, expressed as the capability **`pcsTrustee`**, and standing to act as `trustee`. It is not `sign`: the VTA signs only this task's document as the trustee.

## Definitions

- **`targets`** — what to decrypt.
- **`closedAt`** — when voting closed: the time of the board's `closed` or `closedEarly` entry, or — for a client that cannot read the board — any time at or after the close and not in the future, such as the earlier of now and the manifest's `votingCloses` once [`voting/trustees/status`](../../../../../voting/trustees/status/0.1/spec.md) reports `decrypting`. It only holds the VTA back before the close; the ballot box refuses partials until voting has closed whatever it says.

## Request

The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c1001",
  "type": "https://trusttasks.org/spec/vta/pcs/trustee/decrypt/0.1#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-11-02T09:00:00Z",
  "payload": {
    "contextId": "trustees",
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "electionId": "board-2026",
    "trustee": "did:webvh:QmTrustee:alice.example",
    "targets": [
      {
        "question": "chair",
        "option": "ana",
        "ciphertext": {
          "a": "z4Lp9VmXkCq2Jt7HbR1NfWd8YsGa3Ue6ZoQi5MxKcTvPy",
          "b": "z4Lp9VmXkCq2Jt7HbR1NfWd8YsGa3Ue6ZoQi5MxKcTvPy"
        }
      }
    ],
    "closedAt": "2026-11-15T00:00:00Z"
  }
}
```

## Response

The answer is the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c1002",
  "type": "https://trusttasks.org/spec/vta/pcs/trustee/decrypt/0.1#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-11-02T09:00:01Z",
  "threadId": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c1001",
  "payload": {
    "document": {
      "id": "urn:uuid:0e2f9b1c-5d4a-4e3b-9a8c-7d6e5f4a3b2c",
      "type": "https://trusttasks.org/spec/voting/trustees/decrypt/0.1",
      "issuer": "did:webvh:QmTrustee:alice.example",
      "recipient": "did:webvh:QmBallotBox:vote.example.net",
      "issuedAt": "2026-11-15T09:00:01Z",
      "payload": {},
      "proof": {}
    }
  }
}
```

## Security & Privacy

### Data carried

Partial decryptions out; the key share stays.

### Correlation

Trustees are public by design (VTI-VOT-090); nothing here concerns a voter.

### Retention

Nothing new is kept.

### Consent/purpose

The VTA acts as the trustee only for this election's ceremony and decryption.
