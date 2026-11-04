---
slug: vtn/listings/reinstate
version: "0.1"
title: "VTN Listings — Reinstate"
summary: "An administrator lifts a member's suspension: the network clears the grant's suspension bit and the entry is a member again."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtn, curation, suspension, status-list]
parties:
  - role: administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: network
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The proof is the administrator's authentication: reinstatement makes every verifier of the grant accept the member again, so an unattributed request would let anyone restore a party the network suspended.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A reinstatement replayed after a later suspension would undo it, so a duplicate must be placeable in a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Clears the grant's bit in the network's suspension status list and moves the entry back to `acknowledged`. Undone by vtn/listings/suspend.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: a member's DID and an optional reason. Out: the entry. Public identifiers of organisations only.
retention:
  class: durable
  rationale: >-
    The reinstatement and its reason are part of the entry's history and audit trail, the record of how the network governed its members.
errorCodes:
  - code: vtn/listings/reinstate:invalidTransition
    meaning: "The entry is neither `suspended` nor `acknowledged`: it is `listed`, `repudiated`, `removed` or `departed`, so there is no membership to reinstate. `details` names the state and the event."
    retryable: false
    detailsSchema:
      type: object
      required: [from, event]
      properties:
        from:
          type: string
          enum: [listed, acknowledged, repudiated, suspended, removed, departed]
        event:
          type: string
related:
  - vtn/listings/suspend
  - vtn/listings/remove
  - vtn/listings/show
  - auth/signing-key/enroll
---

## Abstract

**VTN Listings — Reinstate** ends a suspension made with [`vtn/listings/suspend`](../../suspend/0.1/spec.md). The network clears the membership grant's bit in its suspension status list and moves the entry from `suspended` back to `acknowledged`. The grant and the member's acknowledgement are the same credentials as before, so verifiers accept the membership again with nothing reissued, and the member's groups receive broadcasts again.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **administrator** (`issuer`):

1. **MUST NOT** put anything in `reason` that the network's audit policy would not keep. It is recorded in the entry's history.

A conforming **network** (`recipient`):

1. **MUST** refuse with `permissionDenied` unless `issuer` is one of its configured administrators, or a key delegated by one through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md), read from its configuration when it executes the request.
2. **MUST** refuse with `notFound` when it has no entry for `did`.
3. **MUST** answer with the entry unchanged when it is already `acknowledged`. This is a setting operation ([SPEC §7.2](/SPEC.md#72-consumer-requirements)), so it records no new transition.
4. **MUST** refuse with `vtn/listings/reinstate:invalidTransition`, with `details` `{ "from": <state>, "event": "reinstate" }`, when the entry is in any state other than `suspended` or `acknowledged`. Reinstatement never makes a party a member that has not acknowledged (VTI-VTN-013).
5. **MUST** clear the bit at the grant's `statusIndex` in its suspension status list, move the entry to `acknowledged`, append the transition with `reason` to the entry's history, record a `reinstate` event in its audit trail naming `issuer` as the actor, and answer with the entry.
6. **MUST NOT** issue new credentials, and **MUST NOT** change the member's own membership, trust contexts or keys (VTI-VTN-060 – VTI-VTN-063).

## Authorization

The authority this task presupposes is **administrator standing on the network**. The `issuer` is one of the administrators named in the network's configuration, or a key that one of them delegated through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md). The network reads that standing when it executes the request (network item 1), so an administrator removed from the configuration, or a delegation revoked, loses the authority at once. The proof establishes who signed and that the document is unaltered ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). The match between the signer and the configured administrators is what entitles it. The member already agreed when it acknowledged, and nothing here asks it again. A consumer **MAY** require an operation-bound step-up; that is its own policy.

## Definitions

- **`did`**: the suspended member.
- **`reason`**: why, recorded in the entry's history and the audit trail. Optional.
- **`entry`**: the administrator's view of the entry ([`Entry`](../../../_shared/0.1/entry.schema.json)).

## Request

An administrator (`issuer`) asks the network (`recipient`) to reinstate a member. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### An administrator reinstates a community

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000c31",
  "type": "https://trusttasks.org/spec/vtn/listings/reinstate/0.1#request",
  "issuer": "did:example:admin",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T14:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000c3f",
  "payload": {
    "did": "did:example:community",
    "reason": "Accreditation renewed."
  }
}
```

## Response

The network answers with the sub-schema reachable via `$anchor: "response"`: the entry, `acknowledged` again. A refusal is a `trust-task-error`.

### The member is reinstated

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000c32",
  "type": "https://trusttasks.org/spec/vtn/listings/reinstate/0.1#response",
  "issuer": "did:example:whn",
  "recipient": "did:example:admin",
  "issuedAt": "2026-11-03T14:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000c3f",
  "payload": {
    "entry": {
      "did": "did:example:community",
      "kind": "community",
      "label": "St Vincent's",
      "profile": {},
      "state": "acknowledged",
      "createdAt": "2026-11-03T10:00:01Z",
      "updatedAt": "2026-11-03T14:00:01Z",
      "listing": {
        "credential": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0011" },
        "statusIndex": 41
      },
      "grant": {
        "credential": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0012" },
        "statusIndex": 42
      },
      "ack": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0021" },
      "history": [
        { "changedAt": "2026-11-03T10:00:01Z", "to": "listed" },
        { "changedAt": "2026-11-03T10:30:00Z", "from": "listed", "to": "acknowledged" },
        { "changedAt": "2026-11-03T12:00:01Z", "from": "acknowledged", "to": "suspended", "reason": "Accreditation lapsed; pending renewal." },
        { "changedAt": "2026-11-03T14:00:01Z", "from": "suspended", "to": "acknowledged", "reason": "Accreditation renewed." }
      ]
    }
  }
}
```

The credentials are abbreviated to their `id`; the network returns them in full.

## Security & Privacy

### Data carried

The request carries a member's DID and an optional free-text reason; the response carries the entry. There is no personal data unless an administrator writes some into `reason`, and it **MUST NOT**: the reason is kept in the history that administrators read.

### Correlation

Reinstatement is observable by design: the grant's status changes back, and the directory shows the party as a member again. Both parties declare `identifierScope: public`. The member is named by the public DID its grant names, and the network by the DID that issued the grant and publishes the status list, so a pairwise identifier could not name either. The `reason` is not published in the status list.

### Retention

The network keeps the transition and its reason in the entry's history and audit trail for as long as it keeps those, because together with the suspension they record how the network governed the member.

### Consent/purpose

The reason is collected to explain the reinstatement to the network's administrators and auditors, and is used for nothing else. The member agreed to membership when it acknowledged. Reinstatement restores that membership and asks nothing new of it.
