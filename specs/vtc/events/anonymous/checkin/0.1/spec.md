---
slug: vtc/events/anonymous/checkin
version: '0.1'
title: "VTC Events — Anonymous — Check In"
summary: "Check in to an anonymous event by a fresh attestation carrying the registration's tag, bound to the door's challenge."
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- pcs
- zero-knowledge
- privacy
parties:
- role: "registrant (one-time key)"
  requirement: REQUIRED
  member: issuer
  identifierScope: pairwise
- role: "community maintainer"
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "The document is signed by a key made for it alone, so the proof binds the attestation to this document (VTI-EVT-214)."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor: a replay could only be absorbed inside a bounded window."
sideEffects:
  level: mutating
  rationale: "Records the registration held by the tag as checked in; no attendance is counted for anyone (VTI-EVT-216)."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request carries an attestation bound to the door's challenge; the response says when the check-in was recorded."
retention:
  class: durable
  rationale: "The check-in follows the event's retention (VTI-EVT-166)."
errorCodes:
- code: vtc/events/anonymous/checkin:notAnonymous
  meaning: "The event is not `anonymous`."
  retryable: false
- code: vtc/events/anonymous/checkin:attestationInvalid
  meaning: "The attestation does not verify under the community's key, to this event's identity, bound to this document (VTI-EVT-213, 214)."
  retryable: false
- code: vtc/events/anonymous/checkin:classNotInAudience
  meaning: "The attestation's class is not a live class of the event's audience in the current period (VTI-EVT-213)."
  retryable: false
- code: vtc/events/anonymous/checkin:attestationReplayed
  meaning: "This attestation was carried before (VTI-EVT-214)."
  retryable: false
- code: vtc/events/anonymous/checkin:challengeInvalid
  meaning: "The challenge is not the door's, is stale, or is for another event or occurrence."
  retryable: false
- code: vtc/events/anonymous/checkin:notGoing
  meaning: "The registration held by this tag is not going."
  retryable: false
- code: vtc/events/anonymous/checkin:alreadyCheckedIn
  meaning: "The registration held by this tag is already checked in."
  retryable: false
- code: vtc/events/anonymous/checkin:notDuringEvent
  meaning: "The event, or the occurrence, is not running."
  retryable: false
- code: vtc/events/anonymous/checkin:occurrenceRequired
  meaning: "The event is a series registered by occurrence, and the request names no `occurrence`."
  retryable: false
- code: vtc/events/anonymous/checkin:occurrenceUnknown
  meaning: "`occurrence` is not an original start the series' rule gives, or the event is not a series."
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/anonymous/register
- vtc/events/checkin/challenge
- vta/pcs/member/attest
---
## Abstract

The **VTC Events — Anonymous — Check In** Trust Task checks a registrant in to an `anonymous` event by a fresh attestation carrying the registration's tag and bound to the door's challenge (VTI-EVT-161, 215). Nothing identifies the registrant, and no attendance is counted (VTI-EVT-216).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`) and the VTI Community Events chapter (VTI-EVT-190 – 232).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** **MUST**:

1. Accept the document from any signer and charge it to the operation budget for senders without standing.
2. Verify the attestation as vtc/events/anonymous/register does, and that it is also bound to the challenge (VTI-EVT-215).
3. Refuse `challengeInvalid`, `notGoing`, `alreadyCheckedIn` and `notDuringEvent` as their meanings state.
4. Record the check-in against the tag, count no attendance for it (VTI-EVT-216) and record no identifier of the registrant.

## Authorization

**The holder of a going registration's tag**, at the door, proven by a fresh attestation bound to the door's challenge.

## Definitions

- `EventPcsAttestation`, `EventDoorChallengeValue` — see [`event.schema.json`](../../../_shared/0.4/event.schema.json).

## Request

The registrant (one-time key) (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Check in anonymously

```json
{
  "id": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000001",
  "type": "https://trusttasks.org/spec/vtc/events/anonymous/checkin/0.1#request",
  "issuer": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-28T10:00:00Z",
  "payload": {
    "eventId": "evt_W5k2N8xRb3Qz7TpLAAAA",
    "challenge": "u7Hk3NqP9xL2mVb8RtY4wZc6Jd1sFg5A",
    "attestation": {
      "suite": "pcs-bls12-381-v1",
      "attestation": "z3u2en7t5LR2WJQ2YcVPYa1XxYz8XoJ7v1q5pFh3pE6Kp6uJmKq9ZcN2",
      "tag": "zQ3shVc2UkAfJCdc1TR8E66J85h48P19mxcSvquTzPSnSqgxS",
      "classLabel": "member/2026-12"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH#z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
    "created": "2026-11-28T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/anonymous/checkin/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Checked in

```json
{
  "id": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000002",
  "type": "https://trusttasks.org/spec/vtc/events/anonymous/checkin/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "issuedAt": "2026-11-28T10:00:01Z",
  "threadId": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000001",
  "payload": {
    "tag": "zQ3shVc2UkAfJCdc1TR8E66J85h48P19mxcSvquTzPSnSqgxS",
    "checkedInAt": "2026-12-03T18:05:00Z"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:community#key-1",
    "created": "2026-11-28T10:00:01Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Security & Privacy

### Data carried

In: the event, an attestation (a tag, a re-randomised class credential and a proof) and the registration's state. Out: the registration as held by its tag. No member identifier travels in either direction.

### Correlation

The document is signed by a key made for it alone (VTI-EVT-214) and SHOULD reach the community over a connection that does not identify the member, after a pause. The tag is the same for one member at one event and unlinkable to their tag at any other event, or to their identity, under the decisional Diffie–Hellman assumption the predicate credential system rests on.

### Retention

The community keeps the tag, the registration's state and its guests until the event's records are deleted (VTI-EVT-090), and the attestation's binding only as long as it needs to refuse a replay.

### Consent/purpose

Anonymity hides a member among the holders of the class; the event shows how many there are (VTI-EVT-217) and the period after which a departed member's credential lapses (VTI-EVT-218).
