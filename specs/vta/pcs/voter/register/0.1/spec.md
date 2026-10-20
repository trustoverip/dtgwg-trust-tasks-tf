---
slug: vta/pcs/voter/register
version: "0.1"
title: "VTA PCS — Voter Register"
summary: "A voter's client asks its VTA for the blinded request that obtains the voter's ticket for one election; the VTA derives a voter key for that election alone and keeps the blinding state."
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
    The request makes the VTA derive, and act with, the key the voter's ticket will be issued
    to. Where the transport does not already authenticate the caller, the proof is the only
    thing between that key and anyone who can reach the VTA.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential under SPEC §3, so SPEC §7.3 item 17 sets REQUIRED as the floor: a duplicate
    can only be absorbed inside a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    On first use for an election, allocates the election's voter-key path and records the
    election (manifest digest, voting key, ballot box). Records the blinding state of the
    request, which voter/accept needs to unblind the community's answer.
subjectPath: /contextId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    The VTA exercises the election's voter key to build a blinded request in the voter's name.
    Out: the request the community blind-signs — a commitment it cannot open — and the PCS
    identifier of a key used for this election alone. The key and the blinding state stay in
    the VTA.
retention:
  class: durable
  rationale: >-
    The election's record and the blinding state last until the ticket is accepted; the record
    lasts until the election is tallied or cancelled (CONVENTIONS §5.3).
errorCodes:
  - code: vta/pcs/voter/register:electionChanged
    meaning: "The engine holds this election under another manifest digest or voting key."
    retryable: false
  - code: vta/pcs/voter/register:unsupportedSuite
    meaning: "The parameters name a suite the engine does not implement."
    retryable: false
  - code: vta/pcs/voter/register:parametersInvalid
    meaning: "The voting parameters cannot be decoded."
    retryable: false
  - code: vta/pcs/voter/register:unsupportedSecrecy
    meaning: "The manifest's secrecy level is one whose cast task this engine does not serve: `open` is cast with vta/pcs/vote, `sealed` and `talliedEncrypted` with vta/pcs/vote-encrypted."
    retryable: false
  - code: vta/pcs/voter/register:ticketHeld
    meaning: "The engine already holds the voter's ticket for this election. One key and one ticket per election; there is nothing to request."
    retryable: false
  - code: vta/pcs/voter/register:memberNotHeld
    meaning: "`member` names a DID the engine holds no `authentication` key of, or one the caller may not act as."
    retryable: false
related:
  - vtc/elections/tickets/issue
  - vta/pcs/voter/accept
  - vta/pcs/vote
  - vta/pcs/root-request
---

## Abstract

The **VTA PCS — Voter Register** Trust Task is the first half of obtaining a voter's **ticket** for one election. The VTA derives a voter key for that election alone, builds the blinded request of [`vtc/elections/tickets/issue`](../../../../../vtc/elections/tickets/issue/0.1/spec.md) for the ticket class `voter/<electionId>`, and keeps the blinding state. The client sends the request to the community, signed by the voter's member DID, and hands the community's answer to [`vta/pcs/voter/accept`](../../../../../vta/pcs/voter/accept/0.1/spec.md). The engine's custody rules are those of the PCS engine ([VTA conventions §2](../../../../../vta/_shared/0.4/CONVENTIONS.md#2-custody)). The family's shared rules are in [`CONVENTIONS.md`](../../../../../voting/_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the [voting conventions](../../../../../voting/_shared/0.1/CONVENTIONS.md) and of the [PCS engine conventions](../../../../../vta/_shared/0.4/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the client) **MUST**:

1. Pass the manifest and the voting parameters exactly as the community published them.
2. Send the community the response's `ticketDocument` unchanged when it carries one; otherwise send the response payload — `electionId`, `manifestDigest` and `request` — as the payload of a `vtc/elections/tickets/issue/0.1` document signed by the DID the roll admits. Pass the community's response to `vta/pcs/voter/accept`.
3. Name the voter's membership DID as `member` where the engine holds its key, so that the request reaches the community signed as the voter without the client holding that key.

A conforming **consumer** (the PCS engine) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse a caller without standing under [Authorization](#authorization) with `permissionDenied`.
3. Refuse a suite it does not implement with `vta/pcs/voter/register:unsupportedSuite`, and a `votingKey` it cannot decode with `vta/pcs/voter/register:parametersInvalid`.
4. Refuse with `malformedRequest` a manifest whose `community` is not `community`.
5. Refuse with `vta/pcs/voter/register:unsupportedSecrecy` a manifest whose `secrecy` is a level whose cast task it does not serve: [`vta/pcs/vote`](../../../vote/0.1/spec.md) for `open`, [`vta/pcs/vote-encrypted`](../../../vote-encrypted/0.1/spec.md) for `sealed` and `talliedEncrypted`. A ticket is the same at every level; only the cast differs.
6. Compute `manifestDigest`, the SHA-256 multihash of the manifest's RFC 8785 (JCS) canonicalization.
7. Where it already records this election for (`contextId`, `community`, `manifest.electionId`):
   1. refuse with `vta/pcs/voter/register:ticketHeld` if it holds the ticket;
   2. refuse with `vta/pcs/voter/register:electionChanged` if the recorded manifest digest or voting key differs from this request's;
   3. otherwise return the request it kept, unchanged — under the same key and the same blinding state — so that the community's answer to a request already sent can still be unblinded; and, where the request names `member`, the `ticketDocument` it kept for that `member`, unchanged — so that the community answers a resend with its first answer rather than as a second request — or, if it kept none for it, one built as item 9 provides.
8. Otherwise allocate a fresh derivation path under the context's base for this election alone (VTI-VOT-023, [CONVENTIONS §5.2](../../../../../voting/_shared/0.1/CONVENTIONS.md#5-custody-vta)), derive the voter key from it, and build the blinded root request for the class `voter/<electionId>` under the deployment `<community>#voting-pcs` ([CONVENTIONS §3.1–3.2](../../../../../voting/_shared/0.1/CONVENTIONS.md#3-cryptographic-binding)).
9. Where the request names `member`: refuse with `vta/pcs/voter/register:memberNotHeld` unless the engine holds an `authentication` key of `member` that the caller may act as, under the same rule by which it acts as a persona elsewhere; then build `ticketDocument` — a `vtc/elections/tickets/issue/0.1` request document with a fresh random `id`, `issuer` `member`, `recipient` `community`, an `issuedAt` of now and, where it sets one, an `expiresAt` no later than the manifest's `votingCloses`, whose payload is exactly this response's `electionId`, `manifestDigest` and `request` — and sign it with that key for `authentication`. The engine signs only this document type, only with this payload: `member` confers no other signature. These checks come before item 8, so a refused `member` allocates nothing.
10. Record the election — the path, `manifestDigest`, `votingKey` and the manifest's `ballotBox` — keep the request, its blinding state and any `ticketDocument` under the protection of the context's key records ([VTA conventions §2.3](../../../../../vta/_shared/0.4/CONVENTIONS.md#2-custody)), and return the request and the `ticketDocument`: never the key, nor the blinding state.

A consumer **MUST NOT** derive a voter key from a path used for any other key, nor reuse one election's voter key for another. A key per election is what keeps the PCS identifier the community records from linking one election's ticket to the next.

## Authorization

The authority is **standing over the context** the voter belongs to ([conventions §4](../../../../../voting/_shared/0.1/CONVENTIONS.md#4-authorization)). Where it is expressed as a capability, the registered value is **`pcsVoter`** — see `Capability` in [`device/_shared`](../../../../../device/_shared/0.2/device-binding.schema.json). It is not `sign`, and a VTA **MUST NOT** require `sign` for this task, nor satisfy it under `pcsVetter` or `pcsApplicant`. Signing `ticketDocument` as `member` is part of this task, not a general signature: it needs `pcsVoter` and standing to act as `member`, and nothing more — exactly as `vta/pcs/vote` signs the cast. Verifying the `issuer`, the transport identity or the `proof` establishes who asked ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)), never that they may.

## Definitions

- **`contextId`** — the context the voter key belongs to.
- **`community`** — the community's DID.
- **`parameters`** — the community's published voting parameters (`VotingParameters`).
- **`manifest`** — the election's manifest (`Manifest`).
- **`electionId`**, **`manifestDigest`** (response) — the election, and the digest the engine computed. The client compares it with the digest the community and the ballot box show.
- **`request`** (response) — the blinded ticket request. With `electionId` and `manifestDigest` it is exactly the `vtc/elections/tickets/issue` payload to send.
- **`member`** (request) — the voter's membership DID at the community, when the engine holds its key.
- **`ticketDocument`** (response) — the `vtc/elections/tickets/issue/0.1` request, signed as `member`, to send unchanged. Present exactly when the request named `member`.

## Request

The client sends the request to its VTA. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000000a01001",
  "type": "https://trusttasks.org/spec/vta/pcs/voter/register/0.1#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-11-02T09:00:00Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000000a01009",
  "payload": {
    "contextId": "personal/communities",
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "parameters": {
      "suite": "ps-ddh-bls12381",
      "votingKey": "z6MkvVotingKey1abcXYZ"
    },
    "manifest": {
      "electionId": "board-2026",
      "community": "did:webvh:QmCommunity:vtc.example.org",
      "title": "Board election 2026",
      "questions": [
        {
          "key": "chair",
          "type": "choice",
          "text": "Who should chair the board?",
          "options": [
            { "key": "ana", "label": "Ana" },
            { "key": "ben", "label": "Ben" }
          ],
          "minSelections": 1,
          "maxSelections": 1,
          "optional": false
        },
        {
          "key": "budget",
          "type": "choice",
          "text": "Approve the 2027 budget?",
          "options": [
            { "key": "yes" },
            { "key": "no" }
          ],
          "minSelections": 1,
          "maxSelections": 1,
          "optional": true
        }
      ],
      "secrecy": "open",
      "roll": { "rule": "members" },
      "registrationOpens": "2026-11-01T00:00:00Z",
      "votingOpens": "2026-11-08T00:00:00Z",
      "votingCloses": "2026-11-15T00:00:00Z",
      "revote": "replace",
      "board": { "visibility": "public", "auditors": [] },
      "ballotBox": "did:webvh:QmBallotBox:vote.example.net",
      "binding": true,
      "allowEarlyClose": false,
      "resultsBeforeClose": false,
      "textReleaseThreshold": 10
    }
  }
}
```

## Response

The VTA answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000000a01002",
  "type": "https://trusttasks.org/spec/vta/pcs/voter/register/0.1#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-11-02T09:00:01Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000000a01009",
  "payload": {
    "electionId": "board-2026",
    "manifestDigest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
    "request": {
      "label": "voter/board-2026",
      "id": "z4FvpxtP31f2HAEo4V9ffW3VRH1yZJAXY",
      "request": {
        "encoding": "zdjd2scJfUZZT8yfFVfG4UK8RADYCWWni",
        "t0": "zpKvJZS5uaUfGsbMyNW1JVTu1PXZPynmc",
        "proof": "zUCxgm6Xt5yph6DZFZW57AHAA1PdCt1Ny"
      }
    }
  }
}
```

## Security & Privacy

### Data carried

In: the manifest and the community's voting parameters, both public to the voter. Out: a PCS identifier and a commitment the community cannot open. The community learns, when the client sends the request, that the signer asked for a ticket — which it must, to apply the roll — and nothing it can later match against a cast: the ticket it signs is one it never sees ([CONVENTIONS §2](../../../../../voting/_shared/0.1/CONVENTIONS.md#2-separation)).

### Correlation

The identifier is that of a key used for this election alone, so two elections' requests are not linked by it. Within the election the community pairs the identifier with the signer by design; the blind issuance is what keeps that pairing from reaching the cast. The engine's audit records the operation, the caller, the context, the community, the election and the time, and never a tag (VTI-VOT-100).

### Retention

The blinding state lasts until `vta/pcs/voter/accept` consumes it. The election's record lasts until the election is tallied or cancelled; the voter key is never stored, only its path.

### Consent/purpose

To obtain the ticket that lets the voter cast a ballot in this election. Nothing else. This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
