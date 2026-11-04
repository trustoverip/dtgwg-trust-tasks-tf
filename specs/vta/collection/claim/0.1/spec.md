---
slug: vta/collection/claim
version: "0.1"
title: "VTA Collection — Claim"
summary: "A client asks its VTA to make it, until a stated end, the one collector of the messages queued at the mediator for a persona DID whose keys the VTA holds — or renews a claim it holds — so that the client and the VTA never both hold that DID's one mediator connection."
status: draft
targetFrameworkVersion: "0.6.0"
category: notifications
keywords:
  - collection
  - mediator
  - persona
  - vta
  - notifications
  - lease
parties:
  - role: Client acting for the principal
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: >-
    Normally carried by an authenticated transport between a client and its own VTA. The claim
    confers no authority and moves no key: it records which of two parties that may already collect
    for the identifier does so, and the caller's standing in the context is what the VTA checks.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task changes a record and closes the VTA's connections, so the VTA must be able to place the
    request in a bounded window to absorb a repeat (SPEC §7.2 item 11).
sideEffects:
  level: mutating
  rationale: >-
    Records or extends a collection claim, and stops the VTA collecting for the identifier until the
    claim ends. Nothing is granted beyond that, and a repeat moves the end again.
subjectPath: /contextId
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: a context, an identifier and a requested duration. Out: the identifier and the claim's end.
retention:
  class: durable
  rationale: >-
    The VTA keeps the claim until it ends, so that it does not collect for the identifier meanwhile
    and can show the claim in vta/collection/list (VTI-NTF-091).
errorCodes:
  - code: vta/collection/claim:notCollectable
    meaning: >-
      `identifier` is not one the VTA would collect for in `contextId`, whether or not collection
      by default is enabled (VTI-NTF-093): the VTA does not hold its
      messaging keys there, its DID document advertises no transport through a mediator, or it is
      the VTA's own durable identity, another node's or host service's, or an identity the VTA
      provisioned for another party, each of which its own holder collects for (VTI-NTF-080).
    retryable: false
  - code: vta/collection/claim:claimHeld
    meaning: >-
      Another client holds a collection claim for the identifier that is in force. The VTA grants at
      most one claim per identifier (VTI-NTF-081, VTI-NTF-082). The other claim can be released with
      vta/collection/release, after which it still ends at its end; the identifier can be claimed
      once it has.
    retryable: true
  - code: vta/collection/claim:claimNotRenewable
    meaning: >-
      The caller's own claim for the identifier was released by another party, and ends at its end
      without renewal (VTI-NTF-084). The caller stops collecting before that end, and may claim the
      identifier again once it has ended.
    retryable: true
related:
  - vta/collection/release
  - vta/collection/list
  - vta/notifications/deposit
  - notifications/notify
  - notifications/relay
---

## Abstract

The **VTA Collection — Claim** Trust Task decides who empties a persona's mailbox. A VTA holds the
keys of the DIDs it mints for its principal — a persona presented to a community, for instance —
and those DIDs advertise a mediator. Messages addressed to them wait at that mediator until some
party collects them, and the mediator permits **one connection per DID**: a second evicts the
first. A VTA collects for every such DID that no client has claimed (VTI-NTF-080), so that a
principal whose only client is a browser, which cannot hold the connection, still receives what is
addressed to its personas. A client that does hold the connection — and hands what it receives for
the persona to the VTA with [`vta/notifications/deposit`](../../../notifications/deposit/0.1/spec.md)
— claims the DID with this task first.

A claim is a **lease**. The VTA grants it until a stated end, after stopping its own collection and
closing its own connection for the DID; the client connects only once it holds the grant, renews
the claim before it ends with the same task, and closes its connection before the end unless it has
renewed. A client that disappears without releasing its claim — uninstalled, lost, revoked — leaves
the DID with the VTA again once the claim ends, and the mediator holds what arrived meanwhile.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **producer** (the client) **MUST**:

1. Claim an identifier with this task before it opens any connection that collects for it, and
   **MUST NOT** open one until it holds the grant (VTI-NTF-083). A claim covers every protocol the
   mediator carries for the identifier on that connection — DIDComm and TSP alike — because the
   connection is one.
2. Stop collecting for the identifier, and close every such connection, before `endsAt` less its
   clock-skew tolerance as its own clock reads, unless it has renewed the claim by then. It
   **SHOULD** ask for `durationSeconds` `86400` (twenty-four hours), and **SHOULD** renew once
   half of the period granted has passed, so that a failed renewal leaves time to retry before it
   must close (VTI-NTF-083).
3. On `vta/collection/claim:claimHeld` or `vta/collection/claim:notCollectable`, not collect for
   the identifier; and on `vta/collection/claim:claimNotRenewable`, stop collecting before the end
   it was last granted.
4. Hand every `notifications/notify` and `notifications/relay` document it collects to the VTA with
   [`vta/notifications/deposit`](../../../notifications/deposit/0.1/spec.md) (VTI-NTF-054,
   VTI-NTF-092).

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements), and refuse with
   `permissionDenied` a caller without standing under [Authorization](#authorization).
2. Refuse with `vta/collection/claim:notCollectable` an `identifier` VTI-NTF-080 does not name in
   `contextId`, comparing identifiers as VTI-KEY-160 – VTI-KEY-169 require for
   the form they take.
3. Refuse with `vta/collection/claim:claimHeld` a claim while another client's claim for the
   identifier is in force, and with `vta/collection/claim:claimNotRenewable` a renewal of a claim
   that is no longer renewable (VTI-NTF-084).
4. Before it answers a grant, stop collecting for the identifier and close every connection it
   holds for it (VTI-NTF-082). It **MUST NOT** answer a grant while a connection of its own for the
   identifier remains open.
5. Set the claim's end no later than the maximum period it states, which **SHOULD NOT** exceed
   thirty days (VTI-NTF-082), and no later than the requested
   `durationSeconds` from the grant where one is given; answer with that end; and, for a renewal by
   the claimant, move the end and answer `renewed: true`.
6. Not collect for the identifier until the claim has ended and, unless its claimant released it,
   until `endsAt` plus its own clock-skew tolerance has passed (VTI-NTF-084).

**Retry class** (VTI-OPS-060): the second — a durable effect whose repeat is harmless. A repeat of
the same document is answered with the answer first given
([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 11); a new claim document from the claimant is
a renewal.

**A repeated request** sets a state: a claim by the client that already holds the claim succeeds,
as a renewal.

Claims are served whether or not the VTA's operator has enabled collection by default
(VTI-NTF-093). While it is disabled the VTA collects for no unclaimed identifier, so a grant changes
only the record; a client claims all the same, so that its claim is in place when collection is
enabled.

## Authorization

**Standing in the trust context** named by `contextId`, as for
[`vta/notifications/list`](../../../notifications/list/0.1/spec.md#authorization): an access control
entry whose act scope covers that context. The claim belongs to the caller's entry: it becomes
unrenewable when the entry is removed, expires or stops covering the context, and passes to the
entry's successor on a rollover or a key rotation (VTI-NTF-084). Holding a claim is not holding the
identifier's keys; a client that collects needs the identifier's `messaging` keys by whatever means
the VTA already offers it, and the claim decides only that it, and not the VTA, uses them to
collect.

## Definitions

- **`contextId`** — the trust context the identifier belongs to.
- **`identifier`** — the DID to collect for.
- **`durationSeconds`** — the requested length of the claim.
- **Collect** — retrieve the messages queued for the identifier at the mediator its DID document
  advertises. **Collector** — the party that does so. **Claimant** — the client holding the claim.
- **`endsAt`** (response) — when the claim ends unless renewed.
- **`renewed`** (response) — whether this grant moved the end of a claim the caller already held.

## Request

The client sends its VTA the top-level schema in [`payload.schema.json`](payload.schema.json).

### A desktop client claims its persona before connecting

```json
{
  "id": "urn:uuid:5d6e7f80-91a2-4b3c-8d4e-5f6a7b8c9d0e",
  "type": "https://trusttasks.org/spec/vta/collection/claim/0.1",
  "issuer": "did:key:z6MkDesktopClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2027-03-01T09:00:00Z",
  "payload": {
    "contextId": "personal",
    "identifier": "did:webvh:QmPersona:persona.example",
    "durationSeconds": 86400
  }
}
```

## Response

The VTA answers with the sub-schema reachable via `$anchor: "response"`. A claim it does not grant
is answered with `trust-task-error`.

### Granted

```json
{
  "id": "urn:uuid:6e7f8091-a2b3-4c4d-9e5f-6a7b8c9d0e1f",
  "type": "https://trusttasks.org/spec/vta/collection/claim/0.1#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkDesktopClient",
  "issuedAt": "2027-03-01T09:00:01Z",
  "threadId": "urn:uuid:5d6e7f80-91a2-4b3c-8d4e-5f6a7b8c9d0e",
  "payload": {
    "identifier": "did:webvh:QmPersona:persona.example",
    "endsAt": "2027-03-02T09:00:01Z"
  }
}
```

## Security & Privacy

### Data carried

A context, an identifier and a duration in; the identifier and an instant out. Nothing about the
messages queued for the identifier.

### Correlation

The claim tells the VTA which of the principal's clients collects for which persona, which it
already knows from the client's standing. Nothing about the claim reaches the mediator or any
sender: what a sender receives in reply to a notification is the same whichever party collected it
(VTI-NTF-086).

### Retention

The VTA keeps the claim until it ends, and **SHOULD NOT** keep it afterwards beyond what its audit
trail records.

### Consent/purpose

A claim exists only to keep two collectors of one identifier from evicting each other. It confers
nothing: it does not give the claimant the identifier's keys, any authority at the VTA, or any say
over the VTA's acceptance policy, which still decides every notification the claimant hands over.
