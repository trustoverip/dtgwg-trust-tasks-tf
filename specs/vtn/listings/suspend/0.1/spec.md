---
slug: vtn/listings/suspend
version: "0.1"
title: "VTN Listings — Suspend"
summary: "An administrator suspends a member: the network marks its membership grant suspended in its status list, and the member stops receiving broadcasts until reinstated."
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
    The proof is the administrator's authentication: a suspension changes what every verifier of the member's grant concludes, so an unattributed request would let anyone cut a member off.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A suspension replayed after a reinstatement would undo it, so a duplicate must be placeable in a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Sets the grant's bit in the network's suspension status list and moves the entry to `suspended`. Undone by vtn/listings/reinstate.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: a member's DID and an optional reason. Out: the entry. Public identifiers of organisations only; the reason is recorded, and an administrator chooses what it says.
retention:
  class: durable
  rationale: >-
    The suspension and its reason are part of the entry's history and audit trail, the record of how the network governed its members.
errorCodes:
  - code: vtn/listings/suspend:invalidTransition
    meaning: "The entry is not a member: it is `listed`, `repudiated`, `removed` or `departed`. Only an `acknowledged` membership can be suspended. `details` names the state and the event."
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
  - vtn/listings/reinstate
  - vtn/listings/remove
  - vtn/listings/show
  - auth/signing-key/enroll
---

## Abstract

A network can pause a membership without ending it. **VTN Listings — Suspend** moves a member's entry from `acknowledged` to `suspended`, and sets the membership grant's bit in the network's suspension status list. A verifier that checks the grant's `credentialStatus` therefore sees the suspension, and the directory no longer shows the party as a member. A suspended member's groups receive no broadcasts. The credentials are not revoked, so [`vtn/listings/reinstate`](../../reinstate/0.1/spec.md) restores the membership as it was.

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
3. **MUST** answer with the entry unchanged when it is already `suspended`. This is a setting operation ([SPEC §7.2](/SPEC.md#72-consumer-requirements)), so it records no new transition.
4. **MUST** refuse with `vtn/listings/suspend:invalidTransition`, with `details` `{ "from": <state>, "event": "suspend" }`, when the entry is in any state other than `acknowledged` or `suspended`.
5. **MUST** set the bit at the grant's `statusIndex` in its suspension status list, move the entry to `suspended`, append the transition with `reason` to the entry's history, record a `suspend` event in its audit trail naming `issuer` as the actor, and answer with the entry.
6. **MUST NOT** publish a suspended entry as a member, and **MUST NOT** deliver broadcasts to its groups while it is suspended.
7. **MUST NOT** revoke the listing or the grant, and **MUST NOT** change the member's own membership, trust contexts or keys (VTI-VTN-060 – VTI-VTN-063). The suspension is of the network's own credential only.

## Authorization

The authority this task presupposes is **administrator standing on the network**. The `issuer` is one of the administrators named in the network's configuration, or a key that one of them delegated through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md). The network reads that standing when it executes the request (network item 1), so an administrator removed from the configuration, or a delegation revoked, loses the authority at once. The proof establishes who signed and that the document is unaltered ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). The match between the signer and the configured administrators is what entitles it. The member's agreement is not needed: the grant is the network's credential, and its status is the network's to set. A consumer **MAY** require an operation-bound step-up; that is its own policy.

## Definitions

- **`did`**: the member to suspend.
- **`reason`**: why, recorded in the entry's history and the audit trail. Optional.
- **Suspension status list**: the network's Bitstring Status List with purpose `suspension`, which the grant's `credentialStatus` points into.
- **`entry`**: the administrator's view of the entry ([`Entry`](../../../_shared/0.1/entry.schema.json)).

## Request

An administrator (`issuer`) asks the network (`recipient`) to suspend a member. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### An administrator suspends a community

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000c21",
  "type": "https://trusttasks.org/spec/vtn/listings/suspend/0.1#request",
  "issuer": "did:example:admin",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T12:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000c2f",
  "payload": {
    "did": "did:example:community",
    "reason": "Accreditation lapsed; pending renewal."
  }
}
```

## Response

The network answers with the sub-schema reachable via `$anchor: "response"`: the entry, now `suspended`. A refusal is a `trust-task-error`.

### The member is suspended

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000c22",
  "type": "https://trusttasks.org/spec/vtn/listings/suspend/0.1#response",
  "issuer": "did:example:whn",
  "recipient": "did:example:admin",
  "issuedAt": "2026-11-03T12:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000c2f",
  "payload": {
    "entry": {
      "did": "did:example:community",
      "kind": "community",
      "label": "St Vincent's",
      "profile": {},
      "state": "suspended",
      "createdAt": "2026-11-03T10:00:01Z",
      "updatedAt": "2026-11-03T12:00:01Z",
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
        { "changedAt": "2026-11-03T12:00:01Z", "from": "acknowledged", "to": "suspended", "reason": "Accreditation lapsed; pending renewal." }
      ]
    }
  }
}
```

The credentials are abbreviated to their `id`; the network returns them in full.

### The entry is not a member

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000c23",
  "type": "https://trusttasks.org/spec/trust-task-error/0.6",
  "issuer": "did:example:whn",
  "recipient": "did:example:admin",
  "issuedAt": "2026-11-03T12:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000c2f",
  "payload": {
    "code": "vtn/listings/suspend:invalidTransition",
    "message": "transition `suspend` is not allowed from state `listed`",
    "retryable": false,
    "details": { "from": "listed", "event": "suspend" }
  }
}
```

## Security & Privacy

### Data carried

The request carries a member's DID and an optional free-text reason; the response carries the entry. There is no personal data unless an administrator writes some into `reason`, and it **MUST NOT**: the reason is kept in the history that administrators read, and a suspension is about an organisation, not a person.

### Correlation

The suspension is observable by design. Anyone who checks the grant's status, or reads the directory, can see that the member is suspended and roughly when. That is what a suspension is for. Both parties declare `identifierScope: public`. The member is named by the public DID its grant and listing name, and the network by the DID that issued them and publishes the status list, so a pairwise identifier could not name either. The `reason` is not published in the status list; it is visible to administrators in the history.

### Retention

The network keeps the transition and its reason in the entry's history and audit trail for as long as it keeps those, after the entry ends as well, because they record how the network governed its members. The status bit stays set until reinstatement or removal.

### Consent/purpose

The reason is collected to explain the suspension to the network's administrators and auditors, and is used for nothing else. The member's consent is not part of this task: the suspension acts only on the network's own credential (VTI-VTN-060 – VTI-VTN-063). Its remedies are reinstatement by the network, or departure on its own initiative.
