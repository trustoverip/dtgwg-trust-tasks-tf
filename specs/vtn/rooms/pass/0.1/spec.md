---
slug: vtn/rooms/pass
version: "0.1"
title: "VTN Rooms — Pass"
summary: "A person presents their membership credential and the chain of group registrations up to the network, and receives a short-lived pass to a room."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtn, rooms, room-pass, vac, nested-membership]
parties:
  - role: person
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: network
    requirement: REQUIRED
    member: recipient
    identifierScope: public
subjectPath: /membership/credentialSubject/id
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The proof is the person's authentication: the pass is issued to the identity the membership credential names, so the network must know the requester holds it. The response carries a credential the person will present, so it is signed too.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed request would mint a fresh pass on evidence that may since have been revoked, so a duplicate must be placeable in a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Issues and signs a room pass. It expires after the room's `passMinutes` and is not otherwise revoked.
consequences:
  - "Issues a pass, signed by the network, that a room host honours as entry to the room until it expires."
exposure:
  discloses: secret
  ingests: personal
  actsAsSubject: false
  rationale: >-
    In: the person's identity and their membership credential from their group, which are personal data, and the registration chain from that group up to the network. Out: the pass — a credential the person retains and presents to enter the room, returned here and nowhere else.
retention:
  class: transient
  rationale: >-
    The network verifies the evidence and keeps none of it: it caches no roster, and a pass is issued on evidence checked at the time. It keeps only the request identifiers its duplicate-detection window needs and an operational record that a pass was issued.
errorCodes:
  - code: vtn/rooms/pass:unknownRoom
    meaning: "The network opens no room with id `room`."
    retryable: false
  - code: vtn/rooms/pass:notEligible
    meaning: "The evidence does not admit the requester: the membership credential does not verify, does not name the requester's identity, or is revoked, suspended or of indeterminate status; or the chain does not end at a group registered here for the room's role by a member whose entry is `acknowledged`. `details.message` says which."
    retryable: false
    detailsSchema:
      type: object
      required: [message]
      properties:
        message:
          type: string
  - code: vtn/rooms/pass:chainInvalid
    meaning: "A chain link does not verify — a proof fails, the acceptance is not a groupRegistered credential from the registration's recipient or its digest does not match the registration, the acceptance is revoked or suspended or its status cannot be determined, the link is out of order, or the chain is longer than the network follows. `details.index` names the link and `details.message` says why."
    retryable: false
    detailsSchema:
      type: object
      required: [message]
      properties:
        index:
          type: integer
          minimum: 0
        message:
          type: string
related:
  - vtn/rooms/list
  - vtn/groups/register
  - vtn/entries/show
---

## Abstract

A Verifiable Trust Network can open a **room** for a role — everyone in the groups its members registered for `admins`, however deep the nesting ([`vtn/rooms/list`](../../list/0.1/spec.md)). The network holds no roster of who that is, and must not ([VTI specification, *The Verifiable Trust Network*](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-VTN-032, VTI-VTN-033, VTI-VTN-060). So a person proves eligibility with evidence they carry, checked at the time:

1. their **membership credential** (VMC) from a group — a community's group they belong to;
2. a **chain** of registrations from that group up to this network: for each network between them, the group owner's signed [`vtn/groups/register`](../../../groups/register/0.1/spec.md) request and that network's groupRegistered credential accepting it, which the network revokes when the registration ends. Their community hands them these.

The chain must end at a group registered *here*, for the room's role, by a current member. The network then issues a short-lived **room pass**: an authority credential for `room:enter`, scoped to the room, which a room host honours.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **person** (`issuer`):

1. **MUST** sign the request as the identity their membership credential names, or with a key that identity delegated through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md) or approved through `auth/oob/*`.
2. **MUST** order `chain` nearest first: the first link registers the group that issued their membership credential, and each later link registers the network group of the network that accepted the link before it. An empty `chain` says their group is registered with this network directly.
3. **MUST NOT** present the pass to anyone but a host of the room it names, or after it expires.

A conforming **network** (`recipient`):

1. **MUST** refuse with `vtn/rooms/pass:unknownRoom` when it opens no room with id `room`.
2. **MUST** verify `membership` as a membership credential issued by its `issuer` (the person's group), signed for `assertionMethod`, within its validity period, a grant rather than an acknowledgement (its subject carries no `digestMultibase`), naming the requester's identity as its subject, and with a status, resolved against its issuer's status list, that is valid. It **MUST** refuse with `vtn/rooms/pass:notEligible` on any failure, and **MUST** treat an unreachable or unverifiable status list as a failure.
3. **MUST** then walk the chain, starting with the group that issued `membership`, and following at most 8 links:
   - if the current group is registered with this network for the room's role, by a member whose entry is `acknowledged`, the walk succeeds; if it is registered but its member is not `acknowledged`, the walk fails with `notEligible`;
   - otherwise the next link **MUST** verify exactly as [`vtn/groups/register`](../../../groups/register/0.1/spec.md) states for a verifier: the registration's proof; the acceptance, a `groupRegistered/1` statement signed for `assertionMethod` by the accepting network (the registration's `recipient`), within its validity period, whose subject is the registration's `group` and whose `digestMultibase` equals the registration digest of the registration; and the acceptance's status, resolved against the accepting network's revocation status list. A revoked or suspended acceptance, and a status that cannot be determined — an indeterminate status is never valid (VTI-REG-012) — fail the link. The registration's `group` **MUST** be the current group; the network then records the accepting network in `via` and continues with that network's network group for the registered role, `<acceptingNetwork>#role-<role>`.

   A link that fails these checks, or a chain longer than 8 links, **MUST** be refused with `vtn/rooms/pass:chainInvalid`, with `details.index` naming the link. Running out of links before reaching a group registered here **MUST** be refused with `vtn/rooms/pass:notEligible`.
4. On success **MUST** issue and sign a pass: an authority credential issued by the network, its subject the requester's identity, for action `room:enter` on scope `<networkDid>#room-<room>`, valid from now until now plus the room's `passMinutes`.
5. **MUST NOT** retain the membership credential or the chain beyond the request, and **MUST NOT** build or keep a roster from them (VTI-VTN-060).

## Authorization

The authority this task presupposes is **holding a member's membership credential**: the person is entitled to a pass by belonging to a group that a current member — directly, or through a chain of member networks — registered for the room's role. The request's proof establishes who asks; the membership credential, issued by the group and naming that identity, establishes that they belong to it; and the chain establishes, under each accepting network's own signature, that the group serves the role all the way up. This network's own standing decision is the one it reads at execution time: which of its members are `acknowledged`, and which groups they registered. A consumer **MAY** require an operation-bound step-up before issuing a pass, as policy of its own.

## Definitions

- **`room`** — the id of a room the network opens.
- **`membership`** — the person's membership credential from their group ([`Membership`](../../../_shared/0.1/entry.schema.json)): a grant issued by the group, naming the person's identity as its subject.
- **`chain`** — the registrations from the person's group up to a group registered here, nearest first ([`ChainLink`](../../../_shared/0.1/groups.schema.json)): each a `registration`, a member's signed `vtn/groups/register` request, and its `acceptance`, the accepting network's revocable groupRegistered credential over that request's digest.
- **Requester's identity** — the request's `issuer`, or, where the issuer is a delegated key, the identity it was delegated by.
- **`pass`** — the room pass ([`RoomPass`](../../../_shared/0.1/rooms.schema.json)): the room as configured, the signed pass credential, and `via`, the networks whose acceptances the chain carried, nearest first.

## Request

The person (`issuer`) asks the network (`recipient`) for a pass. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A hospital administrator, two networks down

Alice belongs to St Vincent's `admins` group. St Vincent's registered that group with Health-NSW for `admins`; Health-NSW registered its network group with Health-AU; Health-AU registered its own with WHN, which opens the room.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000001201",
  "type": "https://trusttasks.org/spec/vtn/rooms/pass/0.1#request",
  "issuer": "did:example:alice",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T13:01:00Z",
  "payload": {
    "room": "pandemic-response",
    "membership": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0061" },
    "chain": [
      {
        "registration": { "id": "urn:uuid:00000000-0000-4000-8000-000000000811" },
        "acceptance": { "id": "urn:uuid:00000000-0000-4000-8000-000000000812" }
      },
      {
        "registration": { "id": "urn:uuid:00000000-0000-4000-8000-000000000821" },
        "acceptance": { "id": "urn:uuid:00000000-0000-4000-8000-000000000822" }
      }
    ]
  }
}
```

The membership credential and the registration documents are abbreviated to their `id`; the person sends them in full, with their proofs.

## Response

The network answers with the sub-schema reachable via `$anchor: "response"`: the pass. A refusal is a `trust-task-error`, and issues nothing.

### The pass

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000001202",
  "type": "https://trusttasks.org/spec/vtn/rooms/pass/0.1#response",
  "issuer": "did:example:whn",
  "recipient": "did:example:alice",
  "issuedAt": "2026-11-03T13:01:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000001201",
  "payload": {
    "pass": {
      "room": {
        "id": "pandemic-response",
        "name": "Pandemic Response",
        "role": "admins",
        "url": "https://rooms.whn.example/pandemic-response",
        "passMinutes": 60
      },
      "pass": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0071" },
      "via": ["did:example:health-nsw", "did:example:health-au"]
    }
  }
}
```

The pass credential is abbreviated to its `id`; the network returns it in full.

## Security & Privacy

### Data carried

The request carries the person's identity, their membership credential from their group, and the registration chain — documents between organisations, which name groups and networks but no other person. The response carries the pass, which names the person and the room. A person **MUST NOT** add anything to the request beyond the credential and chain the task needs; the network **MUST NOT** put anything in the pass beyond the room scope, the action and its validity.

### Correlation

The network learns the person's identity, their group, and the path from that group up to it, and the room host later sees the same identity on the pass. The network declares `identifierScope: public` because the pass is only worth honouring if a room host can check it against the network's DID, named the same way every time, and the room's scope is built on that DID. The person's scope is `any`: the pass is bound to whatever identity their group's membership credential names, and nothing here requires that identity to be shared with any other relationship — a person who wants their room passes unlinkable to anything else can hold a membership credential for an identity used only for that.

### Retention

The network keeps neither the membership credential nor the chain (network item 5); it keeps the request's identifiers for its duplicate-detection window and an operational record that it issued a pass. The pass expires after the room's `passMinutes`. Every link's acceptance is checked for status at the time, so a registration that has ended anywhere on the path stops admitting anyone as soon as its network revokes the acceptance, subject only to how long a fetched status list is cached.

### Consent/purpose

The person presents their membership credential to obtain a pass to one room, and the network uses it for that decision alone. The community that handed the person their chain consented, by registering its group, to that registration being evidence of the group's role.
