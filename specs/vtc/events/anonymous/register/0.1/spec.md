---
slug: vtc/events/anonymous/register
version: '0.1'
title: "VTC Events — Anonymous — Register"
summary: "Register for an anonymous event by an attestation of class membership; the registration is held by its tag alone."
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
  rationale: "The document is signed by a key made for it alone, so the proof binds the attestation to this document (VTI-EVT-214) rather than to a member."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor: a replay could only be absorbed inside a bounded window."
sideEffects:
  level: mutating
  rationale: "Creates or changes a registration held by its tag, and records the attestation's binding to refuse a replay."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request carries an attestation that names no member; the response is the registration as held by its tag."
retention:
  class: durable
  rationale: "The registration is kept until the event's records are deleted."
errorCodes:
- code: vtc/events/anonymous/register:notAnonymous
  meaning: "The event is not `anonymous`: register with vtc/events/registrations/register."
  retryable: false
- code: vtc/events/anonymous/register:attestationInvalid
  meaning: "The attestation does not verify under the community's key, to this event's identity, bound to this document (VTI-EVT-213, 214)."
  retryable: false
- code: vtc/events/anonymous/register:classNotInAudience
  meaning: "The attestation's class is not a live class of the event's audience in the current period (VTI-EVT-213)."
  retryable: false
- code: vtc/events/anonymous/register:attestationReplayed
  meaning: "This attestation was carried before (VTI-EVT-214)."
  retryable: false
- code: vtc/events/anonymous/register:eventCancelled
  meaning: "The event is cancelled."
  retryable: false
- code: vtc/events/anonymous/register:notOpen
  meaning: "The event takes no registrations."
  retryable: false
- code: vtc/events/anonymous/register:windowClosed
  meaning: "Registrations are not open now."
  retryable: false
- code: vtc/events/anonymous/register:full
  meaning: "The event is full and keeps no waitlist."
  retryable: true
- code: vtc/events/anonymous/register:tooManyGuests
  meaning: "More guests than the event allows."
  retryable: false
- code: vtc/events/anonymous/register:occurrenceRequired
  meaning: "The event is a series registered by occurrence, and the request names no `occurrence`."
  retryable: false
- code: vtc/events/anonymous/register:occurrenceUnknown
  meaning: "`occurrence` is not an original start the series' rule gives, or the event is not a series."
  retryable: false
- code: vtc/events/anonymous/register:occurrenceCancelled
  meaning: "The occurrence named is cancelled."
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/anonymous/withdraw
- vtc/events/anonymous/checkin
- vtc/events/classes/show
- vta/pcs/member/attest
---
## Abstract

The **VTC Events — Anonymous — Register** Trust Task registers for an `anonymous` event (VTI-EVT-210) by an attestation that its sender holds a credential for a live class of the event's audience (VTI-EVT-213). The registration is held by the attestation's tag alone; a second registration with the same tag changes the first. The document is signed by a key made for it alone (VTI-EVT-214).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`) and the VTI Community Events chapter (VTI-EVT-190 – 232).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** **MUST**:

1. Accept the document from any signer — it is signed by a key made for it alone (VTI-EVT-214) — and charge it to the operation budget for senders without standing.
2. Refuse `notAnonymous` for an event that is not `anonymous`, and answer `notFound` for an event that is not published.
3. Verify the attestation under its own verification key, to the event's identity, bound to the event, this task's name, the document's `id` and its signer (VTI-EVT-213, 214); refuse `attestationInvalid`, `classNotInAudience` or `attestationReplayed` as their meanings state, before looking at anything else of the registration.
4. Apply the event's registration settings — window, capacity, waitlist, guests (VTI-EVT-031 – 038) — and, on a series registered by occurrence, the occurrence (VTI-EVT-194 – 196).
5. Hold the registration by the attestation's tag alone: one per tag, a second registration with the same tag changing the first; record no identifier of the registrant (VTI-EVT-210, 213).
6. Answer the registration as held, and record the act in its audit trail by tag, never more (VTI-EVT-091).

## Authorization

**Anyone holding a credential for a live class of the event's audience**, proven by the attestation alone; the signer is a key made for this document and carries no standing.

## Definitions

- `EventPcsAttestation`, `EventRef` — see [`event.schema.json`](../../../_shared/0.4/event.schema.json).
- **Tag** — the value an attestation carries, determined by the member's key and the event's identity (VTI-EVT-213).

## Request

The registrant (one-time key) (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Register anonymously

```json
{
  "id": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000001",
  "type": "https://trusttasks.org/spec/vtc/events/anonymous/register/0.1#request",
  "issuer": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-28T10:00:00Z",
  "payload": {
    "eventId": "evt_W5k2N8xRb3Qz7TpLAAAA",
    "status": "going",
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/anonymous/register/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Registered

```json
{
  "id": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000002",
  "type": "https://trusttasks.org/spec/vtc/events/anonymous/register/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "issuedAt": "2026-11-28T10:00:01Z",
  "threadId": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000001",
  "payload": {
    "tag": "zQ3shVc2UkAfJCdc1TR8E66J85h48P19mxcSvquTzPSnSqgxS",
    "status": "going",
    "guests": 0,
    "event": {
      "eventId": "evt_W5k2N8xRb3Qz7TpLAAAA",
      "title": "Thursday Build Night",
      "when": {
        "start": "2026-12-03T18:00:00",
        "timeZone": "Europe/London",
        "duration": "PT3H"
      },
      "status": "published",
      "sequence": 2
    }
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
