---
slug: vtc/events/anonymous/withdraw
version: '0.1'
title: "VTC Events — Anonymous — Withdraw"
summary: "Withdraw from an anonymous event by a fresh attestation carrying the registration's tag."
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
  rationale: "Sets the registration held by the tag to withdrawn, freeing its places for the waitlist."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request carries an attestation that names no member; the response is the registration as held by its tag."
retention:
  class: durable
  rationale: "The registration's state is kept until the event's records are deleted."
errorCodes:
- code: vtc/events/anonymous/withdraw:notAnonymous
  meaning: "The event is not `anonymous`."
  retryable: false
- code: vtc/events/anonymous/withdraw:attestationInvalid
  meaning: "The attestation does not verify under the community's key, to this event's identity, bound to this document (VTI-EVT-213, 214)."
  retryable: false
- code: vtc/events/anonymous/withdraw:classNotInAudience
  meaning: "The attestation's class is not a live class of the event's audience in the current period (VTI-EVT-213)."
  retryable: false
- code: vtc/events/anonymous/withdraw:attestationReplayed
  meaning: "This attestation was carried before (VTI-EVT-214)."
  retryable: false
- code: vtc/events/anonymous/withdraw:notRegistered
  meaning: "No registration is held by this tag."
  retryable: false
- code: vtc/events/anonymous/withdraw:occurrenceRequired
  meaning: "The event is a series registered by occurrence, and the request names no `occurrence`."
  retryable: false
- code: vtc/events/anonymous/withdraw:occurrenceUnknown
  meaning: "`occurrence` is not an original start the series' rule gives, or the event is not a series."
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/anonymous/register
- vta/pcs/member/attest
---
## Abstract

The **VTC Events — Anonymous — Withdraw** Trust Task withdraws an anonymous registration by a fresh attestation carrying its tag (VTI-EVT-215): only the member who registered can make it again.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`) and the VTI Community Events chapter (VTI-EVT-190 – 232).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** **MUST**:

1. Accept the document from any signer and charge it to the operation budget for senders without standing.
2. Verify the attestation as vtc/events/anonymous/register does (VTI-EVT-213 – 215), before anything else.
3. Refuse `notRegistered` where no registration is held by the tag; otherwise set it to `withdrawn`, freeing its places (VTI-EVT-033, 037), answering `changed` false where it was not in play.

## Authorization

**The holder of the registration's tag**, proven by a fresh attestation carrying it (VTI-EVT-215).

## Definitions

- `EventPcsAttestation`, `EventRef` — see [`event.schema.json`](../../../_shared/0.4/event.schema.json).

## Request

The registrant (one-time key) (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Withdraw anonymously

```json
{
  "id": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000001",
  "type": "https://trusttasks.org/spec/vtc/events/anonymous/withdraw/0.1#request",
  "issuer": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-28T10:00:00Z",
  "payload": {
    "eventId": "evt_W5k2N8xRb3Qz7TpLAAAA",
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/anonymous/withdraw/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Withdrawn

```json
{
  "id": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000002",
  "type": "https://trusttasks.org/spec/vtc/events/anonymous/withdraw/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "issuedAt": "2026-11-28T10:00:01Z",
  "threadId": "urn:uuid:5d1c8e2a-9b4f-4c7e-8a3d-000000000001",
  "payload": {
    "tag": "zQ3shVc2UkAfJCdc1TR8E66J85h48P19mxcSvquTzPSnSqgxS",
    "status": "withdrawn",
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
    },
    "changed": true
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
