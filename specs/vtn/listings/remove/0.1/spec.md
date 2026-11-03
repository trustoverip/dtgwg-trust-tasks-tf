---
slug: vtn/listings/remove
version: "0.1"
title: "VTN Listings — Remove"
summary: "An administrator removes an entry: the network revokes every credential it issued for it and drops its registered groups, ending the listing and any membership."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtn, curation, delisting, revocation]
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
    The proof is the administrator's authentication: removal permanently revokes the network's credentials for a party, so an unattributed request would let anyone end a membership irreversibly.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A removal replayed after the party was listed again would end the new listing, so a duplicate must be placeable in a bounded window.
sideEffects:
  level: destructive
  rationale: >-
    Revokes the listing credential, the offer and the grant in the network's revocation status list. Revocation is permanent: listing the party again issues new credentials and needs a new acknowledgement, and never restores these. The party's registered groups are dropped.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: an entry's DID and an optional reason. Out: the entry. Public identifiers of organisations only.
retention:
  class: durable
  rationale: >-
    The removed entry, its history and the revoked credentials remain the record of what the network once said about the party and when it stopped.
errorCodes:
  - code: vtn/listings/remove:invalidTransition
    meaning: "The entry is `departed`: the party already left, and the network's credentials for it already ended. `details` names the state and the event."
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
  - vtn/listings/create
  - vtn/listings/suspend
  - vtn/listings/show
  - vtn/membership/repudiate
  - auth/signing-key/enroll
---

## Abstract

**VTN Listings — Remove** ends an entry on the network's side. It applies whether the party is only listed, a member, suspended, or has repudiated its listing. The network revokes every credential it issued for the entry in its revocation status list: the listing credential, the offer and, for a member, the membership grant (VTI-VTN-053). It drops the groups the party registered, and moves the entry to `removed`. Removal ends the network's own credentials only. The party's own credentials, its members, its trust contexts and its keys are the party's, and the network neither touches nor can touch them (VTI-VTN-060 – VTI-VTN-063).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **administrator** (`issuer`):

1. **SHOULD** use [`vtn/listings/suspend`](../../suspend/0.1/spec.md) instead where the membership may resume. A removed party can be listed again, but only with new credentials and a new acknowledgement.
2. **MUST NOT** put anything in `reason` that the network's audit policy would not keep.

A conforming **network** (`recipient`):

1. **MUST** refuse with `permissionDenied` unless `issuer` is one of its configured administrators, or a key delegated by one through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md), read from its configuration when it executes the request.
2. **MUST** refuse with `notFound` when it has no entry for `did`.
3. **MUST** answer with the entry unchanged when it is already `removed`. This is a setting operation ([SPEC §7.2](/SPEC.md#72-consumer-requirements)), so it records no new transition.
4. **MUST** refuse with `vtn/listings/remove:invalidTransition`, with `details` `{ "from": "departed", "event": "remove" }`, when the entry is `departed`.
5. **MUST** otherwise set, in its revocation status list, the bit of every credential it issued for the entry that it still holds (the listing, the offer and the grant), and **MUST** drop the groups the party registered so that no broadcast is delivered to them.
6. **MUST** discard the offer, move the entry to `removed`, append the transition with `reason` to the entry's history, record a `remove` event in its audit trail naming `issuer` as the actor, and answer with the entry.
7. **MUST NOT** publish a removed entry in its index or directory.
8. **MUST NOT** revoke, alter or claim authority over any credential the party issued, including its acknowledgement, or over the party's own members, trust contexts or keys (VTI-VTN-060 – VTI-VTN-063).

## Authorization

The authority this task presupposes is **administrator standing on the network**. The `issuer` is one of the administrators named in the network's configuration, or a key that one of them delegated through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md). The network reads that standing when it executes the request (network item 1), so an administrator removed from the configuration, or a delegation revoked, loses the authority at once. The proof establishes who signed and that the document is unaltered ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). The match between the signer and the configured administrators is what entitles it. The authority reaches only the network's own credentials, which is why the party's agreement is not needed and why nothing of the party's is affected (network item 8). A consumer **MAY** require an operation-bound step-up before an irreversible removal; that is its own policy.

## Definitions

- **`did`**: the entry to remove.
- **`reason`**: why, recorded in the entry's history and the audit trail. Optional.
- **Revocation status list**: the network's Bitstring Status List with purpose `revocation`, into which the `credentialStatus` of each credential the network issued points.
- **`entry`**: the administrator's view of the entry ([`Entry`](../../../_shared/0.1/entry.schema.json)), now `removed`, with the revoked credentials kept as the record.

## Request

An administrator (`issuer`) asks the network (`recipient`) to remove an entry. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### An administrator removes a member

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000c41",
  "type": "https://trusttasks.org/spec/vtn/listings/remove/0.1#request",
  "issuer": "did:example:admin",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T15:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000c4f",
  "payload": {
    "did": "did:example:community",
    "reason": "Withdrew from the governance framework."
  }
}
```

## Response

The network answers with the sub-schema reachable via `$anchor: "response"`: the entry, now `removed`. A refusal is a `trust-task-error`.

### The entry is removed

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000c42",
  "type": "https://trusttasks.org/spec/vtn/listings/remove/0.1#response",
  "issuer": "did:example:whn",
  "recipient": "did:example:admin",
  "issuedAt": "2026-11-03T15:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000c4f",
  "payload": {
    "entry": {
      "did": "did:example:community",
      "kind": "community",
      "label": "St Vincent's",
      "profile": {},
      "state": "removed",
      "createdAt": "2026-11-03T10:00:01Z",
      "updatedAt": "2026-11-03T15:00:01Z",
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
        { "changedAt": "2026-11-03T15:00:01Z", "from": "acknowledged", "to": "removed", "reason": "Withdrew from the governance framework." }
      ]
    }
  }
}
```

The credentials are abbreviated to their `id`; the network returns them in full.

## Security & Privacy

### Data carried

The request carries an entry's DID and an optional free-text reason; the response carries the removed entry. There is no personal data unless an administrator writes some into `reason`, and it **MUST NOT**: the reason is kept in the history that administrators read.

### Correlation

Removal is observable by design. A verifier sees the network's credentials revoked, and the party disappears from the directory. Both parties declare `identifierScope: public`. The party is named by the public DID its revoked credentials name, and the network by the DID that issued them and publishes the status lists, so a pairwise identifier could not name either. The `reason` is not published.

### Retention

The network keeps the removed entry, its revoked credentials and its history, and the audit trail, after removal. They are the record of what the network once said about the party and when it stopped saying it, which a verifier presented with an old credential may need to understand. The revocation bits are never cleared.

### Consent/purpose

The reason is collected to explain the removal to the network's administrators and auditors, and is used for nothing else. The party's consent is not part of this task, because removal acts only on what the network issued. The party's own standing, members and keys are untouched (VTI-VTN-060 – VTI-VTN-063).
